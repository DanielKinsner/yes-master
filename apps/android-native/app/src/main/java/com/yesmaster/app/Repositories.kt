package com.yesmaster.app

import android.content.ContentValues
import android.content.Context
import android.net.Uri
import android.provider.MediaStore
import android.provider.OpenableColumns
import java.io.File

/**
 * SAF import: scoped storage hands us content:// URIs the Rust engine can't
 * open, so the MVP copies the picked file into app cache and passes a real
 * path across JNI. File-descriptor passthrough is a later optimization
 * (spec risk #4).
 */
class ImportRepository(private val context: Context) {

    data class Imported(val file: File, val displayName: String)

    fun displayName(uri: Uri): String = queryDisplayName(uri) ?: "imported-audio"

    fun copyToCache(uri: Uri, displayName: String = displayName(uri)): Imported {
        val safeName = displayName.replace(Regex("[^A-Za-z0-9._-]"), "-")
        val dir = File(context.cacheDir, "imports").apply { mkdirs() }
        val target = File(dir, "${System.currentTimeMillis()}-$safeName")
        context.contentResolver.openInputStream(uri)?.use { input ->
            target.outputStream().use { output -> input.copyTo(output) }
        } ?: error("Could not open the selected file")
        return Imported(target, displayName)
    }

    private fun queryDisplayName(uri: Uri): String? =
        context.contentResolver.query(uri, null, null, null, null)?.use { cursor ->
            val index = cursor.getColumnIndex(OpenableColumns.DISPLAY_NAME)
            if (index >= 0 && cursor.moveToFirst()) cursor.getString(index) else null
        }
}

object ImportPolicy {
    val supportedImportExtensions = listOf("wav", "mp3", "m4a", "aac", "flac", "ogg")

    fun extensionToCheck(displayName: String): String? {
        val extension = displayName.substringAfterLast('.', missingDelimiterValue = "")
            .trim()
            .lowercase()
        return extension.ifEmpty { null }
    }

    fun unsupportedMessage(extension: String): String =
        ".${extension.lowercase()} is not supported yet. Use ${supportedImportExtensions.joinToString(", ")}."
}

/**
 * Publishes a rendered master from app cache into the device music library
 * (Music/YES Master) via MediaStore — permission-free for our own files on
 * API 29+, and the closest Android analog to the desktop's "exports never
 * overwrite" rule (MediaStore uniquifies names for us).
 */
class ExportRepository(private val context: Context) {

    data class Published(val uri: Uri, val displayPath: String)

    fun publishToMusic(rendered: File, displayName: String): Published {
        val values = ContentValues().apply {
            put(MediaStore.Audio.Media.DISPLAY_NAME, displayName)
            put(MediaStore.Audio.Media.MIME_TYPE, "audio/wav")
            put(MediaStore.Audio.Media.RELATIVE_PATH, "Music/YES Master")
            put(MediaStore.Audio.Media.IS_PENDING, 1)
        }
        val resolver = context.contentResolver
        val collection = MediaStore.Audio.Media.getContentUri(MediaStore.VOLUME_EXTERNAL_PRIMARY)
        val uri = resolver.insert(collection, values) ?: error("MediaStore insert failed")
        try {
            resolver.openOutputStream(uri)?.use { output ->
                rendered.inputStream().use { input -> input.copyTo(output) }
            } ?: error("Could not write to the music library")
            values.clear()
            values.put(MediaStore.Audio.Media.IS_PENDING, 0)
            resolver.update(uri, values, null, null)
        } catch (e: Exception) {
            // A failed copy or finalize (disk full, source gone, provider
            // error) must not leave an invisible IS_PENDING row orphaned
            // until the OS reaps it — delete the row, then surface the
            // original failure.
            runCatching { resolver.delete(uri, null, null) }
            throw e
        }
        // MediaStore uniquifies on collision ("song (YES Master) (1).wav" on
        // a re-master), so the receipt must name the row that was actually
        // created, not the requested name. A failed read-back must not nuke
        // the published master — fall back to the requested name instead.
        val actualName = runCatching {
            resolver.query(
                uri,
                arrayOf(MediaStore.Audio.Media.DISPLAY_NAME),
                null,
                null,
                null,
            )?.use { cursor -> if (cursor.moveToFirst()) cursor.getString(0) else null }
        }.getOrNull() ?: displayName
        return Published(uri, "Music/YES Master/$actualName")
    }
}
