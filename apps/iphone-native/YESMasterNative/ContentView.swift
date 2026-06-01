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

    var subtitle: String {
        switch self {
        case .balanced:
            "A clean, streaming-ready shape for most mixes."
        case .warm:
            "Fuller body and smoother top-end finish."
        case .open:
            "Air, vocal clarity, and a wider front edge."
        case .punch:
            "Sharper transient impact and forward energy."
        }
    }

    var symbol: String {
        switch self {
        case .balanced:
            "circle.hexagongrid.fill"
        case .warm:
            "flame.fill"
        case .open:
            "snowflake"
        case .punch:
            "bolt.fill"
        }
    }

    var accent: Color {
        switch self {
        case .balanced:
            Color(red: 0.25, green: 0.57, blue: 1.0)
        case .warm:
            Color(red: 1.0, green: 0.49, blue: 0.22)
        case .open:
            Color(red: 0.27, green: 0.88, blue: 0.96)
        case .punch:
            Color(red: 1.0, green: 0.24, blue: 0.24)
        }
    }
}

enum NativeLoudness: String, CaseIterable, Identifiable {
    case low = "Low"
    case medium = "Medium"
    case high = "High"

    var id: String { rawValue }
}

private enum AuditionSide: String, CaseIterable, Identifiable {
    case original = "Original"
    case mastered = "Mastered"

    var id: String { rawValue }
}

struct ContentView: View {
    @State private var selectedPreset: NativeStylePreset = .balanced
    @State private var selectedLoudness: NativeLoudness = .medium
    @State private var selectedAudition: AuditionSide = .original
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
        ZStack {
            appBackground

            ScrollView(showsIndicators: false) {
                VStack(spacing: 8) {
                    header
                    heroPanel
                    trackCard
                    auditionSwitch
                    stylePicker
                    loudnessPicker
                    createMasterButton
                    statusAndAnalysis
                }
                .padding(.horizontal, 16)
                .padding(.top, 8)
                .padding(.bottom, 12)
                .frame(maxWidth: 430)
                .frame(maxWidth: .infinity)
            }
        }
        .preferredColorScheme(.dark)
        .statusBarHidden(true)
        .fileImporter(
            isPresented: $isImportingTrack,
            allowedContentTypes: bridge.supportedImportContentTypes,
            allowsMultipleSelection: false,
            onCompletion: handleImportResult
        )
    }

    private var appBackground: some View {
        LinearGradient(
            colors: [
                Color(red: 0.015, green: 0.027, blue: 0.06),
                Color(red: 0.02, green: 0.04, blue: 0.09),
                Color(red: 0.005, green: 0.01, blue: 0.025)
            ],
            startPoint: .top,
            endPoint: .bottom
        )
        .overlay(
            RadialGradient(
                colors: [
                    Color.blue.opacity(0.28),
                    Color.clear
                ],
                center: .topTrailing,
                startRadius: 20,
                endRadius: 280
            )
        )
        .ignoresSafeArea()
    }

    private var header: some View {
        HStack {
            HStack(spacing: 10) {
                Image(systemName: "waveform")
                    .font(.system(size: 20, weight: .bold))
                    .foregroundStyle(Color(red: 0.29, green: 0.65, blue: 1.0))
                    .shadow(color: .blue.opacity(0.6), radius: 10)

                Text("YES MASTER")
                    .font(.system(size: 16, weight: .heavy))
                    .tracking(1.4)
            }

            Spacer()

            Text(headerStatus)
                .font(.system(size: 13, weight: .heavy))
                .foregroundStyle(Color(red: 0.78, green: 0.87, blue: 1.0))
                .padding(.horizontal, 16)
                .frame(height: 34)
                .background(.white.opacity(0.04))
                .clipShape(Capsule())
                .overlay(
                    Capsule()
                        .stroke(Color(red: 0.35, green: 0.58, blue: 1.0).opacity(0.35), lineWidth: 1)
                )
        }
    }

    private var heroPanel: some View {
        ZStack {
            RoundedRectangle(cornerRadius: 18, style: .continuous)
                .fill(
                    LinearGradient(
                        colors: [
                            Color(red: 0.035, green: 0.07, blue: 0.15).opacity(0.96),
                            Color(red: 0.015, green: 0.025, blue: 0.055)
                        ],
                        startPoint: .top,
                        endPoint: .bottom
                    )
                )
                .overlay(
                    RoundedRectangle(cornerRadius: 18, style: .continuous)
                        .stroke(Color(red: 0.34, green: 0.52, blue: 0.88).opacity(0.28), lineWidth: 1)
                )

            Image(systemName: "waveform.path.ecg")
                .font(.system(size: 210, weight: .thin))
                .foregroundStyle(Color.blue.opacity(0.08))
                .rotationEffect(.degrees(-12))
                .offset(x: 114, y: -70)

            VStack(spacing: 0) {
                Spacer(minLength: 12)
                playVisual
                Spacer(minLength: 6)
                listeningOptions
                    .padding(.bottom, 10)
            }
            .padding(.horizontal, 14)
        }
        .frame(height: 250)
    }

    private var playVisual: some View {
        ZStack {
            ForEach(0..<4, id: \.self) { index in
                Circle()
                    .stroke(
                        Color(red: 0.25, green: 0.55, blue: 1.0).opacity(0.42 - Double(index) * 0.075),
                        lineWidth: index == 0 ? 1.4 : 1
                    )
                    .frame(width: CGFloat(68 + index * 36), height: CGFloat(68 + index * 36))
            }

            RoundedRectangle(cornerRadius: 0)
                .fill(
                    LinearGradient(
                        colors: [
                            Color.clear,
                            Color(red: 0.27, green: 0.67, blue: 1.0).opacity(0.55),
                            Color(red: 0.78, green: 0.96, blue: 1.0).opacity(0.52),
                            Color(red: 0.22, green: 0.52, blue: 1.0).opacity(0.42),
                            Color.clear
                        ],
                        startPoint: .leading,
                        endPoint: .trailing
                    )
                )
                .frame(height: 50)
                .blur(radius: 0.4)

            Button {
                if importedTrack == nil {
                    isImportingTrack = true
                } else {
                    toggleOriginalPlayback()
                }
            } label: {
                ZStack {
                    Circle()
                        .fill(
                            RadialGradient(
                                colors: [
                                    Color.white.opacity(0.95),
                                    Color(red: 0.47, green: 0.76, blue: 1.0),
                                    Color(red: 0.10, green: 0.34, blue: 0.96)
                                ],
                                center: .topLeading,
                                startRadius: 2,
                                endRadius: 86
                            )
                        )
                        .overlay(
                            Circle()
                                .stroke(Color(red: 0.65, green: 0.84, blue: 1.0).opacity(0.7), lineWidth: 1)
                        )
                        .shadow(color: Color(red: 0.18, green: 0.48, blue: 1.0).opacity(0.85), radius: 28)

                    Image(systemName: heroSymbol)
                        .font(.system(size: 33, weight: .black))
                        .foregroundStyle(.white)
                        .shadow(color: .white.opacity(0.6), radius: 10)
                }
                .frame(width: 72, height: 72)
            }
            .buttonStyle(.plain)
            .disabled(importedTrack != nil && !canPlayOriginal)
        }
        .frame(maxWidth: .infinity)
        .frame(height: 156)
    }

    private var listeningOptions: some View {
        HStack(spacing: 18) {
            checkboxButton(
                title: "Volume Match",
                active: listeningMode == .volumeMatch,
                mode: .volumeMatch
            )

            checkboxButton(
                title: "LUFS Preview",
                active: listeningMode == .lufsPreview,
                mode: .lufsPreview
            )
        }
    }

    private func checkboxButton(title: String, active: Bool, mode: ListeningMode) -> some View {
        Button {
            listeningMode = active ? .normal : mode
        } label: {
            HStack(spacing: 8) {
                RoundedRectangle(cornerRadius: 4, style: .continuous)
                    .fill(active ? Color(red: 0.22, green: 0.55, blue: 1.0) : Color.clear)
                    .frame(width: 16, height: 16)
                    .overlay(
                        RoundedRectangle(cornerRadius: 4, style: .continuous)
                            .stroke(active ? Color.clear : Color.white.opacity(0.42), lineWidth: 1.4)
                    )
                    .overlay {
                        if active {
                            Image(systemName: "checkmark")
                                .font(.system(size: 11, weight: .black))
                                .foregroundStyle(Color(red: 0.03, green: 0.06, blue: 0.10))
                        }
                    }

                Text(title)
                    .font(.system(size: 12, weight: .bold))
                    .foregroundStyle(active ? .white : Color(red: 0.74, green: 0.79, blue: 0.88))
            }
        }
        .buttonStyle(.plain)
    }

    private var trackCard: some View {
        HStack(spacing: 10) {
            VStack(alignment: .leading, spacing: 5) {
                Text(trackCardStatus)
                    .font(.system(size: 11, weight: .heavy))
                    .tracking(2.4)
                    .foregroundStyle(Color(red: 0.55, green: 0.64, blue: 0.82))

                Text(importedTrack?.displayName ?? "No track loaded")
                    .font(.system(size: 17, weight: .heavy))
                    .foregroundStyle(.white)
                    .lineLimit(1)

                Text(importedTrack == nil ? "WAV  MP3  M4A  AAC  FLAC  OGG" : fileTypeLabel)
                    .font(.system(size: 11, weight: .heavy))
                    .foregroundStyle(Color(red: 0.76, green: 0.84, blue: 1.0))
                    .padding(.horizontal, 10)
                    .frame(height: 20)
                    .background(Color(red: 0.03, green: 0.06, blue: 0.12).opacity(0.7))
                    .clipShape(Capsule())
                    .overlay(
                        Capsule()
                            .stroke(Color.white.opacity(0.08), lineWidth: 1)
                    )
            }

            Spacer(minLength: 8)

            Button {
                isImportingTrack = true
            } label: {
                Text(importedTrack == nil ? "Import" : "Change")
                    .font(.system(size: 14, weight: .heavy))
                    .foregroundStyle(Color(red: 0.86, green: 0.90, blue: 1.0))
                    .padding(.horizontal, 17)
                    .frame(height: 36)
                    .background(.white.opacity(0.06))
                    .clipShape(Capsule())
                    .overlay(
                        Capsule()
                            .stroke(Color.white.opacity(0.10), lineWidth: 1)
                    )
            }
            .buttonStyle(.plain)
        }
        .padding(12)
        .background(
            LinearGradient(
                colors: [
                    Color(red: 0.07, green: 0.12, blue: 0.23).opacity(0.92),
                    Color(red: 0.025, green: 0.045, blue: 0.09).opacity(0.96)
                ],
                startPoint: .top,
                endPoint: .bottom
            )
        )
        .clipShape(RoundedRectangle(cornerRadius: 16, style: .continuous))
        .overlay(
            RoundedRectangle(cornerRadius: 16, style: .continuous)
                .stroke(Color(red: 0.38, green: 0.52, blue: 0.84).opacity(0.18), lineWidth: 1)
        )
    }

    private var auditionSwitch: some View {
        HStack(spacing: 4) {
            ForEach(AuditionSide.allCases) { side in
                Button {
                    if side == .original {
                        selectedAudition = side
                    } else {
                        statusText = "Mastered preview is next after render wiring."
                    }
                } label: {
                    Text(side.rawValue)
                        .font(.system(size: 15, weight: .heavy))
                        .foregroundStyle(selectedAudition == side ? .white : Color(red: 0.66, green: 0.70, blue: 0.82))
                        .frame(maxWidth: .infinity)
                        .frame(height: 42)
                        .background(
                            Group {
                                if selectedAudition == side {
                                    LinearGradient(
                                        colors: [
                                            Color(red: 0.25, green: 0.56, blue: 1.0),
                                            Color(red: 0.08, green: 0.33, blue: 0.93)
                                        ],
                                        startPoint: .top,
                                        endPoint: .bottom
                                    )
                                } else {
                                    Color.clear
                                }
                            }
                        )
                        .clipShape(Capsule())
                }
                .buttonStyle(.plain)
            }
        }
        .padding(3)
        .background(Color(red: 0.01, green: 0.02, blue: 0.045).opacity(0.88))
        .clipShape(Capsule())
        .overlay(
            Capsule()
                .stroke(Color.white.opacity(0.10), lineWidth: 1)
        )
    }

    private var stylePicker: some View {
        VStack(alignment: .leading, spacing: 7) {
            sectionTitle(step: "1", title: "Style", meta: nil)

            LazyVGrid(columns: [GridItem(.flexible()), GridItem(.flexible())], spacing: 7) {
                ForEach(NativeStylePreset.allCases) { preset in
                    Button {
                        selectedPreset = preset
                    } label: {
                        HStack(spacing: 10) {
                            ZStack {
                                Circle()
                                    .fill(preset.accent.opacity(0.20))
                                    .frame(width: 32, height: 32)
                                    .shadow(color: preset.accent.opacity(0.55), radius: 12)

                                Image(systemName: preset.symbol)
                                    .font(.system(size: 17, weight: .bold))
                                    .foregroundStyle(preset.accent)
                            }

                            VStack(alignment: .leading, spacing: 3) {
                                Text(preset.rawValue)
                                    .font(.system(size: 13, weight: .heavy))
                                    .foregroundStyle(.white)
                                    .lineLimit(1)

                                Text(preset.subtitle)
                                    .font(.system(size: 10, weight: .bold))
                                    .foregroundStyle(Color(red: 0.60, green: 0.67, blue: 0.80))
                                    .lineLimit(2)
                                    .multilineTextAlignment(.leading)
                            }
                        }
                        .frame(maxWidth: .infinity, alignment: .leading)
                        .frame(height: 58)
                        .padding(.horizontal, 9)
                        .background(
                            LinearGradient(
                                colors: [
                                    preset.accent.opacity(selectedPreset == preset ? 0.24 : 0.10),
                                    Color(red: 0.035, green: 0.065, blue: 0.13).opacity(0.92)
                                ],
                                startPoint: .topLeading,
                                endPoint: .bottomTrailing
                            )
                        )
                        .clipShape(RoundedRectangle(cornerRadius: 12, style: .continuous))
                        .overlay(
                            RoundedRectangle(cornerRadius: 12, style: .continuous)
                                .stroke(
                                    selectedPreset == preset ? preset.accent.opacity(0.75) : Color.white.opacity(0.09),
                                    lineWidth: 1
                                )
                        )
                    }
                    .buttonStyle(.plain)
                }
            }
        }
    }

    private var loudnessPicker: some View {
        VStack(alignment: .leading, spacing: 7) {
            sectionTitle(step: "2", title: "Loudness", meta: "-11.0 LUFS · 44.1 KHZ WAV / 24-BIT")

            HStack(spacing: 4) {
                ForEach(NativeLoudness.allCases) { loudness in
                    Button {
                        selectedLoudness = loudness
                    } label: {
                        Text(loudness.rawValue)
                            .font(.system(size: 15, weight: .heavy))
                            .foregroundStyle(selectedLoudness == loudness ? .white : Color(red: 0.78, green: 0.82, blue: 0.92))
                            .frame(maxWidth: .infinity)
                            .frame(height: 42)
                            .background(
                                Group {
                                    if selectedLoudness == loudness {
                                        LinearGradient(
                                            colors: [
                                                Color(red: 0.25, green: 0.56, blue: 1.0),
                                                Color(red: 0.08, green: 0.33, blue: 0.93)
                                            ],
                                            startPoint: .top,
                                            endPoint: .bottom
                                        )
                                    } else {
                                        Color.clear
                                    }
                                }
                            )
                            .clipShape(Capsule())
                    }
                    .buttonStyle(.plain)
                }
            }
            .padding(3)
            .background(Color(red: 0.01, green: 0.02, blue: 0.045).opacity(0.88))
            .clipShape(Capsule())
            .overlay(
                Capsule()
                    .stroke(Color.white.opacity(0.10), lineWidth: 1)
            )
        }
    }

    private var createMasterButton: some View {
        Button {
            if analysisResult == nil {
                statusText = "Import and analyze a track before creating a master."
            } else {
                statusText = "Render bridge is ready. Button wiring is the next slice."
            }
        } label: {
            Text("Create Master")
                .font(.system(size: 18, weight: .heavy))
                .foregroundStyle(.white)
                .frame(maxWidth: .infinity)
                .frame(height: 52)
                .background(
                    LinearGradient(
                        colors: [
                            Color(red: 0.31, green: 0.67, blue: 1.0),
                            Color(red: 0.06, green: 0.35, blue: 0.98)
                        ],
                        startPoint: .top,
                        endPoint: .bottom
                    )
                )
                .clipShape(RoundedRectangle(cornerRadius: 12, style: .continuous))
                .shadow(color: Color.blue.opacity(0.34), radius: 26, y: 16)
        }
        .buttonStyle(.plain)
    }

    private var statusAndAnalysis: some View {
        VStack(spacing: 10) {
            if isAnalyzing {
                HStack(spacing: 10) {
                    ProgressView()
                        .tint(Color(red: 0.32, green: 0.65, blue: 1.0))
                    Text("Analyzing track")
                }
                .font(.system(size: 13, weight: .bold))
                .foregroundStyle(Color(red: 0.68, green: 0.76, blue: 0.92))
            } else if let analysisResult {
                analysisSummary(for: analysisResult)
            }

            Text(statusText)
                .font(.system(size: 12, weight: .bold))
                .foregroundStyle(Color(red: 0.50, green: 0.58, blue: 0.74))
                .multilineTextAlignment(.center)
                .lineLimit(3)
        }
    }

    private func sectionTitle(step: String, title: String, meta: String?) -> some View {
        HStack(alignment: .firstTextBaseline, spacing: 8) {
            Text(step)
                .font(.system(size: 13, weight: .heavy))
                .foregroundStyle(Color(red: 0.27, green: 0.62, blue: 1.0))
                .shadow(color: .blue.opacity(0.7), radius: 8)

            Text(title.uppercased())
                .font(.system(size: 14, weight: .heavy))
                .tracking(3)
                .foregroundStyle(.white)

            if let meta {
                Text(meta)
                    .font(.system(size: 12, weight: .heavy))
                    .tracking(1.2)
                    .foregroundStyle(Color(red: 0.56, green: 0.62, blue: 0.76))
                    .lineLimit(1)
                    .minimumScaleFactor(0.7)
            }
        }
    }

    private func analysisSummary(for analysis: NativeAnalysisResult) -> some View {
        HStack(spacing: 8) {
            analysisMetric(label: "LUFS", value: String(format: "%.1f", analysis.lufsIntegrated))
            analysisMetric(label: "TP", value: String(format: "%.1f", analysis.truePeakDbtp))
            analysisMetric(label: "DR", value: String(format: "%.1f", analysis.dynamicRangeLu))
        }
    }

    private func analysisMetric(label: String, value: String) -> some View {
        VStack(spacing: 3) {
            Text(label)
                .font(.system(size: 10, weight: .heavy))
                .foregroundStyle(Color(red: 0.55, green: 0.63, blue: 0.80))
            Text(value)
                .font(.system(size: 14, weight: .heavy, design: .rounded))
                .monospacedDigit()
                .foregroundStyle(.white)
        }
        .frame(maxWidth: .infinity)
        .frame(height: 50)
        .background(Color.white.opacity(0.045))
        .clipShape(RoundedRectangle(cornerRadius: 10, style: .continuous))
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
                selectedAudition = .original
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

    private var headerStatus: String {
        if isAnalyzing {
            return "Analyzing"
        }
        if analysisResult != nil {
            return "Ready"
        }
        return importedTrack == nil ? "Ready" : "Loaded"
    }

    private var heroSymbol: String {
        if importedTrack == nil {
            return "square.and.arrow.down"
        }
        return playbackController.isPlaying ? "pause.fill" : "play.fill"
    }

    private var trackCardStatus: String {
        if isAnalyzing {
            return "ANALYZING"
        }
        if analysisResult != nil {
            return "READY"
        }
        return importedTrack == nil ? "IMPORT" : "LOADED"
    }

    private var fileTypeLabel: String {
        importedTrack?.localURL.pathExtension.uppercased().isEmpty == false
            ? importedTrack?.localURL.pathExtension.uppercased() ?? "AUDIO"
            : "AUDIO"
    }

    private var canPlayOriginal: Bool {
        importedTrack != nil && analysisResult != nil && !isAnalyzing && selectedAudition == .original
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
                statusText = "Analysis complete."
            case .failure(let error):
                analysisResult = nil
                statusText = "Analysis failed: \(error.localizedDescription)"
            }
        }
    }
}

#Preview {
    ContentView()
}
