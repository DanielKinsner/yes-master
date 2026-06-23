import Foundation

/// dB to apply to `side` so both sides play at the quieter side's loudness.
/// Reference = the quieter of the two, so the returned gain is always <= 0 (never boosts/clips).
func volumeMatchGainDb(sideLufs: Double, otherLufs: Double) -> Double {
    let reference = min(sideLufs, otherLufs)
    return reference - sideLufs
}

func volumeMatchLinearGain(sideLufs: Double, otherLufs: Double) -> Float {
    // §2 — non-finite inputs (e.g. -inf LUFS for digital silence) fall back to
    // unity, mirroring the Android-Rust copy (audition.rs volume_match_linear_gain)
    // so a non-finite measured LUFS can never produce a non-finite gain.
    guard sideLufs.isFinite, otherLufs.isFinite else { return 1.0 }
    return Float(pow(10.0, volumeMatchGainDb(sideLufs: sideLufs, otherLufs: otherLufs) / 20.0))
}
