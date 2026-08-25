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
    val Accent = Color(0xFF5EA8FF)

    /** App background — near-black console. */
    val Background = Color(0xFF0B0D10)

    /** Card / sheet surface. */
    val Surface = Color(0xFF14181D)

    /** Secondary surface (unselected tiles, field chrome). */
    val SurfaceVariant = Color(0xFF1B2026)
}

internal val YesMasterDarkScheme = darkColorScheme(
    primary = YesMasterColors.Accent,
    background = YesMasterColors.Background,
    surface = YesMasterColors.Surface,
    surfaceVariant = YesMasterColors.SurfaceVariant,
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
