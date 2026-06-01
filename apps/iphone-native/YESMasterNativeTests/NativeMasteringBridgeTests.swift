import XCTest
@testable import YES_Master_Native

final class NativeMasteringBridgeTests: XCTestCase {
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
        let outputURL = URL(fileURLWithPath: NSTemporaryDirectory())
            .appendingPathComponent("yes-master-native-missing-render-output.wav")

        XCTAssertThrowsError(try bridge.renderMaster(from: missingSourceURL, to: outputURL)) { error in
            XCTAssertTrue(
                error.localizedDescription.contains("source file not found"),
                "got \(error.localizedDescription)"
            )
        }
    }
}
