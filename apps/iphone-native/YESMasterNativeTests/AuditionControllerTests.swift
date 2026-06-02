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

    // MARK: - Fixture

    private struct Context {
        let controller: AuditionController
        let stream: RecordingStream
        let output: RecordingOutput
        let renderer: FakeRenderer
        let tempBase: URL
    }

    private func makeLoadedController() throws -> Context {
        let tempBase = FileManager.default.temporaryDirectory.appendingPathComponent(UUID().uuidString)
        try FileManager.default.createDirectory(at: tempBase, withIntermediateDirectories: true)
        let storage = RenderStorage(baseDirectory: tempBase)
        let importStore = ImportedTrackStore(importedTracksDirectory: storage.importsDirectory)
        let stream = RecordingStream()
        let output = RecordingOutput()
        let renderer = FakeRenderer()
        let engine = LiveAudioEngine(output: output, makeStream: { _, _, _, _ in stream })
        let controller = AuditionController(
            engine: engine, renderer: renderer, importStore: importStore, renderStorage: storage
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

    func render(into buffer: UnsafeMutablePointer<Float>, frames: UInt32) -> UInt32 { frames }
    func setOriginal(_ original: Bool) { lastOriginal = original }
    func setParams(preset: String, intensity: Float, loudnessTarget: Float) {
        lastParams = (preset, intensity, loudnessTarget)
    }
    func setVolumeMatch(linearGain: Float) { lastVolumeMatch = linearGain }
    func setLandingGain(linearGain: Float) {}
    func seek(toSeconds seconds: Double) {
        lastSeek = seconds
        positionSeconds = seconds
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

    let supportedImportExtensions = ["wav", "mp3"]
    let supportedImportContentTypes: [UTType] = [.wav]

    func analyzeTrack(at url: URL) throws -> NativeAnalysisResult {
        analyzeCount += 1
        return NativeAnalysisResult(lufsIntegrated: -12, truePeakDbtp: -1, dynamicRangeLu: 8)
    }

    func renderMaster(from sourceURL: URL, toDirectory: URL, options: NativeRenderOptions) throws -> NativeRenderJob {
        renderCount += 1
        lastRenderOptions = options
        return NativeRenderJob(
            outputPaths: [toDirectory.appendingPathComponent("master.wav").path],
            measurements: NativeRenderedMeasurements(
                lufsIntegrated: options.lufsTarget == 0 ? -11 : Double(options.lufsTarget),
                truePeakDbtp: -1, dynamicRangeLu: 7, sampleRate: 44_100, bitDepth: 24
            )
        )
    }
}
