import AVFoundation
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

enum AuditionErrorState: Equatable {
    case noSelection
    case importCancelled
    case unsupportedExtension(String)
    case sourceUnavailable
    case emptyFile
    case unreadableContainer(String)
    case importFailed
    case decodeFailed
    case playbackFailed
    case analysisFailed(String)
    case masterUnavailable
    case renderNoOutput
    case renderFailed(String)

    var message: String {
        switch self {
        case .noSelection:
            "No track was selected."
        case .importCancelled:
            "Import was cancelled."
        case .unsupportedExtension(let fileExtension):
            fileExtension.isEmpty
                ? "That file is not supported yet."
                : ".\(fileExtension) is not supported yet."
        case .sourceUnavailable:
            "That file is not available. Make sure it finished downloading, then try again."
        case .emptyFile:
            "That file looks empty. Make sure it finished downloading, then try again."
        case .unreadableContainer(let fileExtension):
            ".\(fileExtension) was selected, but it does not look like readable audio."
        case .importFailed:
            "Track could not be imported. Try another supported audio file."
        case .decodeFailed:
            "The file imported, but the audio could not be read. Try a standard WAV, MP3, or M4A."
        case .playbackFailed:
            "Playback could not start. Try another supported audio file."
        case .analysisFailed(let message), .renderFailed(let message):
            message
        case .masterUnavailable:
            "Import and analyze a track before creating a master."
        case .renderNoOutput:
            "Master render finished but no WAV was returned."
        }
    }
}

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
    @Published private(set) var errorState: AuditionErrorState?
    @Published var statusText = "Import a track to begin."

    let supportedImportContentTypes: [UTType]

    private let engine: LiveAudioEngine
    private let renderer: MasteringRenderer
    private let importStore: ImportedTrackStore
    private let renderStorage: RenderStorage

    private(set) var analysisTask: Task<Void, Never>?
    private(set) var renderTask: Task<Void, Never>?
    private var landingTask: Task<Void, Never>?
    private var landingGeneration: UInt64 = 0
    private var positionTimer: Timer?
    private var wasPlayingBeforeInterruption = false
    private var notificationObservers: [NSObjectProtocol] = []

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
        observeAudioSession()
    }

    deinit {
        notificationObservers.forEach { NotificationCenter.default.removeObserver($0) }
    }

    private func setStatus(_ message: String) {
        errorState = nil
        statusText = message
    }

    private func setError(_ state: AuditionErrorState) {
        errorState = state
        statusText = state.message
    }

    @discardableResult
    private func bumpLandingGeneration() -> UInt64 {
        landingGeneration &+= 1
        return landingGeneration
    }

    // MARK: - Import / analyze

    func handleImportResult(_ result: Result<[URL], Error>) {
        switch result {
        case .success(let urls):
            guard let sourceURL = urls.first else {
                setError(.noSelection)
                return
            }
            importTrack(from: sourceURL)
        case .failure:
            setError(.importCancelled)
        }
    }

    func importTrack(from sourceURL: URL) {
        do {
            landingTask?.cancel()
            bumpLandingGeneration()
            analysisTask?.cancel()
            renderTask?.cancel()
            isAnalyzing = false
            isRendering = false
            setStatus("Importing track...")
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
                setError(.decodeFailed)
                return
            }
            analyze(track)
            scheduleLandingRefresh()
        } catch ImportedTrackStore.ImportError.unsupportedExtension(let fileExtension) {
            setError(.unsupportedExtension(fileExtension))
        } catch ImportedTrackStore.ImportError.sourceUnavailable {
            setError(.sourceUnavailable)
        } catch ImportedTrackStore.ImportError.emptyFile {
            setError(.emptyFile)
        } catch ImportedTrackStore.ImportError.unreadableContainer(let fileExtension) {
            setError(.unreadableContainer(fileExtension))
        } catch {
            setError(.importFailed)
        }
    }

    private func analyze(_ track: ImportedTrack) {
        analysisTask?.cancel()
        analysisResult = nil
        isAnalyzing = true
        setStatus("Analyzing audio...")

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
                applyVolumeMatch() // original LUFS is now known; landing fills mastered LUFS
                setStatus("Ready. Press play to audition.")
            case .failure(let error):
                analysisResult = nil
                setError(audioErrorState(error, fallback: .analysisFailed))
            }
        }
    }

    // MARK: - Transport

    var canPlay: Bool {
        importedTrack != nil && analysisResult != nil && !isAnalyzing && !isRendering
    }

    func togglePlayback() {
        guard canPlay else {
            if importedTrack == nil {
                setError(.analysisFailed("Import and analyze a track before playback."))
            } else {
                setStatus("Analyzing — just a moment.")
            }
            return
        }
        if isPlaying {
            stopPlayback()
            setStatus("Playback paused.")
        } else {
            startPlayback()
            if isPlaying {
                setStatus("Playing \(selectedSide.rawValue.lowercased()) track.")
            }
        }
    }

    /// Begin (or resume) playback from the current playhead, restarting from the
    /// top if parked at the end.
    private func startPlayback() {
        guard canPlay, !isPlaying else { return }
        if engine.durationSeconds > 0, engine.positionSeconds >= engine.durationSeconds - 0.05 {
            engine.seek(toSeconds: 0)
        }
        do {
            try engine.play()
            isPlaying = true
            startPositionTimer()
        } catch {
            setError(.playbackFailed)
        }
    }

    /// Called ~10x/sec while playing. When the cursor reaches the end, stop so
    /// the play button returns to "play"; the next play restarts from the top.
    /// Also halts the silent past-EOF render pull.
    func handlePlaybackTick() {
        guard isPlaying, engine.durationSeconds > 0 else { return }
        if engine.positionSeconds >= engine.durationSeconds - 0.02 {
            stopPlayback()
            setStatus("Reached the end. Press play to listen again.")
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

    // MARK: - Audio session interruptions / route changes

    /// A phone call (or other interruption) began: pause, remembering whether we
    /// were playing so we can resume when it ends.
    func handleInterruptionBegan() {
        wasPlayingBeforeInterruption = isPlaying
        if isPlaying {
            stopPlayback()
            setStatus("Paused for an interruption.")
        }
    }

    /// The interruption ended: resume only if we were playing and the system says
    /// we may (`AVAudioSession.InterruptionOptions.shouldResume`).
    func handleInterruptionEnded(shouldResume: Bool) {
        guard wasPlayingBeforeInterruption else { return }
        wasPlayingBeforeInterruption = false
        if shouldResume {
            startPlayback()
            if isPlaying {
                setStatus("Playing \(selectedSide.rawValue.lowercased()) track.")
            }
        }
    }

    /// The current output route went away (e.g. headphones unplugged). Apple's
    /// guidance is to pause rather than suddenly play out the speaker.
    func handleAudioRouteLost() {
        if isPlaying {
            stopPlayback()
            setStatus("Output changed. Press play to continue.")
        }
    }

    private func observeAudioSession() {
        let center = NotificationCenter.default
        let session = AVAudioSession.sharedInstance()

        notificationObservers.append(center.addObserver(
            forName: AVAudioSession.interruptionNotification, object: session, queue: .main
        ) { [weak self] note in
            guard let info = note.userInfo,
                  let raw = info[AVAudioSessionInterruptionTypeKey] as? UInt,
                  let type = AVAudioSession.InterruptionType(rawValue: raw) else { return }
            let shouldResume: Bool
            if type == .ended, let optionRaw = info[AVAudioSessionInterruptionOptionKey] as? UInt {
                shouldResume = AVAudioSession.InterruptionOptions(rawValue: optionRaw).contains(.shouldResume)
            } else {
                shouldResume = false
            }
            Task { @MainActor in
                guard let self else { return }
                switch type {
                case .began: self.handleInterruptionBegan()
                case .ended: self.handleInterruptionEnded(shouldResume: shouldResume)
                @unknown default: break
                }
            }
        })

        notificationObservers.append(center.addObserver(
            forName: AVAudioSession.routeChangeNotification, object: session, queue: .main
        ) { [weak self] note in
            guard let info = note.userInfo,
                  let raw = info[AVAudioSessionRouteChangeReasonKey] as? UInt,
                  let reason = AVAudioSession.RouteChangeReason(rawValue: raw),
                  reason == .oldDeviceUnavailable else { return }
            Task { @MainActor in self?.handleAudioRouteLost() }
        })
    }

    /// Switch Original/Mastered on the single live timeline. The cursor never
    /// moves, so the playhead is preserved exactly — no pause/resume/re-render.
    func selectSide(_ side: Side) {
        guard side != selectedSide else { return }
        selectedSide = side
        engine.setOriginal(side == .original)
        applyVolumeMatch()
        setStatus("Switched to \(side.rawValue.lowercased()) at the same spot.")
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
        // The chain updates live immediately; the loudness landing re-measures
        // shortly after the change settles (it's heavier — a window through the
        // chain + LUFS), so Low/Med/High and intensity re-land the level.
        scheduleLandingRefresh()
    }

    /// Measure the loudness landing for the current settings and apply it: sets
    /// the landing gain (so Loudness is audible) and the mastered LUFS that
    /// Volume Match needs. Routes through the same landing math as Create Master,
    /// so the live preview lands ≈ where the full render will.
    func refreshLanding() async {
        await refreshLanding(generation: landingGeneration)
    }

    private func refreshLanding(generation: UInt64) async {
        guard let stream = engine.stream else { return }
        let preset = selectedPreset.bridgeIdentifier
        let intensity = Float(presetIntensity)
        let loudness = selectedLoudness.lufsTarget
        let result = await Task.detached {
            stream.measureLanding(preset: preset, intensity: intensity, loudnessTarget: loudness)
        }.value
        guard generation == landingGeneration, !Task.isCancelled else { return }
        engine.setLandingGain(linearGain: result.gain)
        if result.masteredLufs.isFinite {
            masteredLufs = result.masteredLufs
        }
        applyVolumeMatch()
    }

    private func scheduleLandingRefresh() {
        landingTask?.cancel()
        let generation = bumpLandingGeneration()
        landingTask = Task { [weak self] in
            try? await Task.sleep(nanoseconds: 250_000_000) // settle after the last change
            if Task.isCancelled { return }
            await self?.refreshLanding(generation: generation)
        }
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
            setError(.masterUnavailable)
            return
        }
        renderTask?.cancel()
        isRendering = true
        shareMasterURL = nil
        setStatus("Creating master...")

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
                    setError(.renderNoOutput)
                    return
                }
                shareMasterURL = URL(fileURLWithPath: outputPath)
                masteredLufs = job.measurements?.lufsIntegrated
                renderStorage.enforceLimit(in: renderStorage.mastersDirectory, max: 20)
                applyVolumeMatch()
                setStatus("Master created. Share it anytime.")
            case .failure(let error):
                shareMasterURL = nil
                setError(audioErrorState(error, fallback: .renderFailed))
            }
        }
    }

    private func audioErrorState(
        _ error: Error,
        fallback: (String) -> AuditionErrorState
    ) -> AuditionErrorState {
        let message = error.localizedDescription
        if message.localizedCaseInsensitiveContains("no suitable format reader")
            || message.localizedCaseInsensitiveContains("decode error") {
            return .decodeFailed
        }
        return fallback(message)
    }

}
