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

    func testNonFiniteInputFallsBackToUnity() {
        // §2 — a non-finite measured LUFS (e.g. -inf for digital silence) must
        // never produce a non-finite gain. Mirrors the Android-Rust guard test
        // (audition.rs volume_match_linear_gain non-finite cases).
        XCTAssertEqual(volumeMatchLinearGain(sideLufs: -.infinity, otherLufs: -10), 1.0)
        XCTAssertEqual(volumeMatchLinearGain(sideLufs: -10, otherLufs: .nan), 1.0)
        XCTAssertEqual(volumeMatchLinearGain(sideLufs: .infinity, otherLufs: .infinity), 1.0)
        XCTAssertTrue(volumeMatchLinearGain(sideLufs: -.infinity, otherLufs: -10).isFinite)
    }
}
