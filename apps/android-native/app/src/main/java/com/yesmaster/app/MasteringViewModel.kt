package com.yesmaster.app

import android.app.Application
import android.net.Uri
import androidx.lifecycle.AndroidViewModel
import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import java.io.File

/** One linear flow for the MVP: import → analyze → choose → master → done. */
sealed interface UiState {
    data object Idle : UiState
    data class Working(val label: String) : UiState
    data class Ready(
        val displayName: String,
        val sourcePath: String,
        val analysis: WireAnalysis,
        // The last choices made on this track, so "Master again" reopens the
        // screen on what the user actually rendered with — these live in the
        // state (not in composable `remember`) because the Ready screen
        // leaves the composition during Working/Done.
        val style: StandardStyle = StandardStyle.BALANCED,
        val loudness: StandardLoudness = StandardLoudness.MEDIUM,
        val intensity: Float = 0.5f,
    ) : UiState
    data class Done(
        val displayName: String,
        val savedTo: String,
        val savedUri: String,
        val measurements: WireMeasurements?,
        /** Kept so "Master again" can return to Ready without re-analyzing. */
        val previous: Ready,
    ) : UiState
    data class Error(
        val message: String,
        val previous: Ready? = null,
        /** Set when analysis failed AFTER the import already cached the
         *  file — lets the user retry without re-picking through SAF. */
        val retrySourcePath: String? = null,
        val retryDisplayName: String? = null,
    ) : UiState
}

class MasteringViewModel(
    application: Application,
    private val savedState: SavedStateHandle,
) : AndroidViewModel(application) {

    private val imports = ImportRepository(application)
    private val exports = ExportRepository(application)

    /** Live audition for the Ready screen — same Rust chain as the render. */
    val audition = AuditionController(application, viewModelScope)

    private val _state = MutableStateFlow<UiState>(UiState.Idle)
    val state: StateFlow<UiState> = _state

    init {
        restoreReadyFromSavedState()?.let { ready ->
            _state.value = ready
            armAudition(ready)
        }
    }

    fun importTrack(uri: Uri) {
        audition.release()
        _state.value = UiState.Working("Importing…")
        viewModelScope.launch {
            val displayName = try {
                withContext(Dispatchers.IO) { imports.displayName(uri) }
            } catch (e: Exception) {
                _state.value = UiState.Error(e.message ?: "Could not read the selected file name.")
                return@launch
            }
            val extension = ImportPolicy.extensionToCheck(displayName)
            if (extension != null) {
                val supported = try {
                    NativeBridge.supportsImportExtension(extension)
                } catch (e: LinkageError) {
                    _state.value = UiState.Error(nativeBridgeUnavailableMessage(e))
                    return@launch
                }
                if (!supported) {
                    _state.value = UiState.Error(ImportPolicy.unsupportedMessage(extension))
                    return@launch
                }
            }
            val imported = try {
                withContext(Dispatchers.IO) { imports.copyToCache(uri, displayName) }
            } catch (e: Exception) {
                _state.value = UiState.Error(e.message ?: "Import failed")
                return@launch
            }
            analyze(imported.file.absolutePath, imported.displayName)
        }
    }

    fun retryAnalysis() {
        val error = _state.value as? UiState.Error ?: return
        val path = error.retrySourcePath ?: return
        val name = error.retryDisplayName ?: "imported-audio"
        viewModelScope.launch { analyze(path, name) }
    }

    private suspend fun analyze(sourcePath: String, displayName: String) {
        _state.value = UiState.Working("Analyzing…")
        try {
            val analysis = withContext(Dispatchers.IO) {
                Wire.analysis(NativeBridge.analyzeFileJson(sourcePath))
            }
            analysis.error?.let {
                _state.value = UiState.Error(message = it)
                return
            }
            val ready = UiState.Ready(
                displayName = displayName,
                sourcePath = sourcePath,
                analysis = analysis,
            )
            persistReady(ready)
            _state.value = ready
            armAudition(ready)
        } catch (e: LinkageError) {
            _state.value = UiState.Error(
                message = nativeBridgeUnavailableMessage(e),
                retrySourcePath = sourcePath,
                retryDisplayName = displayName,
            )
        } catch (e: Exception) {
            _state.value = UiState.Error(
                message = e.message ?: "Analysis failed",
                retrySourcePath = sourcePath,
                retryDisplayName = displayName,
            )
        }
    }

    /** Live-retune the audition chain as the user tweaks the Ready screen. */
    fun auditionParams(style: StandardStyle, loudness: StandardLoudness, intensity: Float) {
        audition.updateParams(style.id, intensity, loudness.lufs)
    }

    private fun armAudition(ready: UiState.Ready) {
        audition.attach(
            path = ready.sourcePath,
            analysisLufs = ready.analysis.lufsIntegrated,
            preset = ready.style.id,
            intensity = ready.intensity,
            lufsTarget = ready.loudness.lufs,
        )
    }

    fun master(style: StandardStyle, loudness: StandardLoudness, intensity: Float) {
        val ready = (_state.value as? UiState.Ready)
            ?.copy(style = style, loudness = loudness, intensity = intensity)
            ?: return
        persistReady(ready)
        // The render shares the process with the live chain; pausing keeps
        // the audition handle warm for "Master again" without competing for
        // the audio path mid-render.
        audition.pause()
        _state.value = UiState.Working("Mastering…")
        viewModelScope.launch {
            try {
                val job = withContext(Dispatchers.IO) {
                    val outDir = File(getApplication<Application>().cacheDir, "renders")
                        .apply { mkdirs() }
                    Wire.renderJob(
                        NativeBridge.renderMasterWithOptionsJson(
                            sourcePath = ready.sourcePath,
                            outputDir = outDir.absolutePath,
                            preset = style.id,
                            intensity = intensity,
                            lufsTarget = loudness.lufs,
                        )
                    )
                }
                job.error?.let { error(it) }
                val rendered = job.outputPaths.firstOrNull()?.let(::File)
                    ?: error("Render produced no output file")
                // SAF display names arrive raw from the provider; strip path
                // separators so MediaStore can't reject the export name
                // (the import side already sanitizes its cache copy).
                val baseName = ready.displayName.substringBeforeLast('.')
                    .replace(Regex("[/\\\\]"), "-")
                val published = withContext(Dispatchers.IO) {
                    try {
                        exports.publishToMusic(rendered, "$baseName (YES Master).wav")
                    } finally {
                        // The cache copy has no further consumer once
                        // MediaStore owns (or rejected) the master.
                        rendered.delete()
                    }
                }
                _state.value = UiState.Done(
                    displayName = ready.displayName,
                    savedTo = published.displayPath,
                    savedUri = published.uri.toString(),
                    measurements = job.measurements,
                    previous = ready,
                )
            } catch (e: LinkageError) {
                _state.value = UiState.Error(nativeBridgeUnavailableMessage(e), previous = ready)
            } catch (e: Exception) {
                _state.value = UiState.Error(masteringFailureMessage(e), previous = ready)
            }
        }
    }

    fun backToReady() {
        when (val s = _state.value) {
            is UiState.Done -> {
                _state.value = s.previous
                armAudition(s.previous)
            }
            is UiState.Error -> {
                _state.value = s.previous ?: UiState.Idle
                s.previous?.let(::armAudition)
            }
            else -> Unit
        }
    }

    fun reset() {
        audition.release()
        clearSavedReady()
        _state.value = UiState.Idle
    }

    override fun onCleared() {
        audition.release()
    }

    private fun nativeBridgeUnavailableMessage(error: LinkageError): String =
        "Native audio bridge could not load. Reinstall YES Master and try again. (${error.javaClass.simpleName})"

    private fun masteringFailureMessage(error: Exception): String = when (error) {
        is SecurityException -> "Android blocked access to the music library. Check storage permissions and try again."
        is IllegalArgumentException -> "Android could not create that music-library entry. Try a shorter track name."
        else -> error.message ?: "Mastering failed"
    }

    private fun persistReady(ready: UiState.Ready) {
        savedState[ReadyStatePersistence.KEY_SOURCE_PATH] = ready.sourcePath
        savedState[ReadyStatePersistence.KEY_DISPLAY_NAME] = ready.displayName
        savedState[ReadyStatePersistence.KEY_ANALYSIS_JSON] = Wire.gson.toJson(ready.analysis)
        savedState[ReadyStatePersistence.KEY_STYLE_ID] = ready.style.id
        savedState[ReadyStatePersistence.KEY_LOUDNESS_ID] = ready.loudness.id
        savedState[ReadyStatePersistence.KEY_INTENSITY] = ready.intensity
    }

    private fun restoreReadyFromSavedState(): UiState.Ready? {
        val ready = ReadyStatePersistence.restore(
            sourcePath = savedState[ReadyStatePersistence.KEY_SOURCE_PATH],
            displayName = savedState[ReadyStatePersistence.KEY_DISPLAY_NAME],
            analysisJson = savedState[ReadyStatePersistence.KEY_ANALYSIS_JSON],
            styleId = savedState[ReadyStatePersistence.KEY_STYLE_ID],
            loudnessId = savedState[ReadyStatePersistence.KEY_LOUDNESS_ID],
            intensity = savedState[ReadyStatePersistence.KEY_INTENSITY],
        )
        if (ready == null || !File(ready.sourcePath).exists()) {
            clearSavedReady()
            return null
        }
        return ready
    }

    private fun clearSavedReady() {
        ReadyStatePersistence.keys.forEach { key -> savedState.remove<Any>(key) }
    }
}

object ReadyStatePersistence {
    const val KEY_SOURCE_PATH = "ready.sourcePath"
    const val KEY_DISPLAY_NAME = "ready.displayName"
    const val KEY_ANALYSIS_JSON = "ready.analysisJson"
    const val KEY_STYLE_ID = "ready.styleId"
    const val KEY_LOUDNESS_ID = "ready.loudnessId"
    const val KEY_INTENSITY = "ready.intensity"

    val keys = listOf(
        KEY_SOURCE_PATH,
        KEY_DISPLAY_NAME,
        KEY_ANALYSIS_JSON,
        KEY_STYLE_ID,
        KEY_LOUDNESS_ID,
        KEY_INTENSITY,
    )

    fun restore(
        sourcePath: String?,
        displayName: String?,
        analysisJson: String?,
        styleId: String?,
        loudnessId: String?,
        intensity: Float?,
    ): UiState.Ready? {
        if (sourcePath.isNullOrBlank() || displayName.isNullOrBlank() || analysisJson.isNullOrBlank()) {
            return null
        }
        // Wire.analysis no longer THROWS on malformed JSON — it returns an
        // error-bearing default (§6) — so skip the restore when the parse failed
        // or flagged an error, rather than resurrecting a default-zero analysis.
        val analysis = runCatching { Wire.analysis(analysisJson) }.getOrNull()
        if (analysis == null || analysis.error != null) {
            return null
        }
        return UiState.Ready(
            displayName = displayName,
            sourcePath = sourcePath,
            analysis = analysis,
            style = StandardStyle.entries.firstOrNull { it.id == styleId } ?: StandardStyle.BALANCED,
            loudness = StandardLoudness.entries.firstOrNull { it.id == loudnessId }
                ?: StandardLoudness.MEDIUM,
            intensity = intensity?.coerceIn(0f, 1f) ?: 0.5f,
        )
    }
}
