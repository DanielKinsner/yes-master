package com.yesmaster.app

import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test

/**
 * Pins the pure audition decisions (no native library on the JVM lane) and
 * the landing wire decode. The gain FORMULA itself is single-sourced in
 * Rust and pinned by the android crate's host tests against the iPhone's
 * VolumeMatch.swift values; what Kotlin owns is WHICH side feeds it and the
 * transport-edge epsilons, mirrored from the iPhone AuditionController.
 */
class AuditionLogicTest {

    @Test
    fun volumeMatchUsesTheHeardSideAgainstTheOther() {
        // Listening to Original: original is the side being attenuated.
        assertEquals(
            -8.0 to -11.5,
            AuditionMath.volumeMatchSides(
                listeningOriginal = true,
                originalLufs = -8.0,
                masteredLufs = -11.5,
            ),
        )
        // Listening to Mastered: mastered becomes the heard side.
        assertEquals(
            -11.5 to -8.0,
            AuditionMath.volumeMatchSides(
                listeningOriginal = false,
                originalLufs = -8.0,
                masteredLufs = -11.5,
            ),
        )
    }

    @Test
    fun playAtTheEndRestartsFromTheTop() {
        // iPhone epsilon: within 50 ms of the end counts as parked.
        assertTrue(AuditionMath.shouldRestartFromTop(199.96, 200.0))
        assertFalse(AuditionMath.shouldRestartFromTop(199.90, 200.0))
        assertFalse(AuditionMath.shouldRestartFromTop(0.0, 200.0))
        // No duration yet (stream not prepared): never restart-seek.
        assertFalse(AuditionMath.shouldRestartFromTop(0.0, 0.0))
    }

    @Test
    fun reachingTheEndAutoPauses() {
        // iPhone tick epsilon: 20 ms.
        assertTrue(AuditionMath.reachedEnd(199.99, 200.0))
        assertFalse(AuditionMath.reachedEnd(199.90, 200.0))
        assertFalse(AuditionMath.reachedEnd(5.0, 0.0))
    }

    /** Decode side of the Rust bridge's landing payload (the producer keys
     *  are pinned by landing_json_carries_every_key_kotlin_decodes). */
    @Test
    fun landingDecodesTheBridgePayloadShapes() {
        val happy = Wire.landing("""{"gain_lin":0.84,"mastered_lufs":-11.2}""")
        assertEquals(0.84f, happy.gainLin, 1e-6f)
        assertEquals(-11.2, happy.masteredLufs!!, 1e-9)
        assertNull(happy.error)

        // Unavailable measurement: the Rust side maps its non-finite
        // sentinel to JSON null.
        val unavailable = Wire.landing("""{"gain_lin":1.0,"mastered_lufs":null}""")
        assertEquals(1.0f, unavailable.gainLin, 1e-6f)
        assertNull(unavailable.masteredLufs)

        val error = Wire.landing("""{"error":"invalid audition handle"}""")
        assertEquals("invalid audition handle", error.error)
    }
}
