package com.yesmaster.app

import androidx.compose.runtime.Composable
import androidx.compose.runtime.CompositionLocalProvider
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.semantics.LiveRegionMode
import androidx.compose.ui.semantics.SemanticsProperties
import androidx.compose.ui.test.SemanticsMatcher
import androidx.compose.ui.test.assert
import androidx.compose.ui.test.assertIsDisplayed
import androidx.compose.ui.test.assertIsEnabled
import androidx.compose.ui.test.assertIsNotEnabled
import androidx.compose.ui.test.assertIsNotSelected
import androidx.compose.ui.test.assertIsOff
import androidx.compose.ui.test.assertIsOn
import androidx.compose.ui.test.assertIsSelected
import androidx.compose.ui.test.junit4.createComposeRule
import androidx.compose.ui.test.onNodeWithContentDescription
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.performClick
import androidx.compose.ui.test.performScrollTo
import androidx.compose.ui.unit.Density
import com.yesmaster.app.ui.theme.YesMasterColors
import com.yesmaster.app.ui.theme.YesMasterTheme
import androidx.compose.material3.MaterialTheme
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner
import org.robolectric.annotation.Config

/**
 * U19 Compose semantics lane — pins what TalkBack actually receives, on the
 * JVM (Robolectric), so it gates CI's `gradlew test` with no emulator. A
 * physical-device TalkBack pass remains a separate pre-release gate (U20);
 * nothing here claims it.
 */
@RunWith(RobolectricTestRunner::class)
class UiSemanticsTest {
    @get:Rule
    val compose = createComposeRule()

    private fun readyState() = UiState.Ready(
        displayName = "song.wav",
        sourcePath = "/cache/imports/song.wav",
        analysis = WireAnalysis(
            lufsIntegrated = -12.0,
            truePeakDbtp = -1.0,
            dynamicRangeLu = 8.0,
        ),
    )

    private fun readyAudition() = AuditionUi(
        status = AuditionUi.Status.Ready,
        positionSeconds = 32.0,
        durationSeconds = 130.0,
    )

    private fun setReadyScreen(
        auditionUi: AuditionUi = readyAudition(),
        actions: AuditionActions = AuditionActions(),
        onChoices: (StandardStyle, StandardLoudness, Float) -> Unit = { _, _, _ -> },
    ) {
        compose.setContent {
            YesMasterTheme {
                ReadyScreen(
                    ready = readyState(),
                    auditionUi = auditionUi,
                    auditionActions = actions,
                    onChoices = onChoices,
                    onMaster = { _, _, _ -> },
                    onImportOther = {},
                )
            }
        }
    }

    // ---- Named value controls -------------------------------------------

    @Test
    fun intensitySliderHasAnAccessibleName() {
        setReadyScreen()
        compose.onNodeWithContentDescription("Intensity")
            .assertExists()
            .assertIsEnabled()
    }

    @Test
    fun seekSliderAnnouncesTimeNotAFraction() {
        setReadyScreen()
        compose.onNodeWithContentDescription("Playback position")
            .assertExists()
            .assert(
                SemanticsMatcher.expectValue(
                    SemanticsProperties.StateDescription,
                    "0:32 of 2:10",
                ),
            )
    }

    // ---- Volume Match is one named switch --------------------------------

    @Test
    fun volumeMatchIsOneNamedToggleThatToggles() {
        var toggled = 0
        setReadyScreen(
            actions = AuditionActions(onToggleVolumeMatch = { toggled++ }),
        )
        compose.onNodeWithText("Volume Match")
            .assertExists()
            .assertIsOff()
            .performClick()
        assertEquals(1, toggled)
    }

    @Test
    fun volumeMatchReadsOnWhenOn() {
        setReadyScreen(auditionUi = readyAudition().copy(volumeMatch = true))
        compose.onNodeWithText("Volume Match").assertIsOn()
    }

    // ---- Disabled states carry a visible reason --------------------------

    @Test
    fun preparingDisablesTransportAndShowsTheReason() {
        setReadyScreen(auditionUi = AuditionUi(status = AuditionUi.Status.Preparing))
        compose.onNodeWithText("Play").assertIsNotEnabled()
        compose.onNodeWithContentDescription("Playback position").assertIsNotEnabled()
        compose.onNodeWithText("Mastered").assertIsNotEnabled()
        compose.onNodeWithText("Original").assertIsNotEnabled()
        compose.onNodeWithText("Volume Match").assertIsNotEnabled()
        compose.onNodeWithText("Preparing…").assertExists()
    }

    // ---- Style tiles expose selection, not just color --------------------

    @Test
    fun styleTilesExposeSelectionState() {
        var chosen: StandardStyle? = null
        setReadyScreen(onChoices = { style, _, _ -> chosen = style })
        compose.onNodeWithText("Universal").assertIsSelected()
        compose.onNodeWithText("Clarity").assertIsNotSelected()
        compose.onNodeWithText("Clarity").performClick()
        compose.onNodeWithText("Clarity").assertIsSelected()
        compose.onNodeWithText("Universal").assertIsNotSelected()
        assertEquals(StandardStyle.BRIGHT, chosen)
    }

    // ---- Interruption notices are announced ------------------------------

    @Test
    fun interruptionNoticeIsAPoliteLiveRegion() {
        compose.setContent {
            YesMasterTheme {
                AuditionCard(
                    ui = readyAudition().copy(
                        playing = false,
                        notice = "Paused for an interruption.",
                    ),
                    actions = AuditionActions(),
                )
            }
        }
        compose.onNodeWithText("Paused for an interruption.")
            .assertExists()
            .assert(
                SemanticsMatcher.expectValue(
                    SemanticsProperties.LiveRegion,
                    LiveRegionMode.Polite,
                ),
            )
    }

    @Test
    fun workingLabelIsAPoliteLiveRegion() {
        compose.setContent { YesMasterTheme { WorkingScreen("Analyzing…") } }
        compose.onNodeWithText("Analyzing…")
            .assert(
                SemanticsMatcher.expectValue(
                    SemanticsProperties.LiveRegion,
                    LiveRegionMode.Polite,
                ),
            )
    }

    // ---- Receipt screen: actions present, reachable, wired ---------------

    private fun doneState() = UiState.Done(
        displayName = "song.wav",
        savedTo = "Music/YES Master/song - Master.wav",
        savedUri = "content://media/external/audio/1",
        measurements = WireMeasurements(
            lufsIntegrated = -11.2,
            truePeakDbtp = -1.1,
            dynamicRangeLu = 7.5,
            sampleRate = 44100,
            bitDepth = 24,
        ),
        previous = readyState(),
    )

    @Test
    fun receiptActionsArePresentAndWired() {
        var again = 0
        var fresh = 0
        compose.setContent {
            YesMasterTheme {
                DoneScreen(doneState(), onAgain = { again++ }, onNew = { fresh++ })
            }
        }
        compose.onNodeWithText("Share Master").assertExists()
        compose.onNodeWithText("Play Master").assertExists()
        compose.onNodeWithText("Master again with different settings")
            .performScrollTo()
            .performClick()
        compose.onNodeWithText("Start over")
            .performScrollTo()
            .performClick()
        assertEquals(1, again)
        assertEquals(1, fresh)
    }

    // Small display + doubled font scale: every receipt action must still be
    // reachable by scrolling. Before U19 the Done column did not scroll, so
    // overflowing content simply pushed its own buttons off-screen.
    @Test
    @Config(qualifiers = "w320dp-h420dp")
    fun receiptScrollsToItsActionsOnASmallDisplayAtLargeFont() {
        compose.setContent {
            val base = LocalDensity.current
            CompositionLocalProvider(
                LocalDensity provides Density(base.density, fontScale = 2f),
            ) {
                YesMasterTheme {
                    DoneScreen(doneState(), onAgain = {}, onNew = {})
                }
            }
        }
        compose.onNodeWithText("Start over")
            .performScrollTo()
            .assertIsDisplayed()
    }

    @Test
    @Config(qualifiers = "w320dp-h420dp")
    fun errorScreenScrollsToItsActionsAtLargeFont() {
        compose.setContent {
            val base = LocalDensity.current
            CompositionLocalProvider(
                LocalDensity provides Density(base.density, fontScale = 2f),
            ) {
                YesMasterTheme {
                    ErrorScreen(
                        UiState.Error(
                            message = "Analysis failed: the file could not be decoded. " +
                                "Try re-exporting it from your DAW as WAV or FLAC.",
                            retrySourcePath = "/cache/imports/song.wav",
                        ),
                        onBack = {},
                        onRetry = {},
                    )
                }
            }
        }
        compose.onNodeWithText("Back")
            .performScrollTo()
            .assertIsDisplayed()
    }

    @Test
    fun errorScreenOffersRetryOnlyWhenRetryIsPossible() {
        compose.setContent {
            YesMasterTheme {
                ErrorScreen(UiState.Error(message = "boom"), onBack = {}, onRetry = null)
            }
        }
        compose.onNodeWithText("Retry analysis").assertDoesNotExist()
        compose.onNodeWithText("Back").assertExists()
    }

    // ---- Dark/light policy: pinned dark console in BOTH modes ------------

    private fun themedBackground(onRead: (Color) -> Unit): @Composable () -> Unit = {
        YesMasterTheme { onRead(MaterialTheme.colorScheme.background) }
    }

    @Test
    fun themeIsTheDarkConsoleInLightMode() {
        var background: Color? = null
        compose.setContent(themedBackground { background = it })
        assertEquals(YesMasterColors.Background, background)
    }

    @Test
    @Config(qualifiers = "night")
    fun themeIsTheDarkConsoleInDarkMode() {
        var background: Color? = null
        compose.setContent(themedBackground { background = it })
        assertEquals(YesMasterColors.Background, background)
        assertTrue(
            "policy: one pinned scheme, not an OS-driven pair",
            YesMasterColors.Background != Color.White,
        )
    }
}
