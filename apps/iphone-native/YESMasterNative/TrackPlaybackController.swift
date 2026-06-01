import AVFoundation
import Combine
import Foundation

protocol TrackAudioPlayer: AnyObject {
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

    func play(url: URL) throws {
        try activateForPlayback()

        if loadedURL != url {
            player = try makePlayer(url)
            loadedURL = url
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
