package com.yesmaster.app

import com.google.gson.Gson
import com.google.gson.annotations.SerializedName

/**
 * Kotlin mirror of the FE-relevant wire subset, decoding the exact keys the
 * desktop drift gate pins in src/wire-samples.json (this module's JVM unit
 * test is that gate's fourth consumer) and the iPhone app decodes in Swift.
 * Plain Gson: no codegen, runs on the JVM test lane.
 */

data class WireAnalysis(
    @SerializedName("lufs_integrated") val lufsIntegrated: Double,
    @SerializedName("true_peak_dbtp") val truePeakDbtp: Double,
    @SerializedName("dynamic_range_lu") val dynamicRangeLu: Double,
    @SerializedName("error") val error: String? = null,
)

data class WireMeasurements(
    @SerializedName("lufs_integrated") val lufsIntegrated: Double,
    @SerializedName("true_peak_dbtp") val truePeakDbtp: Double,
    @SerializedName("dynamic_range_lu") val dynamicRangeLu: Double,
    @SerializedName("sample_rate") val sampleRate: Int,
    @SerializedName("bit_depth") val bitDepth: Int,
    @SerializedName("effective_adaptive_strength") val effectiveAdaptiveStrength: Double = 0.0,
    @SerializedName("source_profile_digest") val sourceProfileDigest: String? = null,
    @SerializedName("confidence_digest") val confidenceDigest: String? = null,
    @SerializedName("compression_digest") val compressionDigest: String? = null,
)

data class WireRenderJob(
    @SerializedName("output_paths") val outputPaths: List<String> = emptyList(),
    @SerializedName("measurements") val measurements: WireMeasurements? = null,
    @SerializedName("error") val error: String? = null,
)

/**
 * Live-audition landing measurement. `mastered_lufs` arrives as JSON null
 * when the measurement was unavailable (the Rust side maps its non-finite
 * sentinel to null — pinned by `landing_json_carries_every_key_kotlin_decodes`
 * in the android bridge crate).
 */
data class WireLanding(
    @SerializedName("gain_lin") val gainLin: Float = 1f,
    @SerializedName("mastered_lufs") val masteredLufs: Double? = null,
    @SerializedName("error") val error: String? = null,
)

object Wire {
    val gson: Gson = Gson()

    // §6 — the native bridge hands these JSON strings to Gson on a background
    // dispatcher (e.g. AuditionController.measureLanding). A malformed or empty
    // payload would otherwise throw JsonSyntaxException (or return null) and
    // crash the coroutine. Decode defensively: any parse failure becomes an
    // error-bearing default that callers already short-circuit on via `.error`.
    private fun <T> parse(json: String, klass: Class<T>, fallback: () -> T): T =
        try {
            gson.fromJson(json, klass) ?: fallback()
        } catch (_: com.google.gson.JsonParseException) {
            fallback()
        }

    fun analysis(json: String): WireAnalysis =
        parse(json, WireAnalysis::class.java) {
            WireAnalysis(0.0, 0.0, 0.0, error = "malformed analysis payload")
        }

    fun renderJob(json: String): WireRenderJob =
        parse(json, WireRenderJob::class.java) {
            WireRenderJob(error = "malformed render-job payload")
        }

    fun landing(json: String): WireLanding =
        parse(json, WireLanding::class.java) {
            WireLanding(error = "malformed landing payload")
        }
}
