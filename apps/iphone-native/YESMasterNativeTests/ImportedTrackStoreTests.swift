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
        try writeMinimalWavHeader(to: sourceURL)

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

    func testImportRejectsMissingSourceFilesWithPlainError() throws {
        let sourceURL = temporaryDirectory.appendingPathComponent("deleted.wav")

        let store = ImportedTrackStore(
            importedTracksDirectory: temporaryDirectory.appendingPathComponent("Imported", isDirectory: true)
        )

        XCTAssertThrowsError(
            try store.importTrack(from: sourceURL, supportedExtensions: ["wav"])
        ) { error in
            XCTAssertEqual(error as? ImportedTrackStore.ImportError, .sourceUnavailable)
        }
    }

    func testImportCreatesUniqueCopiesInsteadOfOverwriting() throws {
        let sourceURL = temporaryDirectory.appendingPathComponent("mix.wav")
        try writeMinimalWavHeader(to: sourceURL)

        let store = ImportedTrackStore(
            importedTracksDirectory: temporaryDirectory.appendingPathComponent("Imported", isDirectory: true)
        )

        let firstTrack = try store.importTrack(from: sourceURL, supportedExtensions: ["wav"])
        let secondTrack = try store.importTrack(from: sourceURL, supportedExtensions: ["wav"])

        XCTAssertNotEqual(firstTrack.localURL, secondTrack.localURL)
        XCTAssertTrue(FileManager.default.fileExists(atPath: firstTrack.localURL.path))
        XCTAssertTrue(FileManager.default.fileExists(atPath: secondTrack.localURL.path))
    }

    func testImportRejectsEmptyFiles() throws {
        let sourceURL = temporaryDirectory.appendingPathComponent("empty.wav")
        try Data().write(to: sourceURL)

        let store = ImportedTrackStore(
            importedTracksDirectory: temporaryDirectory.appendingPathComponent("Imported", isDirectory: true)
        )

        XCTAssertThrowsError(
            try store.importTrack(from: sourceURL, supportedExtensions: ["wav"])
        ) { error in
            XCTAssertEqual(error as? ImportedTrackStore.ImportError, .emptyFile)
        }
    }

    func testImportRemovesCopiedFileWhenValidationFails() throws {
        let sourceURL = temporaryDirectory.appendingPathComponent("fake.wav")
        let importedDirectory = temporaryDirectory.appendingPathComponent("Imported", isDirectory: true)
        try Data("not really a wave file".utf8).write(to: sourceURL)

        let store = ImportedTrackStore(importedTracksDirectory: importedDirectory)

        XCTAssertThrowsError(
            try store.importTrack(from: sourceURL, supportedExtensions: ["wav"])
        )

        let copiedFiles = (try? FileManager.default.contentsOfDirectory(
            at: importedDirectory,
            includingPropertiesForKeys: nil
        )) ?? []
        XCTAssertTrue(copiedFiles.isEmpty, "Rejected imports should not leave copied files behind.")
    }

    func testImportRejectsWavFilesThatDoNotLookLikeWavAudio() throws {
        let sourceURL = temporaryDirectory.appendingPathComponent("fake.wav")
        try Data("not really a wave file".utf8).write(to: sourceURL)

        let store = ImportedTrackStore(
            importedTracksDirectory: temporaryDirectory.appendingPathComponent("Imported", isDirectory: true)
        )

        XCTAssertThrowsError(
            try store.importTrack(from: sourceURL, supportedExtensions: ["wav"])
        ) { error in
            XCTAssertEqual(error as? ImportedTrackStore.ImportError, .unreadableContainer("wav"))
        }
    }

    func testImportStampsCopiedFileWithRecentModificationDate() throws {
        let sourceURL = temporaryDirectory.appendingPathComponent("old.wav")
        try writeMinimalWavHeader(to: sourceURL)
        try FileManager.default.setAttributes(
            [.modificationDate: Date(timeIntervalSince1970: 1_000)],
            ofItemAtPath: sourceURL.path
        )

        let store = ImportedTrackStore(
            importedTracksDirectory: temporaryDirectory.appendingPathComponent("Imported", isDirectory: true)
        )
        let track = try store.importTrack(from: sourceURL, supportedExtensions: ["wav"])

        let attributes = try FileManager.default.attributesOfItem(atPath: track.localURL.path)
        let modificationDate = try XCTUnwrap(attributes[.modificationDate] as? Date)
        XCTAssertGreaterThan(
            modificationDate.timeIntervalSinceNow,
            -60,
            "Imported copy should be stamped with a recent mtime, not the source's old date."
        )
    }

    private func writeMinimalWavHeader(to url: URL) throws {
        var data = Data()
        data.append(contentsOf: [0x52, 0x49, 0x46, 0x46])
        data.append(contentsOf: [0x24, 0x00, 0x00, 0x00])
        data.append(contentsOf: [0x57, 0x41, 0x56, 0x45])
        try data.write(to: url)
    }
}
