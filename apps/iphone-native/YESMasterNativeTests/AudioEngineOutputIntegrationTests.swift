import AVFoundation
import XCTest
@testable import YES_Master_Native

/// Exercises the REAL `AVAudioEngineOutput` (session + AVAudioEngine +
/// AVAudioSourceNode) against a real `LiveAuditionBridge`. The unit tests use a
/// fake output, so only this catches audio-graph misconfiguration (the kind that
/// traps with EXC_BREAKPOINT on play). Runs on the simulator.
final class AudioEngineOutputIntegrationTests: XCTestCase {
    func testRealEngineStartsAndRendersWithoutTrapping() throws {
        let url = try Self.writeWav(seconds: 0.5, sampleRate: 44_100)
        defer { try? FileManager.default.removeItem(at: url) }

        let bridge = try XCTUnwrap(
            LiveAuditionBridge(sourceURL: url, preset: "balanced", intensity: 0.6, loudnessTarget: -11)
        )
        let output = AVAudioEngineOutput()

        try output.start(pulling: bridge)
        Thread.sleep(forTimeInterval: 0.2) // let the render thread pull a few buffers
        XCTAssertTrue(output.isRunning)
        output.stop()
    }

    private static func writeWav(seconds: Double, sampleRate: Int) throws -> URL {
        let channels = 2, bits = 16
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
        let url = FileManager.default.temporaryDirectory.appendingPathComponent(UUID().uuidString + ".wav")
        try data.write(to: url)
        return url
    }
}
