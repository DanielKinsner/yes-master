import AVFoundation
import Combine
import Foundation

/// The output graph that pulls PCM from a live stream. The real implementation
/// owns `AVAudioEngine` + one `AVAudioSourceNode`; tests inject a fake so the
/// engine's control logic is exercised without audio hardware.
protocol LiveAudioOutput: AnyObject {
    var isRunning: Bool { get }
    /// (Re)install the render path for `stream`, configure the session, and start.
    func start(pulling stream: LiveAuditionStreaming) throws
    /// Halt output but keep the graph/session so a later `start` resumes cheaply.
    func stop()
}

/// Owns the iPhone live-audition output. Swift owns playback; the Rust stream
/// owns the PCM, cursor, and DSP. One engine, one source node, one stream — so
/// Original/Mastered, seek, and live param changes all run on a single timeline
/// with the playhead preserved (the bypass/seek/param work happens inside the
/// stream; this layer never tears down a second player).
final class LiveAudioEngine: ObservableObject {
    @Published private(set) var isPlaying = false

    private let output: LiveAudioOutput
    private let makeStream: (URL, String, Float, Float) -> LiveAuditionStreaming?
    private(set) var stream: LiveAuditionStreaming?

    init(
        output: LiveAudioOutput = AVAudioEngineOutput(),
        makeStream: @escaping (URL, String, Float, Float) -> LiveAuditionStreaming? = { url, preset, intensity, loudness in
            LiveAuditionBridge(sourceURL: url, preset: preset, intensity: intensity, loudnessTarget: loudness)
        }
    ) {
        self.output = output
        self.makeStream = makeStream
    }

    /// Decode + build the live stream for a track. Does not start playback.
    /// Returns `false` if the file could not be decoded.
    @discardableResult
    func load(url: URL, preset: String, intensity: Float, loudnessTarget: Float) -> Bool {
        output.stop()
        isPlaying = false
        stream = makeStream(url, preset, intensity, loudnessTarget)
        return stream != nil
    }

    func play() throws {
        guard let stream else { return }
        try output.start(pulling: stream)
        isPlaying = true
    }

    /// Pause without discarding the stream, so resuming keeps the playhead.
    func pause() {
        output.stop()
        isPlaying = false
    }

    /// `true` = Original (dry), `false` = Mastered. Playhead is preserved.
    func setOriginal(_ original: Bool) { stream?.setOriginal(original) }

    func setParams(preset: String, intensity: Float, loudnessTarget: Float) {
        stream?.setParams(preset: preset, intensity: intensity, loudnessTarget: loudnessTarget)
    }

    func setVolumeMatch(linearGain: Float) { stream?.setVolumeMatch(linearGain: linearGain) }
    func setLandingGain(linearGain: Float) { stream?.setLandingGain(linearGain: linearGain) }
    func seek(toSeconds seconds: Double) { stream?.seek(toSeconds: seconds) }

    var positionSeconds: Double { stream?.positionSeconds ?? 0 }
    var durationSeconds: Double { stream?.durationSeconds ?? 0 }
}

/// Real `LiveAudioOutput`: an `AVAudioEngine` with a single `AVAudioSourceNode`
/// whose render callback pulls mastered PCM from the Rust stream. The source
/// node's format is pinned to the decoded source rate/channels (interleaved);
/// the engine resamples to the hardware rate for monitoring only, so the live
/// chain runs at the same rate the export path uses.
final class AVAudioEngineOutput: LiveAudioOutput {
    private let engine = AVAudioEngine()
    private var sourceNode: AVAudioSourceNode?
    private var sessionConfigured = false

    var isRunning: Bool { engine.isRunning }

    func start(pulling stream: LiveAuditionStreaming) throws {
        try configureSessionIfNeeded()
        installNode(pulling: stream)
        if !engine.isRunning {
            try engine.start()
        }
    }

    func stop() {
        if engine.isRunning {
            engine.pause()
        }
    }

    private func configureSessionIfNeeded() throws {
        guard !sessionConfigured else { return }
        let session = AVAudioSession.sharedInstance()
        try session.setCategory(.playback, mode: .default, options: [])
        // Preferred values are hints; the OS may pick others. We pin the source
        // node to the decoded rate regardless, so a mismatch is just resampled.
        try? session.setPreferredSampleRate(48_000)
        try? session.setPreferredIOBufferDuration(0.005)
        try session.setActive(true)
        sessionConfigured = true
    }

    private func installNode(pulling stream: LiveAuditionStreaming) {
        if let existing = sourceNode {
            engine.detach(existing)
            sourceNode = nil
        }

        let channels = AVAudioChannelCount(max(1, stream.channelCount))
        let sampleRate = stream.sampleRate > 0 ? stream.sampleRate : 48_000
        guard let format = AVAudioFormat(
            commonFormat: .pcmFormatFloat32,
            sampleRate: sampleRate,
            channels: channels,
            interleaved: true
        ) else { return }

        // RT render callback. Captures `stream` once (no per-call allocation);
        // the body is a single forward into the Rust C ABI. NOTE: this is the
        // hot path to profile in the on-device spike — if existential dispatch
        // shows ARC traffic, capture the concrete handle instead.
        let node = AVAudioSourceNode(format: format) { _, _, frameCount, audioBufferList in
            let buffers = UnsafeMutableAudioBufferListPointer(audioBufferList)
            guard let raw = buffers[0].mData else { return noErr }
            let out = raw.assumingMemoryBound(to: Float.self)
            _ = stream.render(into: out, frames: frameCount)
            return noErr
        }

        engine.attach(node)
        engine.connect(node, to: engine.mainMixerNode, format: format)
        sourceNode = node
    }
}
