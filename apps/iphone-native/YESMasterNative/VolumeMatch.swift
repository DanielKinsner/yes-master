import Foundation

/// dB to apply to `side` so both sides play at the quieter side's loudness.
/// Reference = the quieter of the two, so the returned gain is always <= 0 (never boosts/clips).
func volumeMatchGainDb(sideLufs: Double, otherLufs: Double) -> Double {
    let reference = min(sideLufs, otherLufs)
    return reference - sideLufs
}

func volumeMatchLinearGain(sideLufs: Double, otherLufs: Double) -> Float {
    Float(pow(10.0, volumeMatchGainDb(sideLufs: sideLufs, otherLufs: otherLufs) / 20.0))
}
