import XCTest
@testable import YES_Master_Native

/// Exercises the real Rust live-stream FFI end-to-end from Swift (no audio
/// device required), so the simulator can prove the bridge works.
final class LiveAuditionBridgeTests: XCTestCase {
    private var tempURL: URL!

    override func setUpWithError() throws {
        tempURL = try Self.writeSineWav(seconds: 0.5)
    }

    override func tearDownWithError() throws {
        if let tempURL { try? FileManager.default.removeItem(at: tempURL) }
    }

    func testCreateFromValidFileExposesFormat() throws {
        let stream = try XCTUnwrap(
            LiveAuditionBridge(sourceURL: tempURL, preset: "balanced", intensity: 0.7, loudnessTarget: -11)
        )
        XCTAssertEqual(stream.channelCount, 2)
        XCTAssertEqual(stream.sampleRate, 48_000, accuracy: 1)
        XCTAssertEqual(stream.durationSeconds, 0.5, accuracy: 0.05)
        XCTAssertEqual(stream.positionSeconds, 0, accuracy: 1e-6)
    }

    func testCreateFromMissingFileReturnsNil() {
        let missing = FileManager.default.temporaryDirectory
            .appendingPathComponent("yes-master-live-missing.wav")
        XCTAssertNil(
            LiveAuditionBridge(sourceURL: missing, preset: "balanced", intensity: 0.5, loudnessTarget: -11)
        )
    }

    func testRenderFillsBufferAndAdvancesPlayhead() throws {
        let stream = try XCTUnwrap(
            LiveAuditionBridge(sourceURL: tempURL, preset: "punch", intensity: 0.8, loudnessTarget: -9)
        )
        let frames: UInt32 = 1_024
        var buffer = [Float](repeating: 0, count: Int(frames) * stream.channelCount)
        let written = buffer.withUnsafeMutableBufferPointer { ptr in
            stream.render(into: ptr.baseAddress!, frames: frames)
        }
        XCTAssertEqual(written, frames)
        XCTAssertTrue(buffer.allSatisfy { $0.isFinite })
        XCTAssertGreaterThan(buffer.map { abs($0) }.max() ?? 0, 0.001, "mastered render should be audible")
        XCTAssertGreaterThan(stream.positionSeconds, 0)
    }

    func testTogglesAndSettersDoNotCrashAndSeekMovesPlayhead() throws {
        let stream = try XCTUnwrap(
            LiveAuditionBridge(sourceURL: tempURL, preset: "balanced", intensity: 0.5, loudnessTarget: -11)
        )
        stream.setOriginal(true)
        stream.setParams(preset: "warm", intensity: 0.9, loudnessTarget: -14)
        stream.setVolumeMatch(linearGain: 0.5)
        stream.setLandingGain(linearGain: 1.2)
        stream.seek(toSeconds: 0.25)

        let frames: UInt32 = 256
        var buffer = [Float](repeating: 0, count: Int(frames) * stream.channelCount)
        _ = buffer.withUnsafeMutableBufferPointer { stream.render(into: $0.baseAddress!, frames: frames) }

        XCTAssertTrue(buffer.allSatisfy { $0.isFinite })
        XCTAssertGreaterThan(stream.positionSeconds, 0.2, "seek to 0.25s should move the playhead")
    }

    // MARK: - Helpers

    /// Hand-write a canonical 16-bit PCM stereo WAV. Deterministic and free of
    /// AVAudioFile container/format quirks, so the Rust decoder always accepts it.
    private static func writeSineWav(seconds: Double) throws -> URL {
        let sampleRate = 48_000
        let channels = 2
        let bitsPerSample = 16
        let frameCount = Int(Double(sampleRate) * seconds)
        let blockAlign = channels * bitsPerSample / 8
        let byteRate = sampleRate * blockAlign
        let dataSize = frameCount * blockAlign

        var data = Data()
        func appendLE<T: FixedWidthInteger>(_ value: T) {
            withUnsafeBytes(of: value.littleEndian) { data.append(contentsOf: $0) }
        }
        data.append(Data("RIFF".utf8))
        appendLE(UInt32(36 + dataSize))
        data.append(Data("WAVE".utf8))
        data.append(Data("fmt ".utf8))
        appendLE(UInt32(16))            // PCM fmt chunk size
        appendLE(UInt16(1))             // PCM
        appendLE(UInt16(channels))
        appendLE(UInt32(sampleRate))
        appendLE(UInt32(byteRate))
        appendLE(UInt16(blockAlign))
        appendLE(UInt16(bitsPerSample))
        data.append(Data("data".utf8))
        appendLE(UInt32(dataSize))
        for frame in 0..<frameCount {
            let sample = sin(2.0 * Double.pi * 440.0 * Double(frame) / Double(sampleRate)) * 0.3
            let value = Int16(sample * Double(Int16.max))
            appendLE(value) // L
            appendLE(value) // R
        }

        let url = FileManager.default.temporaryDirectory
            .appendingPathComponent(UUID().uuidString + ".wav")
        try data.write(to: url)
        return url
    }
}
