package com.yesmaster.app

/**
 * JNI surface over the Rust audition engine (`audition` module of
 * apps/android-native/rust). One handle = one imported track auditioning
 * through the facade's LiveStream — the exact MasteringChain desktop and the
 * iPhone app run, so what plays here is what Create Master renders.
 *
 * Contract mirrors NativeBridge: the bridge never throws; the one
 * string-returning call follows the error-JSON shape and nullable externs
 * fold a worst-case null back into it.
 */
object AuditionBridge {
    init {
        System.loadLibrary("yes_master_android_bridge")
    }

    private const val NULL_BRIDGE = """{"error":"native bridge returned null"}"""

    private external fun measureLandingNative(
        handle: Long,
        preset: String?,
        intensity: Float,
        lufsTarget: Float,
    ): String?

    /** 0 = creation failed (missing/undecodable file). Decodes the whole
     *  file — call from a background dispatcher. */
    external fun createNative(
        sourcePath: String,
        preset: String?,
        intensity: Float,
        lufsTarget: Float,
    ): Long

    external fun destroyNative(handle: Long)
    external fun startNative(handle: Long): Boolean
    external fun pauseNative(handle: Long)
    external fun isPlayingNative(handle: Long): Boolean

    /** true = Original (dry); false = Mastered. Playhead is preserved. */
    external fun setBypassNative(handle: Long, original: Boolean)
    external fun setParamsNative(
        handle: Long,
        preset: String?,
        intensity: Float,
        lufsTarget: Float,
    )

    external fun setVolumeMatchNative(handle: Long, linearGain: Float)
    external fun setLandingGainNative(handle: Long, linearGain: Float)
    external fun seekNative(handle: Long, positionSeconds: Double)
    external fun positionSecondsNative(handle: Long): Double
    external fun durationSecondsNative(handle: Long): Double

    /** Volume Match gain for the side being heard — single-source Rust math
     *  (mirrors the iPhone's VolumeMatch.swift, pinned by host tests). */
    external fun volumeMatchGainNative(sideLufs: Double, otherLufs: Double): Float

    /** Slow (masters a measurement window) — call from a background
     *  dispatcher. Safe while playback runs. */
    fun measureLanding(
        handle: Long,
        preset: String?,
        intensity: Float,
        lufsTarget: Float,
    ): WireLanding = Wire.landing(
        measureLandingNative(handle, preset, intensity, lufsTarget) ?: NULL_BRIDGE
    )
}
