package com.yesmaster.app

import android.app.Application
import android.net.Uri
import androidx.lifecycle.AndroidViewModel
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

class MasteringViewModel(application: Application) : AndroidViewModel(application) {

    private val imports = ImportRepository(application)
    private val exports = ExportRepository(application)

    private val _state = MutableStateFlow<UiState>(UiState.Idle)
    val state: StateFlow<UiState> = _state

    fun importTrack(uri: Uri) {
        _state.value = UiState.Working("Importing…")
        viewModelScope.launch {
            val imported = try {
                withContext(Dispatchers.IO) { imports.copyToCache(uri) }
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
            analysis.error?.let { error(it) }
            _state.value = UiState.Ready(
                displayName = displayName,
                sourcePath = sourcePath,
                analysis = analysis,
            )
        } catch (e: Exception) {
            _state.value = UiState.Error(
                message = e.message ?: "Analysis failed",
                retrySourcePath = sourcePath,
                retryDisplayName = displayName,
            )
        }
    }

    fun master(style: StandardStyle, loudness: StandardLoudness, intensity: Float) {
        val ready = (_state.value as? UiState.Ready)
            ?.copy(style = style, loudness = loudness, intensity = intensity)
            ?: return
        _state.value = UiState.Working("Mastering…")
        viewModelScope.launch {
            try {
                val outDir = File(getApplication<Application>().cacheDir, "renders")
                    .apply { mkdirs() }
                val job = withContext(Dispatchers.IO) {
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
                val baseName = ready.displayName.substringBeforeLast('.')
                val published = withContext(Dispatchers.IO) {
                    exports.publishToMusic(rendered, "$baseName (YES Master).wav")
                }
                _state.value = UiState.Done(
                    displayName = ready.displayName,
                    savedTo = published.displayPath,
                    measurements = job.measurements,
                    previous = ready,
                )
            } catch (e: Exception) {
                _state.value = UiState.Error(e.message ?: "Mastering failed", previous = ready)
            }
        }
    }

    fun backToReady() {
        when (val s = _state.value) {
            is UiState.Done -> _state.value = s.previous
            is UiState.Error -> _state.value = s.previous ?: UiState.Idle
            else -> Unit
        }
    }

    fun reset() {
        _state.value = UiState.Idle
    }
}
