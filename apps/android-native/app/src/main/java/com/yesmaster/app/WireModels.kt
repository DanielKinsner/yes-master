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

    fun analysis(json: String): WireAnalysis = gson.fromJson(json, WireAnalysis::class.java)
    fun renderJob(json: String): WireRenderJob = gson.fromJson(json, WireRenderJob::class.java)
    fun landing(json: String): WireLanding = gson.fromJson(json, WireLanding::class.java)
}
