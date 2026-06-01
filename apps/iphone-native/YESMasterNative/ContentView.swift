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
    @State private var isImportingTrack = false
    @State private var statusText = "Import a track to begin."

    private let audioSession = AudioSessionController()
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
                        do {
                            try audioSession.activateForPlayback()
                            statusText = "Audio session ready."
                        } catch {
                            statusText = "Audio session could not start."
                        }
                    } label: {
                        Label("Play", systemImage: "play.fill")
                            .frame(maxWidth: .infinity, minHeight: 48)
                    }
                    .buttonStyle(.bordered)
                    .disabled(true)

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
                statusText = "Track imported. Analysis comes next before preview or export."
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
}

#Preview {
    ContentView()
}
