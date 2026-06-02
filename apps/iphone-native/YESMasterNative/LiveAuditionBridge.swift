import Foundation

/// Swift-facing live-audition stream. The real implementation
/// (``LiveAuditionBridge``) wraps the Rust C ABI; tests inject a fake.
///
/// `render(into:frames:)` is called from the `AVAudioSourceNode` render thread
/// and must stay allocation/lock free (it forwards straight to one C call).
/// Every other member is for non-audio (UI) threads. The underlying Rust handle
/// is internally synchronized, so the two thread classes can share one stream.
protocol LiveAuditionStreaming: AnyObject {
    var channelCount: Int { get }
    var sampleRate: Double { get }
    var durationSeconds: Double { get }
    var positionSeconds: Double { get }

    /// Fill `buffer` with up to `frames` interleaved frames; returns frames
    /// written (short at end-of-file, remainder zero-filled). Real-time safe.
    func render(into buffer: UnsafeMutablePointer<Float>, frames: UInt32) -> UInt32

    /// `true` = play Original (dry); `false` = Mastered. Playhead is preserved.
    func setOriginal(_ original: Bool)
    func setParams(preset: String, intensity: Float, loudnessTarget: Float)
    /// Audition-only Volume Match gain (linear, `1.0` = unity). Never exported.
    func setVolumeMatch(linearGain: Float)
    /// Loudness landing gain (linear, `1.0` = unity) for the Mastered path.
    func setLandingGain(linearGain: Float)
    func seek(toSeconds seconds: Double)

    /// Measure the loudness landing for the given Simple controls — off the audio
    /// thread. Returns the linear landing gain and the resulting mastered
    /// integrated LUFS (for audition Volume Match).
    func measureLanding(preset: String, intensity: Float, loudnessTarget: Float) -> (gain: Float, masteredLufs: Double)
}

/// Concrete `LiveAuditionStreaming` backed by the Rust live-stream C ABI. Owns
/// the handle and frees it on `deinit`. This is the ONLY type that touches the
/// raw FFI pointer.
final class LiveAuditionBridge: LiveAuditionStreaming {
    private let handle: OpaquePointer

    /// Decode `sourceURL` and build a live handle initialized with the Simple
    /// controls. Returns `nil` if the file is missing or cannot be decoded.
    init?(sourceURL: URL, preset: String, intensity: Float, loudnessTarget: Float) {
        let created = sourceURL.path.withCString { pathPointer in
            preset.withCString { presetPointer in
                yes_master_native_live_create(pathPointer, presetPointer, intensity, loudnessTarget)
            }
        }
        guard let created else { return nil }
        handle = created
    }

    deinit {
        yes_master_native_live_destroy(handle)
    }

    var channelCount: Int { Int(yes_master_native_live_channels(handle)) }
    var sampleRate: Double { yes_master_native_live_sample_rate(handle) }
    var durationSeconds: Double { yes_master_native_live_duration_seconds(handle) }
    var positionSeconds: Double { yes_master_native_live_position_seconds(handle) }

    func render(into buffer: UnsafeMutablePointer<Float>, frames: UInt32) -> UInt32 {
        yes_master_native_live_process(handle, buffer, frames)
    }

    func setOriginal(_ original: Bool) {
        yes_master_native_live_set_bypass(handle, original)
    }

    func setParams(preset: String, intensity: Float, loudnessTarget: Float) {
        preset.withCString { presetPointer in
            yes_master_native_live_set_params(handle, presetPointer, intensity, loudnessTarget)
        }
    }

    func setVolumeMatch(linearGain: Float) {
        yes_master_native_live_set_volume_match(handle, linearGain)
    }

    func setLandingGain(linearGain: Float) {
        yes_master_native_live_set_landing_gain(handle, linearGain)
    }

    func seek(toSeconds seconds: Double) {
        yes_master_native_live_seek(handle, seconds)
    }

    func measureLanding(preset: String, intensity: Float, loudnessTarget: Float) -> (gain: Float, masteredLufs: Double) {
        var masteredLufs: Float = -.infinity
        let gain = preset.withCString { pointer in
            yes_master_native_live_measure_landing(handle, pointer, intensity, loudnessTarget, &masteredLufs)
        }
        return (gain, Double(masteredLufs))
    }
}
