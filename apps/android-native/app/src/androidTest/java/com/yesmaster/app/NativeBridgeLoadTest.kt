package com.yesmaster.app

import androidx.test.ext.junit.runners.AndroidJUnit4
import org.junit.Assert.assertTrue
import org.junit.Test
import org.junit.runner.RunWith

/** Device/emulator smoke gate for the exact failure a tester reported. */
@RunWith(AndroidJUnit4::class)
class NativeBridgeLoadTest {
    @Test
    fun packagedBridgeLoadsAndRecognizesMp3() {
        assertTrue(NativeBridge.supportsImportExtension("mp3"))
    }
}
