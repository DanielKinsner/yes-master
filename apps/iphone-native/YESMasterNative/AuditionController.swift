import Foundation
import UniformTypeIdentifiers

/// Analyze + full-song render surface the controller needs. `NativeMasteringBridge`
/// is the real implementation; tests inject a fake to assert that audition never
/// triggers a render and that Create Master is the only render path.
protocol MasteringRenderer {
    var supportedImportExtensions: [String] { get }
    var supportedImportContentTypes: [UTType] { get }
    func analyzeTrack(at url: URL) throws -> NativeAnalysisResult
    func renderMaster(from sourceURL: URL, toDirectory: URL, options: NativeRenderOptions) throws -> NativeRenderJob
}

extension NativeMasteringBridge: MasteringRenderer {}

/// Owns audition behavior: import/analyze once, play Original immediately, hear
/// Mastered live, switch sides on one preserved timeline, and change
/// Style/Intensity/Loudness live — all without rendering a preview WAV. Create
/// Master is the only full-song render. The view binds to this; the live DSP and
/// transport live in `LiveAudioEngine` + the Rust stream.
@MainActor
final class AuditionController: ObservableObject {
    enum Side: String, CaseIterable, Identifiable {
        case original = "Original"
        case mastered = "Mastered"
        var id: String { rawValue }
    }

    // Selection state (drives the controls).
    @Published var selectedPreset: NativeStylePreset = .balanced
    @Published var selectedLoudness: NativeLoudness = .medium
    @Published private(set) var selectedSide: Side = .original
    @Published private(set) var volumeMatchEnabled = false
    @Published var presetIntensity: Double = 0.5

    // Track / analysis / render state.
    @Published private(set) var importedTrack: ImportedTrack?
    @Published private(set) var analysisResult: NativeAnalysisResult?
    @Published private(set) var masteredLufs: Double?
    @Published private(set) var shareMasterURL: URL?
    @Published private(set) var isAnalyzing = false
    @Published private(set) var isRendering = false
    @Published private(set) var isPlaying = false
    @Published var statusText = "Import a track to begin."

    let supportedImportContentTypes: [UTType]

    private let engine: LiveAudioEngine
    private let renderer: MasteringRenderer
    private let importStore: ImportedTrackStore
    private let renderStorage: RenderStorage

    private(set) var analysisTask: Task<Void, Never>?
    private(set) var renderTask: Task<Void, Never>?
    private var positionTimer: Timer?

    init(
        engine: LiveAudioEngine = LiveAudioEngine(),
        renderer: MasteringRenderer = NativeMasteringBridge(),
        importStore: ImportedTrackStore = ImportedTrackStore(),
        renderStorage: RenderStorage = RenderStorage()
    ) {
        self.engine = engine
        self.renderer = renderer
        self.importStore = importStore
        self.renderStorage = renderStorage
        self.supportedImportContentTypes = renderer.supportedImportContentTypes
    }

    // MARK: - Import / analyze

    func handleImportResult(_ result: Result<[URL], Error>) {
        switch result {
        case .success(let urls):
            guard let sourceURL = urls.first else {
                statusText = "No track was selected."
                return
            }
            importTrack(from: sourceURL)
        case .failure:
            statusText = "Import was cancelled."
        }
    }

    func importTrack(from sourceURL: URL) {
        do {
            let track = try importStore.importTrack(
                from: sourceURL,
                supportedExtensions: renderer.supportedImportExtensions
            )
            stopPlayback()
            importedTrack = track
            analysisResult = nil
            masteredLufs = nil
            shareMasterURL = nil
            selectedSide = .original

            // Decode once for live playback. No mastered preview is rendered.
            let loaded = engine.load(
                url: track.localURL,
                preset: selectedPreset.bridgeIdentifier,
                intensity: Float(presetIntensity),
                loudnessTarget: selectedLoudness.lufsTarget
            )
            engine.setOriginal(true)
            renderStorage.enforceLimit(in: renderStorage.importsDirectory, max: 20)

            guard loaded else {
                statusText = "The file imported, but the audio could not be read. Try a standard WAV, MP3, or M4A."
                return
            }
            analyze(track)
        } catch ImportedTrackStore.ImportError.unsupportedExtension(let fileExtension) {
            let label = fileExtension.isEmpty ? "that file" : ".\(fileExtension)"
            statusText = "\(label) is not supported yet. Use \(renderer.supportedImportExtensions.joined(separator: ", "))."
        } catch ImportedTrackStore.ImportError.sourceUnavailable {
            statusText = "That file is not available. Make sure it finished downloading, then try again."
        } catch ImportedTrackStore.ImportError.emptyFile {
            statusText = "That file looks empty. Make sure it finished downloading, then try again."
        } catch ImportedTrackStore.ImportError.unreadableContainer(let fileExtension) {
            statusText = ".\(fileExtension) was selected, but it does not look like readable audio."
        } catch {
            statusText = "Track could not be imported. Try another supported audio file."
        }
    }

    private func analyze(_ track: ImportedTrack) {
        analysisTask?.cancel()
        analysisResult = nil
        isAnalyzing = true
        statusText = "Analyzing with the YES Master engine."

        let renderer = renderer
        analysisTask = Task {
            let result = await Task.detached {
                Result { try renderer.analyzeTrack(at: track.localURL) }
            }.value

            guard !Task.isCancelled else { return }
            isAnalyzing = false
            switch result {
            case .success(let analysis):
                analysisResult = analysis
                statusText = "Ready. Press play to audition."
            case .failure(let error):
                analysisResult = nil
                statusText = friendlyAudioErrorMessage(error)
            }
        }
    }

    // MARK: - Transport

    var canPlay: Bool {
        importedTrack != nil && analysisResult != nil && !isAnalyzing && !isRendering
    }

    func togglePlayback() {
        guard canPlay else {
            statusText = importedTrack == nil
                ? "Import and analyze a track before playback."
                : "Analyzing — just a moment."
            return
        }
        if isPlaying {
            stopPlayback()
            statusText = "Playback paused."
            return
        }
        // Parked at the end? Restart from the top so play always plays.
        if engine.durationSeconds > 0, engine.positionSeconds >= engine.durationSeconds - 0.05 {
            engine.seek(toSeconds: 0)
        }
        do {
            try engine.play()
            isPlaying = true
            startPositionTimer()
            statusText = "Playing \(selectedSide.rawValue.lowercased()) track."
        } catch {
            statusText = "Playback could not start. Try another supported audio file."
        }
    }

    /// Called ~10x/sec while playing. When the cursor reaches the end, stop so
    /// the play button returns to "play"; the next play restarts from the top.
    /// Also halts the silent past-EOF render pull.
    func handlePlaybackTick() {
        guard isPlaying, engine.durationSeconds > 0 else { return }
        if engine.positionSeconds >= engine.durationSeconds - 0.02 {
            stopPlayback()
            statusText = "Reached the end. Press play to listen again."
        }
    }

    private func stopPlayback() {
        engine.pause()
        isPlaying = false
        stopPositionTimer()
    }

    private func startPositionTimer() {
        stopPositionTimer()
        positionTimer = Timer.scheduledTimer(withTimeInterval: 0.1, repeats: true) { [weak self] _ in
            Task { @MainActor in self?.handlePlaybackTick() }
        }
    }

    private func stopPositionTimer() {
        positionTimer?.invalidate()
        positionTimer = nil
    }

    /// Switch Original/Mastered on the single live timeline. The cursor never
    /// moves, so the playhead is preserved exactly — no pause/resume/re-render.
    func selectSide(_ side: Side) {
        guard side != selectedSide else { return }
        selectedSide = side
        engine.setOriginal(side == .original)
        applyVolumeMatch()
        statusText = "Switched to \(side.rawValue.lowercased()) at the same spot."
    }

    // MARK: - Live settings (no render)

    func setStyle(_ preset: NativeStylePreset) {
        selectedPreset = preset
        pushParams()
    }

    func setIntensity(_ value: Double) {
        presetIntensity = value
        pushParams()
    }

    func setLoudness(_ loudness: NativeLoudness) {
        selectedLoudness = loudness
        pushParams()
    }

    private func pushParams() {
        engine.setParams(
            preset: selectedPreset.bridgeIdentifier,
            intensity: Float(presetIntensity),
            loudnessTarget: selectedLoudness.lufsTarget
        )
    }

    // MARK: - Volume Match (audition only)

    func toggleVolumeMatch() {
        volumeMatchEnabled.toggle()
        applyVolumeMatch()
    }

    private func currentSideGain() -> Float {
        guard volumeMatchEnabled,
              let original = analysisResult?.lufsIntegrated,
              let master = masteredLufs else { return 1.0 }
        switch selectedSide {
        case .original: return volumeMatchLinearGain(sideLufs: original, otherLufs: master)
        case .mastered: return volumeMatchLinearGain(sideLufs: master, otherLufs: original)
        }
    }

    private func applyVolumeMatch() {
        engine.setVolumeMatch(linearGain: currentSideGain())
    }

    // MARK: - Create Master (the only full-song render)

    var currentRenderOptions: NativeRenderOptions {
        NativeRenderOptions(
            preset: selectedPreset.bridgeIdentifier,
            intensity: Float(presetIntensity),
            lufsTarget: selectedLoudness.lufsTarget
        )
    }

    func createMaster() {
        guard let track = importedTrack, analysisResult != nil else {
            statusText = "Import and analyze a track before creating a master."
            return
        }
        renderTask?.cancel()
        isRendering = true
        shareMasterURL = nil
        statusText = "Creating master with the YES Master engine."

        let renderer = renderer
        let sourceURL = track.localURL
        let outputDirectoryURL = renderStorage.mastersDirectory
        let options = currentRenderOptions

        renderTask = Task {
            let result = await Task.detached {
                Result {
                    try renderer.renderMaster(from: sourceURL, toDirectory: outputDirectoryURL, options: options)
                }
            }.value

            guard !Task.isCancelled else { return }
            isRendering = false
            switch result {
            case .success(let job):
                guard let outputPath = job.outputPaths.first else {
                    statusText = "Master render finished but no WAV was returned."
                    return
                }
                shareMasterURL = URL(fileURLWithPath: outputPath)
                masteredLufs = job.measurements?.lufsIntegrated
                renderStorage.enforceLimit(in: renderStorage.mastersDirectory, max: 20)
                applyVolumeMatch()
                statusText = "Master created. Share it anytime."
            case .failure(let error):
                shareMasterURL = nil
                statusText = friendlyAudioErrorMessage(error)
            }
        }
    }

    private func friendlyAudioErrorMessage(_ error: Error) -> String {
        let message = error.localizedDescription
        if message.localizedCaseInsensitiveContains("no suitable format reader")
            || message.localizedCaseInsensitiveContains("decode error") {
            return "The file imported, but the audio could not be read. Try a standard WAV, MP3, or M4A."
        }
        return message
    }
}
