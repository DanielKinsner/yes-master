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
        stream.snapControlsToTargets()
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
/// whose render callback pulls mastered PCM from the Rust stream. AVAudioEngine
/// accepts a standard non-interleaved Float32 source format on device; Rust still
/// renders interleaved PCM into a preallocated scratch buffer, and this layer
/// copies each channel into the node's buffers.
final class AVAudioEngineOutput: LiveAudioOutput {
    private static let maxRenderScratchFrames = 16_384

    private let engine = AVAudioEngine()
    private var sourceNode: AVAudioSourceNode?
    private var sessionConfigured = false
    /// Preallocated interleaved scratch. The Rust stream renders interleaved
    /// here; the render block then deinterleaves into AVAudioEngine's per-channel
    /// buffers. The engine's buses reject interleaved formats (OSStatus -10868),
    /// so the node itself runs a standard deinterleaved float format.
    private var scratch: UnsafeMutableBufferPointer<Float>?

    var isRunning: Bool { engine.isRunning }

    deinit { scratch?.deallocate() }

    static func makeSourceFormat(sampleRate: Double, channels: Int) -> AVAudioFormat? {
        AVAudioFormat(
            commonFormat: .pcmFormatFloat32,
            sampleRate: sampleRate > 0 ? sampleRate : 48_000,
            channels: AVAudioChannelCount(max(1, channels)),
            interleaved: false
        )
    }

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

        let channelCount = max(1, stream.channelCount)
        let sampleRate = stream.sampleRate > 0 ? stream.sampleRate : 48_000
        guard let format = Self.makeSourceFormat(sampleRate: sampleRate, channels: channelCount) else { return }
        prepareScratch(channels: channelCount)
        guard let scratchBase = scratch?.baseAddress else { return }
        let scratchCapacityFrames = Self.maxRenderScratchFrames

        let node = AVAudioSourceNode(format: format) { _, _, frameCount, audioBufferList in
            let frames = Int(frameCount)
            let buffers = UnsafeMutableAudioBufferListPointer(audioBufferList)
            guard frames <= scratchCapacityFrames, buffers.count >= channelCount else {
                Self.clear(buffers: buffers, frames: frames)
                return noErr
            }

            let written = min(Int(stream.render(into: scratchBase, frames: frameCount)), frames)
            for channel in 0..<channelCount {
                guard let raw = buffers[channel].mData else {
                    Self.clear(buffers: buffers, frames: frames)
                    return noErr
                }
                let availableSamples = Int(buffers[channel].mDataByteSize) / MemoryLayout<Float>.size
                guard availableSamples >= frames else {
                    Self.clear(buffers: buffers, frames: frames)
                    return noErr
                }

                let destination = raw.assumingMemoryBound(to: Float.self)
                for frame in 0..<written {
                    destination[frame] = scratchBase[frame * channelCount + channel]
                }
                if written < frames {
                    for frame in written..<frames {
                        destination[frame] = 0.0
                    }
                }
            }

            if buffers.count > channelCount {
                for channel in channelCount..<buffers.count {
                    guard let raw = buffers[channel].mData else { continue }
                    let destination = raw.assumingMemoryBound(to: Float.self)
                    let availableSamples = min(frames, Int(buffers[channel].mDataByteSize) / MemoryLayout<Float>.size)
                    for frame in 0..<availableSamples {
                        destination[frame] = 0.0
                    }
                }
            }
            return noErr
        }

        engine.attach(node)
        engine.connect(node, to: engine.mainMixerNode, format: format)
        sourceNode = node
    }

    private func prepareScratch(channels: Int) {
        scratch?.deallocate()
        let capacity = Self.maxRenderScratchFrames * max(1, channels)
        scratch = UnsafeMutableBufferPointer<Float>.allocate(capacity: capacity)
        scratch?.initialize(repeating: 0.0)
    }

    private static func clear(buffers: UnsafeMutableAudioBufferListPointer, frames: Int) {
        for index in 0..<buffers.count {
            guard let raw = buffers[index].mData else { continue }
            let destination = raw.assumingMemoryBound(to: Float.self)
            let availableSamples = min(frames, Int(buffers[index].mDataByteSize) / MemoryLayout<Float>.size)
            for sample in 0..<availableSamples {
                destination[sample] = 0.0
            }
        }
    }
}
