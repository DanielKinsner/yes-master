package com.yesmaster.app.ui.theme

import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.darkColorScheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.graphics.Color

/**
 * YES Master's brand colors, named once so future visual branding edits one
 * file instead of hunting literals through screens (U19).
 */
object YesMasterColors {
    /** Accent blue — primary actions, selected states. */
    val Accent = Color(0xFF4F9EFF)
    val AccentBright = Color(0xFF8CCBFF)
    val Cyan = Color(0xFF45E0F5)
    val Amber = Color(0xFFFFB84D)
    val Oomph = Color(0xFFF77171)

    /** App background — near-black console. */
    val Background = Color(0xFF020817)
    val BackgroundMid = Color(0xFF07132B)
    val BackgroundDeep = Color(0xFF01040C)

    /** Card / sheet surface. */
    val Surface = Color(0xFF07142E)
    val SurfaceDeep = Color(0xFF030816)

    /** Secondary surface (unselected tiles, field chrome). */
    val SurfaceVariant = Color(0xFF0A1936)
    val Border = Color(0xFF3E64A8)
    val TextPrimary = Color(0xFFF2F6FF)
    val TextSecondary = Color(0xFFA6B5D5)
    val TextMuted = Color(0xFF7284A8)
}

internal val YesMasterDarkScheme = darkColorScheme(
    primary = YesMasterColors.Accent,
    background = YesMasterColors.Background,
    surface = YesMasterColors.Surface,
    surfaceVariant = YesMasterColors.SurfaceVariant,
    onPrimary = YesMasterColors.TextPrimary,
    onBackground = YesMasterColors.TextPrimary,
    onSurface = YesMasterColors.TextPrimary,
    onSurfaceVariant = YesMasterColors.TextSecondary,
)

/**
 * The app theme. **Policy (deliberate): dark console in BOTH system modes.**
 * YES Master is a metering/audition surface like the desktop app — a light
 * variant is a future brand decision, not something the OS toggle should
 * improvise. So this theme ignores `isSystemInDarkTheme()` on purpose; when a
 * light brand exists, this is the one place it gets wired.
 */
@Composable
fun YesMasterTheme(content: @Composable () -> Unit) {
    MaterialTheme(colorScheme = YesMasterDarkScheme, content = content)
}
