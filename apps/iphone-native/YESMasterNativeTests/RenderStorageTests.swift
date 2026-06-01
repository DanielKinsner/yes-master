import XCTest
@testable import YES_Master_Native

final class RenderStorageTests: XCTestCase {
    private var root: URL!

    override func setUpWithError() throws {
        root = FileManager.default.temporaryDirectory.appendingPathComponent(UUID().uuidString)
        try FileManager.default.createDirectory(at: root, withIntermediateDirectories: true)
    }
    override func tearDownWithError() throws { try? FileManager.default.removeItem(at: root) }

    func testMastersDirectoryIsDurableNotCaches() {
        let storage = RenderStorage(baseDirectory: root)
        XCTAssertTrue(storage.mastersDirectory.path.contains("RenderedMasters"))
        XCTAssertFalse(storage.mastersDirectory.path.lowercased().contains("caches"))
    }

    func testEnforceLimitKeepsNewest() throws {
        let storage = RenderStorage(baseDirectory: root)
        try FileManager.default.createDirectory(at: storage.mastersDirectory, withIntermediateDirectories: true)
        for i in 0..<5 {
            let u = storage.mastersDirectory.appendingPathComponent("m\(i).wav")
            try Data([UInt8(i)]).write(to: u)
            try FileManager.default.setAttributes(
                [.modificationDate: Date(timeIntervalSince1970: TimeInterval(1000 + i))], ofItemAtPath: u.path
            )
        }
        storage.enforceLimit(in: storage.mastersDirectory, max: 2)
        let remaining = try FileManager.default.contentsOfDirectory(atPath: storage.mastersDirectory.path)
        XCTAssertEqual(Set(remaining), ["m3.wav", "m4.wav"])
    }

    func testPruneObsoletePreviewsKeepsOnlyCurrent() throws {
        let storage = RenderStorage(baseDirectory: root)
        try FileManager.default.createDirectory(at: storage.previewsDirectory, withIntermediateDirectories: true)
        let keep = storage.previewsDirectory.appendingPathComponent("keep.wav")
        let drop = storage.previewsDirectory.appendingPathComponent("drop.wav")
        try Data([0]).write(to: keep)
        try Data([0]).write(to: drop)

        storage.pruneObsoletePreviews(keeping: keep)

        XCTAssertTrue(FileManager.default.fileExists(atPath: keep.path))
        XCTAssertFalse(FileManager.default.fileExists(atPath: drop.path))
    }
}
