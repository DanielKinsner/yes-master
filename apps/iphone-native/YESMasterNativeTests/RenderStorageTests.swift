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

    /// Apple aliases the same file across symlinked paths (e.g. `/var` vs
    /// `/private/var`). Raw `URL` equality treats those as different files, so a
    /// just-rendered preview addressed via one alias gets deleted when pruning
    /// lists it via the other. Pruning must compare symlink-resolved paths.
    /// We reproduce the alias deterministically with a symlink so the test
    /// holds on both Simulator and device (Simulator temp paths are not under
    /// `/private/var`, which would make a `/private/var`→`/var` string swap a
    /// no-op and silently pass).
    func testPruneObsoletePreviewsKeepsSymlinkEquivalentCurrentURL() throws {
        let storage = RenderStorage(baseDirectory: root)
        try FileManager.default.createDirectory(at: storage.previewsDirectory, withIntermediateDirectories: true)
        let keep = storage.previewsDirectory.appendingPathComponent("keep.wav")
        let drop = storage.previewsDirectory.appendingPathComponent("drop.wav")
        try Data([0]).write(to: keep)
        try Data([0]).write(to: drop)

        // A symlinked alias of the previews directory: `keep.wav` addressed
        // through it resolves to the same real file by a different path.
        let aliasDir = root.appendingPathComponent("previews-alias")
        try FileManager.default.createSymbolicLink(at: aliasDir, withDestinationURL: storage.previewsDirectory)
        let keepViaAlias = aliasDir.appendingPathComponent("keep.wav")

        storage.pruneObsoletePreviews(keeping: keepViaAlias)

        XCTAssertTrue(
            FileManager.default.fileExists(atPath: keep.path),
            "current preview must survive pruning even when addressed via a symlink-equivalent path"
        )
        XCTAssertFalse(FileManager.default.fileExists(atPath: drop.path), "obsolete preview should be deleted")
    }
}
