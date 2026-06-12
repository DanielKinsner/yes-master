import Foundation
import XCTest
@testable import YES_Master_Native

final class NativeLoudnessTests: XCTestCase {
    func testLoudnessMapsToExportTargets() throws {
        let fixture = try Self.loadParityFixture()

        for loudness in NativeLoudness.allCases {
            let key = loudness.rawValue.lowercased()
            XCTAssertEqual(
                loudness.lufsTarget,
                try XCTUnwrap(fixture.loudness[key], "Missing \(key) in standard-mapping-parity.json")
            )
        }
    }

    func testStyleBridgeIdentifiersMatchTheSharedParityFixture() throws {
        let fixture = try Self.loadParityFixture()
        let expectedIds = Set(fixture.styles.keys)
        let actualIds = Set(NativeStylePreset.allCases.map(\.bridgeIdentifier))

        XCTAssertEqual(actualIds, expectedIds)
        for preset in NativeStylePreset.allCases {
            XCTAssertNotNil(
                fixture.styles[preset.bridgeIdentifier],
                "\(preset) bridgeIdentifier is not pinned by standard-mapping-parity.json"
            )
        }
    }

    private struct StandardParityFixture: Decodable {
        let styles: [String: String]
        let loudness: [String: Float]
    }

    private static func loadParityFixture() throws -> StandardParityFixture {
        let url = try XCTUnwrap(
            Bundle(for: NativeLoudnessTests.self).url(
                forResource: "standard-mapping-parity",
                withExtension: "json"
            ),
            "standard-mapping-parity.json must be bundled into YESMasterNativeTests"
        )
        let data = try Data(contentsOf: url)
        return try JSONDecoder().decode(StandardParityFixture.self, from: data)
    }
}
