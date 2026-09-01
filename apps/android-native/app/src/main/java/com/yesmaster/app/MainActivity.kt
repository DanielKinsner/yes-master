package com.yesmaster.app

import android.content.Context
import android.content.Intent
import android.net.Uri
import android.os.Bundle
import android.provider.Settings
import androidx.activity.ComponentActivity
import androidx.activity.compose.rememberLauncherForActivityResult
import androidx.activity.compose.setContent
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.animation.core.RepeatMode
import androidx.compose.animation.core.animateFloat
import androidx.compose.animation.core.infiniteRepeatable
import androidx.compose.animation.core.rememberInfiniteTransition
import androidx.compose.animation.core.tween
import androidx.compose.foundation.Canvas
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.BoxScope
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.selection.toggleable
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
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
import androidx.compose.material3.Switch
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.alpha
import androidx.compose.ui.draw.blur
import androidx.compose.ui.draw.clip
import androidx.compose.ui.draw.shadow
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.StrokeCap
import androidx.compose.ui.graphics.drawscope.Stroke
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.semantics.LiveRegionMode
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.heading
import androidx.compose.ui.semantics.liveRegion
import androidx.compose.ui.semantics.role
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.semantics.selected
import androidx.compose.ui.semantics.stateDescription
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.lifecycle.viewmodel.compose.viewModel
import com.yesmaster.app.ui.theme.YesMasterColors
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
        ActivityResultContracts.OpenDocument(),
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

/** Plain callbacks keep the Compose lane independent of native audio. */
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

private typealias ColumnScope = androidx.compose.foundation.layout.ColumnScope

@Composable
private fun StudioBackground(content: @Composable BoxScope.() -> Unit) {
    Box(
        Modifier
            .fillMaxSize()
            .background(
                Brush.verticalGradient(
                    listOf(
                        YesMasterColors.Background,
                        YesMasterColors.BackgroundMid,
                        YesMasterColors.BackgroundDeep,
                    ),
                ),
            ),
    ) {
        Canvas(Modifier.fillMaxSize()) {
            drawCircle(
                brush = Brush.radialGradient(
                    colors = listOf(
                        YesMasterColors.Accent.copy(alpha = 0.24f),
                        Color.Transparent,
                    ),
                    center = Offset(size.width * 0.88f, 0f),
                    radius = size.width * 0.72f,
                ),
                radius = size.width * 0.72f,
                center = Offset(size.width * 0.88f, 0f),
            )
        }
        content()
    }
}

@Composable
private fun BrandMark(modifier: Modifier = Modifier) {
    Box(
        modifier
            .size(30.dp)
            .shadow(12.dp, RoundedCornerShape(8.dp), ambientColor = YesMasterColors.Cyan)
            .clip(RoundedCornerShape(8.dp))
            .background(
                Brush.linearGradient(
                    listOf(YesMasterColors.AccentBright, YesMasterColors.Accent),
                ),
            ),
        contentAlignment = Alignment.Center,
    ) {
        Row(
            horizontalArrangement = Arrangement.spacedBy(2.dp),
            verticalAlignment = Alignment.Bottom,
        ) {
            listOf(8.dp, 14.dp, 10.dp).forEach { barHeight ->
                Box(
                    Modifier
                        .width(3.dp)
                        .height(barHeight)
                        .clip(RoundedCornerShape(2.dp))
                        .background(YesMasterColors.BackgroundDeep),
                )
            }
        }
    }
}

@Composable
private fun BrandHeader(status: String) {
    Row(
        Modifier.fillMaxWidth(),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        BrandMark()
        Spacer(Modifier.width(10.dp))
        Text(
            "YES MASTER",
            fontSize = 16.sp,
            fontWeight = FontWeight.Black,
            letterSpacing = 1.4.sp,
            color = YesMasterColors.TextPrimary,
            modifier = Modifier.semantics { heading() },
        )
        Spacer(Modifier.weight(1f))
        Text(
            status,
            fontSize = 12.sp,
            fontWeight = FontWeight.Bold,
            color = Color(0xFFC7DEFF),
            modifier = Modifier
                .clip(CircleShape)
                .background(Color.White.copy(alpha = 0.045f))
                .border(1.dp, YesMasterColors.Border.copy(alpha = 0.55f), CircleShape)
                .padding(horizontal = 16.dp, vertical = 8.dp),
        )
    }
}

@Composable
internal fun ImportHero(onImport: () -> Unit) {
    WorkbenchScreen(
        displayName = null,
        analysis = null,
        auditionUi = AuditionUi(),
        auditionActions = AuditionActions(),
        style = StandardStyle.BALANCED,
        loudness = StandardLoudness.MEDIUM,
        intensity = 0.5f,
        controlsEnabled = false,
        onStyle = {},
        onLoudness = {},
        onIntensity = {},
        onMaster = {},
        onImport = onImport,
    )
}

@Composable
internal fun WorkingScreen(label: String) {
    StudioBackground {
        Box(
            Modifier
                .fillMaxSize()
                .blur(7.dp)
                .alpha(0.76f),
        ) {
            WorkbenchContent(
                displayName = null,
                analysis = null,
                auditionUi = AuditionUi(),
                auditionActions = AuditionActions(),
                style = StandardStyle.BALANCED,
                loudness = StandardLoudness.MEDIUM,
                intensity = 0.5f,
                controlsEnabled = false,
                onStyle = {},
                onLoudness = {},
                onIntensity = {},
                onMaster = {},
                onImport = {},
            )
        }
        AnalysisOverlay(label)
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
    var styleName by rememberSaveable(ready.sourcePath) { mutableStateOf(ready.style.name) }
    var loudnessName by rememberSaveable(ready.sourcePath) { mutableStateOf(ready.loudness.name) }
    var intensity by rememberSaveable(ready.sourcePath) { mutableStateOf(ready.intensity) }
    val style = StandardStyle.valueOf(styleName)
    val loudness = StandardLoudness.valueOf(loudnessName)

    WorkbenchScreen(
        displayName = ready.displayName,
        analysis = ready.analysis,
        auditionUi = auditionUi,
        auditionActions = auditionActions,
        style = style,
        loudness = loudness,
        intensity = intensity,
        controlsEnabled = true,
        onStyle = { selected ->
            styleName = selected.name
            onChoices(selected, loudness, intensity)
        },
        onLoudness = { selected ->
            loudnessName = selected.name
            onChoices(style, selected, intensity)
        },
        onIntensity = { selected ->
            intensity = selected
            onChoices(style, loudness, selected)
        },
        onMaster = { onMaster(style, loudness, intensity) },
        onImport = onImportOther,
    )
}

@Composable
private fun WorkbenchScreen(
    displayName: String?,
    analysis: WireAnalysis?,
    auditionUi: AuditionUi,
    auditionActions: AuditionActions,
    style: StandardStyle,
    loudness: StandardLoudness,
    intensity: Float,
    controlsEnabled: Boolean,
    onStyle: (StandardStyle) -> Unit,
    onLoudness: (StandardLoudness) -> Unit,
    onIntensity: (Float) -> Unit,
    onMaster: () -> Unit,
    onImport: () -> Unit,
) {
    StudioBackground {
        WorkbenchContent(
            displayName,
            analysis,
            auditionUi,
            auditionActions,
            style,
            loudness,
            intensity,
            controlsEnabled,
            onStyle,
            onLoudness,
            onIntensity,
            onMaster,
            onImport,
        )
    }
}

@Composable
private fun WorkbenchContent(
    displayName: String?,
    analysis: WireAnalysis?,
    auditionUi: AuditionUi,
    auditionActions: AuditionActions,
    style: StandardStyle,
    loudness: StandardLoudness,
    intensity: Float,
    controlsEnabled: Boolean,
    onStyle: (StandardStyle) -> Unit,
    onLoudness: (StandardLoudness) -> Unit,
    onIntensity: (Float) -> Unit,
    onMaster: () -> Unit,
    onImport: () -> Unit,
) {
    Column(
        Modifier
            .fillMaxSize()
            .verticalScroll(rememberScrollState())
            .padding(horizontal = 16.dp, vertical = 14.dp),
        verticalArrangement = Arrangement.spacedBy(10.dp),
    ) {
        BrandHeader(if (analysis == null) "Ready" else "Ready")
        AuditionCard(
            ui = auditionUi,
            actions = auditionActions,
            displayName = displayName,
            onImport = onImport,
        )
        StylePicker(style, controlsEnabled, onStyle)
        IntensityControl(intensity, controlsEnabled, onIntensity)
        LoudnessPicker(loudness, controlsEnabled, onLoudness)
        PrimaryGradientButton(
            text = "Create Master",
            enabled = controlsEnabled,
            onClick = onMaster,
            modifier = Modifier.padding(top = 12.dp),
        )
        analysis?.let {
            Text(
                "Source  %.1f LUFS · TP %.2f dBTP · LRA %.1f LU".format(
                    it.lufsIntegrated,
                    it.truePeakDbtp,
                    it.dynamicRangeLu,
                ),
                fontSize = 11.sp,
                fontWeight = FontWeight.SemiBold,
                color = YesMasterColors.TextMuted,
                textAlign = TextAlign.Center,
                modifier = Modifier.fillMaxWidth().padding(top = 2.dp),
            )
        }
        Spacer(Modifier.height(8.dp))
    }
}

private fun formatClock(seconds: Double): String {
    val total = seconds.toInt().coerceAtLeast(0)
    return "%d:%02d".format(total / 60, total % 60)
}

@Composable
internal fun AuditionCard(
    ui: AuditionUi,
    actions: AuditionActions,
    displayName: String? = "Track",
    onImport: () -> Unit = {},
) {
    val ready = ui.status == AuditionUi.Status.Ready
    val title = displayName ?: "Import track"
    val extension = displayName
        ?.substringAfterLast('.', missingDelimiterValue = "")
        ?.uppercase()
        ?.ifBlank { "AUDIO" }

    Column(verticalArrangement = Arrangement.spacedBy(8.dp)) {
        Box(
            Modifier
                .fillMaxWidth()
                .height(304.dp)
                .clip(RoundedCornerShape(18.dp))
                .background(
                    Brush.verticalGradient(
                        listOf(Color(0xFF091B3B), YesMasterColors.SurfaceDeep),
                    ),
                )
                .border(
                    1.dp,
                    YesMasterColors.Border.copy(alpha = 0.48f),
                    RoundedCornerShape(18.dp),
                ),
        ) {
            Row(
                Modifier.fillMaxWidth().padding(start = 16.dp, end = 14.dp, top = 14.dp),
                verticalAlignment = Alignment.CenterVertically,
            ) {
                Row(
                    Modifier
                        .weight(1f)
                        .height(38.dp)
                        .clip(CircleShape)
                        .background(Color(0xFF07152F).copy(alpha = 0.9f))
                        .border(1.dp, YesMasterColors.Border.copy(alpha = 0.42f), CircleShape)
                        .clickable(onClick = onImport)
                        .padding(horizontal = 9.dp),
                    verticalAlignment = Alignment.CenterVertically,
                ) {
                    BrandMark(Modifier.size(23.dp))
                    Spacer(Modifier.width(8.dp))
                    Text(
                        title,
                        fontSize = 13.sp,
                        fontWeight = FontWeight.Bold,
                        color = Color(0xFFD8E8FF),
                        maxLines = 1,
                        modifier = Modifier.weight(1f),
                    )
                    if (extension != null) {
                        Text(
                            extension,
                            fontSize = 9.sp,
                            fontWeight = FontWeight.Black,
                            color = Color(0xFF7895C7),
                        )
                    }
                }
                Spacer(Modifier.width(8.dp))
                Box(
                    Modifier
                        .size(38.dp)
                        .clip(CircleShape)
                        .background(Color(0xFF07152F).copy(alpha = 0.9f))
                        .border(1.dp, YesMasterColors.Border.copy(alpha = 0.48f), CircleShape)
                        .clickable(onClick = onImport),
                    contentAlignment = Alignment.Center,
                ) {
                    Text(
                        if (displayName == null) "+" else "✎",
                        fontSize = 18.sp,
                        fontWeight = FontWeight.Black,
                        color = Color(0xFFD6E7FF),
                    )
                }
            }

            Column(
                Modifier.align(Alignment.Center).padding(top = 28.dp),
                horizontalAlignment = Alignment.CenterHorizontally,
            ) {
                PlaybackOrb(
                    playing = ui.playing,
                    enabled = ready || displayName == null,
                    empty = displayName == null,
                    onClick = if (displayName == null) onImport else actions.onTogglePlay,
                )
                Text(
                    when {
                        displayName == null -> "Import Audio"
                        ui.playing -> "Pause"
                        else -> "Play"
                    },
                    fontSize = 11.sp,
                    fontWeight = FontWeight.Bold,
                    color = YesMasterColors.TextSecondary,
                    modifier = Modifier.padding(top = 8.dp),
                )
            }

            Column(
                Modifier.align(Alignment.BottomCenter).fillMaxWidth()
                    .padding(horizontal = 16.dp, vertical = 13.dp),
                verticalArrangement = Arrangement.spacedBy(5.dp),
            ) {
                if (ui.status == AuditionUi.Status.Preparing) {
                    Row(
                        verticalAlignment = Alignment.CenterVertically,
                        horizontalArrangement = Arrangement.spacedBy(8.dp),
                    ) {
                        CircularProgressIndicator(Modifier.size(16.dp), strokeWidth = 2.dp)
                        Text(
                            "Preparing…",
                            fontSize = 11.sp,
                            color = YesMasterColors.TextSecondary,
                            modifier = Modifier.semantics { liveRegion = LiveRegionMode.Polite },
                        )
                    }
                }

                var dragValue by remember { mutableStateOf<Float?>(null) }
                val duration = ui.durationSeconds
                val positionLabel = "${formatClock(ui.positionSeconds)} of ${formatClock(duration)}"
                Slider(
                    value = dragValue
                        ?: if (duration > 0) (ui.positionSeconds / duration).toFloat() else 0f,
                    onValueChange = { dragValue = it },
                    onValueChangeFinished = {
                        dragValue?.let { actions.onSeek(it.toDouble() * duration) }
                        dragValue = null
                    },
                    enabled = ready && duration > 0,
                    modifier = Modifier.height(28.dp).semantics {
                        contentDescription = "Playback position"
                        stateDescription = positionLabel
                    },
                )

                Row(
                    Modifier.fillMaxWidth(),
                    verticalAlignment = Alignment.CenterVertically,
                ) {
                    Row(
                        verticalAlignment = Alignment.CenterVertically,
                        horizontalArrangement = Arrangement.spacedBy(7.dp),
                        modifier = Modifier.toggleable(
                            value = ui.volumeMatch,
                            role = Role.Switch,
                            enabled = ready,
                            onValueChange = { actions.onToggleVolumeMatch() },
                        ),
                    ) {
                        Switch(
                            checked = ui.volumeMatch,
                            onCheckedChange = null,
                            enabled = ready,
                            modifier = Modifier.height(28.dp),
                        )
                        Text(
                            "Volume Match",
                            fontSize = 12.sp,
                            fontWeight = FontWeight.Bold,
                            color = if (ready) YesMasterColors.TextPrimary else YesMasterColors.TextMuted,
                        )
                    }
                    Spacer(Modifier.weight(1f))
                    Text(
                        "${formatClock(ui.positionSeconds)} / ${formatClock(duration)}",
                        fontSize = 10.sp,
                        fontWeight = FontWeight.SemiBold,
                        color = YesMasterColors.TextMuted,
                    )
                }
            }
        }

        SingleChoiceSegmentedButtonRow(Modifier.fillMaxWidth()) {
            listOf(false to "Mastered", true to "Original").forEachIndexed { index, (original, label) ->
                SegmentedButton(
                    selected = ui.listeningOriginal == original,
                    onClick = { actions.onListenOriginal(original) },
                    enabled = ready,
                    shape = SegmentedButtonDefaults.itemShape(index = index, count = 2),
                ) { Text(label, fontWeight = FontWeight.Bold) }
            }
        }

        ui.notice?.let {
            Text(
                it,
                fontSize = 11.sp,
                fontWeight = FontWeight.SemiBold,
                color = YesMasterColors.TextSecondary,
                modifier = Modifier.semantics { liveRegion = LiveRegionMode.Polite },
            )
        }
    }
}

@Composable
private fun PlaybackOrb(
    playing: Boolean,
    enabled: Boolean,
    empty: Boolean,
    onClick: () -> Unit,
) {
    val motionEnabled = motionEnabled()
    val transition = rememberInfiniteTransition(label = "hero-pulse")
    val pulse by transition.animateFloat(
        initialValue = 1f,
        targetValue = if (motionEnabled) 1.035f else 1f,
        animationSpec = infiniteRepeatable(
            animation = tween(1_400),
            repeatMode = RepeatMode.Reverse,
        ),
        label = "hero-pulse-scale",
    )

    Box(Modifier.size(180.dp), contentAlignment = Alignment.Center) {
        Canvas(Modifier.fillMaxSize()) {
            repeat(4) { index ->
                drawCircle(
                    color = YesMasterColors.Accent.copy(alpha = 0.34f - index * 0.055f),
                    radius = (42 + index * 17).dp.toPx() * pulse,
                    style = Stroke(width = if (index == 0) 1.4.dp.toPx() else 1.dp.toPx()),
                )
            }
            drawLine(
                brush = Brush.horizontalGradient(
                    listOf(
                        Color.Transparent,
                        YesMasterColors.Accent.copy(alpha = 0.58f),
                        YesMasterColors.AccentBright.copy(alpha = 0.72f),
                        YesMasterColors.Accent.copy(alpha = 0.5f),
                        Color.Transparent,
                    ),
                ),
                start = Offset(0f, size.height / 2),
                end = Offset(size.width, size.height / 2),
                strokeWidth = 3.dp.toPx(),
                cap = StrokeCap.Round,
            )
        }
        Box(
            Modifier
                .size(74.dp)
                .shadow(28.dp, CircleShape, ambientColor = YesMasterColors.Accent)
                .clip(CircleShape)
                .background(
                    Brush.radialGradient(
                        listOf(Color(0xFFE8F6FF), Color(0xFF79BFFF), Color(0xFF1955EE)),
                    ),
                )
                .clickable(enabled = enabled, role = Role.Button, onClick = onClick)
                .semantics {
                    contentDescription = when {
                        empty -> "Import Audio"
                        playing -> "Pause"
                        else -> "Play"
                    }
                },
            contentAlignment = Alignment.Center,
        ) {
            Text(
                when {
                    empty -> "⇩"
                    playing -> "Ⅱ"
                    else -> "▶"
                },
                fontSize = if (empty) 28.sp else 24.sp,
                fontWeight = FontWeight.Black,
                color = Color.White,
            )
        }
    }
}

private fun StandardStyle.accent(): Color = when (this) {
    StandardStyle.BALANCED -> YesMasterColors.Accent
    StandardStyle.BRIGHT -> YesMasterColors.Cyan
    StandardStyle.WARM -> YesMasterColors.Amber
    StandardStyle.HEAVY -> YesMasterColors.Oomph
}

private fun StandardStyle.symbol(): String = when (this) {
    StandardStyle.BALANCED -> "◉"
    StandardStyle.BRIGHT -> "✦"
    StandardStyle.WARM -> "●"
    StandardStyle.HEAVY -> "ϟ"
}

@Composable
private fun SectionTitle(step: String, title: String, meta: String? = null) {
    Row(verticalAlignment = Alignment.CenterVertically) {
        Text(step, fontSize = 13.sp, fontWeight = FontWeight.Black, color = YesMasterColors.Accent)
        Spacer(Modifier.width(8.dp))
        Text(
            title.uppercase(),
            fontSize = 14.sp,
            fontWeight = FontWeight.Black,
            letterSpacing = 3.sp,
            color = YesMasterColors.TextPrimary,
            modifier = Modifier.semantics { heading() },
        )
        if (meta != null) {
            Spacer(Modifier.width(8.dp))
            Text(
                meta,
                fontSize = 11.sp,
                fontWeight = FontWeight.Bold,
                color = YesMasterColors.TextMuted,
            )
        }
    }
}

@Composable
private fun StylePicker(
    selected: StandardStyle,
    enabled: Boolean,
    onSelected: (StandardStyle) -> Unit,
) {
    Column(verticalArrangement = Arrangement.spacedBy(7.dp)) {
        SectionTitle("1", "Style")
        StandardStyle.entries.chunked(2).forEach { rowStyles ->
            Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                rowStyles.forEach { style ->
                    val active = selected == style
                    val accent = style.accent()
                    Card(
                        onClick = { onSelected(style) },
                        enabled = enabled,
                        shape = RoundedCornerShape(12.dp),
                        colors = CardDefaults.cardColors(containerColor = Color.Transparent),
                        border = androidx.compose.foundation.BorderStroke(
                            1.dp,
                            if (active) accent.copy(alpha = 0.78f) else Color.White.copy(alpha = 0.10f),
                        ),
                        modifier = Modifier
                            .weight(1f)
                            .heightIn(min = 62.dp)
                            .testTag("style-${style.id}")
                            .semantics(mergeDescendants = true) {
                                this.selected = active
                                role = Role.RadioButton
                            },
                    ) {
                        Box(
                            Modifier
                                .fillMaxWidth()
                                .background(
                                    Brush.linearGradient(
                                        listOf(
                                            accent.copy(alpha = if (active) 0.26f else 0.10f),
                                            YesMasterColors.Surface.copy(alpha = 0.94f),
                                        ),
                                    ),
                                ),
                        ) {
                            Row(
                                Modifier.fillMaxWidth().padding(horizontal = 12.dp, vertical = 13.dp),
                                verticalAlignment = Alignment.CenterVertically,
                            ) {
                                Box(
                                    Modifier.size(32.dp).clip(CircleShape)
                                        .background(accent.copy(alpha = 0.20f)),
                                    contentAlignment = Alignment.Center,
                                ) {
                                    Text(
                                        style.symbol(),
                                        color = accent,
                                        fontSize = 17.sp,
                                        fontWeight = FontWeight.Black,
                                    )
                                }
                                Spacer(Modifier.width(10.dp))
                                Text(
                                    style.label,
                                    fontSize = 13.sp,
                                    fontWeight = FontWeight.Black,
                                    color = if (enabled) YesMasterColors.TextPrimary else YesMasterColors.TextMuted,
                                )
                            }
                        }
                    }
                }
            }
        }
    }
}

@Composable
private fun IntensityControl(value: Float, enabled: Boolean, onValue: (Float) -> Unit) {
    Column(verticalArrangement = Arrangement.spacedBy(8.dp)) {
        SectionTitle("2", "Intensity", "${"%.0f".format(value * 100)}%")
        Column(
            Modifier
                .fillMaxWidth()
                .clip(RoundedCornerShape(16.dp))
                .background(YesMasterColors.SurfaceDeep.copy(alpha = 0.9f))
                .border(1.dp, Color.White.copy(alpha = 0.10f), RoundedCornerShape(16.dp))
                .padding(horizontal = 16.dp, vertical = 10.dp),
        ) {
            Slider(
                value = value,
                onValueChange = onValue,
                enabled = enabled,
                modifier = Modifier.semantics { contentDescription = "Intensity" },
            )
            Row(Modifier.fillMaxWidth()) {
                Text("Subtle", fontSize = 10.sp, fontWeight = FontWeight.Bold, color = YesMasterColors.TextMuted)
                Spacer(Modifier.weight(1f))
                Text("Full", fontSize = 10.sp, fontWeight = FontWeight.Bold, color = YesMasterColors.TextMuted)
                Spacer(Modifier.weight(1f))
                Text("Pushed", fontSize = 10.sp, fontWeight = FontWeight.Bold, color = YesMasterColors.TextMuted)
            }
        }
    }
}

@Composable
private fun LoudnessPicker(
    selected: StandardLoudness,
    enabled: Boolean,
    onSelected: (StandardLoudness) -> Unit,
) {
    Column(verticalArrangement = Arrangement.spacedBy(7.dp)) {
        SectionTitle("3", "Loudness")
        SingleChoiceSegmentedButtonRow(Modifier.fillMaxWidth()) {
            StandardLoudness.entries.forEachIndexed { index, candidate ->
                SegmentedButton(
                    selected = selected == candidate,
                    onClick = { onSelected(candidate) },
                    enabled = enabled,
                    shape = SegmentedButtonDefaults.itemShape(
                        index = index,
                        count = StandardLoudness.entries.size,
                    ),
                ) { Text(candidate.label, fontWeight = FontWeight.Bold) }
            }
        }
    }
}

@Composable
private fun PrimaryGradientButton(
    text: String,
    enabled: Boolean,
    onClick: () -> Unit,
    modifier: Modifier = Modifier,
) {
    Box(
        modifier
            .fillMaxWidth()
            .height(54.dp)
            .shadow(
                if (enabled) 22.dp else 0.dp,
                RoundedCornerShape(12.dp),
                ambientColor = YesMasterColors.Accent,
            )
            .clip(RoundedCornerShape(12.dp))
            .background(
                if (enabled) {
                    Brush.verticalGradient(listOf(Color(0xFF55AEFF), Color(0xFF0F57F4)))
                } else {
                    Brush.verticalGradient(listOf(Color(0xFF24344F), Color(0xFF18233A)))
                },
            )
            .clickable(enabled = enabled, role = Role.Button, onClick = onClick),
        contentAlignment = Alignment.Center,
    ) {
        Text(
            text,
            fontSize = 18.sp,
            fontWeight = FontWeight.Black,
            color = if (enabled) Color.White else YesMasterColors.TextMuted,
        )
    }
}

@Composable
private fun motionEnabled(): Boolean {
    val context = LocalContext.current
    return remember(context) {
        Settings.Global.getFloat(
            context.contentResolver,
            Settings.Global.ANIMATOR_DURATION_SCALE,
            1f,
        ) > 0f
    }
}

@Composable
private fun AnalysisOverlay(label: String) {
    val motionEnabled = motionEnabled()
    val transition = rememberInfiniteTransition(label = "analysis")
    val rotation by transition.animateFloat(
        initialValue = 0f,
        targetValue = if (motionEnabled) 360f else 0f,
        animationSpec = infiniteRepeatable(tween(1_150), RepeatMode.Restart),
        label = "analysis-rotation",
    )
    val pulse by transition.animateFloat(
        initialValue = 1f,
        targetValue = if (motionEnabled) 1.04f else 1f,
        animationSpec = infiniteRepeatable(tween(1_400), RepeatMode.Reverse),
        label = "analysis-pulse",
    )

    Box(
        Modifier
            .fillMaxSize()
            .background(YesMasterColors.BackgroundDeep.copy(alpha = 0.72f))
            .semantics {
                contentDescription = "Analyzing track. Listening for loudness, dynamics, and tone."
            },
        contentAlignment = Alignment.Center,
    ) {
        Column(horizontalAlignment = Alignment.CenterHorizontally) {
            Box(Modifier.size(224.dp), contentAlignment = Alignment.Center) {
                Canvas(Modifier.fillMaxSize()) {
                    repeat(4) { index ->
                        drawCircle(
                            color = YesMasterColors.Accent.copy(alpha = 0.32f - index * 0.05f),
                            radius = (52 + index * 18).dp.toPx() * pulse,
                            style = Stroke(if (index == 0) 1.6.dp.toPx() else 1.dp.toPx()),
                        )
                    }
                    drawArc(
                        brush = Brush.sweepGradient(
                            listOf(
                                YesMasterColors.AccentBright,
                                YesMasterColors.Amber,
                                YesMasterColors.AccentBright,
                            ),
                        ),
                        startAngle = rotation,
                        sweepAngle = 252f,
                        useCenter = false,
                        topLeft = Offset(65.dp.toPx(), 65.dp.toPx()),
                        size = androidx.compose.ui.geometry.Size(94.dp.toPx(), 94.dp.toPx()),
                        style = Stroke(3.dp.toPx(), cap = StrokeCap.Round),
                    )
                    val centerY = size.height / 2
                    val bars = listOf(12f, 28f, 18f, 42f, 24f, 34f, 14f)
                    val gap = 8.dp.toPx()
                    val startX = size.width / 2 - gap * 3
                    bars.forEachIndexed { index, height ->
                        drawLine(
                            color = Color(0xFFC9EBFF),
                            start = Offset(startX + gap * index, centerY - height.dp.toPx() / 2),
                            end = Offset(startX + gap * index, centerY + height.dp.toPx() / 2),
                            strokeWidth = 3.dp.toPx(),
                            cap = StrokeCap.Round,
                        )
                    }
                }
            }
            Text(
                label,
                fontSize = 15.sp,
                fontWeight = FontWeight.Black,
                letterSpacing = 2.8.sp,
                color = YesMasterColors.TextPrimary,
                modifier = Modifier.semantics { liveRegion = LiveRegionMode.Polite },
            )
            Spacer(Modifier.height(8.dp))
            Text(
                "Listening for loudness, dynamics, and tone",
                fontSize = 12.sp,
                fontWeight = FontWeight.SemiBold,
                color = YesMasterColors.TextSecondary,
            )
        }
    }
}

@Composable
private fun ScreenColumn(content: @Composable ColumnScope.() -> Unit) {
    StudioBackground {
        Column(
            Modifier
                .fillMaxSize()
                .verticalScroll(rememberScrollState())
                .padding(horizontal = 20.dp, vertical = 28.dp),
            verticalArrangement = Arrangement.spacedBy(16.dp),
            content = content,
        )
    }
}

@Composable
internal fun DoneScreen(done: UiState.Done, onAgain: () -> Unit, onNew: () -> Unit) {
    val context = LocalContext.current
    ScreenColumn {
        BrandHeader("Complete")
        Text(
            "Master saved",
            fontSize = 28.sp,
            fontWeight = FontWeight.Black,
            modifier = Modifier.semantics { heading() },
        )
        Text(done.savedTo, color = YesMasterColors.TextSecondary)
        done.measurements?.let { measurements ->
            Card(
                colors = CardDefaults.cardColors(containerColor = YesMasterColors.Surface.copy(alpha = 0.92f)),
                border = androidx.compose.foundation.BorderStroke(
                    1.dp,
                    YesMasterColors.Border.copy(alpha = 0.45f),
                ),
            ) {
                Column(Modifier.padding(18.dp), verticalArrangement = Arrangement.spacedBy(6.dp)) {
                    Text(
                        "Delivered",
                        fontSize = 16.sp,
                        fontWeight = FontWeight.Black,
                        modifier = Modifier.semantics { heading() },
                    )
                    Text("Master  %.1f LUFS".format(measurements.lufsIntegrated))
                    Text("True peak  %.2f dBTP".format(measurements.truePeakDbtp))
                    Text("Dynamics  %.1f LU".format(measurements.dynamicRangeLu))
                    Text("${measurements.sampleRate / 1000.0} kHz · ${measurements.bitDepth}-bit")
                }
            }
        }
        Button(
            onClick = {
                context.tryStartActivity(
                    Intent.createChooser(DoneIntents.share(done.savedUri), "Share Master"),
                )
            },
            modifier = Modifier.fillMaxWidth(),
        ) { Text("Share Master") }
        OutlinedButton(
            onClick = { context.tryStartActivity(DoneIntents.play(done.savedUri)) },
            modifier = Modifier.fillMaxWidth(),
        ) { Text("Play Master") }
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
        BrandHeader("Needs attention")
        Text(
            "Something went wrong",
            fontSize = 26.sp,
            fontWeight = FontWeight.Black,
            modifier = Modifier.semantics { heading() },
        )
        Text(error.message, color = Color(0xFFFFB7B7), lineHeight = 22.sp)
        Spacer(Modifier.height(6.dp))
        if (onRetry != null) {
            Button(onClick = onRetry, modifier = Modifier.fillMaxWidth()) {
                Text("Retry analysis")
            }
        }
        OutlinedButton(onClick = onBack, modifier = Modifier.fillMaxWidth()) { Text("Back") }
    }
}
