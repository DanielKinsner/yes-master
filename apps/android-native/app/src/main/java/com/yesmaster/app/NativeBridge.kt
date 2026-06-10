package com.yesmaster.app

/**
 * JNI surface over the shared Rust engine (apps/android-native/rust). Same
 * contract the iPhone app drives through Swift FFI: paths in, JSON out,
 * errors as an `error` key in the payload — the bridge never throws.
 */
object NativeBridge {
    init {
        System.loadLibrary("yes_master_android_bridge")
    }

    external fun bridgeVersion(): String
    external fun supportsImportExtension(extension: String): Boolean
    external fun analyzeFileJson(path: String): String
    external fun renderMasterWithOptionsJson(
        sourcePath: String,
        outputDir: String,
        preset: String?,
        intensity: Float,
        lufsTarget: Float,
    ): String
}
