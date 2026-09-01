package com.yesmaster.app

import org.junit.Assert.assertEquals
import org.junit.Test

class AnalysisPresentationTest {
    @Test
    fun fastAnalysisHoldsTheProcessingMomentForThreeSeconds() {
        assertEquals(2_880L, remainingAnalysisPresentationMillis(1_000L, 1_120L))
    }

    @Test
    fun naturallyLongAnalysisIsNeverDelayed() {
        assertEquals(0L, remainingAnalysisPresentationMillis(1_000L, 4_500L))
    }
}
