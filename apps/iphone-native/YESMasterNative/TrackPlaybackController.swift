import AVFoundation
import Combine
import Foundation

protocol TrackAudioPlayer: AnyObject {
    var currentTime: TimeInterval { get set }

    func play() throws
    func pause()
}

enum TrackPlaybackError: LocalizedError, Equatable {
    case playbackCouldNotStart

    var errorDescription: String? {
        switch self {
        case .playbackCouldNotStart:
            "Playback could not start."
        }
    }
}

final class TrackPlaybackController: ObservableObject {
    @Published private(set) var isPlaying = false

    private let activateForPlayback: () throws -> Void
    private let makePlayer: (URL) throws -> any TrackAudioPlayer
    private var player: (any TrackAudioPlayer)?
    private var loadedURL: URL?

    var currentTime: TimeInterval {
        player?.currentTime ?? 0
    }

    init(
        activateForPlayback: @escaping () throws -> Void = {
            try AudioSessionController().activateForPlayback()
        },
        makePlayer: @escaping (URL) throws -> any TrackAudioPlayer = {
            try AVFoundationTrackAudioPlayer(url: $0)
        }
    ) {
        self.activateForPlayback = activateForPlayback
        self.makePlayer = makePlayer
    }

    func play(url: URL, startingAt startTime: TimeInterval? = nil) throws {
        try activateForPlayback()

        if loadedURL != url {
            player = try makePlayer(url)
            loadedURL = url
        }

        if let startTime {
            player?.currentTime = max(0, startTime)
        }

        try player?.play()
        isPlaying = true
    }

    func pause() {
        player?.pause()
        isPlaying = false
    }
}

private final class AVFoundationTrackAudioPlayer: TrackAudioPlayer {
    private let player: AVAudioPlayer

    var currentTime: TimeInterval {
        get { player.currentTime }
        set { player.currentTime = newValue }
    }

    init(url: URL) throws {
        player = try AVAudioPlayer(contentsOf: url)
        player.prepareToPlay()
    }

    func play() throws {
        guard player.play() else {
            throw TrackPlaybackError.playbackCouldNotStart
        }
    }

    func pause() {
        player.pause()
    }
}
