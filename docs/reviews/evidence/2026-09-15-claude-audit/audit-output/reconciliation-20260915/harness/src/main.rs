//! Independent audit harness for YES Master.
//!
//! Every DSP path here is the production implementation from the unmodified
//! `yes-master` crate (path dependency). The harness only (a) builds
//! `MasteringSettings` from a compact spec, (b) mirrors the desktop backend's
//! pre-render resolution (source profile / confidence / compression guards)
//! exactly as `render_track_master` + `populate_profile_store` do, and
//! (c) exposes stage-isolation by overriding public `ChainCoeffs` fields.
//! Nothing in the crate is re-implemented; any place that departs from the
//! desktop path is labelled ADAPTATION in a comment.

mod codex_src_fixed;
mod recon;

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use yes_master_lib::dsp::{BiquadCoeffs, ChainCoeffs, MasteringChain};
use yes_master_lib::engine::{
    analyze_tracks_core_with_progress_sync, mastering_render_to_path, preview_landing,
    AnalyzeRequest,
};
use yes_master_lib::types::*;

// ---------------------------------------------------------------------------
// Compact settings spec
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
struct Spec {
    preset: Option<String>,
    intensity: Option<f32>,
    delivery_profile: Option<String>,
    /// Custom profile only. `null` (absent) = no loudness landing at all.
    lufs_target: Option<f32>,
    ceiling_dbtp: Option<f32>,
    sample_rate: Option<u32>,
    bit_depth: Option<u16>,
    compression_mode: Option<String>,
    compression_density: Option<f32>,
    adapt_strength: Option<f32>,
    input_gain_db: Option<f32>,
    output_gain_db: Option<f32>,
    eq_sub_db: Option<f32>,
    eq_low_db: Option<f32>,
    eq_low_mid_db: Option<f32>,
    eq_mid_db: Option<f32>,
    eq_high_mid_db: Option<f32>,
    eq_high_db: Option<f32>,
    eq_sparkle_db: Option<f32>,
    width: Option<f32>,
    warmth: Option<f32>,
    presence_air: Option<f32>,
    volume_match: Option<bool>,
    manual_low_threshold_db: Option<f32>,
    manual_low_ratio: Option<f32>,
    manual_low_attack_ms: Option<f32>,
    manual_low_release_ms: Option<f32>,
    manual_mid_threshold_db: Option<f32>,
    manual_mid_ratio: Option<f32>,
    manual_mid_attack_ms: Option<f32>,
    manual_mid_release_ms: Option<f32>,
    manual_high_threshold_db: Option<f32>,
    manual_high_ratio: Option<f32>,
    manual_high_attack_ms: Option<f32>,
    manual_high_release_ms: Option<f32>,
    manual_link_stereo: Option<bool>,
    /// Mirror the desktop backend's adaptive resolution (profile, confidence,
    /// compression guards) from a deep analysis of the source. Default true.
    resolve_adaptive: Option<bool>,
}

fn parse_preset(s: &str) -> Preset {
    match s.to_ascii_lowercase().as_str() {
        "universal" => Preset::Universal,
        "clarity" => Preset::Clarity,
        "tape" => Preset::Tape,
        "spatial" => Preset::Spatial,
        "oomph" => Preset::Oomph,
        "warmth" => Preset::Warmth,
        "punch" => Preset::Punch,
        "loud" => Preset::Loud,
        "custom" => Preset::Custom {
            id: "audit-custom".to_string(),
        },
        other => panic!("unknown preset {other}"),
    }
}

fn parse_profile(s: &str) -> DeliveryProfile {
    match s.to_ascii_lowercase().as_str() {
        "streaming-universal" | "streaming" => DeliveryProfile::StreamingUniversal,
        "apple-music" | "apple" => DeliveryProfile::AppleMusic,
        "cd" => DeliveryProfile::Cd,
        "vinyl-premaster" | "vinyl" => DeliveryProfile::VinylPremaster,
        "loud-rock" => DeliveryProfile::LoudRock,
        "broadcast-eu" => DeliveryProfile::BroadcastEu,
        "broadcast-us" => DeliveryProfile::BroadcastUs,
        "custom" => DeliveryProfile::Custom,
        other => panic!("unknown delivery profile {other}"),
    }
}

fn parse_comp_mode(s: &str) -> CompressionMode {
    match s.to_ascii_lowercase().as_str() {
        "preset" => CompressionMode::Preset,
        "manual" => CompressionMode::Manual,
        "off" => CompressionMode::Off,
        other => panic!("unknown compression mode {other}"),
    }
}

/// Build `MasteringSettings` from the spec. Field defaults are the same the
/// desktop frontend sends for a fresh track (Universal, intensity 0.5,
/// Streaming Universal profile, flat EQ, Preset compression, default
/// advanced block) — cf. `tests/common/mod.rs::default_master_settings` and
/// `analysis.rs::recommended_universal`.
fn settings_from_spec(spec: &Spec) -> MasteringSettings {
    let profile = spec
        .delivery_profile
        .as_deref()
        .map(parse_profile)
        .unwrap_or(DeliveryProfile::StreamingUniversal);
    let mut advanced = AdvancedSettings::default();
    advanced.lufs_offset_db = spec.lufs_target;
    advanced.ceiling_dbtp = spec.ceiling_dbtp;
    advanced.bit_depth = spec.bit_depth;
    advanced.target_sample_rate = spec.sample_rate;
    advanced.compression_mode = spec
        .compression_mode
        .as_deref()
        .map(parse_comp_mode)
        .unwrap_or_default();
    advanced.compression_density = spec.compression_density;
    advanced.adaptive_strength = spec.adapt_strength;
    advanced.width = spec.width;
    advanced.warmth = spec.warmth;
    advanced.presence_air = spec.presence_air;
    advanced.compression_low_threshold_db = spec.manual_low_threshold_db;
    advanced.compression_low_ratio = spec.manual_low_ratio;
    advanced.compression_low_attack_ms = spec.manual_low_attack_ms;
    advanced.compression_low_release_ms = spec.manual_low_release_ms;
    advanced.compression_mid_threshold_db = spec.manual_mid_threshold_db;
    advanced.compression_mid_ratio = spec.manual_mid_ratio;
    advanced.compression_mid_attack_ms = spec.manual_mid_attack_ms;
    advanced.compression_mid_release_ms = spec.manual_mid_release_ms;
    advanced.compression_high_threshold_db = spec.manual_high_threshold_db;
    advanced.compression_high_ratio = spec.manual_high_ratio;
    advanced.compression_high_attack_ms = spec.manual_high_attack_ms;
    advanced.compression_high_release_ms = spec.manual_high_release_ms;
    advanced.compression_link_stereo = spec.manual_link_stereo;

    MasteringSettings {
        preset: spec
            .preset
            .as_deref()
            .map(parse_preset)
            .unwrap_or(Preset::Universal),
        intensity: spec.intensity.unwrap_or(0.5),
        eq_sub_db: spec.eq_sub_db.unwrap_or(0.0),
        eq_low_db: spec.eq_low_db.unwrap_or(0.0),
        eq_low_mid_db: spec.eq_low_mid_db.unwrap_or(0.0),
        eq_mid_db: spec.eq_mid_db.unwrap_or(0.0),
        eq_high_mid_db: spec.eq_high_mid_db.unwrap_or(0.0),
        eq_high_db: spec.eq_high_db.unwrap_or(0.0),
        eq_sparkle_db: spec.eq_sparkle_db.unwrap_or(0.0),
        eq_bands: EqBandFrequencies::default(),
        volume_match: spec.volume_match.unwrap_or(false),
        source_lufs_integrated: None,
        input_gain_db: spec.input_gain_db.unwrap_or(0.0),
        output_gain_db: spec.output_gain_db.unwrap_or(0.0),
        delivery_profile: profile,
        album: None,
        advanced,
    }
}

// ---------------------------------------------------------------------------
// Analysis + desktop-equivalent adaptive resolution
// ---------------------------------------------------------------------------

fn analyze(path: &Path) -> AnalysisResult {
    let req = AnalyzeRequest {
        id: TrackId("audit".to_string()),
        path: path.to_string_lossy().to_string(),
    };
    let mut results =
        analyze_tracks_core_with_progress_sync(vec![req], |_, _| {}).expect("analysis");
    results.remove(0)
}

/// Mirror of the desktop render entry (`render_track_master`) after
/// `populate_profile_store`: profile from analysis, confidence resolved from
/// the deep read (gate default OFF -> None), compression guards resolved from
/// the deep read + stand-down (gate default OFF -> None). Same calls, same
/// order, `album = false`.
fn resolve_like_desktop(settings: &mut MasteringSettings, analysis: &AnalysisResult) {
    settings.advanced.source_profile = SourceProfile::from_analysis(analysis);
    yes_master_lib::profile_store::apply_resolved_confidence(
        settings,
        analysis.deep_analysis.clone(),
        false,
    );
    let band_psr = analysis
        .deep_analysis
        .as_deref()
        .and_then(yes_master_lib::deep_analysis::band_psr_p10_db);
    let stand_down = yes_master_lib::guardrails::classify_already_mastered_stand_down(
        analysis.lufs_integrated,
        analysis.true_peak_dbtp,
        analysis.dynamic_range_lu,
        band_psr,
    );
    yes_master_lib::profile_store::apply_resolved_compression_guards(
        settings,
        analysis.deep_analysis.clone(),
        Some(stand_down),
        false,
    );
    // The frontend injects this for Volume Match only; render forces VM off.
    settings.source_lufs_integrated = Some(analysis.lufs_integrated);
}

#[derive(Serialize)]
struct AnalysisDump {
    lufs_integrated: f32,
    true_peak_dbtp: f32,
    dynamic_range_lu: f32,
    lufs_short_term_max_3s: Option<f32>,
    dynamic_range_p95_p10_db: Option<f32>,
    stereo_correlation: Option<f32>,
    stereo_width: f32,
    spectral_balance_6band: Option<SpectralBalance6>,
    transient_flux: Option<f32>,
    energy_density_score: Option<f32>,
    deep_analysis_present: bool,
    source_profile_digest: Option<String>,
    stand_down: Option<StandDownDump>,
    inferred_character: Option<TrackCharacter>,
    inferred_role: Option<TrackRole>,
}

#[derive(Serialize)]
struct StandDownDump {
    stand_down: f32,
    hot_loudness: bool,
    near_ceiling: bool,
    low_lra: bool,
    uniformly_low_psr: bool,
}

fn analysis_dump(a: &AnalysisResult) -> AnalysisDump {
    let band_psr = a
        .deep_analysis
        .as_deref()
        .and_then(yes_master_lib::deep_analysis::band_psr_p10_db);
    let sd = yes_master_lib::guardrails::classify_already_mastered_stand_down(
        a.lufs_integrated,
        a.true_peak_dbtp,
        a.dynamic_range_lu,
        band_psr,
    );
    AnalysisDump {
        lufs_integrated: a.lufs_integrated,
        true_peak_dbtp: a.true_peak_dbtp,
        dynamic_range_lu: a.dynamic_range_lu,
        lufs_short_term_max_3s: a.lufs_short_term_max_3s,
        dynamic_range_p95_p10_db: a.dynamic_range_p95_p10_db,
        stereo_correlation: a.stereo_correlation,
        stereo_width: a.stereo_width,
        spectral_balance_6band: a.spectral_balance_6band,
        transient_flux: a.transient_flux,
        energy_density_score: a.energy_density_score,
        deep_analysis_present: a.deep_analysis.is_some(),
        source_profile_digest: SourceProfile::from_analysis(a).map(|p| p.digest()),
        stand_down: Some(StandDownDump {
            stand_down: sd.stand_down,
            hot_loudness: sd.hot_loudness,
            near_ceiling: sd.near_ceiling,
            low_lra: sd.low_lra,
            uniformly_low_psr: sd.uniformly_low_psr,
        }),
        inferred_character: a.inferred_character,
        inferred_role: a.inferred_role,
    }
}

// ---------------------------------------------------------------------------
// Coefficient dump (for invariants without audio)
// ---------------------------------------------------------------------------

fn biquad_mag_db(c: &BiquadCoeffs, f_hz: f64, sr: f64) -> f64 {
    let w = 2.0 * std::f64::consts::PI * f_hz / sr;
    let (cw, sw) = (w.cos(), w.sin());
    let (c2w, s2w) = ((2.0 * w).cos(), (2.0 * w).sin());
    let nr = c.b0 as f64 + c.b1 as f64 * cw + c.b2 as f64 * c2w;
    let ni = -(c.b1 as f64 * sw + c.b2 as f64 * s2w);
    let dr = 1.0 + c.a1 as f64 * cw + c.a2 as f64 * c2w;
    let di = -(c.a1 as f64 * sw + c.a2 as f64 * s2w);
    let num = nr * nr + ni * ni;
    let den = dr * dr + di * di;
    10.0 * (num / den).log10()
}

#[derive(Serialize)]
struct CoeffDump {
    sample_rate: u32,
    input_gain_db: f64,
    saturation_amount: f32,
    saturation_small_signal_gain_db: f64,
    ceiling_dbfs: f64,
    width_side_scale: f32,
    transient_amount: f32,
    compression_active: bool,
    comp_low_threshold_db: f32,
    comp_mid_threshold_db: f32,
    comp_high_threshold_db: f32,
    comp_low_ratio: f32,
    comp_mid_ratio: f32,
    comp_high_ratio: f32,
    comp_low_makeup_db: f32,
    comp_mid_makeup_db: f32,
    comp_high_makeup_db: f32,
    comp_link_stereo: bool,
    volume_match_gain_db: f64,
    user_output_gain_db: f64,
    /// Magnitude of each EQ stage at its own band centre (dB) — the
    /// effective, guardrail-trimmed gain.
    eq_sub_80_db: f64,
    eq_low_200_db: f64,
    eq_low_mid_400_db: f64,
    eq_mid_1500_db: f64,
    eq_high_mid_3500_db: f64,
    eq_high_6000_db: f64,
    eq_sparkle_12000_db: f64,
    eq_warmth_300_db: f64,
    eq_presence_air_10000_db: f64,
    /// Combined static EQ response (all stages incl. HPF) at probe freqs.
    static_response_db: Vec<(f64, f64)>,
    biquads: serde_json::Value,
}

fn coeff_dump(c: &ChainCoeffs, sr: u32) -> CoeffDump {
    let srf = sr as f64;
    let stages: [(&str, &BiquadCoeffs); 12] = [
        ("sub_highpass_1", &c.sub_highpass),
        ("sub_highpass_2", &c.sub_highpass),
        ("sub", &c.sub),
        ("low", &c.low),
        ("low_mid", &c.low_mid),
        ("mid", &c.mid),
        ("high_mid", &c.high_mid),
        ("high", &c.high),
        ("sparkle", &c.sparkle),
        ("warmth", &c.warmth),
        ("presence_air", &c.presence_air),
        ("comp_low_lp", &c.comp_low_lp),
    ];
    let probes = [
        20.0, 25.0, 30.0, 40.0, 50.0, 63.0, 80.0, 100.0, 125.0, 160.0, 200.0, 250.0, 315.0, 400.0,
        500.0, 630.0, 800.0, 1000.0, 1250.0, 1600.0, 2000.0, 2500.0, 3150.0, 4000.0, 5000.0,
        6300.0, 8000.0, 10000.0, 12500.0, 16000.0, 20000.0,
    ];
    let static_response_db = probes
        .iter()
        .filter(|f| **f < srf / 2.0)
        .map(|&f| {
            let total: f64 = stages[..11]
                .iter()
                .map(|(_, bq)| biquad_mag_db(bq, f, srf))
                .sum();
            (f, total)
        })
        .collect();
    let biquads = serde_json::json!(stages
        .iter()
        .map(|(n, bq)| (n.to_string(), serde_json::json!([bq.b0, bq.b1, bq.b2, bq.a1, bq.a2])))
        .collect::<serde_json::Map<String, serde_json::Value>>());
    let drive = 1.0 + c.saturation_amount as f64 * 2.0;
    let small_signal = if c.saturation_amount > 0.0 {
        20.0 * (drive / drive.tanh()).log10()
    } else {
        0.0
    };
    CoeffDump {
        sample_rate: sr,
        input_gain_db: 20.0 * (c.input_gain_lin as f64).log10(),
        saturation_amount: c.saturation_amount,
        saturation_small_signal_gain_db: small_signal,
        ceiling_dbfs: 20.0 * (c.ceiling_lin as f64).log10(),
        width_side_scale: c.width_side_scale,
        transient_amount: c.transient_amount,
        compression_active: c.compression_active,
        comp_low_threshold_db: c.comp_low_threshold_db,
        comp_mid_threshold_db: c.comp_mid_threshold_db,
        comp_high_threshold_db: c.comp_high_threshold_db,
        comp_low_ratio: c.comp_low_ratio,
        comp_mid_ratio: c.comp_mid_ratio,
        comp_high_ratio: c.comp_high_ratio,
        comp_low_makeup_db: c.comp_low_makeup_db,
        comp_mid_makeup_db: c.comp_mid_makeup_db,
        comp_high_makeup_db: c.comp_high_makeup_db,
        comp_link_stereo: c.comp_link_stereo,
        volume_match_gain_db: 20.0 * (c.volume_match_gain_lin as f64).log10(),
        user_output_gain_db: 20.0 * (c.user_output_gain_lin as f64).log10(),
        eq_sub_80_db: biquad_mag_db(&c.sub, 80.0, srf),
        eq_low_200_db: biquad_mag_db(&c.low, 200.0, srf),
        eq_low_mid_400_db: biquad_mag_db(&c.low_mid, 400.0, srf),
        eq_mid_1500_db: biquad_mag_db(&c.mid, 1500.0, srf),
        eq_high_mid_3500_db: biquad_mag_db(&c.high_mid, 3500.0, srf),
        eq_high_6000_db: biquad_mag_db(&c.high, 6000.0, srf),
        eq_sparkle_12000_db: biquad_mag_db(&c.sparkle, 12000.0, srf),
        eq_warmth_300_db: biquad_mag_db(&c.warmth, 300.0, srf),
        eq_presence_air_10000_db: biquad_mag_db(&c.presence_air, 10000.0, srf),
        static_response_db,
        biquads,
    }
}

// ---------------------------------------------------------------------------
// Chain-only processing with stage overrides (ADAPTATION: coefficient
// overrides on the production chain, used solely for stage attribution).
// ---------------------------------------------------------------------------

fn apply_overrides(c: &mut ChainCoeffs, overrides: &[String]) {
    for o in overrides {
        match o.as_str() {
            "no_sat" => c.saturation_amount = 0.0,
            "no_transient" => c.transient_amount = 0.0,
            "no_comp" => c.compression_active = false,
            "no_width" => c.width_side_scale = 1.0,
            "no_input_gain" => c.input_gain_lin = 1.0,
            "no_hpf" => c.sub_highpass = BiquadCoeffs::identity(),
            "no_eq" => {
                c.sub = BiquadCoeffs::identity();
                c.low = BiquadCoeffs::identity();
                c.low_mid = BiquadCoeffs::identity();
                c.mid = BiquadCoeffs::identity();
                c.high_mid = BiquadCoeffs::identity();
                c.high = BiquadCoeffs::identity();
                c.sparkle = BiquadCoeffs::identity();
                c.warmth = BiquadCoeffs::identity();
                c.presence_air = BiquadCoeffs::identity();
            }
            // Limiter "bypass": raise the ceiling far above any sample so the
            // limiter's constraints stay at unity. Uses the production
            // `with_coeffs_inheriting_state` path, which copies
            // `coeffs.ceiling_lin` into the limiter.
            "no_limiter" => c.ceiling_lin = 1.0e6,
            // Reconciliation: set the saturation amount directly (claim 4
            // mapping/continuity probes). Positive values use the production
            // curve unchanged; 0 is the production bypass.
            o if o.starts_with("sat=") => c.saturation_amount = o[4..].parse().expect("sat= value"),
            other => panic!("unknown override {other}"),
        }
    }
}

fn write_wav_f32(path: &Path, samples: &[f32], sr: u32, channels: u16) {
    let spec = hound::WavSpec {
        channels,
        sample_rate: sr,
        bits_per_sample: 32,
        sample_format: hound::SampleFormat::Float,
    };
    let mut w = hound::WavWriter::create(path, spec).expect("create wav");
    for s in samples {
        w.write_sample(*s).expect("write");
    }
    w.finalize().expect("finalize");
}

fn measure_crate(samples: &[f32], sr: u32, channels: u16) -> (f64, f64, f64) {
    use ebur128::{EbuR128, Mode};
    let mut ebu = EbuR128::new(
        u32::from(channels),
        sr,
        Mode::I | Mode::LRA | Mode::TRUE_PEAK,
    )
    .expect("ebur128");
    ebu.add_frames_f32(samples).expect("feed");
    let lufs = ebu.loudness_global().expect("global");
    let lra = ebu.loudness_range().expect("lra");
    let mut peak = 0.0f64;
    for ch in 0..u32::from(channels) {
        peak = peak.max(ebu.true_peak(ch).expect("tp"));
    }
    let tp = if peak > 0.0 { 20.0 * peak.log10() } else { -60.0 };
    (lufs, tp, lra)
}

// ---------------------------------------------------------------------------
// CLI
// ---------------------------------------------------------------------------

fn arg_value(args: &[String], key: &str) -> Option<String> {
    args.iter()
        .position(|a| a == key)
        .and_then(|i| args.get(i + 1).cloned())
}

fn arg_values(args: &[String], key: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut i = 0;
    while i < args.len() {
        if args[i] == key {
            if let Some(v) = args.get(i + 1) {
                out.push(v.clone());
            }
            i += 2;
        } else {
            i += 1;
        }
    }
    out
}

fn load_spec(args: &[String]) -> Spec {
    match arg_value(args, "--spec") {
        Some(p) => {
            let text = std::fs::read_to_string(&p).expect("read spec");
            serde_json::from_str(&text).expect("parse spec")
        }
        None => match arg_value(args, "--spec-json") {
            Some(j) => serde_json::from_str(&j).expect("parse spec json"),
            None => Spec::default(),
        },
    }
}

fn abs(p: &str) -> PathBuf {
    let path = PathBuf::from(p);
    if path.is_absolute() {
        path
    } else {
        std::env::current_dir().unwrap().join(path)
    }
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let Some(cmd) = args.first() else {
        eprintln!("usage: analyze | render | coeffs | chain | measure | preview-landing | src-probe | src-file | src-impulse | stats");
        std::process::exit(2);
    };
    match cmd.as_str() {
        "analyze" => {
            let input = abs(&args[1]);
            let a = analyze(&input);
            let dump = analysis_dump(&a);
            println!("{}", serde_json::to_string_pretty(&dump).unwrap());
        }
        "render" => {
            // render <in> <out.wav> --spec s.json | --spec-json '{...}' [--json out.json]
            let input = abs(&args[1]);
            let output = abs(&args[2]);
            let spec = load_spec(&args);
            let mut settings = settings_from_spec(&spec);
            let analysis = if spec.resolve_adaptive.unwrap_or(true) {
                let a = analyze(&input);
                resolve_like_desktop(&mut settings, &a);
                Some(a)
            } else {
                None
            };
            if output.exists() {
                // Production never overwrites: it would divert to a `__n`
                // sibling. Remove our own previous render first so the path
                // in the ledger is the file that was written.
                std::fs::remove_file(&output).expect("remove previous render");
            }
            let out_dir = output.parent().unwrap().to_path_buf();
            let t0 = std::time::Instant::now();
            let job = mastering_render_to_path(
                TrackId("audit-render".to_string()),
                &input,
                &settings,
                &out_dir,
                RenderKind::Master,
                &output,
            )
            .expect("render");
            let elapsed = t0.elapsed().as_secs_f64();
            let measurements = job.measurements.clone().expect("measurements");
            let report = ExportReport {
                delivered_format: job.delivered_format.clone(),
                track_id: TrackId("audit-render".to_string()),
                output_path: job.output_paths[0].clone(),
                measured_lufs: measurements.lufs_integrated,
                measured_true_peak_dbtp: measurements.true_peak_dbtp,
                measured_dynamic_range_lu: measurements.dynamic_range_lu,
                source_format: "wav".to_string(),
                destination_format: "wav".to_string(),
                sample_rate: measurements.sample_rate,
                bit_depth: measurements.bit_depth,
                effective_adaptive_strength: measurements.effective_adaptive_strength,
                source_profile_digest: measurements.source_profile_digest.clone(),
                confidence_digest: measurements.confidence_digest.clone(),
                compression_digest: measurements.compression_digest.clone(),
                measurements_are_rendered: true,
                checks: Vec::new(),
            };
            let checks = yes_master_lib::exports::export_checks_for_report(
                &report,
                analysis.as_ref(),
                Some(&settings),
            );
            let out = serde_json::json!({
                "input": input.to_string_lossy(),
                "output": job.output_paths[0],
                "status": format!("{:?}", job.status),
                "elapsed_s": elapsed,
                "effective_target_lufs": settings.effective_target_lufs(),
                "effective_ceiling_dbtp": settings.effective_ceiling_dbtp(),
                "effective_bit_depth": settings.effective_bit_depth(),
                "requested_sample_rate": settings.requested_delivery_sample_rate(),
                "measurements": measurements,
                "checks": checks,
                "settings": settings,
                "analysis": analysis.as_ref().map(analysis_dump),
            });
            let text = serde_json::to_string_pretty(&out).unwrap();
            if let Some(p) = arg_value(&args, "--json") {
                std::fs::write(abs(&p), &text).expect("write json");
            }
            println!("{text}");
        }
        "coeffs" => {
            // coeffs --sr 48000 --spec s.json [--analysis in.wav]
            let sr: u32 = arg_value(&args, "--sr")
                .map(|s| s.parse().unwrap())
                .unwrap_or(48_000);
            let spec = load_spec(&args);
            let mut settings = settings_from_spec(&spec);
            if let Some(p) = arg_value(&args, "--analysis") {
                let a = analyze(&abs(&p));
                resolve_like_desktop(&mut settings, &a);
            }
            let c = ChainCoeffs::from_settings(sr, &settings);
            println!(
                "{}",
                serde_json::to_string_pretty(&coeff_dump(&c, sr)).unwrap()
            );
        }
        "chain" => {
            // chain <in> <out.f32.wav> --spec s.json [--override no_sat ...]
            //       [--land] [--json out.json] [--analysis-from in.wav]
            // Runs decode -> MasteringChain (native rate) -> flush tail, and
            // writes 32-bit float. No SRC, no dither. `--land` additionally
            // applies the production ceiling-bounded landing (via the public
            // preview_landing helper, which shares the same rule).
            let input = abs(&args[1]);
            let output = abs(&args[2]);
            let spec = load_spec(&args);
            let mut settings = settings_from_spec(&spec);
            if spec.resolve_adaptive.unwrap_or(true) {
                let src = arg_value(&args, "--analysis-from").map(|p| abs(&p));
                let a = analyze(src.as_deref().unwrap_or(&input));
                resolve_like_desktop(&mut settings, &a);
            }
            settings.volume_match = false;
            let pcm = yes_master_lib::decode::decode_full(&input).expect("decode");
            let ch = pcm.channels as usize;
            let overrides = arg_values(&args, "--override");
            let base = MasteringChain::new(pcm.sample_rate, ch, &settings);
            let mut coeffs = base.coeffs;
            apply_overrides(&mut coeffs, &overrides);
            let mut chain = MasteringChain::with_coeffs_inheriting_state(coeffs, &base);
            let mut samples = pcm.samples.clone();
            chain.process_interleaved(&mut samples, ch);
            chain.flush_render_tail(&mut samples, ch);
            let (lufs_pre, tp_pre, lra_pre) = measure_crate(&samples, pcm.sample_rate, pcm.channels);
            let mut landing_db = 0.0f64;
            if args.iter().any(|a| a == "--land") {
                if let Some(target) = settings.effective_target_lufs() {
                    let ceiling = settings.effective_ceiling_dbtp() as f64;
                    let delta = target as f64 - lufs_pre;
                    let cap = ceiling - tp_pre;
                    landing_db = delta.min(cap);
                    if landing_db.abs() <= 1.0e-4 {
                        landing_db = 0.0;
                    }
                    let g = 10f64.powf(landing_db / 20.0) as f32;
                    for s in samples.iter_mut() {
                        *s *= g;
                    }
                }
            }
            write_wav_f32(&output, &samples, pcm.sample_rate, pcm.channels);
            let (lufs, tp, lra) = measure_crate(&samples, pcm.sample_rate, pcm.channels);
            let out = serde_json::json!({
                "input": input.to_string_lossy(),
                "output": output.to_string_lossy(),
                "sample_rate": pcm.sample_rate,
                "channels": pcm.channels,
                "overrides": overrides,
                "pre_landing": {"lufs": lufs_pre, "true_peak_dbtp": tp_pre, "lra": lra_pre},
                "landing_db": landing_db,
                "post": {"lufs": lufs, "true_peak_dbtp": tp, "lra": lra},
                "coeffs": coeff_dump(&chain.coeffs, pcm.sample_rate),
            });
            let text = serde_json::to_string_pretty(&out).unwrap();
            if let Some(p) = arg_value(&args, "--json") {
                std::fs::write(abs(&p), &text).expect("write json");
            }
            println!("{text}");
        }
        "measure" => {
            let input = abs(&args[1]);
            let pcm = yes_master_lib::decode::decode_full(&input).expect("decode");
            let (lufs, tp, lra) = measure_crate(&pcm.samples, pcm.sample_rate, pcm.channels);
            println!(
                "{}",
                serde_json::json!({
                    "input": input.to_string_lossy(),
                    "sample_rate": pcm.sample_rate,
                    "channels": pcm.channels,
                    "frames": pcm.samples.len() / pcm.channels.max(1) as usize,
                    "lufs": lufs, "true_peak_dbtp": tp, "lra": lra
                })
            );
        }
        "preview-landing" => {
            // preview-landing <in> --spec s.json
            let input = abs(&args[1]);
            let spec = load_spec(&args);
            let mut settings = settings_from_spec(&spec);
            if spec.resolve_adaptive.unwrap_or(true) {
                let a = analyze(&input);
                resolve_like_desktop(&mut settings, &a);
            }
            let pcm = yes_master_lib::decode::decode_full(&input).expect("decode");
            let pl = preview_landing(&pcm.samples, pcm.sample_rate, pcm.channels, &settings)
                .expect("preview landing");
            println!(
                "{}",
                serde_json::json!({
                    "gain_db": 20.0 * (pl.gain_lin as f64).log10(),
                    "mastered_lufs": pl.mastered_lufs,
                })
            );
        }
        "src-probe" => recon::cmd_src_probe(&args),
        "src-file" => recon::cmd_src_file(&args),
        "src-impulse" => recon::cmd_src_impulse(&args),
        "stats" => {
            // stats <in> --spec s.json [--json out.json] [--analysis-from in.wav]
            let input = abs(&args[1]);
            let spec = load_spec(&args);
            let mut settings = settings_from_spec(&spec);
            if spec.resolve_adaptive.unwrap_or(true) {
                let src = arg_value(&args, "--analysis-from").map(|p| abs(&p));
                let a = analyze(src.as_deref().unwrap_or(&input));
                resolve_like_desktop(&mut settings, &a);
            }
            settings.volume_match = false;
            let pcm = yes_master_lib::decode::decode_full(&input).expect("decode");
            let ch = pcm.channels as usize;
            let run = |ovr: &[&str]| -> (Vec<f32>, [u32; 3]) {
                let base = MasteringChain::new(pcm.sample_rate, ch, &settings);
                let mut coeffs = base.coeffs;
                let o: Vec<String> = ovr.iter().map(|s| s.to_string()).collect();
                apply_overrides(&mut coeffs, &o);
                let mut chain = MasteringChain::with_coeffs_inheriting_state(coeffs, &base);
                let mut samples = pcm.samples.clone();
                chain.process_interleaved(&mut samples, ch);
                chain.flush_render_tail(&mut samples, ch);
                use std::sync::atomic::Ordering;
                let gr = [
                    chain.gr_snapshots.low.load(Ordering::Relaxed),
                    chain.gr_snapshots.mid.load(Ordering::Relaxed),
                    chain.gr_snapshots.high.load(Ordering::Relaxed),
                ];
                (samples, gr)
            };
            let (full, gr) = run(&[]);
            let (nolim, _) = run(&["no_limiter"]);
            let (nolim_nosat, _) = run(&["no_limiter", "no_sat"]);
            let st = recon::limiter_stats(&full, &nolim, ch);
            let sat = recon::level_stats(&nolim_nosat, &nolim, ch);
            let (lufs_pre, tp_pre, lra_pre) = measure_crate(&full, pcm.sample_rate, pcm.channels);
            let out = serde_json::json!({
                "input": input.to_string_lossy(),
                "sample_rate": pcm.sample_rate,
                "frames": full.len() / ch,
                "pre_landing": {"lufs": lufs_pre, "true_peak_dbtp": tp_pre, "lra": lra_pre},
                "comp_max_gr_db": [gr[0] as f64 / 100.0, gr[1] as f64 / 100.0, gr[2] as f64 / 100.0],
                "limiter": st,
                "saturation": sat,
                "coeffs": coeff_dump(&ChainCoeffs::from_settings(pcm.sample_rate, &settings), pcm.sample_rate),
            });
            let text = serde_json::to_string_pretty(&out).unwrap();
            if let Some(p) = arg_value(&args, "--json") {
                std::fs::write(abs(&p), &text).expect("write json");
            }
            println!("{text}");
        }
        other => {
            eprintln!("unknown command {other}");
            std::process::exit(2);
        }
    }
}
