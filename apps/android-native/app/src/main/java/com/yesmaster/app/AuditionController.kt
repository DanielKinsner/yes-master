package com.yesmaster.app

import android.content.BroadcastReceiver
import android.content.Context
import android.content.Intent
import android.content.IntentFilter
import android.media.AudioAttributes
import android.media.AudioFocusRequest
import android.media.AudioManager
import androidx.core.content.ContextCompat
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.Job
import kotlinx.coroutines.currentCoroutineContext
import kotlinx.coroutines.delay
import kotlinx.coroutines.ensureActive
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.isActive
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext

/**
 * Pure decision helpers, split out so the JVM test lane can pin them
 * without loading the native library. The numbers mirror the iPhone's
 * AuditionController exactly.
 */
internal object AuditionMath {
    /**
     * (side, other) LUFS for the Volume Match gain of the side currently
     * being heard — Original listens against the mastered loudness and vice
     * versa (iPhone's currentSideGain side selection).
     */
    fun volumeMatchSides(
        listeningOriginal: Boolean,
        originalLufs: Double,
        masteredLufs: Double,
    ): Pair<Double, Double> =
        if (listeningOriginal) originalLufs to masteredLufs else masteredLufs to originalLufs

    /** Pressing play while parked at the end restarts from the top. */
    fun shouldRestartFromTop(positionSeconds: Double, durationSeconds: Double): Boolean =
        durationSeconds > 0 && positionSeconds >= durationSeconds - 0.05

    /** Playback tick: reaching the end auto-pauses so play means "again". */
    fun reachedEnd(positionSeconds: Double, durationSeconds: Double): Boolean =
        durationSeconds > 0 && positionSeconds >= durationSeconds - 0.02
}

/**
 * Coroutine helper for the one ownership hazard in [AuditionController.attach]:
 * the background decode + handle creation. Split out (like [AuditionMath]) so
 * the JVM test lane can pin the cancellation behavior without loading the
 * native library or an Android [Context].
 */
internal object AuditionAttach {
    /**
     * Run [create] (the whole-file decode + native engine alloc) on
     * [ioDispatcher] and return its handle for the caller to own — UNLESS this
     * coroutine was cancelled while the decode was in flight (a different track
     * was attached). [create] is an uninterruptible JNI call, so cancellation
     * cannot stop it: it always completes and yields a live handle. If we are
     * no longer the active attach, that orphaned handle is freed via [destroy]
     * here so it can never leak.
     */
    suspend fun createOrRelease(
        ioDispatcher: CoroutineDispatcher,
        create: () -> Long,
        destroy: (Long) -> Unit,
    ): Long {
        // The handle is captured into an outer var INSIDE the block, not via
        // `val h = withContext { create() }`: if we were cancelled during the
        // decode, `withContext` throws CancellationException as it resumes and
        // discards the block's return value, so the handle would never reach
        // an assignment outside the block — and leak. Held here, the `finally`
        // can always free it if ownership is never transferred to the caller.
        var created = 0L
        try {
            withContext(ioDispatcher) { created = create() }
            currentCoroutineContext().ensureActive()
            val handle = created
            created = 0L // ownership transferred to the caller
            return handle
        } finally {
            if (created != 0L) destroy(created)
        }
    }
}

data class AuditionUi(
    val status: Status = Status.Unavailable,
    val playing: Boolean = false,
    val positionSeconds: Double = 0.0,
    val durationSeconds: Double = 0.0,
    val listeningOriginal: Boolean = false,
    val volumeMatch: Boolean = false,
    val masteredLufs: Double? = null,
    val notice: String? = null,
) {
    enum class Status { Unavailable, Preparing, Ready, Failed }
}

/**
 * Live audition for the Ready screen — the Android mirror of the iPhone's
 * AuditionController + AudioSessionController, over the same Rust engine:
 *
 * - One native handle per imported track ([AuditionBridge]); the playhead
 *   survives Original/Mastered flips by construction.
 * - The loudness landing is ALWAYS applied (desktop Standard's forced
 *   WYSIWYG): re-measured in the background ~250 ms after settings settle,
 *   so what you hear lands where Create Master will.
 * - Volume Match is the only audition-loudness toggle (off by default,
 *   audition-only, never touches export) — the gain matches the heard side
 *   to the quieter side via the single-source Rust formula.
 * - Audio focus + becoming-noisy: pause on loss/unplug (Android's analog of
 *   the iPhone's interruption/route handling), resume after a transient
 *   loss ends.
 */
class AuditionController(
    private val context: Context,
    private val scope: CoroutineScope,
) {
    private val _state = MutableStateFlow(AuditionUi())
    val state: StateFlow<AuditionUi> = _state.asStateFlow()

    private var handle = 0L
    private var sourcePath: String? = null
    private var originalLufs: Double? = null
    private var preset: String? = null
    private var intensity = 0.5f
    private var lufsTarget = -11f

    private var prepareJob: Job? = null
    private var landingJob: Job? = null
    private var pollJob: Job? = null
    private var resumeAfterFocusLoss = false

    private val audioManager =
        context.getSystemService(Context.AUDIO_SERVICE) as AudioManager
    private var focusRequest: AudioFocusRequest? = null
    private val focusListener = AudioManager.OnAudioFocusChangeListener { change ->
        when (change) {
            AudioManager.AUDIOFOCUS_LOSS -> {
                resumeAfterFocusLoss = false
                pause("Paused — another app took over audio.")
            }
            AudioManager.AUDIOFOCUS_LOSS_TRANSIENT,
            AudioManager.AUDIOFOCUS_LOSS_TRANSIENT_CAN_DUCK -> {
                resumeAfterFocusLoss = _state.value.playing
                pause("Paused for an interruption.")
            }
            AudioManager.AUDIOFOCUS_GAIN -> {
                if (resumeAfterFocusLoss) {
                    resumeAfterFocusLoss = false
                    play()
                }
            }
        }
    }

    /** Headphones unplugged: pause rather than blast the speaker. */
    private val noisyReceiver = object : BroadcastReceiver() {
        override fun onReceive(context: Context?, intent: Intent?) {
            pause("Output changed. Press play to continue.")
        }
    }
    private var noisyRegistered = false

    /**
     * Bind the controller to an analyzed track. Re-attaching the same path
     * (e.g. "Master again" returning to Ready) keeps the existing decoded
     * handle; a new path decodes in the background.
     */
    fun attach(
        path: String,
        analysisLufs: Double,
        preset: String?,
        intensity: Float,
        lufsTarget: Float,
    ) {
        this.originalLufs = analysisLufs
        this.preset = preset
        this.intensity = intensity
        this.lufsTarget = lufsTarget
        if (path == sourcePath && handle != 0L) {
            pushParams()
            return
        }
        release()
        sourcePath = path
        _state.update { AuditionUi(status = AuditionUi.Status.Preparing) }
        prepareJob = scope.launch {
            val created = AuditionAttach.createOrRelease(
                ioDispatcher = Dispatchers.IO,
                create = { AuditionBridge.createNative(path, preset, intensity, lufsTarget) },
                destroy = { AuditionBridge.destroyNative(it) },
            )
            if (created == 0L) {
                _state.update {
                    it.copy(
                        status = AuditionUi.Status.Failed,
                        notice = "Live audition is unavailable for this file.",
                    )
                }
                return@launch
            }
            handle = created
            val duration = AuditionBridge.durationSecondsNative(created)
            AuditionBridge.setBypassNative(created, _state.value.listeningOriginal)
            _state.update {
                it.copy(
                    status = AuditionUi.Status.Ready,
                    durationSeconds = duration,
                    positionSeconds = 0.0,
                    playing = false,
                    notice = null,
                )
            }
            scheduleLandingRefresh()
        }
    }

    fun togglePlay() {
        if (_state.value.playing) pause() else play()
    }

    private fun play() {
        val h = handle
        if (h == 0L || _state.value.status != AuditionUi.Status.Ready) return
        val ui = _state.value
        if (AuditionMath.shouldRestartFromTop(ui.positionSeconds, ui.durationSeconds)) {
            AuditionBridge.seekNative(h, 0.0)
        }
        if (!requestFocus()) {
            _state.update { it.copy(notice = "Couldn't get audio focus.") }
            return
        }
        if (!AuditionBridge.startNative(h)) {
            abandonFocus()
            _state.update { it.copy(notice = "Playback could not start.") }
            return
        }
        registerNoisy()
        _state.update { it.copy(playing = true, notice = null) }
        startPolling()
    }

    fun pause(notice: String? = null) {
        val h = handle
        pollJob?.cancel()
        if (h != 0L) {
            AuditionBridge.pauseNative(h)
            _state.update {
                it.copy(
                    playing = false,
                    positionSeconds = AuditionBridge.positionSecondsNative(h),
                    notice = notice,
                )
            }
        }
        unregisterNoisy()
        abandonFocus()
    }

    fun seek(seconds: Double) {
        val h = handle
        if (h == 0L) return
        AuditionBridge.seekNative(h, seconds)
        // The native cursor moves at the next processed block; while paused,
        // the UI owns the displayed position until playback resumes.
        _state.update {
            it.copy(positionSeconds = seconds.coerceIn(0.0, it.durationSeconds))
        }
    }

    /** Original/Mastered on one live timeline — the cursor never moves. */
    fun setListeningOriginal(original: Boolean) {
        if (_state.value.listeningOriginal == original) return
        _state.update { it.copy(listeningOriginal = original) }
        val h = handle
        if (h != 0L) {
            AuditionBridge.setBypassNative(h, original)
            applyVolumeMatch()
        }
    }

    fun toggleVolumeMatch() {
        _state.update { it.copy(volumeMatch = !it.volumeMatch) }
        applyVolumeMatch()
    }

    /** Style/Loudness/Intensity changed — retune the live chain, then
     *  re-land the loudness once the change settles. */
    fun updateParams(preset: String?, intensity: Float, lufsTarget: Float) {
        this.preset = preset
        this.intensity = intensity
        this.lufsTarget = lufsTarget
        pushParams()
    }

    private fun pushParams() {
        val h = handle
        if (h == 0L) return
        AuditionBridge.setParamsNative(h, preset, intensity, lufsTarget)
        scheduleLandingRefresh()
    }

    /**
     * Measure the loudness landing for the current settings and apply it
     * (forced WYSIWYG) plus the mastered LUFS Volume Match needs. Mirrors
     * the iPhone's 250 ms settle debounce.
     */
    private fun scheduleLandingRefresh() {
        landingJob?.cancel()
        val h = handle
        if (h == 0L) return
        val (preset, intensity, lufsTarget) = Triple(preset, intensity, lufsTarget)
        landingJob = scope.launch {
            delay(250)
            val landing = withContext(Dispatchers.Default) {
                AuditionBridge.measureLanding(h, preset, intensity, lufsTarget)
            }
            if (handle != h || landing.error != null) return@launch
            AuditionBridge.setLandingGainNative(h, landing.gainLin)
            landing.masteredLufs?.let { lufs ->
                _state.update { it.copy(masteredLufs = lufs) }
            }
            applyVolumeMatch()
        }
    }

    private fun applyVolumeMatch() {
        val h = handle
        if (h == 0L) return
        val ui = _state.value
        val original = originalLufs
        val mastered = ui.masteredLufs
        val gain = if (ui.volumeMatch && original != null && mastered != null) {
            val (side, other) =
                AuditionMath.volumeMatchSides(ui.listeningOriginal, original, mastered)
            AuditionBridge.volumeMatchGainNative(side, other)
        } else {
            1f
        }
        AuditionBridge.setVolumeMatchNative(h, gain)
    }

    private fun startPolling() {
        pollJob?.cancel()
        pollJob = scope.launch {
            while (isActive) {
                val h = handle
                if (h == 0L) break
                val position = AuditionBridge.positionSecondsNative(h)
                val ui = _state.value
                if (AuditionMath.reachedEnd(position, ui.durationSeconds)) {
                    pause("Reached the end. Press play to listen again.")
                    break
                }
                if (ui.playing && !AuditionBridge.isPlayingNative(h)) {
                    // The device stream died (route change); the engine
                    // rebuilds it on the next start.
                    pause("Output changed. Press play to continue.")
                    break
                }
                _state.update { it.copy(positionSeconds = position) }
                delay(100)
            }
        }
    }

    fun release() {
        prepareJob?.cancel()
        landingJob?.cancel()
        pollJob?.cancel()
        unregisterNoisy()
        abandonFocus()
        val h = handle
        handle = 0L
        sourcePath = null
        if (h != 0L) {
            AuditionBridge.destroyNative(h)
        }
        _state.update { AuditionUi() }
    }

    private fun requestFocus(): Boolean {
        val request = focusRequest ?: AudioFocusRequest.Builder(AudioManager.AUDIOFOCUS_GAIN)
            .setAudioAttributes(
                AudioAttributes.Builder()
                    .setUsage(AudioAttributes.USAGE_MEDIA)
                    .setContentType(AudioAttributes.CONTENT_TYPE_MUSIC)
                    .build()
            )
            .setOnAudioFocusChangeListener(focusListener)
            .build()
            .also { focusRequest = it }
        return audioManager.requestAudioFocus(request) ==
            AudioManager.AUDIOFOCUS_REQUEST_GRANTED
    }

    private fun abandonFocus() {
        focusRequest?.let { audioManager.abandonAudioFocusRequest(it) }
    }

    private fun registerNoisy() {
        if (noisyRegistered) return
        ContextCompat.registerReceiver(
            context,
            noisyReceiver,
            IntentFilter(AudioManager.ACTION_AUDIO_BECOMING_NOISY),
            ContextCompat.RECEIVER_NOT_EXPORTED,
        )
        noisyRegistered = true
    }

    private fun unregisterNoisy() {
        if (!noisyRegistered) return
        runCatching { context.unregisterReceiver(noisyReceiver) }
        noisyRegistered = false
    }
}
