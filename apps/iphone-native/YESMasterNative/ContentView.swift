import SwiftUI

enum NativeStylePreset: String, CaseIterable, Identifiable {
    case balanced = "Universal"
    case bright = "Clarity"
    case warm = "Tape"
    case heavy = "Oomph"

    var id: String { rawValue }

    var symbol: String {
        switch self {
        case .balanced: "circle.hexagongrid.fill"
        case .bright: "snowflake"
        case .warm: "flame.fill"
        case .heavy: "bolt.fill"
        }
    }

    var accent: Color {
        switch self {
        case .balanced: Color(red: 0.25, green: 0.57, blue: 1.0)   // blue
        case .bright: Color(red: 0.27, green: 0.88, blue: 0.96)    // cyan
        case .warm: Color(red: 1.0, green: 0.72, blue: 0.30)       // amber
        case .heavy: Color(red: 0.97, green: 0.44, blue: 0.44)     // Oomph red
        }
    }

    var bridgeIdentifier: String {
        switch self {
        case .balanced: "balanced"
        case .bright: "bright"
        case .warm: "warm"
        case .heavy: "heavy"
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

struct ContentView: View {
    @Environment(\.accessibilityReduceMotion) private var reduceMotion
    @StateObject private var controller = AuditionController()
    @State private var isImportingTrack = false
    @State private var heroPulse = false

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
            allowedContentTypes: controller.supportedImportContentTypes,
            allowsMultipleSelection: false,
            onCompletion: controller.handleImportResult
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
                if controller.importedTrack == nil {
                    isImportingTrack = true
                } else {
                    controller.togglePlayback()
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
                .scaleEffect(controller.isPlaying && !reduceMotion ? 1.04 : 1.0)
                .animation(.easeOut(duration: 0.18), value: controller.isPlaying)
            }
            .buttonStyle(.plain)
            .disabled(controller.importedTrack != nil && !controller.canPlay)
        }
        .frame(maxWidth: .infinity)
        .frame(height: 166)
    }

    @ViewBuilder
    private var processingBanner: some View {
        if controller.isAnalyzing || controller.isRendering {
            VStack(spacing: 8) {
                HStack(spacing: 10) {
                    ProcessingSpinner()

                    Text(controller.statusText)
                        .font(.system(size: 12, weight: .heavy))
                        .foregroundStyle(Color(red: 0.72, green: 0.82, blue: 1.0))
                        .lineLimit(1)
                        .minimumScaleFactor(0.72)

                    Spacer(minLength: 0)
                }

            }
            .padding(.horizontal, 14)
            .padding(.vertical, 9)
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
            checkboxButton(title: "Volume Match", active: controller.volumeMatchEnabled) {
                controller.toggleVolumeMatch()
            }
            Spacer(minLength: 0)
        }
        .padding(.horizontal, 18)
    }

    private func checkboxButton(title: String, active: Bool, action: @escaping () -> Void) -> some View {
        Button(action: action) {
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
                    Image(systemName: controller.importedTrack == nil ? "tray.and.arrow.down.fill" : "waveform")
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

                    if controller.importedTrack != nil {
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
                Image(systemName: controller.importedTrack == nil ? "plus" : "pencil")
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
            ForEach(AuditionController.Side.allCases) { side in
                Button {
                    controller.selectSide(side)
                } label: {
                    Text(side.rawValue)
                        .font(.system(size: 15, weight: .heavy))
                        .foregroundStyle(controller.selectedSide == side ? .white : Color(red: 0.66, green: 0.70, blue: 0.82))
                        .frame(maxWidth: .infinity)
                        .frame(height: 42)
                        .background(
                            Group {
                                if controller.selectedSide == side {
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
                        controller.setStyle(preset)
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

                            Text(preset.rawValue)
                                .font(.system(size: 13, weight: .heavy))
                                .foregroundStyle(.white)
                                .lineLimit(1)
                        }
                        .frame(maxWidth: .infinity, alignment: .leading)
                        .frame(height: 58)
                        .padding(.horizontal, 9)
                        .background(
                            LinearGradient(
                                colors: [
                                    preset.accent.opacity(controller.selectedPreset == preset ? 0.24 : 0.10),
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
                                    controller.selectedPreset == preset ? preset.accent.opacity(0.75) : Color.white.opacity(0.09),
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
                        get: { controller.presetIntensity },
                        set: { controller.setIntensity($0) }
                    ),
                    in: 0...1
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
                        controller.setLoudness(loudness)
                    } label: {
                        Text(loudness.rawValue)
                            .font(.system(size: 15, weight: .heavy))
                            .foregroundStyle(controller.selectedLoudness == loudness ? .white : Color(red: 0.78, green: 0.82, blue: 0.92))
                            .frame(maxWidth: .infinity)
                            .frame(height: 42)
                            .background(
                                Group {
                                    if controller.selectedLoudness == loudness {
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
            controller.createMaster()
        } label: {
            Text(controller.isRendering ? "Creating Master..." : "Create Master")
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
        .disabled(controller.isRendering)
    }

    @ViewBuilder
    private var shareMasterButton: some View {
        if let shareMasterURL = controller.shareMasterURL {
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
        if let errorState = controller.errorState {
            Text(errorState.message)
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

    private var headerStatus: String {
        if controller.isRendering {
            return "Rendering"
        }
        if controller.isAnalyzing {
            return "Analyzing"
        }
        if controller.analysisResult != nil {
            return "Ready"
        }
        return controller.importedTrack == nil ? "Ready" : "Loaded"
    }

    private var heroSymbol: String {
        if controller.importedTrack == nil {
            return "square.and.arrow.down"
        }
        return controller.isPlaying ? "pause.fill" : "play.fill"
    }

    private var fileTypeLabel: String {
        controller.importedTrack?.localURL.pathExtension.uppercased().isEmpty == false
            ? controller.importedTrack?.localURL.pathExtension.uppercased() ?? "AUDIO"
            : "AUDIO"
    }

    private var trackChipTitle: String {
        controller.importedTrack?.displayName ?? "Import track"
    }

    private var intensityLabel: String {
        let percent = Int((controller.presetIntensity * 100).rounded())
        return "\(percent)%"
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
