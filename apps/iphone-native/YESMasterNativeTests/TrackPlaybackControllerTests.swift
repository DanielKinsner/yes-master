import XCTest
@testable import YES_Master_Native

final class TrackPlaybackControllerTests: XCTestCase {
    func testPlayActivatesAudioSessionBeforeStartingPlayer() throws {
        var events: [String] = []
        let player = FakeTrackAudioPlayer(events: &events)
        let controller = TrackPlaybackController(
            activateForPlayback: { events.append("activate") },
            makePlayer: { _ in player }
        )

        try controller.play(url: URL(fileURLWithPath: "/tmp/example.wav"))

        XCTAssertEqual(events, ["activate", "play"])
        XCTAssertTrue(controller.isPlaying)
    }
}

private final class FakeTrackAudioPlayer: TrackAudioPlayer {
    private let record: (String) -> Void

    init(events: UnsafeMutablePointer<[String]>) {
        record = { events.pointee.append($0) }
    }

    func play() throws {
        record("play")
    }

    func pause() {
        record("pause")
    }
}
