import Foundation

struct NativeMasteringBridge {
    let supportedImportExtensions = ["wav", "mp3", "m4a", "aac", "flac", "ogg"]

    var supportedImportSummary: String {
        "Supported now: " + supportedImportExtensions.joined(separator: ", ")
    }

    var fixedExportSummary: String {
        "Rust bridge scaffold: Medium maps to -11 LUFS, WAV 44.1 kHz, 24-bit, -1 dBTP."
    }
}
