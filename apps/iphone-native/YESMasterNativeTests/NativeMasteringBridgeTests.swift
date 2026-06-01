import Foundation
import XCTest
@testable import YES_Master_Native

final class NativeMasteringBridgeTests: XCTestCase {
    private var temporaryDirectory: URL!

    override func setUpWithError() throws {
        temporaryDirectory = FileManager.default.temporaryDirectory
            .appendingPathComponent(UUID().uuidString, isDirectory: true)
        try FileManager.default.createDirectory(
            at: temporaryDirectory,
            withIntermediateDirectories: true
        )
    }

    override func tearDownWithError() throws {
        try? FileManager.default.removeItem(at: temporaryDirectory)
    }

    func testAnalyzeTrackSurfacesRustErrors() {
        let bridge = NativeMasteringBridge()
        let missingURL = URL(fileURLWithPath: "/tmp/yes-master-native-missing.wav")

        XCTAssertThrowsError(try bridge.analyzeTrack(at: missingURL)) { error in
            XCTAssertTrue(
                error.localizedDescription.contains("source file not found"),
                "got \(error.localizedDescription)"
            )
        }
    }

    func testRenderMasterSurfacesRustErrors() {
        let bridge = NativeMasteringBridge()
        let missingSourceURL = URL(fileURLWithPath: "/tmp/yes-master-native-missing-render-source.wav")
        let outputDirectoryURL = URL(fileURLWithPath: NSTemporaryDirectory())
            .appendingPathComponent("yes-master-native-missing-render-output", isDirectory: true)

        XCTAssertThrowsError(try bridge.renderMaster(from: missingSourceURL, toDirectory: outputDirectoryURL)) { error in
            XCTAssertTrue(
                error.localizedDescription.contains("source file not found"),
                "got \(error.localizedDescription)"
            )
        }
    }

    func testAnalyzeAndRenderMasterUseRustBridgeForRealWav() throws {
        let bridge = NativeMasteringBridge()
        let sourceURL = temporaryDirectory.appendingPathComponent("source.wav")
        let outputDirectoryURL = temporaryDirectory.appendingPathComponent("renders", isDirectory: true)
        try writeSineWave(to: sourceURL)

        let analysis = try bridge.analyzeTrack(at: sourceURL)
        XCTAssertLessThan(analysis.lufsIntegrated, 0)

        let renderJob = try bridge.renderMaster(from: sourceURL, toDirectory: outputDirectoryURL)

        let outputPath = try XCTUnwrap(renderJob.outputPaths.first)
        XCTAssertTrue(FileManager.default.fileExists(atPath: outputPath))
        XCTAssertEqual(URL(fileURLWithPath: outputPath).pathExtension.lowercased(), "wav")
        XCTAssertEqual(renderJob.measurements?.sampleRate, 44_100)
        XCTAssertEqual(renderJob.measurements?.bitDepth, 24)
    }

    private func writeSineWave(to url: URL) throws {
        let sampleRate = 44_100
        let channels = 2
        let bitsPerSample = 16
        let frameCount = sampleRate / 4
        let byteRate = sampleRate * channels * bitsPerSample / 8
        let blockAlign = channels * bitsPerSample / 8
        let dataByteCount = frameCount * blockAlign

        var data = Data()
        data.append(contentsOf: [0x52, 0x49, 0x46, 0x46])
        data.appendLittleEndianUInt32(UInt32(36 + dataByteCount))
        data.append(contentsOf: [0x57, 0x41, 0x56, 0x45])
        data.append(contentsOf: [0x66, 0x6d, 0x74, 0x20])
        data.appendLittleEndianUInt32(16)
        data.appendLittleEndianUInt16(1)
        data.appendLittleEndianUInt16(UInt16(channels))
        data.appendLittleEndianUInt32(UInt32(sampleRate))
        data.appendLittleEndianUInt32(UInt32(byteRate))
        data.appendLittleEndianUInt16(UInt16(blockAlign))
        data.appendLittleEndianUInt16(UInt16(bitsPerSample))
        data.append(contentsOf: [0x64, 0x61, 0x74, 0x61])
        data.appendLittleEndianUInt32(UInt32(dataByteCount))

        for frame in 0..<frameCount {
            let phase = Double(frame) / Double(sampleRate) * 440.0 * 2.0 * Double.pi
            let sample = Int16((sin(phase) * 0.2 * Double(Int16.max)).rounded())
            data.appendLittleEndianInt16(sample)
            data.appendLittleEndianInt16(sample)
        }

        try data.write(to: url)
    }
}

private extension Data {
    mutating func appendLittleEndianUInt16(_ value: UInt16) {
        var littleEndianValue = value.littleEndian
        Swift.withUnsafeBytes(of: &littleEndianValue) { append(contentsOf: $0) }
    }

    mutating func appendLittleEndianInt16(_ value: Int16) {
        var littleEndianValue = value.littleEndian
        Swift.withUnsafeBytes(of: &littleEndianValue) { append(contentsOf: $0) }
    }

    mutating func appendLittleEndianUInt32(_ value: UInt32) {
        var littleEndianValue = value.littleEndian
        Swift.withUnsafeBytes(of: &littleEndianValue) { append(contentsOf: $0) }
    }
}
