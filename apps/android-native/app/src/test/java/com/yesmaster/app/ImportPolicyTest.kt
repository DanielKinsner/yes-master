package com.yesmaster.app

import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Test

class ImportPolicyTest {
    @Test
    fun extensionlessSafNamesAreAllowedThroughToDecode() {
        assertNull(ImportPolicy.extensionToCheck("content"))
        assertNull(ImportPolicy.extensionToCheck("imported-audio"))
    }

    @Test
    fun displayNameExtensionIsNormalizedBeforeNativeSupportCheck() {
        assertEquals("wav", ImportPolicy.extensionToCheck("My Mix.WAV"))
        assertEquals("mp3", ImportPolicy.extensionToCheck("song.final.mp3"))
    }

    @Test
    fun unsupportedMessageNamesTheSupportedSet() {
        assertEquals(
            ".aiff is not supported yet. Use wav, mp3, m4a, aac, flac, ogg.",
            ImportPolicy.unsupportedMessage("AIFF"),
        )
    }
}
