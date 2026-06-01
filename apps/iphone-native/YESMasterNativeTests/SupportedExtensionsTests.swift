import XCTest
@testable import YES_Master_Native

final class SupportedExtensionsTests: XCTestCase {
    func testSupportedExtensionsMatchRustDecoderSupport() {
        let bridge = NativeMasteringBridge()
        XCTAssertEqual(Set(bridge.supportedImportExtensions), ["wav", "mp3", "m4a", "aac", "flac", "ogg"])
    }
}
