// Pure constructor for the track-export receipt payload, extracted from
// useTrackMaster's runExport (consolidated backlog B4.3) so its fallback
// semantics are unit-testable.
//
// Codex audit 2026-05-13 P0: the receipt must describe the RENDERED output,
// not the source analysis. Single-track renders return post-chain
// measurements on job.measurements; the source-analysis fallback exists only
// for render paths that don't measure yet (album masters).

import type {
  AnalysisResult,
  ExportReport,
  MasteringSettings,
  RenderJob,
  TrackId,
} from "../bindings";

export function buildExportReport(args: {
  trackId: TrackId;
  outputPath: string;
  job: RenderJob;
  sourceAnalysis: AnalysisResult;
  sourceFormat: string;
  exportSettings: MasteringSettings;
}): ExportReport {
  const { trackId, outputPath, job, sourceAnalysis, sourceFormat, exportSettings } = args;
  const m = job.measurements ?? null;
  return {
    track_id: trackId,
    output_path: outputPath,
    measured_lufs: m?.lufs_integrated ?? sourceAnalysis.lufs_integrated,
    measured_true_peak_dbtp: m?.true_peak_dbtp ?? sourceAnalysis.true_peak_dbtp,
    measured_dynamic_range_lu:
      m?.dynamic_range_lu ?? sourceAnalysis.dynamic_range_lu,
    source_format: sourceFormat,
    destination_format: "wav",
    sample_rate: m?.sample_rate ?? 44_100,
    bit_depth: m?.bit_depth ?? exportSettings.advanced.bit_depth ?? 24,
    // B5 — adaptive traceability, sourced from the backend render (which
    // resolved the profile; the FE no longer holds it).
    effective_adaptive_strength: m?.effective_adaptive_strength ?? 0,
    source_profile_digest: m?.source_profile_digest ?? null,
    confidence_digest: m?.confidence_digest ?? null,
    compression_digest: m?.compression_digest ?? null,
    // Gates the backend's target_not_reached check: only a rendered-output
    // measurement may be compared against the delivery target. MUST stay
    // exactly `m != null`.
    measurements_are_rendered: m != null,
    checks: [],
  };
}
