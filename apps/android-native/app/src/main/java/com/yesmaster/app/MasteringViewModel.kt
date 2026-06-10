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
    ) : UiState
    data class Done(
        val displayName: String,
        val savedTo: String,
        val measurements: WireMeasurements?,
        /** Kept so "Master again" can return to Ready without re-analyzing. */
        val previous: Ready,
    ) : UiState
    data class Error(val message: String, val previous: Ready? = null) : UiState
}

class MasteringViewModel(application: Application) : AndroidViewModel(application) {

    private val imports = ImportRepository(application)
    private val exports = ExportRepository(application)

    private val _state = MutableStateFlow<UiState>(UiState.Idle)
    val state: StateFlow<UiState> = _state

    fun importTrack(uri: Uri) {
        _state.value = UiState.Working("Importing…")
        viewModelScope.launch {
            try {
                val imported = withContext(Dispatchers.IO) { imports.copyToCache(uri) }
                _state.value = UiState.Working("Analyzing…")
                val analysis = withContext(Dispatchers.IO) {
                    Wire.analysis(NativeBridge.analyzeFileJson(imported.file.absolutePath))
                }
                analysis.error?.let { error(it) }
                _state.value = UiState.Ready(
                    displayName = imported.displayName,
                    sourcePath = imported.file.absolutePath,
                    analysis = analysis,
                )
            } catch (e: Exception) {
                _state.value = UiState.Error(e.message ?: "Import failed")
            }
        }
    }

    fun master(style: StandardStyle, loudness: StandardLoudness, intensity: Float) {
        val ready = _state.value as? UiState.Ready ?: return
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
