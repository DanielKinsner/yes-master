import Foundation
import UniformTypeIdentifiers

struct NativeAnalysisResult: Decodable, Equatable {
    let lufsIntegrated: Double
    let truePeakDbtp: Double
    let dynamicRangeLu: Double
}

struct NativeRenderJob: Decodable, Equatable {
    let outputPaths: [String]
    let measurements: NativeRenderedMeasurements?
}

struct NativeRenderedMeasurements: Decodable, Equatable {
    let lufsIntegrated: Double
    let truePeakDbtp: Double
    let dynamicRangeLu: Double
    let sampleRate: Int
    let bitDepth: Int
}

struct NativeRenderOptions: Equatable {
    let preset: String
    let intensity: Float

    static let `default` = NativeRenderOptions(preset: "balanced", intensity: 0.5)
}

enum NativeMasteringBridgeError: LocalizedError, Equatable {
    case rust(String)
    case invalidResponse

    var errorDescription: String? {
        switch self {
        case .rust(let message):
            message
        case .invalidResponse:
            "Rust bridge returned an invalid response."
        }
    }
}

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

    func analyzeTrack(at url: URL) throws -> NativeAnalysisResult {
        let pointer = url.path.withCString { pathPointer in
            yes_master_native_analyze_file_json(pathPointer)
        }

        guard let pointer else {
            throw NativeMasteringBridgeError.invalidResponse
        }
        defer { yes_master_native_free_string(pointer) }

        let json = String(cString: pointer)
        let data = Data(json.utf8)
        let decoder = JSONDecoder()
        decoder.keyDecodingStrategy = .convertFromSnakeCase

        if let errorPayload = try? decoder.decode(BridgeErrorPayload.self, from: data),
           !errorPayload.error.isEmpty {
            throw NativeMasteringBridgeError.rust(errorPayload.error)
        }

        do {
            return try decoder.decode(NativeAnalysisResult.self, from: data)
        } catch {
            throw NativeMasteringBridgeError.invalidResponse
        }
    }

    func renderMaster(
        from sourceURL: URL,
        toDirectory outputDirectoryURL: URL,
        options: NativeRenderOptions = .default
    ) throws -> NativeRenderJob {
        let pointer = sourceURL.path.withCString { sourcePathPointer in
            outputDirectoryURL.path.withCString { outputDirectoryPointer in
                options.preset.withCString { presetPointer in
                    yes_master_native_render_master_with_options_json(
                        sourcePathPointer,
                        outputDirectoryPointer,
                        presetPointer,
                        options.intensity
                    )
                }
            }
        }

        guard let pointer else {
            throw NativeMasteringBridgeError.invalidResponse
        }
        defer { yes_master_native_free_string(pointer) }

        let json = String(cString: pointer)
        let data = Data(json.utf8)
        let decoder = JSONDecoder()
        decoder.keyDecodingStrategy = .convertFromSnakeCase

        if let errorPayload = try? decoder.decode(BridgeErrorPayload.self, from: data),
           !errorPayload.error.isEmpty {
            throw NativeMasteringBridgeError.rust(errorPayload.error)
        }

        do {
            return try decoder.decode(NativeRenderJob.self, from: data)
        } catch {
            throw NativeMasteringBridgeError.invalidResponse
        }
    }
}

private struct BridgeErrorPayload: Decodable {
    let error: String
}
