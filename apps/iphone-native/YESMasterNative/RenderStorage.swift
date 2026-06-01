import Foundation

struct RenderStorage {
    let importsDirectory: URL
    let mastersDirectory: URL
    let previewsDirectory: URL
    private let fileManager: FileManager

    /// Production: imports/masters in Application Support (durable), previews in Caches (evictable).
    init(fileManager: FileManager = .default) {
        let appSupport = fileManager.urls(for: .applicationSupportDirectory, in: .userDomainMask)[0]
        let caches = fileManager.urls(for: .cachesDirectory, in: .userDomainMask)[0]
        self.importsDirectory = appSupport.appendingPathComponent("ImportedTracks", isDirectory: true)
        self.mastersDirectory = appSupport.appendingPathComponent("RenderedMasters", isDirectory: true)
        self.previewsDirectory = caches.appendingPathComponent("MasteredPreviews", isDirectory: true)
        self.fileManager = fileManager
    }

    /// Tests: everything under one base directory.
    init(baseDirectory: URL, fileManager: FileManager = .default) {
        self.importsDirectory = baseDirectory.appendingPathComponent("ImportedTracks", isDirectory: true)
        self.mastersDirectory = baseDirectory.appendingPathComponent("RenderedMasters", isDirectory: true)
        self.previewsDirectory = baseDirectory.appendingPathComponent("MasteredPreviews", isDirectory: true)
        self.fileManager = fileManager
    }

    /// Delete every preview WAV except the one currently in use.
    func pruneObsoletePreviews(keeping current: URL?) {
        guard let files = try? fileManager.contentsOfDirectory(
            at: previewsDirectory, includingPropertiesForKeys: nil
        ) else { return }
        for file in files where file != current && file.pathExtension.lowercased() == "wav" {
            try? fileManager.removeItem(at: file)
        }
    }

    /// Keep the newest `max` files in a directory by modification date; delete the rest.
    func enforceLimit(in directory: URL, max: Int) {
        guard let files = try? fileManager.contentsOfDirectory(
            at: directory,
            includingPropertiesForKeys: [.contentModificationDateKey]
        ) else { return }
        let sorted = files.sorted {
            let a = (try? $0.resourceValues(forKeys: [.contentModificationDateKey]))?.contentModificationDate ?? .distantPast
            let b = (try? $1.resourceValues(forKeys: [.contentModificationDateKey]))?.contentModificationDate ?? .distantPast
            return a > b
        }
        for file in sorted.dropFirst(max) {
            try? fileManager.removeItem(at: file)
        }
    }
}
