package com.yesmaster.app

/**
 * JNI surface over the shared Rust engine (apps/android-native/rust). Same
 * contract the iPhone app drives through Swift FFI: paths in, JSON out,
 * errors as an `error` key in the payload — the bridge never throws.
 *
 * The externs are declared nullable because JNI can, in the worst case
 * (JVM OOM while allocating even the fallback error string), hand back a
 * null reference; the public wrappers fold that back into the error-JSON
 * contract so callers never see a null or an exception.
 */
object NativeBridge {
    init {
        System.loadLibrary("yes_master_android_bridge")
    }

    private const val NULL_BRIDGE = """{"error":"native bridge returned null"}"""

    private external fun bridgeVersionNative(): String?
    private external fun analyzeFileJsonNative(path: String): String?
    private external fun renderMasterWithOptionsJsonNative(
        sourcePath: String,
        outputDir: String,
        preset: String?,
        intensity: Float,
        lufsTarget: Float,
    ): String?

    external fun supportsImportExtension(extension: String): Boolean

    fun bridgeVersion(): String = bridgeVersionNative() ?: NULL_BRIDGE

    fun analyzeFileJson(path: String): String = analyzeFileJsonNative(path) ?: NULL_BRIDGE

    fun renderMasterWithOptionsJson(
        sourcePath: String,
        outputDir: String,
        preset: String?,
        intensity: Float,
        lufsTarget: Float,
    ): String = renderMasterWithOptionsJsonNative(
        sourcePath, outputDir, preset, intensity, lufsTarget,
    ) ?: NULL_BRIDGE
}
