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
import androidx.compose.foundation.verticalScroll
import androidx.compose.foundation.lazy.grid.GridCells
import androidx.compose.foundation.lazy.grid.LazyVerticalGrid
import androidx.compose.foundation.lazy.grid.items
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
import androidx.compose.material3.darkColorScheme
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
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.unit.dp
import androidx.lifecycle.viewmodel.compose.viewModel

private val Scheme = darkColorScheme(
    primary = Color(0xFF5EA8FF),
    background = Color(0xFF0B0D10),
    surface = Color(0xFF14181D),
    surfaceVariant = Color(0xFF1B2026),
)

class MainActivity : ComponentActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContent {
            MaterialTheme(colorScheme = Scheme) {
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
        is UiState.Ready -> ReadyScreen(
            s,
            audition = vm.audition,
            onChoices = vm::auditionParams,
            onMaster = vm::master,
            onImportOther = pick,
        )
        is UiState.Done -> DoneScreen(s, onAgain = vm::backToReady, onNew = vm::reset)
        is UiState.Error -> ErrorScreen(
            s,
            onBack = vm::backToReady,
            onRetry = if (s.retrySourcePath != null) vm::retryAnalysis else null,
        )
    }
}

@Composable
private fun ScreenColumn(content: @Composable ColumnScope.() -> Unit) {
    Column(
        Modifier
            .fillMaxSize()
            .padding(horizontal = 24.dp, vertical = 32.dp),
        verticalArrangement = Arrangement.spacedBy(16.dp),
        content = content,
    )
}

private typealias ColumnScope = androidx.compose.foundation.layout.ColumnScope

@Composable
private fun ImportHero(onImport: () -> Unit) {
    Column(
        Modifier.fillMaxSize().padding(32.dp),
        verticalArrangement = Arrangement.Center,
        horizontalAlignment = Alignment.CenterHorizontally,
    ) {
        Text("YES Master", style = MaterialTheme.typography.headlineLarge)
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
private fun WorkingScreen(label: String) {
    Column(
        Modifier.fillMaxSize(),
        verticalArrangement = Arrangement.Center,
        horizontalAlignment = Alignment.CenterHorizontally,
    ) {
        CircularProgressIndicator()
        Spacer(Modifier.height(16.dp))
        Text(label, style = MaterialTheme.typography.titleMedium)
    }
}

@Composable
private fun ReadyScreen(
    ready: UiState.Ready,
    audition: AuditionController,
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
        Text(ready.displayName, style = MaterialTheme.typography.titleLarge)
        Text(
            "Source  %.1f LUFS · TP %.2f dBTP · LRA %.1f LU".format(
                ready.analysis.lufsIntegrated,
                ready.analysis.truePeakDbtp,
                ready.analysis.dynamicRangeLu,
            ),
            style = MaterialTheme.typography.bodyMedium,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
        )

        AuditionCard(audition)

        Text("Style", style = MaterialTheme.typography.titleMedium)
        LazyVerticalGrid(
            columns = GridCells.Fixed(2),
            horizontalArrangement = Arrangement.spacedBy(12.dp),
            verticalArrangement = Arrangement.spacedBy(12.dp),
            modifier = Modifier.height(180.dp),
        ) {
            items(StandardStyle.entries) { candidate ->
                val selected = candidate == style
                Card(
                    onClick = { style = candidate; retune() },
                    colors = CardDefaults.cardColors(
                        containerColor = if (selected) MaterialTheme.colorScheme.primary
                        else MaterialTheme.colorScheme.surfaceVariant,
                    ),
                ) {
                    Column(Modifier.padding(16.dp)) {
                        Text(candidate.label, style = MaterialTheme.typography.titleMedium)
                    }
                }
            }
        }

        Text("Loudness", style = MaterialTheme.typography.titleMedium)
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
        )
        Slider(value = intensity, onValueChange = { intensity = it; retune() })

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
 */
@Composable
private fun AuditionCard(audition: AuditionController) {
    val ui by audition.state.collectAsState()

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
                    onClick = audition::togglePlay,
                    enabled = ui.status == AuditionUi.Status.Ready,
                ) { Text(if (ui.playing) "Pause" else "Play") }
                Text(
                    "${formatClock(ui.positionSeconds)} / ${formatClock(ui.durationSeconds)}",
                    style = MaterialTheme.typography.bodyMedium,
                )
                if (ui.status == AuditionUi.Status.Preparing) {
                    CircularProgressIndicator(Modifier.height(20.dp))
                    Text("Preparing…", style = MaterialTheme.typography.bodySmall)
                }
            }

            // Local drag state so the thumb tracks the finger; the engine
            // seeks once on release (applied at the next processed block).
            var dragValue by remember { mutableStateOf<Float?>(null) }
            val duration = ui.durationSeconds
            Slider(
                value = dragValue
                    ?: if (duration > 0) (ui.positionSeconds / duration).toFloat() else 0f,
                onValueChange = { dragValue = it },
                onValueChangeFinished = {
                    dragValue?.let { audition.seek(it.toDouble() * duration) }
                    dragValue = null
                },
                enabled = ui.status == AuditionUi.Status.Ready && duration > 0,
            )

            SingleChoiceSegmentedButtonRow(Modifier.fillMaxWidth()) {
                listOf(false to "Mastered", true to "Original").forEachIndexed { index, (original, label) ->
                    SegmentedButton(
                        selected = ui.listeningOriginal == original,
                        onClick = { audition.setListeningOriginal(original) },
                        enabled = ui.status == AuditionUi.Status.Ready,
                        shape = SegmentedButtonDefaults.itemShape(index = index, count = 2),
                    ) { Text(label) }
                }
            }

            Row(
                verticalAlignment = Alignment.CenterVertically,
                horizontalArrangement = Arrangement.spacedBy(8.dp),
            ) {
                Switch(
                    checked = ui.volumeMatch,
                    onCheckedChange = { audition.toggleVolumeMatch() },
                    enabled = ui.status == AuditionUi.Status.Ready,
                )
                Text("Volume Match", style = MaterialTheme.typography.bodyMedium)
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
                Text(
                    it,
                    style = MaterialTheme.typography.bodySmall,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                )
            }
        }
    }
}

@Composable
private fun DoneScreen(done: UiState.Done, onAgain: () -> Unit, onNew: () -> Unit) {
    val context = LocalContext.current
    ScreenColumn {
        Text("Master saved", style = MaterialTheme.typography.headlineMedium)
        Text(done.savedTo, style = MaterialTheme.typography.bodyMedium)
        done.measurements?.let { m ->
            Card {
                Column(Modifier.padding(16.dp), verticalArrangement = Arrangement.spacedBy(4.dp)) {
                    Text("Delivered", style = MaterialTheme.typography.titleMedium)
                    Text("Master  %.1f LUFS".format(m.lufsIntegrated))
                    Text("True peak  %.2f dBTP".format(m.truePeakDbtp))
                    Text("Dynamics  %.1f LU".format(m.dynamicRangeLu))
                    Text("${m.sampleRate / 1000.0} kHz · ${m.bitDepth}-bit")
                }
            }
        }
        Spacer(Modifier.weight(1f))
        Button(
            onClick = { context.startActivity(Intent.createChooser(DoneIntents.share(done.savedUri), "Share Master")) },
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
private fun ErrorScreen(error: UiState.Error, onBack: () -> Unit, onRetry: (() -> Unit)?) {
    ScreenColumn {
        Text("Something went wrong", style = MaterialTheme.typography.headlineSmall)
        Text(error.message, color = MaterialTheme.colorScheme.error)
        Spacer(Modifier.weight(1f))
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
