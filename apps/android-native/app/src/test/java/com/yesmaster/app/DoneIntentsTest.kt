package com.yesmaster.app

import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Test

class DoneIntentsTest {
    @Test
    fun shareSpecCarriesAudioStreamUri() {
        val uri = "content://media/external/audio/media/42"
        val spec = DoneIntents.shareSpec(uri)

        assertEquals("android.intent.action.SEND", spec.action)
        assertEquals("audio/wav", spec.type)
        assertEquals(uri, spec.savedUri)
        assertTrue(spec.streamExtra)
    }

    @Test
    fun playSpecViewsAudioUri() {
        val uri = "content://media/external/audio/media/42"
        val spec = DoneIntents.playSpec(uri)

        assertEquals("android.intent.action.VIEW", spec.action)
        assertEquals("audio/wav", spec.type)
        assertEquals(uri, spec.savedUri)
        assertTrue(!spec.streamExtra)
    }
}
