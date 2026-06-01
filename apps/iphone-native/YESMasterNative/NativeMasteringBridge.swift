import Foundation
import UniformTypeIdentifiers

struct NativeMasteringBridge {
    private let knownAudioExtensions = ["wav", "mp3", "m4a", "aac", "flac", "ogg", "aiff", "aif", "opus"]

    var bridgeVersion: String {
        guard let pointer = yes_master_native_bridge_version() else {
            return "Rust bridge unavailable"
        }
        return String(cString: pointer)
    }

    var supportedImportExtensions: [String] {
        knownAudioExtensions.filter { fileExtension in
            fileExtension.withCString { pointer in
                yes_master_native_supports_import_extension(pointer)
            }
        }
    }

    var supportedImportContentTypes: [UTType] {
        supportedImportExtensions.map { fileExtension in
            UTType(filenameExtension: fileExtension)
                ?? UTType(importedAs: "com.yesmaster.audio.\(fileExtension)", conformingTo: .audio)
        }
    }

    var supportedImportSummary: String {
        "\(bridgeVersion). Supported now: " + supportedImportExtensions.joined(separator: ", ")
    }

    var fixedExportSummary: String {
        guard let pointer = yes_master_native_fixed_export_settings_json() else {
            return "Rust bridge could not load export settings."
        }
        defer { yes_master_native_free_string(pointer) }
        return String(cString: pointer)
    }
}
