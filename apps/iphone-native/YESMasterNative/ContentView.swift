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
    @State private var selectedFileName: String?
    @State private var statusText = "Import a track to begin."

    private let audioSession = AudioSessionController()
    private let bridge = NativeMasteringBridge()

    var body: some View {
        NavigationStack {
            VStack(alignment: .leading, spacing: 24) {
                VStack(alignment: .leading, spacing: 8) {
                    Text("YES Master")
                        .font(.largeTitle.bold())
                    Text(selectedFileName ?? "No track loaded")
                        .font(.headline)
                        .foregroundStyle(.secondary)
                }

                Button {
                    selectedFileName = "Native import not wired yet"
                    statusText = bridge.supportedImportSummary
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

                    Button {
                        statusText = bridge.fixedExportSummary
                    } label: {
                        Label("Create Master", systemImage: "waveform")
                            .frame(maxWidth: .infinity, minHeight: 48)
                    }
                    .buttonStyle(.borderedProminent)
                }

                Text(statusText)
                    .font(.callout)
                    .foregroundStyle(.secondary)

                Spacer()
            }
            .padding(20)
            .navigationTitle("Track")
            .navigationBarTitleDisplayMode(.inline)
        }
    }
}

#Preview {
    ContentView()
}
