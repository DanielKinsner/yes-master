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
            "Clean balance"
        case .warm:
            "Warm body"
        case .open:
            "Open air"
        case .punch:
            "Punchy impact"
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

    var bridgeIdentifier: String {
        switch self {
        case .balanced:
            "balanced"
        case .warm:
            "warm"
        case .open:
            "open"
        case .punch:
            "punch"
        }
    }
}

enum NativeLoudness: String, CaseIterable, Identifiable {
    case low = "Low"
    case medium = "Medium"
    case high = "High"

    var id: String { rawValue }

    var lufsTarget: Float {
        switch self {
        case .low: -14
        case .medium: -11
        case .high: -9
        }
    }
}

private enum AuditionSide: String, CaseIterable, Identifiable {
    case original = "Original"
    case mastered = "Mastered"

    var id: String { rawValue }
}

struct ContentView: View {
    @Environment(\.accessibilityReduceMotion) private var reduceMotion
    @State private var selectedPreset: NativeStylePreset = .balanced
    @State private var selectedLoudness: NativeLoudness = .medium
    @State private var selectedAudition: AuditionSide = .original
    @State private var listeningMode: ListeningMode = .normal
    @State private var presetIntensity = 0.5
    @State private var importedTrack: ImportedTrack?
    @State private var analysisResult: NativeAnalysisResult?
    @State private var masteredPreviewURL: URL?
    @State private var shareMasterURL: URL?
    @State private var renderTask: Task<Void, Never>?
    @State private var previewTask: Task<Void, Never>?
    @State private var previewDebounceTask: Task<Void, Never>?
    @State private var isPreparingMasterPreview = false
    @State private var isRendering = false
    @State private var isImportingTrack = false
    @State private var isAnalyzing = false
    @State private var analysisTask: Task<Void, Never>?
    @State private var heroPulse = false
    @State private var statusText = "Import a track to begin."
    @StateObject private var playbackController = TrackPlaybackController()

    private let bridge = NativeMasteringBridge()
    private let importStore = ImportedTrackStore()
    private let renderStorage = RenderStorage()

    var body: some View {
        ZStack {
            appBackground

            ScrollView(showsIndicators: false) {
                VStack(spacing: 10) {
                    header
                    heroPanel
                    processingBanner
                    auditionSwitch
                    stylePicker
                    intensitySlider
                    loudnessPicker
                    createMasterButton
                        .padding(.top, 22)
                    shareMasterButton
                    errorStatus
                }
                .padding(.horizontal, 16)
                .padding(.top, 12)
                .padding(.bottom, 18)
                .frame(maxWidth: 430)
                .frame(maxWidth: .infinity)
            }
        }
        .preferredColorScheme(.dark)
        .fileImporter(
            isPresented: $isImportingTrack,
            allowedContentTypes: bridge.supportedImportContentTypes,
            allowsMultipleSelection: false,
            onCompletion: handleImportResult
        )
        .onAppear {
            guard !reduceMotion else { return }
            withAnimation(.easeInOut(duration: 2.8).repeatForever(autoreverses: true)) {
                heroPulse = true
            }
        }
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
                Image("BrandIcon")
                    .resizable()
                    .scaledToFit()
                    .frame(width: 28, height: 28)
                    .clipShape(RoundedRectangle(cornerRadius: 7, style: .continuous))
                    .shadow(color: Color(red: 0.35, green: 0.88, blue: 0.82).opacity(0.38), radius: 10)

                Text("YES MASTER")
                    .font(.system(size: 16, weight: .heavy))
                    .tracking(1.4)
                    .foregroundStyle(.white)
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

            Image("BrandIcon")
                .resizable()
                .scaledToFit()
                .opacity(0.13)
                .blendMode(.screen)
                .frame(width: 268, height: 268)
                .offset(x: 68, y: 8)
                .allowsHitTesting(false)

            VStack {
                heroTrackRow
                    .padding(.top, 16)
                    .padding(.horizontal, 18)

                Spacer()

                playVisual
                    .padding(.horizontal, 14)
                    .padding(.top, 12)

                Spacer()

                heroListeningToggles
                    .padding(.bottom, 14)
            }
        }
        .frame(height: 292)
        .clipShape(RoundedRectangle(cornerRadius: 18, style: .continuous))
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
                    .scaleEffect(!reduceMotion && heroPulse ? 1.035 : 1.0)
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
                    toggleAuditionPlayback()
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
                .scaleEffect(playbackController.isPlaying && !reduceMotion ? 1.04 : 1.0)
                .animation(.easeOut(duration: 0.18), value: playbackController.isPlaying)
            }
            .buttonStyle(.plain)
            .disabled(importedTrack != nil && !canPlaySelectedAudition)
        }
        .frame(maxWidth: .infinity)
        .frame(height: 166)
    }

    @ViewBuilder
    private var processingBanner: some View {
        if isAnalyzing || isPreparingMasterPreview {
            HStack(spacing: 10) {
                ProcessingSpinner()

                Text(isAnalyzing ? "Analyzing track" : "Preparing preview")
                    .font(.system(size: 12, weight: .heavy))
                    .foregroundStyle(Color(red: 0.72, green: 0.82, blue: 1.0))

                Spacer(minLength: 0)
            }
            .padding(.horizontal, 14)
            .frame(height: 38)
            .background(Color(red: 0.02, green: 0.045, blue: 0.10).opacity(0.76))
            .clipShape(Capsule())
            .overlay(
                Capsule()
                    .stroke(Color(red: 0.35, green: 0.62, blue: 1.0).opacity(0.24), lineWidth: 1)
            )
            .transition(.opacity.combined(with: .scale(scale: 0.98)))
        }
    }

    private var heroListeningToggles: some View {
        HStack(spacing: 22) {
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
        .padding(.horizontal, 18)
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
            .frame(height: 44)
        }
        .buttonStyle(.plain)
    }

    private var heroTrackRow: some View {
        HStack(alignment: .top) {
            Button {
                isImportingTrack = true
            } label: {
                HStack(spacing: 8) {
                    Image(systemName: importedTrack == nil ? "tray.and.arrow.down.fill" : "waveform")
                        .font(.system(size: 12, weight: .heavy))
                        .foregroundStyle(Color(red: 0.78, green: 0.90, blue: 1.0))
                        .frame(width: 22, height: 22)
                        .background(
                            LinearGradient(
                                colors: [
                                    Color(red: 0.41, green: 0.73, blue: 1.0),
                                    Color(red: 0.12, green: 0.34, blue: 0.78)
                                ],
                                startPoint: .topLeading,
                                endPoint: .bottomTrailing
                            )
                        )
                        .clipShape(RoundedRectangle(cornerRadius: 6, style: .continuous))

                    Text(trackChipTitle)
                        .font(.system(size: 13, weight: .heavy))
                        .foregroundStyle(Color(red: 0.86, green: 0.92, blue: 1.0))
                        .lineLimit(1)

                    if importedTrack != nil {
                        Text(fileTypeLabel)
                            .font(.system(size: 10, weight: .heavy))
                            .foregroundStyle(Color(red: 0.47, green: 0.62, blue: 0.86))
                            .lineLimit(1)
                    }
                }
                .padding(.leading, 8)
                .padding(.trailing, 12)
                .frame(height: 36)
                .background(Color(red: 0.02, green: 0.05, blue: 0.12).opacity(0.72))
                .clipShape(Capsule())
                .overlay(
                    Capsule()
                        .stroke(Color(red: 0.35, green: 0.55, blue: 0.95).opacity(0.26), lineWidth: 1)
                )
            }
            .buttonStyle(.plain)

            Spacer(minLength: 8)

            Button {
                isImportingTrack = true
            } label: {
                Image(systemName: importedTrack == nil ? "plus" : "pencil")
                    .font(.system(size: 15, weight: .black))
                    .foregroundStyle(Color(red: 0.86, green: 0.90, blue: 1.0))
                    .frame(width: 36, height: 36)
                    .background(Color(red: 0.02, green: 0.05, blue: 0.12).opacity(0.72))
                    .clipShape(Circle())
                    .overlay(
                        Circle()
                            .stroke(Color(red: 0.35, green: 0.55, blue: 0.95).opacity(0.32), lineWidth: 1)
                    )
            }
            .buttonStyle(.plain)
        }
    }

    private var auditionSwitch: some View {
        HStack(spacing: 4) {
            ForEach(AuditionSide.allCases) { side in
                Button {
                    selectAudition(side)
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
                        scheduleMasteredPreviewRefresh()
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
                                    .lineLimit(1)
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

    private var intensitySlider: some View {
        VStack(alignment: .leading, spacing: 9) {
            sectionTitle(step: "2", title: "Intensity", meta: intensityLabel)

            VStack(spacing: 8) {
                Slider(
                    value: Binding(
                        get: { presetIntensity },
                        set: { presetIntensity = $0 }
                    ),
                    in: 0...1,
                    onEditingChanged: { isEditing in
                        if !isEditing {
                            scheduleMasteredPreviewRefresh()
                        }
                    }
                )
                .tint(Color(red: 0.22, green: 0.56, blue: 1.0))
                .frame(height: 34)

                HStack {
                    Text("Subtle")
                    Spacer()
                    Text("Full")
                    Spacer()
                    Text("Pushed")
                }
                .font(.system(size: 10, weight: .heavy))
                .foregroundStyle(Color(red: 0.56, green: 0.64, blue: 0.80))
            }
            .padding(.horizontal, 16)
            .padding(.vertical, 11)
            .background(Color(red: 0.01, green: 0.02, blue: 0.045).opacity(0.88))
            .clipShape(RoundedRectangle(cornerRadius: 16, style: .continuous))
            .overlay(
                RoundedRectangle(cornerRadius: 16, style: .continuous)
                    .stroke(Color.white.opacity(0.10), lineWidth: 1)
            )
            .frame(maxWidth: 382)
            .frame(maxWidth: .infinity, alignment: .center)
        }
    }

    private var loudnessPicker: some View {
        VStack(alignment: .leading, spacing: 7) {
            sectionTitle(step: "3", title: "Loudness", meta: nil)

            HStack(spacing: 4) {
                ForEach(NativeLoudness.allCases) { loudness in
                    Button {
                        selectedLoudness = loudness
                        scheduleMasteredPreviewRefresh()
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
            renderMaster()
        } label: {
            Text(isRendering ? "Creating Master..." : "Create Master")
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
        .disabled(isRendering)
    }

    @ViewBuilder
    private var shareMasterButton: some View {
        if let shareMasterURL {
            ShareLink(item: shareMasterURL) {
                Text("Share Master")
                    .font(.system(size: 15, weight: .heavy))
                    .foregroundStyle(Color(red: 0.84, green: 0.90, blue: 1.0))
                    .frame(maxWidth: .infinity)
                    .frame(height: 44)
                    .background(Color.white.opacity(0.055))
                    .clipShape(RoundedRectangle(cornerRadius: 12, style: .continuous))
                    .overlay(
                        RoundedRectangle(cornerRadius: 12, style: .continuous)
                            .stroke(Color.white.opacity(0.12), lineWidth: 1)
                    )
            }
        }
    }

    @ViewBuilder
    private var errorStatus: some View {
        if shouldShowStatusText {
            Text(statusText)
                .font(.system(size: 12, weight: .bold))
                .foregroundStyle(Color(red: 0.72, green: 0.79, blue: 0.92))
                .multilineTextAlignment(.center)
                .lineLimit(3)
                .padding(.top, 2)
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
                masteredPreviewURL = nil
                shareMasterURL = nil
                selectedAudition = .original
                playbackController.pause()
                analyzeImportedTrack(track)
            } catch ImportedTrackStore.ImportError.unsupportedExtension(let fileExtension) {
                let label = fileExtension.isEmpty ? "that file" : ".\(fileExtension)"
                statusText = "\(label) is not supported yet. Use \(bridge.supportedImportExtensions.joined(separator: ", "))."
            } catch ImportedTrackStore.ImportError.sourceUnavailable {
                statusText = "That file is not available. Make sure it finished downloading, then try again."
            } catch ImportedTrackStore.ImportError.emptyFile {
                statusText = "That file looks empty. Make sure it finished downloading, then try again."
            } catch ImportedTrackStore.ImportError.unreadableContainer(let fileExtension) {
                statusText = ".\(fileExtension) was selected, but it does not look like readable audio."
            } catch {
                statusText = "Track could not be imported. Try another supported audio file."
            }
        case .failure:
            statusText = "Import was cancelled."
        }
    }

    private var headerStatus: String {
        if isRendering {
            return "Rendering"
        }
        if isPreparingMasterPreview {
            return "Preview"
        }
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

    private var fileTypeLabel: String {
        importedTrack?.localURL.pathExtension.uppercased().isEmpty == false
            ? importedTrack?.localURL.pathExtension.uppercased() ?? "AUDIO"
            : "AUDIO"
    }

    private var trackChipTitle: String {
        importedTrack?.displayName ?? "Import track"
    }

    private var shouldShowStatusText: Bool {
        let lowercasedStatus = statusText.lowercased()
        return lowercasedStatus.contains("failed")
            || lowercasedStatus.contains("could not")
            || lowercasedStatus.contains("not supported")
            || lowercasedStatus.contains("empty")
            || lowercasedStatus.contains("not look like")
            || lowercasedStatus.contains("try ")
            || lowercasedStatus.contains("no track was selected")
            || lowercasedStatus.contains("cancelled")
    }

    private var selectedAuditionURL: URL? {
        switch selectedAudition {
        case .original:
            importedTrack?.localURL
        case .mastered:
            masteredPreviewURL
        }
    }

    private var canPlaySelectedAudition: Bool {
        selectedAuditionURL != nil
            && analysisResult != nil
            && !isAnalyzing
            && !isRendering
            && !isPreparingMasterPreview
    }

    private var currentRenderOptions: NativeRenderOptions {
        NativeRenderOptions(
            preset: selectedPreset.bridgeIdentifier,
            intensity: Float(presetIntensity),
            lufsTarget: selectedLoudness.lufsTarget
        )
    }

    private var intensityLabel: String {
        let percent = Int((presetIntensity * 100).rounded())
        return "\(percent)%"
    }

    private func toggleAuditionPlayback() {
        guard let selectedAuditionURL, canPlaySelectedAudition else {
            if selectedAudition == .mastered {
                statusText = isPreparingMasterPreview
                    ? "Preparing the mastered preview."
                    : "Mastered preview is not ready yet."
            } else {
                statusText = "Import and analyze a track before playback."
            }
            return
        }

        if playbackController.isPlaying {
            playbackController.pause()
            statusText = "Playback paused."
            return
        }

        do {
            try playbackController.play(url: selectedAuditionURL)
            statusText = "Playing \(selectedAudition.rawValue.lowercased()) track."
        } catch {
            statusText = "Playback could not start. Try another supported audio file."
        }
    }

    private func selectAudition(_ side: AuditionSide) {
        guard side != selectedAudition else { return }
        if side == .mastered && masteredPreviewURL == nil {
            let shouldResume = playbackController.isPlaying
            let resumeTime = playbackController.currentTime
            playbackController.pause()
            selectedAudition = .mastered
            statusText = isPreparingMasterPreview
                ? "Preparing the mastered preview."
                : "Preparing mastered preview."
            prepareMasteredPreview(
                selectWhenReady: true,
                resumePlayback: shouldResume,
                resumeTime: resumeTime
            )
            return
        }

        let shouldResume = playbackController.isPlaying
        let resumeTime = playbackController.currentTime
        playbackController.pause()
        selectedAudition = side

        guard shouldResume else { return }
        guard let selectedAuditionURL, canPlaySelectedAudition else {
            statusText = "Selected \(side.rawValue.lowercased())."
            return
        }

        do {
            try playbackController.play(url: selectedAuditionURL, startingAt: resumeTime)
            statusText = "Switched to \(side.rawValue.lowercased()) at the same spot."
        } catch {
            statusText = "Playback could not switch. Try playing again."
        }
    }

    private func analyzeImportedTrack(_ track: ImportedTrack) {
        analysisTask?.cancel()
        renderTask?.cancel()
        previewTask?.cancel()
        analysisResult = nil
        masteredPreviewURL = nil
        shareMasterURL = nil
        isRendering = false
        isPreparingMasterPreview = false
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
                statusText = "Analysis complete. Preparing mastered preview."
                prepareMasteredPreview(selectWhenReady: false)
            case .failure(let error):
                analysisResult = nil
                statusText = friendlyAudioErrorMessage(error)
            }
        }
    }

    private func scheduleMasteredPreviewRefresh() {
        previewDebounceTask?.cancel()
        previewDebounceTask = Task {
            try? await Task.sleep(nanoseconds: 300_000_000) // 300ms debounce
            guard !Task.isCancelled else { return }
            refreshMasteredPreviewForCurrentSettings()
        }
    }

    private func refreshMasteredPreviewForCurrentSettings() {
        guard importedTrack != nil, analysisResult != nil else { return }
        previewTask?.cancel()
        renderTask?.cancel()
        playbackController.pause()
        masteredPreviewURL = nil
        shareMasterURL = nil
        if selectedAudition == .mastered {
            selectedAudition = .original
        }
        statusText = "Settings changed. Preparing mastered preview."
        prepareMasteredPreview(selectWhenReady: false)
    }

    private func prepareMasteredPreview(
        selectWhenReady: Bool,
        resumePlayback: Bool = false,
        resumeTime: TimeInterval = 0
    ) {
        guard let track = importedTrack, analysisResult != nil else { return }

        previewTask?.cancel()
        isPreparingMasterPreview = true
        masteredPreviewURL = nil
        shareMasterURL = nil

        let bridge = bridge
        let sourceURL = track.localURL
        let outputDirectoryURL = renderStorage.previewsDirectory
        let options = currentRenderOptions

        previewTask = Task {
            let result = await Task.detached {
                Result {
                    try bridge.renderMaster(
                        from: sourceURL,
                        toDirectory: outputDirectoryURL,
                        options: options
                    )
                }
            }.value

            guard !Task.isCancelled else { return }
            isPreparingMasterPreview = false

            switch result {
            case .success(let job):
                guard let outputPath = job.outputPaths.first else {
                    statusText = "Mastered preview finished but no WAV was returned."
                    return
                }
                masteredPreviewURL = URL(fileURLWithPath: outputPath)
                if selectWhenReady {
                    selectedAudition = .mastered
                }
                if resumePlayback, let masteredPreviewURL {
                    do {
                        try playbackController.play(url: masteredPreviewURL, startingAt: resumeTime)
                        statusText = "Switched to mastered at the same spot."
                        return
                    } catch {
                        statusText = "Mastered preview is ready. Tap play to listen."
                        return
                    }
                }
                statusText = "Mastered preview ready."
            case .failure(let error):
                masteredPreviewURL = nil
                if selectedAudition == .mastered {
                    selectedAudition = .original
                }
                statusText = friendlyAudioErrorMessage(error)
            }
        }
    }

    private func renderMaster() {
        guard let track = importedTrack, analysisResult != nil else {
            statusText = "Import and analyze a track before creating a master."
            return
        }

        renderTask?.cancel()
        previewTask?.cancel()
        playbackController.pause()
        masteredPreviewURL = nil
        shareMasterURL = nil
        isRendering = true
        isPreparingMasterPreview = false
        statusText = "Creating master with the YES Master engine."

        let bridge = bridge
        let sourceURL = track.localURL
        let outputDirectoryURL = renderStorage.mastersDirectory
        let options = currentRenderOptions

        renderTask = Task {
            let result = await Task.detached {
                Result {
                    try bridge.renderMaster(
                        from: sourceURL,
                        toDirectory: outputDirectoryURL,
                        options: options
                    )
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
                masteredPreviewURL = URL(fileURLWithPath: outputPath)
                shareMasterURL = URL(fileURLWithPath: outputPath)
                selectedAudition = .mastered
                statusText = "Master created. You can audition the mastered version."
            case .failure(let error):
                masteredPreviewURL = nil
                shareMasterURL = nil
                selectedAudition = .original
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

private struct ProcessingSpinner: View {
    @Environment(\.accessibilityReduceMotion) private var reduceMotion
    @State private var isRotating = false

    var body: some View {
        Circle()
            .trim(from: 0.18, to: 0.86)
            .stroke(
                AngularGradient(
                    colors: [
                        Color(red: 0.40, green: 0.76, blue: 1.0),
                        Color(red: 0.93, green: 0.78, blue: 0.35),
                        Color(red: 0.40, green: 0.76, blue: 1.0)
                    ],
                    center: .center
                ),
                style: StrokeStyle(lineWidth: 2.4, lineCap: .round)
            )
            .frame(width: 18, height: 18)
            .rotationEffect(.degrees(isRotating ? 360 : 0))
            .onAppear {
                guard !reduceMotion else { return }
                withAnimation(.linear(duration: 0.9).repeatForever(autoreverses: false)) {
                    isRotating = true
                }
            }
            .accessibilityHidden(true)
    }
}

#Preview {
    ContentView()
}
