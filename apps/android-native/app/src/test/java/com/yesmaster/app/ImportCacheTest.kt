package com.yesmaster.app

import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test
import java.io.File
import kotlin.io.path.createTempDirectory

class ImportCacheTest {
    @Test
    fun pruneKeepingDeletesOlderImportSiblingsOnly() {
        val dir = createTempDirectory(prefix = "yes-master-imports").toFile()
        try {
            val keep = File(dir, "current.wav").apply { writeText("current") }
            val old = File(dir, "old.wav").apply { writeText("old") }
            val oldDir = File(dir, "old-dir").apply { mkdirs() }
            File(oldDir, "nested.wav").writeText("nested")

            ImportCache.pruneKeeping(keep)

            assertTrue(keep.exists())
            assertFalse(old.exists())
            assertFalse(oldDir.exists())
        } finally {
            dir.deleteRecursively()
        }
    }
}
