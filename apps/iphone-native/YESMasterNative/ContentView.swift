import SwiftUI

enum ListeningMode: String, CaseIterable, Identifiable {
    case normal = "Normal"
    case volumeMatch = "Volume Match"
    case lufsPreview = "LUFS Preview"

    var id: String { rawValue }
}

enum NativeStylePreset: String, CaseIterable, Identifiable {
    case balanced = "Balanced"
    case warm = "Warm"
    case open = "Open"
    case punch = "Punch"

    var id: String { rawValue }
}

enum NativeLoudness: String, CaseIterable, Identifiable {
    case low = "Low"
    case medium = "Medium"
    case high = "High"

    var id: String { rawValue }
}

struct ContentView: View {
    @State private var selectedPreset: NativeStylePreset = .balanced
    @State private var selectedLoudness: NativeLoudness = .medium
    @State private var listeningMode: ListeningMode = .normal
    @State private var importedTrack: ImportedTrack?
    @State private var analysisResult: NativeAnalysisResult?
    @State private var isImportingTrack = false
    @State private var isAnalyzing = false
    @State private var analysisTask: Task<Void, Never>?
    @State private var statusText = "Import a track to begin."
    @StateObject private var playbackController = TrackPlaybackController()

    private let bridge = NativeMasteringBridge()
    private let importStore = ImportedTrackStore()

    var body: some View {
        NavigationStack {
            VStack(alignment: .leading, spacing: 24) {
                VStack(alignment: .leading, spacing: 8) {
                    Text("YES Master")
                        .font(.largeTitle.bold())
                    Text(importedTrack?.displayName ?? "No track loaded")
                        .font(.headline)
                        .foregroundStyle(.secondary)
                }

                Button {
                    isImportingTrack = true
                } label: {
                    Label("Import Track", systemImage: "square.and.arrow.down")
                        .frame(maxWidth: .infinity, minHeight: 48)
                }
                .buttonStyle(.borderedProminent)

                Picker("Listening", selection: $listeningMode) {
                    ForEach(ListeningMode.allCases) { mode in
                        Text(mode.rawValue).tag(mode)
                    }
                }
                .pickerStyle(.segmented)

                Picker("Style", selection: $selectedPreset) {
                    ForEach(NativeStylePreset.allCases) { preset in
                        Text(preset.rawValue).tag(preset)
                    }
                }
                .pickerStyle(.segmented)

                Picker("Loudness", selection: $selectedLoudness) {
                    ForEach(NativeLoudness.allCases) { loudness in
                        Text(loudness.rawValue).tag(loudness)
                    }
                }
                .pickerStyle(.segmented)

                HStack(spacing: 12) {
                    Button {
                        toggleOriginalPlayback()
                    } label: {
                        Label(
                            playbackController.isPlaying ? "Pause" : "Play Original",
                            systemImage: playbackController.isPlaying ? "pause.fill" : "play.fill"
                        )
                            .frame(maxWidth: .infinity, minHeight: 48)
                    }
                    .buttonStyle(.bordered)
                    .disabled(!canPlayOriginal)

                    Button {
                        statusText = bridge.fixedExportSummary
                    } label: {
                        Label("Create Master", systemImage: "waveform")
                            .frame(maxWidth: .infinity, minHeight: 48)
                    }
                    .buttonStyle(.borderedProminent)
                    .disabled(true)
                }

                Text(statusText)
                    .font(.callout)
                    .foregroundStyle(.secondary)

                if isAnalyzing {
                    ProgressView("Analyzing track")
                        .font(.callout)
                } else if let analysisResult {
                    analysisSummary(for: analysisResult)
                }

                Spacer()
            }
            .padding(20)
            .navigationTitle("Track")
            .navigationBarTitleDisplayMode(.inline)
            .fileImporter(
                isPresented: $isImportingTrack,
                allowedContentTypes: bridge.supportedImportContentTypes,
                allowsMultipleSelection: false,
                onCompletion: handleImportResult
            )
        }
    }

    private func handleImportResult(_ result: Result<[URL], Error>) {
        switch result {
        case .success(let urls):
            guard let sourceURL = urls.first else {
                statusText = "No track was selected."
                return
            }

            do {
                let track = try importStore.importTrack(
                    from: sourceURL,
                    supportedExtensions: bridge.supportedImportExtensions
                )
                importedTrack = track
                playbackController.pause()
                analyzeImportedTrack(track)
            } catch ImportedTrackStore.ImportError.unsupportedExtension(let fileExtension) {
                let label = fileExtension.isEmpty ? "that file" : ".\(fileExtension)"
                statusText = "\(label) is not supported yet. Use \(bridge.supportedImportExtensions.joined(separator: ", "))."
            } catch {
                statusText = "Track could not be imported. Try another supported audio file."
            }
        case .failure:
            statusText = "Import was cancelled."
        }
    }

    private var canPlayOriginal: Bool {
        importedTrack != nil && analysisResult != nil && !isAnalyzing
    }

    private func toggleOriginalPlayback() {
        guard let track = importedTrack, canPlayOriginal else {
            statusText = "Import and analyze a track before playback."
            return
        }

        if playbackController.isPlaying {
            playbackController.pause()
            statusText = "Playback paused."
            return
        }

        do {
            try playbackController.play(url: track.localURL)
            statusText = "Playing original track."
        } catch {
            statusText = "Playback could not start. Try another supported audio file."
        }
    }

    private func analyzeImportedTrack(_ track: ImportedTrack) {
        analysisTask?.cancel()
        analysisResult = nil
        isAnalyzing = true
        statusText = "Analyzing with the YES Master engine."

        let bridge = bridge
        analysisTask = Task {
            let result = await Task.detached {
                Result {
                    try bridge.analyzeTrack(at: track.localURL)
                }
            }.value

            guard !Task.isCancelled else { return }
            isAnalyzing = false

            switch result {
            case .success(let analysis):
                analysisResult = analysis
                statusText = "Analysis complete. Playback and export come next."
            case .failure(let error):
                analysisResult = nil
                statusText = "Analysis failed: \(error.localizedDescription)"
            }
        }
    }

    private func analysisSummary(for analysis: NativeAnalysisResult) -> some View {
        VStack(alignment: .leading, spacing: 10) {
            Text("Analysis")
                .font(.headline)
            HStack(spacing: 14) {
                analysisMetric(label: "LUFS", value: String(format: "%.1f", analysis.lufsIntegrated))
                analysisMetric(label: "True Peak", value: String(format: "%.1f dBTP", analysis.truePeakDbtp))
                analysisMetric(label: "DR", value: String(format: "%.1f LU", analysis.dynamicRangeLu))
            }
        }
    }

    private func analysisMetric(label: String, value: String) -> some View {
        VStack(alignment: .leading, spacing: 4) {
            Text(label)
                .font(.caption)
                .foregroundStyle(.secondary)
            Text(value)
                .font(.headline.monospacedDigit())
        }
        .frame(maxWidth: .infinity, alignment: .leading)
    }
}

#Preview {
    ContentView()
}
