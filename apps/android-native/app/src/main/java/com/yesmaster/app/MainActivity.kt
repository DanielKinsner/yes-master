package com.yesmaster.app

import android.content.Context
import android.content.Intent
import android.net.Uri
import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.rememberLauncherForActivityResult
import androidx.activity.compose.setContent
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.selection.selectable
import androidx.compose.foundation.selection.toggleable
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.Button
import androidx.compose.material3.Card
import androidx.compose.material3.CardDefaults
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedButton
import androidx.compose.material3.SegmentedButton
import androidx.compose.material3.SegmentedButtonDefaults
import androidx.compose.material3.SingleChoiceSegmentedButtonRow
import androidx.compose.material3.Slider
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.material3.Switch
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.semantics.LiveRegionMode
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.heading
import androidx.compose.ui.semantics.liveRegion
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.semantics.stateDescription
import androidx.compose.ui.unit.dp
import androidx.lifecycle.viewmodel.compose.viewModel
import com.yesmaster.app.ui.theme.YesMasterTheme

class MainActivity : ComponentActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContent {
            YesMasterTheme {
                Surface(Modifier.fillMaxSize(), color = MaterialTheme.colorScheme.background) {
                    AppRoot()
                }
            }
        }
    }
}

@Composable
private fun AppRoot(vm: MasteringViewModel = viewModel()) {
    val state by vm.state.collectAsState()
    val picker = rememberLauncherForActivityResult(
        ActivityResultContracts.OpenDocument()
    ) { uri -> uri?.let(vm::importTrack) }
    val pick = { picker.launch(arrayOf("audio/*")) }

    when (val s = state) {
        is UiState.Idle -> ImportHero(onImport = pick)
        is UiState.Working -> WorkingScreen(s.label)
        is UiState.Ready -> {
            val auditionUi by vm.audition.state.collectAsState()
            ReadyScreen(
                s,
                auditionUi = auditionUi,
                auditionActions = AuditionActions.forController(vm.audition),
                onChoices = vm::auditionParams,
                onMaster = vm::master,
                onImportOther = pick,
            )
        }
        is UiState.Done -> DoneScreen(s, onAgain = vm::backToReady, onNew = vm::reset)
        is UiState.Error -> ErrorScreen(
            s,
            onBack = vm::backToReady,
            onRetry = if (s.retrySourcePath != null) vm::retryAnalysis else null,
        )
    }
}

/**
 * The audition callbacks the Ready screen needs, split from
 * [AuditionController] so screens can be composed — and semantics-tested on
 * the JVM — without a native handle, audio focus, or an Android [Context]
 * (U19 testable seam).
 */
internal data class AuditionActions(
    val onTogglePlay: () -> Unit = {},
    val onSeek: (Double) -> Unit = {},
    val onListenOriginal: (Boolean) -> Unit = {},
    val onToggleVolumeMatch: () -> Unit = {},
) {
    companion object {
        fun forController(controller: AuditionController) = AuditionActions(
            onTogglePlay = controller::togglePlay,
            onSeek = controller::seek,
            onListenOriginal = controller::setListeningOriginal,
            onToggleVolumeMatch = controller::toggleVolumeMatch,
        )
    }
}

// Scrollable so small displays and large font scales can always reach every
// action — the Done/Error screens used to rely on everything fitting (U19
// finding: at 2× font on a small phone the receipt pushed its own buttons
// off-screen with no way to scroll to them).
@Composable
private fun ScreenColumn(content: @Composable ColumnScope.() -> Unit) {
    Column(
        Modifier
            .fillMaxSize()
            .verticalScroll(rememberScrollState())
            .padding(horizontal = 24.dp, vertical = 32.dp),
        verticalArrangement = Arrangement.spacedBy(16.dp),
        content = content,
    )
}

private typealias ColumnScope = androidx.compose.foundation.layout.ColumnScope

@Composable
internal fun ImportHero(onImport: () -> Unit) {
    Column(
        Modifier.fillMaxSize().padding(32.dp),
        verticalArrangement = Arrangement.Center,
        horizontalAlignment = Alignment.CenterHorizontally,
    ) {
        Text(
            "YES Master",
            style = MaterialTheme.typography.headlineLarge,
            modifier = Modifier.semantics { heading() },
        )
        Spacer(Modifier.height(8.dp))
        Text(
            "Import a track, pick a style, get a master.",
            style = MaterialTheme.typography.bodyLarge,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
        )
        Spacer(Modifier.height(24.dp))
        Button(onClick = onImport) { Text("Import Audio") }
    }
}

@Composable
internal fun WorkingScreen(label: String) {
    Column(
        Modifier.fillMaxSize(),
        verticalArrangement = Arrangement.Center,
        horizontalAlignment = Alignment.CenterHorizontally,
    ) {
        CircularProgressIndicator()
        Spacer(Modifier.height(16.dp))
        // Polite live region: TalkBack announces stage changes ("Analyzing…"
        // → "Rendering…") without the user having to re-explore the screen.
        Text(
            label,
            style = MaterialTheme.typography.titleMedium,
            modifier = Modifier.semantics { liveRegion = LiveRegionMode.Polite },
        )
    }
}

@Composable
internal fun ReadyScreen(
    ready: UiState.Ready,
    auditionUi: AuditionUi,
    auditionActions: AuditionActions,
    onChoices: (StandardStyle, StandardLoudness, Float) -> Unit,
    onMaster: (StandardStyle, StandardLoudness, Float) -> Unit,
    onImportOther: () -> Unit,
) {
    // Seeded from the state (keyed on it) so "Master again" reopens on the
    // choices that produced the last master, not on defaults — the Ready
    // screen leaves the composition during Working/Done, so plain remember
    // would reset (adversarial-review finding). Saveable so uncommitted
    // tweaks also survive Activity recreation (dark-mode toggle, font-scale
    // change — anything outside the manifest's configChanges opt-outs).
    var style by rememberSaveable(ready) { mutableStateOf(ready.style) }
    var loudness by rememberSaveable(ready) { mutableStateOf(ready.loudness) }
    var intensity by rememberSaveable(ready) { mutableStateOf(ready.intensity) }
    val retune = { onChoices(style, loudness, intensity) }

    // Scrollable (unlike the other screens): transport + style grid +
    // loudness + intensity exceed small displays.
    Column(
        Modifier
            .fillMaxSize()
            .verticalScroll(rememberScrollState())
            .padding(horizontal = 24.dp, vertical = 32.dp),
        verticalArrangement = Arrangement.spacedBy(16.dp),
    ) {
        Text(
            ready.displayName,
            style = MaterialTheme.typography.titleLarge,
            modifier = Modifier.semantics { heading() },
        )
        Text(
            "Source  %.1f LUFS · TP %.2f dBTP · LRA %.1f LU".format(
                ready.analysis.lufsIntegrated,
                ready.analysis.truePeakDbtp,
                ready.analysis.dynamicRangeLu,
            ),
            style = MaterialTheme.typography.bodyMedium,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
        )

        AuditionCard(auditionUi, auditionActions)

        Text(
            "Style",
            style = MaterialTheme.typography.titleMedium,
            modifier = Modifier.semantics { heading() },
        )
        // Plain rows, not a LazyVerticalGrid: the grid's fixed 180 dp clipped
        // the second row at large font scales, and four tiles never needed
        // laziness. Each tile carries RadioButton selection semantics —
        // selection used to be color-only, invisible to TalkBack (U19).
        StandardStyle.entries.chunked(2).forEach { rowStyles ->
            Row(horizontalArrangement = Arrangement.spacedBy(12.dp)) {
                rowStyles.forEach { candidate ->
                    val selected = candidate == style
                    Card(
                        colors = CardDefaults.cardColors(
                            containerColor = if (selected) MaterialTheme.colorScheme.primary
                            else MaterialTheme.colorScheme.surfaceVariant,
                        ),
                        modifier = Modifier
                            .weight(1f)
                            .selectable(
                                selected = selected,
                                role = Role.RadioButton,
                                onClick = { style = candidate; retune() },
                            ),
                    ) {
                        Column(Modifier.padding(16.dp)) {
                            Text(candidate.label, style = MaterialTheme.typography.titleMedium)
                        }
                    }
                }
            }
        }

        Text(
            "Loudness",
            style = MaterialTheme.typography.titleMedium,
            modifier = Modifier.semantics { heading() },
        )
        SingleChoiceSegmentedButtonRow(Modifier.fillMaxWidth()) {
            StandardLoudness.entries.forEachIndexed { index, candidate ->
                SegmentedButton(
                    selected = loudness == candidate,
                    onClick = { loudness = candidate; retune() },
                    shape = SegmentedButtonDefaults.itemShape(
                        index = index,
                        count = StandardLoudness.entries.size,
                    ),
                ) { Text(candidate.label) }
            }
        }

        Text(
            "Intensity  ${"%.0f".format(intensity * 100)}%",
            style = MaterialTheme.typography.titleMedium,
            modifier = Modifier.semantics { heading() },
        )
        Slider(
            value = intensity,
            onValueChange = { intensity = it; retune() },
            // The visible caption above is not associated with the slider, so
            // TalkBack heard an anonymous "50%" — name it (U19).
            modifier = Modifier.semantics { contentDescription = "Intensity" },
        )

        Spacer(Modifier.height(8.dp))
        Button(
            onClick = { onMaster(style, loudness, intensity) },
            modifier = Modifier.fillMaxWidth(),
        ) { Text("Create Master") }
        OutlinedButton(onClick = onImportOther, modifier = Modifier.fillMaxWidth()) {
            Text("Import a different track")
        }
    }
}

private fun formatClock(seconds: Double): String {
    val total = seconds.toInt().coerceAtLeast(0)
    return "%d:%02d".format(total / 60, total % 60)
}

/**
 * Live audition transport: hear Original vs Mastered on one timeline (the
 * playhead never moves on a switch), with the loudness landing always
 * applied — what plays here is what Create Master renders.
 *
 * Takes plain state + callbacks (not the controller) so the JVM Compose
 * lane can drive every semantic state without a native handle (U19).
 */
@Composable
internal fun AuditionCard(ui: AuditionUi, actions: AuditionActions) {
    Card {
        Column(
            Modifier.fillMaxWidth().padding(16.dp),
            verticalArrangement = Arrangement.spacedBy(12.dp),
        ) {
            Row(
                verticalAlignment = Alignment.CenterVertically,
                horizontalArrangement = Arrangement.spacedBy(12.dp),
            ) {
                Button(
                    onClick = actions.onTogglePlay,
                    enabled = ui.status == AuditionUi.Status.Ready,
                ) { Text(if (ui.playing) "Pause" else "Play") }
                Text(
                    "${formatClock(ui.positionSeconds)} / ${formatClock(ui.durationSeconds)}",
                    style = MaterialTheme.typography.bodyMedium,
                )
                if (ui.status == AuditionUi.Status.Preparing) {
                    CircularProgressIndicator(Modifier.height(20.dp))
                    // The visible reason the transport is disabled; polite so
                    // TalkBack hears it without a focus hunt.
                    Text(
                        "Preparing…",
                        style = MaterialTheme.typography.bodySmall,
                        modifier = Modifier.semantics { liveRegion = LiveRegionMode.Polite },
                    )
                }
            }

            // Local drag state so the thumb tracks the finger; the engine
            // seeks once on release (applied at the next processed block).
            var dragValue by remember { mutableStateOf<Float?>(null) }
            val duration = ui.durationSeconds
            val positionLabel =
                "${formatClock(ui.positionSeconds)} of ${formatClock(duration)}"
            Slider(
                value = dragValue
                    ?: if (duration > 0) (ui.positionSeconds / duration).toFloat() else 0f,
                onValueChange = { dragValue = it },
                onValueChangeFinished = {
                    dragValue?.let { actions.onSeek(it.toDouble() * duration) }
                    dragValue = null
                },
                enabled = ui.status == AuditionUi.Status.Ready && duration > 0,
                // Named, and announced as time — the raw 0..1 fraction reads
                // as a meaningless percent to TalkBack (U19).
                modifier = Modifier.semantics {
                    contentDescription = "Playback position"
                    stateDescription = positionLabel
                },
            )

            SingleChoiceSegmentedButtonRow(Modifier.fillMaxWidth()) {
                listOf(false to "Mastered", true to "Original").forEachIndexed { index, (original, label) ->
                    SegmentedButton(
                        selected = ui.listeningOriginal == original,
                        onClick = { actions.onListenOriginal(original) },
                        enabled = ui.status == AuditionUi.Status.Ready,
                        shape = SegmentedButtonDefaults.itemShape(index = index, count = 2),
                    ) { Text(label) }
                }
            }

            Row(
                verticalAlignment = Alignment.CenterVertically,
                horizontalArrangement = Arrangement.spacedBy(8.dp),
            ) {
                // One merged toggleable row: previously the Switch was
                // anonymous ("off, switch") because the label text beside it
                // was a separate node (U19). The Switch itself is display-only
                // (onCheckedChange = null) inside the row's semantics.
                Row(
                    verticalAlignment = Alignment.CenterVertically,
                    horizontalArrangement = Arrangement.spacedBy(8.dp),
                    modifier = Modifier.toggleable(
                        value = ui.volumeMatch,
                        role = Role.Switch,
                        enabled = ui.status == AuditionUi.Status.Ready,
                        onValueChange = { actions.onToggleVolumeMatch() },
                    ),
                ) {
                    Switch(
                        checked = ui.volumeMatch,
                        onCheckedChange = null,
                        enabled = ui.status == AuditionUi.Status.Ready,
                    )
                    Text("Volume Match", style = MaterialTheme.typography.bodyMedium)
                }
                Spacer(Modifier.weight(1f))
                ui.masteredLufs?.let {
                    Text(
                        "Lands %.1f LUFS".format(it),
                        style = MaterialTheme.typography.bodySmall,
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                    )
                }
            }

            ui.notice?.let {
                // Interruptions (audio focus loss, unplugged headphones,
                // reached-the-end) land here; polite live region so the state
                // change is announced, not silently discovered (U19).
                Text(
                    it,
                    style = MaterialTheme.typography.bodySmall,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                    modifier = Modifier.semantics { liveRegion = LiveRegionMode.Polite },
                )
            }
        }
    }
}

@Composable
internal fun DoneScreen(done: UiState.Done, onAgain: () -> Unit, onNew: () -> Unit) {
    val context = LocalContext.current
    ScreenColumn {
        Text(
            "Master saved",
            style = MaterialTheme.typography.headlineMedium,
            modifier = Modifier.semantics { heading() },
        )
        Text(done.savedTo, style = MaterialTheme.typography.bodyMedium)
        done.measurements?.let { m ->
            Card {
                Column(Modifier.padding(16.dp), verticalArrangement = Arrangement.spacedBy(4.dp)) {
                    Text(
                        "Delivered",
                        style = MaterialTheme.typography.titleMedium,
                        modifier = Modifier.semantics { heading() },
                    )
                    Text("Master  %.1f LUFS".format(m.lufsIntegrated))
                    Text("True peak  %.2f dBTP".format(m.truePeakDbtp))
                    Text("Dynamics  %.1f LU".format(m.dynamicRangeLu))
                    Text("${m.sampleRate / 1000.0} kHz · ${m.bitDepth}-bit")
                }
            }
        }
        // No weight spacer: ScreenColumn scrolls now, and a weighted child
        // inside a scrollable column measures to zero anyway — actions sit
        // directly under the receipt so they are always reachable.
        Spacer(Modifier.height(8.dp))
        Button(
            onClick = { context.tryStartActivity(Intent.createChooser(DoneIntents.share(done.savedUri), "Share Master")) },
            modifier = Modifier.fillMaxWidth(),
        ) {
            Text("Share Master")
        }
        OutlinedButton(
            onClick = { context.tryStartActivity(DoneIntents.play(done.savedUri)) },
            modifier = Modifier.fillMaxWidth(),
        ) {
            Text("Play Master")
        }
        Button(onClick = onAgain, modifier = Modifier.fillMaxWidth()) {
            Text("Master again with different settings")
        }
        OutlinedButton(onClick = onNew, modifier = Modifier.fillMaxWidth()) {
            Text("Start over")
        }
    }
}

object DoneIntents {
    data class Spec(val action: String, val type: String, val savedUri: String, val streamExtra: Boolean)

    fun shareSpec(savedUri: String): Spec =
        Spec(Intent.ACTION_SEND, "audio/wav", savedUri, streamExtra = true)

    fun playSpec(savedUri: String): Spec =
        Spec(Intent.ACTION_VIEW, "audio/wav", savedUri, streamExtra = false)

    fun share(savedUri: String): Intent = shareSpec(savedUri).toIntent()

    fun play(savedUri: String): Intent = playSpec(savedUri).toIntent()

    private fun Spec.toIntent(): Intent {
        val uri = Uri.parse(savedUri)
        val intent = Intent(action)
            .setType(type)
            .addFlags(Intent.FLAG_GRANT_READ_URI_PERMISSION)
        return if (streamExtra) {
            intent.putExtra(Intent.EXTRA_STREAM, uri)
        } else {
            intent.setDataAndType(uri, type)
        }
    }
}

private fun Context.tryStartActivity(intent: Intent) {
    runCatching { startActivity(intent) }
}

@Composable
internal fun ErrorScreen(error: UiState.Error, onBack: () -> Unit, onRetry: (() -> Unit)?) {
    ScreenColumn {
        Text(
            "Something went wrong",
            style = MaterialTheme.typography.headlineSmall,
            modifier = Modifier.semantics { heading() },
        )
        Text(error.message, color = MaterialTheme.colorScheme.error)
        Spacer(Modifier.height(8.dp))
        if (onRetry != null) {
            // Analysis failed after the import already cached the file —
            // retry without making the user re-pick through SAF.
            Button(onClick = onRetry, modifier = Modifier.fillMaxWidth()) {
                Text("Retry analysis")
            }
        }
        OutlinedButton(onClick = onBack, modifier = Modifier.fillMaxWidth()) { Text("Back") }
    }
}
