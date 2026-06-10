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

object Wire {
    val gson: Gson = Gson()

    fun analysis(json: String): WireAnalysis = gson.fromJson(json, WireAnalysis::class.java)
    fun renderJob(json: String): WireRenderJob = gson.fromJson(json, WireRenderJob::class.java)
}
