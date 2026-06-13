import { describe, expect, it } from "vitest";

import { buildExportReport } from "./export-receipt";
import type {
  AnalysisResult,
  MasteringSettings,
  RenderJob,
  RenderedMeasurements,
} from "../bindings";

const SETTINGS: MasteringSettings = {
  preset: { kind: "universal" },
  intensity: 0.5,
  eq_sub_db: 0,
  eq_low_db: 0,
  eq_low_mid_db: 0,
  eq_mid_db: 0,
  eq_high_mid_db: 0,
  eq_high_db: 0,
  eq_sparkle_db: 0,
  volume_match: false,
  source_lufs_integrated: null,
  input_gain_db: 0,
  output_gain_db: 0,
  delivery_profile: "custom",
  album: null,
  advanced: {
    lufs_offset_db: null,
    ceiling_dbtp: null,
    width: null,
    warmth: null,
    presence_air: null,
    compression_mode: "preset",
    compression_density: null,
    compression_low_threshold_db: null,
    compression_low_ratio: null,
    compression_low_attack_ms: null,
    compression_low_release_ms: null,
    compression_mid_threshold_db: null,
    compression_mid_ratio: null,
    compression_mid_attack_ms: null,
    compression_mid_release_ms: null,
    compression_high_threshold_db: null,
    compression_high_ratio: null,
    compression_high_attack_ms: null,
    compression_high_release_ms: null,
    compression_link_stereo: null,
    bit_depth: null,
    target_sample_rate: null,
    adaptive_strength: null,
    source_profile: null,
  },
};

const ANALYSIS = {
  track_id: "t1",
  lufs_integrated: -9.5,
  lufs_short_term_max: -7.5,
  true_peak_dbtp: -0.5,
  dynamic_range_lu: 6,
  spectral_balance: { low: 0.33, mid: 0.34, high: 0.33 },
  transient_density: 0.5,
  stereo_width: 0.5,
  recommended_universal: SETTINGS,
  measured_at_iso: "2026-06-09T00:00:00+00:00",
} as AnalysisResult;

const MEASUREMENTS: RenderedMeasurements = {
  lufs_integrated: -13.5,
  true_peak_dbtp: -1.25,
  dynamic_range_lu: 8.5,
  sample_rate: 48_000,
  bit_depth: 24,
  effective_adaptive_strength: 0.5,
  source_profile_digest: "bass +1.2",
  confidence_digest: null,
  compression_digest:
    "compression eased low 20% / mid 16% / high 25%; stand-down 0.00; density confidence 1.00",
};

function job(measurements: RenderedMeasurements | null): RenderJob {
  return {
    id: "job-1",
    kind: "master",
    target_tracks: ["t1"],
    status: { status: "done" },
    progress: 1,
    started_at_iso: "2026-06-09T00:00:00+00:00",
    output_paths: ["out/track.master.wav"],
    measurements,
  };
}

function build(measurements: RenderedMeasurements | null) {
  return buildExportReport({
    trackId: "t1",
    outputPath: "out/track.master.wav",
    job: job(measurements),
    sourceAnalysis: ANALYSIS,
    sourceFormat: "wav",
    exportSettings: SETTINGS,
  });
}

describe("buildExportReport", () => {
  it("uses rendered measurements and marks measurements_are_rendered", () => {
    const report = build(MEASUREMENTS);
    expect(report.measurements_are_rendered).toBe(true);
    expect(report.measured_lufs).toBe(-13.5);
    expect(report.measured_true_peak_dbtp).toBe(-1.25);
    expect(report.measured_dynamic_range_lu).toBe(8.5);
    expect(report.sample_rate).toBe(48_000);
    expect(report.bit_depth).toBe(24);
    expect(report.effective_adaptive_strength).toBe(0.5);
    expect(report.source_profile_digest).toBe("bass +1.2");
    expect(report.compression_digest).toContain("compression eased low 20%");
  });

  it("falls back to source analysis and stays unmarked without measurements", () => {
    // measurements_are_rendered MUST be false here — it gates the backend's
    // target_not_reached advisory, which may never compare a SOURCE loudness
    // against the delivery target.
    const report = build(null);
    expect(report.measurements_are_rendered).toBe(false);
    expect(report.measured_lufs).toBe(-9.5);
    expect(report.measured_true_peak_dbtp).toBe(-0.5);
    expect(report.measured_dynamic_range_lu).toBe(6);
    expect(report.sample_rate).toBe(44_100);
    expect(report.bit_depth).toBe(24);
    expect(report.effective_adaptive_strength).toBe(0);
    expect(report.source_profile_digest).toBeNull();
    expect(report.compression_digest).toBeNull();
  });

  it("prefers the user bit depth over the 24-bit default when unrendered", () => {
    const settings = {
      ...SETTINGS,
      advanced: { ...SETTINGS.advanced, bit_depth: 16 },
    };
    const report = buildExportReport({
      trackId: "t1",
      outputPath: "out/track.master.wav",
      job: job(null),
      sourceAnalysis: ANALYSIS,
      sourceFormat: "wav",
      exportSettings: settings,
    });
    expect(report.bit_depth).toBe(16);
  });
});
