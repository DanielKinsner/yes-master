import XCTest
@testable import YES_Master_Native

final class NativeLoudnessTests: XCTestCase {
    func testLoudnessMapsToExportTargets() {
        XCTAssertEqual(NativeLoudness.low.lufsTarget, -14)
        XCTAssertEqual(NativeLoudness.medium.lufsTarget, -11)
        XCTAssertEqual(NativeLoudness.high.lufsTarget, -9)
    }
}
