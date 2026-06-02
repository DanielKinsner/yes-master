import AVFoundation
import XCTest
@testable import YES_Master_Native

final class LiveAudioEngineTests: XCTestCase {
    private let url = URL(fileURLWithPath: "/tmp/yes-master-live-engine.wav")

    func testLoadBuildsStreamWithoutAutoPlaying() {
        let output = FakeOutput()
        var built = 0
        let engine = LiveAudioEngine(output: output, makeStream: { _, _, _, _ in
            built += 1
            return FakeStream()
        })

        let ok = engine.load(url: url, preset: "balanced", intensity: 0.5, loudnessTarget: -11)

        XCTAssertTrue(ok)
        XCTAssertEqual(built, 1, "load should build exactly one stream")
        XCTAssertNotNil(engine.stream)
        XCTAssertFalse(engine.isPlaying)
        XCTAssertEqual(output.startCount, 0, "load must not auto-play")
    }

    func testLoadFailureReturnsFalseAndClearsStream() {
        let engine = LiveAudioEngine(output: FakeOutput(), makeStream: { _, _, _, _ in nil })
        XCTAssertFalse(engine.load(url: url, preset: "balanced", intensity: 0.5, loudnessTarget: -11))
        XCTAssertNil(engine.stream)
        XCTAssertFalse(engine.isPlaying)
    }

    func testPlayStartsOutputOnceAndPauseKeepsStream() throws {
        let output = FakeOutput()
        let stream = FakeStream()
        let engine = LiveAudioEngine(output: output, makeStream: { _, _, _, _ in stream })
        engine.load(url: url, preset: "balanced", intensity: 0.5, loudnessTarget: -11)

        try engine.play()
        XCTAssertEqual(output.startCount, 1)
        XCTAssertTrue(output.lastStartedStream === stream, "engine should start the loaded stream")
        XCTAssertTrue(engine.isPlaying)

        engine.pause()
        XCTAssertFalse(engine.isPlaying)
        XCTAssertNotNil(engine.stream, "pause must keep the stream so resume preserves the playhead")
    }

    func testPlayWithoutLoadDoesNothing() throws {
        let output = FakeOutput()
        let engine = LiveAudioEngine(output: output, makeStream: { _, _, _, _ in FakeStream() })
        try engine.play()
        XCTAssertEqual(output.startCount, 0)
        XCTAssertFalse(engine.isPlaying)
    }

    func testControlsForwardToStream() {
        let stream = FakeStream()
        let engine = LiveAudioEngine(output: FakeOutput(), makeStream: { _, _, _, _ in stream })
        engine.load(url: url, preset: "balanced", intensity: 0.5, loudnessTarget: -11)

        engine.setOriginal(true)
        engine.setParams(preset: "warm", intensity: 0.9, loudnessTarget: -9)
        engine.setVolumeMatch(linearGain: 0.5)
        engine.setLandingGain(linearGain: 1.25)
        engine.seek(toSeconds: 12.5)

        XCTAssertEqual(stream.lastOriginal, true)
        XCTAssertEqual(stream.lastParams?.preset, "warm")
        XCTAssertEqual(stream.lastParams?.intensity, 0.9)
        XCTAssertEqual(stream.lastParams?.loudness, -9)
        XCTAssertEqual(stream.lastVolumeMatch, 0.5)
        XCTAssertEqual(stream.lastLandingGain, 1.25)
        XCTAssertEqual(stream.lastSeek, 12.5)
    }

    func testPositionAndDurationComeFromStream() {
        let stream = FakeStream()
        stream.positionSeconds = 3.0
        stream.durationSeconds = 60.0
        let engine = LiveAudioEngine(output: FakeOutput(), makeStream: { _, _, _, _ in stream })
        engine.load(url: url, preset: "balanced", intensity: 0.5, loudnessTarget: -11)
        XCTAssertEqual(engine.positionSeconds, 3.0)
        XCTAssertEqual(engine.durationSeconds, 60.0)
    }

    func testSourceNodeFormatUsesNonInterleavedFloat32() throws {
        let format = try XCTUnwrap(AVAudioEngineOutput.makeSourceFormat(sampleRate: 48_000, channels: 2))

        XCTAssertEqual(format.commonFormat, .pcmFormatFloat32)
        XCTAssertFalse(format.isInterleaved)
        XCTAssertEqual(format.channelCount, 2)
        XCTAssertEqual(format.sampleRate, 48_000)
    }
}

private final class FakeStream: LiveAuditionStreaming {
    var channelCount = 2
    var sampleRate = 48_000.0
    var durationSeconds = 0.0
    var positionSeconds = 0.0

    var lastOriginal: Bool?
    var lastParams: (preset: String, intensity: Float, loudness: Float)?
    var lastVolumeMatch: Float?
    var lastLandingGain: Float?
    var lastSeek: Double?

    func render(into buffer: UnsafeMutablePointer<Float>, frames: UInt32) -> UInt32 { frames }
    func setOriginal(_ original: Bool) { lastOriginal = original }
    func setParams(preset: String, intensity: Float, loudnessTarget: Float) {
        lastParams = (preset, intensity, loudnessTarget)
    }
    func setVolumeMatch(linearGain: Float) { lastVolumeMatch = linearGain }
    func setLandingGain(linearGain: Float) { lastLandingGain = linearGain }
    func seek(toSeconds seconds: Double) { lastSeek = seconds }
}

private final class FakeOutput: LiveAudioOutput {
    private(set) var startCount = 0
    private(set) var stopCount = 0
    private(set) var lastStartedStream: LiveAuditionStreaming?
    var isRunning = false

    func start(pulling stream: LiveAuditionStreaming) throws {
        startCount += 1
        lastStartedStream = stream
        isRunning = true
    }

    func stop() {
        stopCount += 1
        isRunning = false
    }
}
