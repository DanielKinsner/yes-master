import XCTest
@testable import YES_Master_Native

final class ImportedTrackStoreTests: XCTestCase {
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

    func testImportCopiesSupportedTrackIntoAppOwnedStorage() throws {
        let sourceURL = temporaryDirectory.appendingPathComponent("Mix Draft.WAV")
        try Data("fake audio".utf8).write(to: sourceURL)

        let store = ImportedTrackStore(
            importedTracksDirectory: temporaryDirectory.appendingPathComponent("Imported", isDirectory: true)
        )

        let track = try store.importTrack(
            from: sourceURL,
            supportedExtensions: ["wav", "mp3"]
        )

        XCTAssertEqual(track.displayName, "Mix Draft.WAV")
        XCTAssertEqual(track.fileExtension, "wav")
        XCTAssertTrue(FileManager.default.fileExists(atPath: track.localURL.path))
        XCTAssertNotEqual(track.localURL, sourceURL)
    }

    func testImportRejectsUnsupportedExtensions() throws {
        let sourceURL = temporaryDirectory.appendingPathComponent("mix.aiff")
        try Data("fake audio".utf8).write(to: sourceURL)

        let store = ImportedTrackStore(
            importedTracksDirectory: temporaryDirectory.appendingPathComponent("Imported", isDirectory: true)
        )

        XCTAssertThrowsError(
            try store.importTrack(from: sourceURL, supportedExtensions: ["wav", "mp3"])
        ) { error in
            XCTAssertEqual(error as? ImportedTrackStore.ImportError, .unsupportedExtension("aiff"))
        }
    }

    func testImportCreatesUniqueCopiesInsteadOfOverwriting() throws {
        let sourceURL = temporaryDirectory.appendingPathComponent("mix.wav")
        try Data("first".utf8).write(to: sourceURL)

        let store = ImportedTrackStore(
            importedTracksDirectory: temporaryDirectory.appendingPathComponent("Imported", isDirectory: true)
        )

        let firstTrack = try store.importTrack(from: sourceURL, supportedExtensions: ["wav"])
        let secondTrack = try store.importTrack(from: sourceURL, supportedExtensions: ["wav"])

        XCTAssertNotEqual(firstTrack.localURL, secondTrack.localURL)
        XCTAssertTrue(FileManager.default.fileExists(atPath: firstTrack.localURL.path))
        XCTAssertTrue(FileManager.default.fileExists(atPath: secondTrack.localURL.path))
    }
}
