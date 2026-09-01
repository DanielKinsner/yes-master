import Foundation
import UniformTypeIdentifiers
import XCTest
@testable import YES_Master_Native

@MainActor
final class AuditionControllerTests: XCTestCase {
    func testImportLoadsStreamAndNeverRenders() async throws {
        let ctx = try makeLoadedController()
        // Import decodes for playback (stream exists) but renders nothing.
        XCTAssertNotNil(ctx.controller.importedTrack)
        XCTAssertEqual(ctx.renderer.renderCount, 0)
        XCTAssertEqual(ctx.output.startCount, 0, "import must not auto-play")

        await ctx.controller.analysisTask?.value
        XCTAssertNotNil(ctx.controller.analysisResult)
        XCTAssertEqual(ctx.renderer.analyzeCount, 1, "import analyzes exactly once")
        XCTAssertEqual(ctx.renderer.renderCount, 0, "analysis must not render")
    }

    func testAnalysisUsesHonestIndeterminateStatusUntilAnalysisFinishes() async throws {
        let renderer = FakeRenderer()
        renderer.analysisDelay = 0.10
        let ctx = try makeLoadedController(renderer: renderer)

        XCTAssertTrue(ctx.controller.isAnalyzing)
        XCTAssertEqual(ctx.controller.statusText, "Analyzing audio...")

        await ctx.controller.analysisTask?.value
        XCTAssertEqual(ctx.controller.statusText, "Ready. Press play to audition.")
    }

    func testFastAnalysisKeepsTheProcessingMomentVisible() async throws {
        let ctx = try makeLoadedController(minimumAnalysisPresentationSeconds: 0.05)

        try await Task.sleep(nanoseconds: 10_000_000)
        XCTAssertTrue(ctx.controller.isAnalyzing)

        await ctx.controller.analysisTask?.value
        XCTAssertFalse(ctx.controller.isAnalyzing)
        XCTAssertNotNil(ctx.controller.analysisResult)
    }

    func testLiveSettingsForwardToEngineAndNeverRender() throws {
        let ctx = try makeLoadedController()

        ctx.controller.setStyle(.warm)
        ctx.controller.setIntensity(0.9)
        ctx.controller.setLoudness(.high)

        XCTAssertEqual(ctx.stream.lastParams?.preset, "warm")
        XCTAssertEqual(ctx.stream.lastParams?.intensity, 0.9)
        XCTAssertEqual(ctx.stream.lastParams?.loudness, NativeLoudness.high.lufsTarget)
        XCTAssertEqual(ctx.renderer.renderCount, 0, "live setting changes must never render a WAV")
    }

    func testSelectSideFlipsBypassWithoutRendering() throws {
        let ctx = try makeLoadedController()

        ctx.controller.selectSide(.mastered)
        XCTAssertEqual(ctx.stream.lastOriginal, false)
        XCTAssertEqual(ctx.controller.selectedSide, .mastered)

        ctx.controller.selectSide(.original)
        XCTAssertEqual(ctx.stream.lastOriginal, true)

        XCTAssertEqual(ctx.renderer.renderCount, 0, "switching sides must not render")
    }

    func testTogglePlaybackStartsThenStopsAfterAnalysis() async throws {
        let ctx = try makeLoadedController()
        await ctx.controller.analysisTask?.value
        XCTAssertTrue(ctx.controller.canPlay)

        ctx.controller.togglePlayback()
        XCTAssertTrue(ctx.controller.isPlaying)
        XCTAssertEqual(ctx.output.startCount, 1)

        ctx.controller.togglePlayback()
        XCTAssertFalse(ctx.controller.isPlaying)
        XCTAssertGreaterThanOrEqual(ctx.output.stopCount, 1)
    }

    func testCreateMasterIsTheOnlyRenderPath() async throws {
        let ctx = try makeLoadedController()
        await ctx.controller.analysisTask?.value
        XCTAssertEqual(ctx.renderer.renderCount, 0)

        ctx.controller.createMaster()
        await ctx.controller.renderTask?.value

        XCTAssertEqual(ctx.renderer.renderCount, 1, "Create Master is the single render path")
        XCTAssertNotNil(ctx.controller.shareMasterURL)
        // Export options carry only preset/intensity/loudness — never a Volume
        // Match flag (the Rust export force-disables it).
        XCTAssertEqual(ctx.renderer.lastRenderOptions?.preset, "balanced")
    }

    func testRenderUsesHonestIndeterminateStatusUntilMasterFinishes() async throws {
        let renderer = FakeRenderer()
        renderer.renderDelay = 0.10
        let ctx = try makeLoadedController(renderer: renderer)
        await ctx.controller.analysisTask?.value

        ctx.controller.createMaster()
        XCTAssertTrue(ctx.controller.isRendering)
        XCTAssertEqual(ctx.controller.statusText, "Creating master...")

        await ctx.controller.renderTask?.value
        XCTAssertEqual(ctx.controller.statusText, "Master created. Share it anytime.")
    }

    func testUnsupportedImportUsesExplicitErrorState() async throws {
        let ctx = try makeLoadedController()
        await ctx.controller.analysisTask?.value
        let unsupported = ctx.tempBase.appendingPathComponent("source.aiff")
        try Self.writeWav(to: unsupported)

        ctx.controller.importTrack(from: unsupported)

        XCTAssertEqual(ctx.controller.errorState, .unsupportedExtension("aiff"))
        XCTAssertEqual(ctx.controller.statusText, ".aiff is not supported yet.")
    }

    func testSuccessfulImportClearsExplicitErrorState() async throws {
        let ctx = try makeLoadedController()
        await ctx.controller.analysisTask?.value
        let unsupported = ctx.tempBase.appendingPathComponent("source.aiff")
        try Self.writeWav(to: unsupported)
        ctx.controller.importTrack(from: unsupported)
        XCTAssertNotNil(ctx.controller.errorState)

        let supported = ctx.tempBase.appendingPathComponent("second.wav")
        try Self.writeWav(to: supported)
        ctx.controller.importTrack(from: supported)

        XCTAssertNil(ctx.controller.errorState)
        XCTAssertEqual(ctx.controller.statusText, "Analyzing audio...")
    }

    func testVolumeMatchToggleForwardsAuditionGain() throws {
        let ctx = try makeLoadedController()
        XCTAssertFalse(ctx.controller.volumeMatchEnabled)

        ctx.controller.toggleVolumeMatch()
        XCTAssertTrue(ctx.controller.volumeMatchEnabled)
        // No mastered loudness yet -> unity gain, but the call is forwarded.
        XCTAssertEqual(ctx.stream.lastVolumeMatch, 1.0)
    }

    func testReachingEndStopsPlaybackSoTheButtonReturns() async throws {
        let ctx = try makeLoadedController()
        await ctx.controller.analysisTask?.value
        ctx.controller.togglePlayback()
        XCTAssertTrue(ctx.controller.isPlaying)

        // Cursor reaches the end.
        ctx.stream.positionSeconds = ctx.stream.durationSeconds
        ctx.controller.handlePlaybackTick()

        XCTAssertFalse(ctx.controller.isPlaying, "playback should stop at EOF so the play button returns")
    }

    func testPressingPlayAtEndRestartsFromStart() async throws {
        let ctx = try makeLoadedController()
        await ctx.controller.analysisTask?.value
        // Parked at the end, not playing.
        ctx.stream.positionSeconds = ctx.stream.durationSeconds

        ctx.controller.togglePlayback() // play from the end

        XCTAssertEqual(ctx.stream.lastSeek, 0, "pressing play at the end should seek back to the start")
        XCTAssertTrue(ctx.controller.isPlaying)
    }

    func testLandingRefreshSetsGainAndMakesVolumeMatchLive() async throws {
        let ctx = try makeLoadedController()
        await ctx.controller.analysisTask?.value // FakeRenderer analysis: originalLufs = -12
        ctx.stream.stubLandingGain = 0.6
        ctx.stream.stubMasteredLufs = -11.0

        await ctx.controller.refreshLanding()

        // Loudness landing reached the stream (Low/Med/High now audible).
        XCTAssertEqual(ctx.stream.lastLandingGain, 0.6)
        XCTAssertEqual(ctx.controller.masteredLufs, -11.0)

        // Volume Match is now live: Mastered (-11) is louder than Original (-12),
        // so matching attenuates it (~ -1 LU) instead of the previous unity no-op.
        ctx.controller.selectSide(.mastered)
        ctx.controller.toggleVolumeMatch()
        let gain = try XCTUnwrap(ctx.stream.lastVolumeMatch)
        XCTAssertLessThan(gain, 1.0)
        XCTAssertGreaterThan(gain, 0.8)
    }

    func testStaleLandingRefreshDoesNotOverwriteNewerSettings() async throws {
        let ctx = try makeLoadedController()
        await ctx.controller.analysisTask?.value
        ctx.stream.measureDelay = 0.08
        ctx.stream.stubLandingGain = 0.4

        let stale = Task { await ctx.controller.refreshLanding() }
        try await Task.sleep(nanoseconds: 10_000_000)
        ctx.stream.stubLandingGain = 0.9
        ctx.controller.setLoudness(.high)
        await stale.value

        XCTAssertNil(ctx.stream.lastLandingGain, "stale landing result must not reach the engine")
        XCTAssertNil(ctx.controller.masteredLufs, "stale mastered LUFS must not reach Volume Match")
    }

    func testInterruptionPausesThenResumesWhenAllowed() async throws {
        let ctx = try makeLoadedController()
        await ctx.controller.analysisTask?.value
        ctx.controller.togglePlayback()
        XCTAssertTrue(ctx.controller.isPlaying)

        ctx.controller.handleInterruptionBegan()
        XCTAssertFalse(ctx.controller.isPlaying, "an interruption should pause")

        ctx.controller.handleInterruptionEnded(shouldResume: true)
        XCTAssertTrue(ctx.controller.isPlaying, "should resume when the system allows it")
    }

    func testInterruptionWithoutResumeStaysPaused() async throws {
        let ctx = try makeLoadedController()
        await ctx.controller.analysisTask?.value
        ctx.controller.togglePlayback()
        ctx.controller.handleInterruptionBegan()
        ctx.controller.handleInterruptionEnded(shouldResume: false)
        XCTAssertFalse(ctx.controller.isPlaying)
    }

    func testInterruptionWhilePausedDoesNotAutoResume() async throws {
        let ctx = try makeLoadedController()
        await ctx.controller.analysisTask?.value
        // Not playing when the interruption begins.
        ctx.controller.handleInterruptionBegan()
        ctx.controller.handleInterruptionEnded(shouldResume: true)
        XCTAssertFalse(ctx.controller.isPlaying, "must not start playing on its own after an interruption")
    }

    func testRouteLossPausesPlayback() async throws {
        let ctx = try makeLoadedController()
        await ctx.controller.analysisTask?.value
        ctx.controller.togglePlayback()
        XCTAssertTrue(ctx.controller.isPlaying)
        ctx.controller.handleAudioRouteLost()
        XCTAssertFalse(ctx.controller.isPlaying, "losing the output route should pause")
    }

    /// S8.3a cross-language pin: these raw values are the FFI wire contract,
    /// asserted on the Rust side by `tests::ffi_error_payloads_carry_stable_codes`
    /// (rust/src/lib.rs). Change the two pins together.
    func testBridgeErrorCodesMatchTheRustWireContract() {
        XCTAssertEqual(
            NativeBridgeErrorCode.allCases.map(\.rawValue),
            ["invalid_path", "decode", "render", "io", "internal", "other"]
        )
    }

    func testDecodeCodedAnalysisErrorMapsToDecodeFailedState() async throws {
        let renderer = FakeRenderer()
        renderer.analysisError = NativeMasteringBridgeError.rust(
            "adaptive context failed: decode error: no suitable format reader", .decode
        )
        let ctx = try makeLoadedController(renderer: renderer)
        await ctx.controller.analysisTask?.value

        XCTAssertEqual(
            ctx.controller.errorState, .decodeFailed,
            "a decode-coded bridge error must map to the typed decode state, not message sniffing"
        )
        XCTAssertFalse(ctx.controller.canPlay)
    }

    func testOtherCodedAnalysisErrorFallsBackToItsMessage() async throws {
        let renderer = FakeRenderer()
        renderer.analysisError = NativeMasteringBridgeError.rust("analysis exploded", .other)
        let ctx = try makeLoadedController(renderer: renderer)
        await ctx.controller.analysisTask?.value

        XCTAssertEqual(ctx.controller.errorState, .analysisFailed("analysis exploded"))
    }

    func testDecodeCodedRenderErrorMapsToDecodeFailedState() async throws {
        let renderer = FakeRenderer()
        let ctx = try makeLoadedController(renderer: renderer)
        await ctx.controller.analysisTask?.value

        renderer.renderError = NativeMasteringBridgeError.rust("decode error: bad frames", .decode)
        ctx.controller.createMaster()
        await ctx.controller.renderTask?.value

        XCTAssertEqual(ctx.controller.errorState, .decodeFailed)
        XCTAssertNil(ctx.controller.shareMasterURL)
    }

    // MARK: - Fixture

    private struct Context {
        let controller: AuditionController
        let stream: RecordingStream
        let output: RecordingOutput
        let renderer: FakeRenderer
        let tempBase: URL
    }

    private func makeLoadedController(
        renderer: FakeRenderer = FakeRenderer(),
        minimumAnalysisPresentationSeconds: TimeInterval = 0
    ) throws -> Context {
        let tempBase = FileManager.default.temporaryDirectory.appendingPathComponent(UUID().uuidString)
        try FileManager.default.createDirectory(at: tempBase, withIntermediateDirectories: true)
        let storage = RenderStorage(baseDirectory: tempBase)
        let importStore = ImportedTrackStore(importedTracksDirectory: storage.importsDirectory)
        let stream = RecordingStream()
        let output = RecordingOutput()
        let engine = LiveAudioEngine(output: output, makeStream: { _, _, _, _ in stream })
        let controller = AuditionController(
            engine: engine,
            renderer: renderer,
            importStore: importStore,
            renderStorage: storage,
            minimumAnalysisPresentationSeconds: minimumAnalysisPresentationSeconds
        )

        let source = tempBase.appendingPathComponent("source.wav")
        try Self.writeWav(to: source)
        controller.importTrack(from: source)
        addTeardownBlock { try? FileManager.default.removeItem(at: tempBase) }
        return Context(controller: controller, stream: stream, output: output, renderer: renderer, tempBase: tempBase)
    }

    private static func writeWav(to url: URL, seconds: Double = 0.3) throws {
        let sampleRate = 48_000, channels = 2, bits = 16
        let frames = Int(Double(sampleRate) * seconds)
        let blockAlign = channels * bits / 8
        let dataSize = frames * blockAlign
        var data = Data()
        func le<T: FixedWidthInteger>(_ v: T) { withUnsafeBytes(of: v.littleEndian) { data.append(contentsOf: $0) } }
        data.append(Data("RIFF".utf8)); le(UInt32(36 + dataSize)); data.append(Data("WAVE".utf8))
        data.append(Data("fmt ".utf8)); le(UInt32(16)); le(UInt16(1)); le(UInt16(channels))
        le(UInt32(sampleRate)); le(UInt32(sampleRate * blockAlign)); le(UInt16(blockAlign)); le(UInt16(bits))
        data.append(Data("data".utf8)); le(UInt32(dataSize))
        for frame in 0..<frames {
            let v = Int16(sin(2.0 * .pi * 440.0 * Double(frame) / Double(sampleRate)) * 0.3 * Double(Int16.max))
            le(v); le(v)
        }
        try data.write(to: url)
    }
}

// MARK: - Fakes

private final class RecordingStream: LiveAuditionStreaming {
    var channelCount = 2
    var sampleRate = 48_000.0
    var durationSeconds = 0.3
    var positionSeconds = 0.0

    var lastOriginal: Bool?
    var lastParams: (preset: String, intensity: Float, loudness: Float)?
    var lastVolumeMatch: Float?
    var lastSeek: Double?
    var lastLandingGain: Float?
    var stubLandingGain: Float = 0.7
    var stubMasteredLufs: Double = -11.0
    var measureDelay: TimeInterval = 0

    func render(into buffer: UnsafeMutablePointer<Float>, frames: UInt32) -> UInt32 { frames }
    func snapControlsToTargets() {}
    func setOriginal(_ original: Bool) { lastOriginal = original }
    func setParams(preset: String, intensity: Float, loudnessTarget: Float) {
        lastParams = (preset, intensity, loudnessTarget)
    }
    func setVolumeMatch(linearGain: Float) { lastVolumeMatch = linearGain }
    func setLandingGain(linearGain: Float) { lastLandingGain = linearGain }
    func seek(toSeconds seconds: Double) {
        lastSeek = seconds
        positionSeconds = seconds
    }
    func measureLanding(preset: String, intensity: Float, loudnessTarget: Float) -> (gain: Float, masteredLufs: Double) {
        if measureDelay > 0 {
            Thread.sleep(forTimeInterval: measureDelay)
        }
        return (stubLandingGain, stubMasteredLufs)
    }
}

private final class RecordingOutput: LiveAudioOutput {
    private(set) var startCount = 0
    private(set) var stopCount = 0
    var isRunning = false
    func start(pulling stream: LiveAuditionStreaming) throws { startCount += 1; isRunning = true }
    func stop() { stopCount += 1; isRunning = false }
}

private final class FakeRenderer: MasteringRenderer {
    private(set) var analyzeCount = 0
    private(set) var renderCount = 0
    private(set) var lastRenderOptions: NativeRenderOptions?
    var analysisDelay: TimeInterval = 0
    var renderDelay: TimeInterval = 0
    var analysisError: Error?
    var renderError: Error?

    let supportedImportExtensions = ["wav", "mp3"]
    let supportedImportContentTypes: [UTType] = [.wav]

    func analyzeTrack(at url: URL) throws -> NativeAnalysisResult {
        if analysisDelay > 0 {
            Thread.sleep(forTimeInterval: analysisDelay)
        }
        analyzeCount += 1
        if let analysisError {
            throw analysisError
        }
        return NativeAnalysisResult(lufsIntegrated: -12, truePeakDbtp: -1, dynamicRangeLu: 8)
    }

    func renderMaster(from sourceURL: URL, toDirectory: URL, options: NativeRenderOptions) throws -> NativeRenderJob {
        if renderDelay > 0 {
            Thread.sleep(forTimeInterval: renderDelay)
        }
        renderCount += 1
        lastRenderOptions = options
        if let renderError {
            throw renderError
        }
        return NativeRenderJob(
            outputPaths: [toDirectory.appendingPathComponent("master.wav").path],
            measurements: NativeRenderedMeasurements(
                lufsIntegrated: options.lufsTarget == 0 ? -11 : Double(options.lufsTarget),
                truePeakDbtp: -1, dynamicRangeLu: 7, sampleRate: 44_100, bitDepth: 24
            )
        )
    }
}
