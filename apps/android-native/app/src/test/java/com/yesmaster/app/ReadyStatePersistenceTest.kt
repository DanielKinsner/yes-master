package com.yesmaster.app

import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Test

class ReadyStatePersistenceTest {
    @Test
    fun restoreBuildsReadyStateFromSavedPrimitives() {
        val analysisJson = Wire.gson.toJson(
            WireAnalysis(lufsIntegrated = -12.0, truePeakDbtp = -1.0, dynamicRangeLu = 8.0),
        )

        val ready = ReadyStatePersistence.restore(
            sourcePath = "/cache/imports/song.wav",
            displayName = "song.wav",
            analysisJson = analysisJson,
            styleId = "heavy",
            loudnessId = "high",
            intensity = 1.2f,
        )

        requireNotNull(ready)
        assertEquals("/cache/imports/song.wav", ready.sourcePath)
        assertEquals("song.wav", ready.displayName)
        assertEquals(StandardStyle.HEAVY, ready.style)
        assertEquals(StandardLoudness.HIGH, ready.loudness)
        assertEquals(1.0f, ready.intensity)
        assertEquals(-12.0, ready.analysis.lufsIntegrated, 1e-9)
    }

    @Test
    fun restoreSkipsMissingCoreFieldsOrInvalidJson() {
        assertNull(
            ReadyStatePersistence.restore(
                sourcePath = null,
                displayName = "song.wav",
                analysisJson = "{}",
                styleId = "balanced",
                loudnessId = "medium",
                intensity = 0.5f,
            ),
        )
        assertNull(
            ReadyStatePersistence.restore(
                sourcePath = "/cache/imports/song.wav",
                displayName = "song.wav",
                analysisJson = "{not-json",
                styleId = "balanced",
                loudnessId = "medium",
                intensity = 0.5f,
            ),
        )
    }
}
