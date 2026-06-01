import Foundation

struct ImportedTrack: Equatable {
    let displayName: String
    let fileExtension: String
    let localURL: URL
}

struct ImportedTrackStore {
    enum ImportError: Error, Equatable {
        case unsupportedExtension(String)
        case missingFileName
    }

    private let importedTracksDirectory: URL
    private let fileManager: FileManager

    init(
        importedTracksDirectory: URL = FileManager.default.urls(
            for: .applicationSupportDirectory,
            in: .userDomainMask
        )[0].appendingPathComponent("ImportedTracks", isDirectory: true),
        fileManager: FileManager = .default
    ) {
        self.importedTracksDirectory = importedTracksDirectory
        self.fileManager = fileManager
    }

    func importTrack(from sourceURL: URL, supportedExtensions: [String]) throws -> ImportedTrack {
        let normalizedExtension = sourceURL.pathExtension.trimmingCharacters(in: .whitespacesAndNewlines)
            .lowercased()
        let normalizedSupportedExtensions = Set(
            supportedExtensions.map {
                $0.trimmingCharacters(in: .whitespacesAndNewlines)
                    .trimmingCharacters(in: CharacterSet(charactersIn: "."))
                    .lowercased()
            }
        )

        guard normalizedSupportedExtensions.contains(normalizedExtension) else {
            throw ImportError.unsupportedExtension(normalizedExtension)
        }

        let fileName = sourceURL.lastPathComponent
        guard !fileName.isEmpty else {
            throw ImportError.missingFileName
        }

        let didStartSecurityScope = sourceURL.startAccessingSecurityScopedResource()
        defer {
            if didStartSecurityScope {
                sourceURL.stopAccessingSecurityScopedResource()
            }
        }

        try fileManager.createDirectory(
            at: importedTracksDirectory,
            withIntermediateDirectories: true
        )

        let destinationURL = uniqueDestinationURL(for: sourceURL)
        try fileManager.copyItem(at: sourceURL, to: destinationURL)

        return ImportedTrack(
            displayName: fileName,
            fileExtension: normalizedExtension,
            localURL: destinationURL
        )
    }

    private func uniqueDestinationURL(for sourceURL: URL) -> URL {
        let baseName = sourceURL.deletingPathExtension().lastPathComponent
        let safeBaseName = baseName.isEmpty ? "Track" : baseName
        let fileExtension = sourceURL.pathExtension

        var candidate = importedTracksDirectory
            .appendingPathComponent(sourceURL.lastPathComponent, isDirectory: false)
        var copyIndex = 2

        while fileManager.fileExists(atPath: candidate.path) {
            let copiedName = "\(safeBaseName)-\(copyIndex)"
            candidate = importedTracksDirectory
                .appendingPathComponent(copiedName, isDirectory: false)
                .appendingPathExtension(fileExtension)
            copyIndex += 1
        }

        return candidate
    }
}
