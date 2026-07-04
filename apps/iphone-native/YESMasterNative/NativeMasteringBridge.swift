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
    let effectiveAdaptiveStrength: Double?
    let sourceProfileDigest: String?
    let confidenceDigest: String?
    let compressionDigest: String?

    init(
        lufsIntegrated: Double,
        truePeakDbtp: Double,
        dynamicRangeLu: Double,
        sampleRate: Int,
        bitDepth: Int,
        effectiveAdaptiveStrength: Double? = nil,
        sourceProfileDigest: String? = nil,
        confidenceDigest: String? = nil,
        compressionDigest: String? = nil
    ) {
        self.lufsIntegrated = lufsIntegrated
        self.truePeakDbtp = truePeakDbtp
        self.dynamicRangeLu = dynamicRangeLu
        self.sampleRate = sampleRate
        self.bitDepth = bitDepth
        self.effectiveAdaptiveStrength = effectiveAdaptiveStrength
        self.sourceProfileDigest = sourceProfileDigest
        self.confidenceDigest = confidenceDigest
        self.compressionDigest = compressionDigest
    }
}

struct NativeRenderOptions: Equatable {
    let preset: String
    let intensity: Float
    let lufsTarget: Float

    static let `default` = NativeRenderOptions(preset: "balanced", intensity: 0.5, lufsTarget: -11)
}

/// Machine-readable class of a Rust bridge error. Decoded from the `code`
/// field every FFI error payload carries (S8.3a). The raw values are the wire
/// contract, pinned on the Rust side by
/// `tests::ffi_error_payloads_carry_stable_codes` (rust/src/lib.rs) and here by
/// `AuditionControllerTests.bridgeErrorCodesMatchTheRustWireContract` — change
/// them together.
enum NativeBridgeErrorCode: String, Decodable, Equatable, CaseIterable {
    case invalidPath = "invalid_path"
    case decode
    case render
    case io
    case `internal`
    case other
}

enum NativeMasteringBridgeError: LocalizedError, Equatable {
    case rust(String, NativeBridgeErrorCode?)
    case invalidResponse

    var errorDescription: String? {
        switch self {
        case .rust(let message, _):
            message
        case .invalidResponse:
            "Rust bridge returned an invalid response."
        }
    }
}

struct NativeMasteringBridge {
    private let knownAudioExtensions = ["wav", "mp3", "m4a", "aac", "flac", "ogg"]

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
            throw NativeMasteringBridgeError.rust(errorPayload.error, errorPayload.errorCode)
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
                        options.intensity,
                        options.lufsTarget
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
            throw NativeMasteringBridgeError.rust(errorPayload.error, errorPayload.errorCode)
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
    /// Stable class string (see `NativeBridgeErrorCode`). Optional so an older
    /// or hand-rolled payload without a code still surfaces its message.
    let code: String?

    var errorCode: NativeBridgeErrorCode? {
        code.flatMap(NativeBridgeErrorCode.init(rawValue:))
    }
}
