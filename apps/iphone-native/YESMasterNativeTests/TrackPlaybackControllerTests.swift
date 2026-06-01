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

    func testPlayAppliesRequestedVolume() throws {
        var events: [String] = []
        let player = FakeTrackAudioPlayer(events: &events)
        let controller = TrackPlaybackController(
            activateForPlayback: { events.append("activate") },
            makePlayer: { _ in player }
        )
        try controller.play(url: URL(fileURLWithPath: "/tmp/x.wav"), volume: 0.5)
        XCTAssertEqual(player.volume, 0.5)
    }

    func testPlayCanStartReplacementTrackAtExistingTime() throws {
        var events: [String] = []
        let firstPlayer = FakeTrackAudioPlayer(events: &events)
        let secondPlayer = FakeTrackAudioPlayer(events: &events)
        var players = [firstPlayer, secondPlayer]
        let controller = TrackPlaybackController(
            activateForPlayback: { events.append("activate") },
            makePlayer: { _ in players.removeFirst() }
        )

        try controller.play(url: URL(fileURLWithPath: "/tmp/original.wav"))
        firstPlayer.currentTime = 37.5
        try controller.play(
            url: URL(fileURLWithPath: "/tmp/mastered.wav"),
            startingAt: controller.currentTime
        )

        XCTAssertEqual(secondPlayer.currentTime, 37.5)
        XCTAssertTrue(controller.isPlaying)
    }
}

private final class FakeTrackAudioPlayer: TrackAudioPlayer {
    private let record: (String) -> Void
    var currentTime: TimeInterval = 0
    var volume: Float = 1.0

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
