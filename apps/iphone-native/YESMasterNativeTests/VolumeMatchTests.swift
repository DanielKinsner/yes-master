import XCTest
@testable import YES_Master_Native

final class VolumeMatchTests: XCTestCase {
    func testQuieterSideIsUnchangedLouderSideAttenuated() {
        XCTAssertEqual(volumeMatchGainDb(sideLufs: -18, otherLufs: -11), 0, accuracy: 0.0001)
        XCTAssertEqual(volumeMatchGainDb(sideLufs: -11, otherLufs: -18), -7, accuracy: 0.0001)
    }

    func testLinearGainNeverExceedsUnity() {
        XCTAssertLessThanOrEqual(volumeMatchLinearGain(sideLufs: -11, otherLufs: -18), 1.0)
        XCTAssertEqual(volumeMatchLinearGain(sideLufs: -18, otherLufs: -11), 1.0, accuracy: 0.0001)
    }
}
