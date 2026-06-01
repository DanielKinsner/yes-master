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
