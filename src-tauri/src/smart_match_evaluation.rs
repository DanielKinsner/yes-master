//! Smart Match evaluation runner (Smart Match plan, Slice 6).
//!
//! A local-only evidence runner. It changes no product behavior. It measures
//! Smart Match against the plan's gates on real songs and writes:
//!
//! - a **private ledger** (file paths, per-case numbers) under the git-ignored
//!   `test-output/`;
//! - a **committable summary** with numbers only. Free-library songs and the
//!   September corpus pieces are named (their credits are already committed);
//!   the owner's private files appear as anonymous IDs.
//!
//! **Independent tone changes.** G3 and G6 need copies of a song with a known
//! tone change. Those are made here with an FFT equalizer (sqrt-Hann
//! overlap-add, brick-wall gains on the meter's band edges, or a log-frequency
//! tilt), deliberately unrelated to the chain's biquads, so the chain is never
//! graded against itself.
//!
//! **Same meter, same decision.** Every profile comes from
//! [`crate::smart_match::measure_shares`] (the app's meter) or the app's own
//! track analysis, and every correction from [`crate::smart_match::decide_with`]
//! or the chain itself.

use crate::dsp::MasteringChain;
use crate::engine::{analyze_tracks_core_with_progress_sync, AnalyzeRequest};
use crate::smart_match::{
    self, below_typical_db, boost_fade, decide_with, gap_db, measure_shares, remove_offset,
    shares_array, shares_to_db, SmartMatchParams, BAND_COUNT,
};
use crate::types::{
    AdvancedSettings, CommandError, CommandResult, DeliveryProfile, EqBandFrequencies,
    MasteringSettings, Preset, SmartMatchReference, SmartMatchSettings, SourceProfile,
    SpectralBalance6, TrackId,
};
use rustfft::num_complex::Complex;
use rustfft::FftPlanner;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;

/// G3 and G6 evaluate at the product's default amount.
pub const EVAL_AMOUNT: f32 = smart_match::DEFAULT_AMOUNT;
/// Size of a band change in the known-answer copies (plan G3: ±4 dB).
pub const BAND_CHANGE_DB: f64 = 4.0;
/// Slope of the tilt case, in dB per octave around 1 kHz.
pub const TILT_DB_PER_OCTAVE: f64 = 1.0;
/// A band counts toward G3 and G6 only when its measured gap is this large.
pub const MIN_GAP_DB: f64 = 1.0;
/// G3: the correction may differ from `amount × gap` by this much.
pub const G3_TOLERANCE_DB: f64 = 0.5;
/// G3: bands the copy did not change may move this much...
pub const G3_OTHER_BAND_DB: f64 = 0.5;
/// ...in at least this share of cases.
pub const G3_OTHER_BAND_SHARE: f64 = 0.9;
/// G6: the re-measured gap must be between these fractions of the original.
pub const G6_RANGE: (f64, f64) = (0.35, 0.65);

/// Frame of the independent equalizer: 0.67–0.73 Hz bins at 44.1–48 kHz, so its
/// band edges are sharp at the meter's 20 and 80 Hz edges.
const FFT_EQ_SIZE: usize = 65_536;

/// Where a song sits in the holdout protocol.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SongGroup {
    LibraryBuild,
    LibraryHoldout,
    CorpusDevelopment,
    CorpusHoldout,
}

impl SongGroup {
    pub fn is_holdout(self) -> bool {
        matches!(self, Self::LibraryHoldout | Self::CorpusHoldout)
    }
}

/// One song to evaluate. `label` is what the committable summary shows.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SongSpec {
    pub label: String,
    pub group: SongGroup,
    pub genre: String,
    #[serde(skip_serializing)]
    pub path: PathBuf,
}

/// A known tone change made by the independent FFT equalizer.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ToneChange {
    Band {
        band: usize,
        db: f64,
    },
    TwoBand {
        a: usize,
        a_db: f64,
        b: usize,
        b_db: f64,
    },
    Tilt {
        db_per_octave: f64,
    },
}

impl ToneChange {
    /// The equalizer's gain at `freq_hz`, in dB.
    pub fn gain_db(&self, freq_hz: f64, sample_rate: u32) -> f64 {
        let band = meter_band(freq_hz, sample_rate);
        match *self {
            Self::Band { band: b, db } => {
                if band == Some(b) {
                    db
                } else {
                    0.0
                }
            }
            Self::TwoBand { a, a_db, b, b_db } => match band {
                Some(x) if x == a => a_db,
                Some(x) if x == b => b_db,
                _ => 0.0,
            },
            Self::Tilt { db_per_octave } => {
                if freq_hz <= 0.0 {
                    0.0
                } else {
                    db_per_octave * (freq_hz / 1000.0).log2()
                }
            }
        }
    }

    /// Which meter bands the change targets.
    pub fn changed_bands(&self) -> [bool; BAND_COUNT] {
        let mut out = [false; BAND_COUNT];
        match *self {
            Self::Band { band, .. } => out[band] = true,
            Self::TwoBand { a, b, .. } => {
                out[a] = true;
                out[b] = true;
            }
            Self::Tilt { .. } => out = [true; BAND_COUNT],
        }
        out
    }

    pub fn label(&self) -> String {
        let name = |b: usize| smart_match::BAND_KEYS[b];
        match *self {
            Self::Band { band, db } => format!("{} {db:+.0} dB", name(band)),
            Self::TwoBand { a, a_db, b, b_db } => {
                format!("{} {a_db:+.0} dB, {} {b_db:+.0} dB", name(a), name(b))
            }
            Self::Tilt { db_per_octave } => format!("tilt {db_per_octave:+.1} dB/oct"),
        }
    }
}

/// The plan's G3 cases: each band ±4 dB, one two-band case, one tilt.
pub fn standard_cases() -> Vec<ToneChange> {
    let mut cases = Vec::new();
    for band in 0..BAND_COUNT {
        for db in [BAND_CHANGE_DB, -BAND_CHANGE_DB] {
            cases.push(ToneChange::Band { band, db });
        }
    }
    cases.push(ToneChange::TwoBand {
        a: 2,
        a_db: BAND_CHANGE_DB,
        b: 4,
        b_db: -BAND_CHANGE_DB,
    });
    cases.push(ToneChange::Tilt {
        db_per_octave: TILT_DB_PER_OCTAVE,
    });
    cases
}

/// The meter band a frequency falls in (the meter's own edges).
fn meter_band(freq_hz: f64, sample_rate: u32) -> Option<usize> {
    let mut edges = smart_match::BAND_EDGES_HZ;
    edges[BAND_COUNT] = (f64::from(sample_rate) / 2.0).min(edges[BAND_COUNT]);
    (0..BAND_COUNT).find(|&b| freq_hz >= edges[b] && freq_hz < edges[b + 1])
}

/// Apply a tone change with an FFT equalizer: sqrt-Hann analysis and
/// synthesis windows at 50% overlap (perfect reconstruction at unity gain),
/// a real zero-phase gain per bin. Independent of the chain's filters.
pub fn apply_tone_change(
    samples: &[f32],
    sample_rate: u32,
    channels: usize,
    change: &ToneChange,
) -> Vec<f32> {
    let n = FFT_EQ_SIZE;
    let hop = n / 2;
    let ch = channels.max(1);
    let frames = samples.len() / ch;
    let window: Vec<f32> = (0..n)
        .map(|i| (0.5 - 0.5 * (std::f64::consts::TAU * i as f64 / n as f64).cos()).sqrt() as f32)
        .collect();
    let gains: Vec<f32> = (0..=n / 2)
        .map(|k| {
            let f = k as f64 * f64::from(sample_rate) / n as f64;
            10f64.powf(change.gain_db(f, sample_rate) / 20.0) as f32
        })
        .collect();
    let mut planner = FftPlanner::<f32>::new();
    let forward = planner.plan_fft_forward(n);
    let inverse = planner.plan_fft_inverse(n);
    let mut out = vec![0.0_f32; frames * ch];
    let padded = frames + 2 * n;
    let mut acc = vec![0.0_f32; padded];
    let mut buf = vec![Complex { re: 0.0, im: 0.0 }; n];
    for c in 0..ch {
        acc.iter_mut().for_each(|v| *v = 0.0);
        let mut start = 0;
        while start + n <= padded {
            for (i, slot) in buf.iter_mut().enumerate() {
                let pos = start + i;
                let x = if pos >= n && pos - n < frames {
                    samples[(pos - n) * ch + c]
                } else {
                    0.0
                };
                *slot = Complex {
                    re: x * window[i],
                    im: 0.0,
                };
            }
            forward.process(&mut buf);
            for (k, slot) in buf.iter_mut().enumerate() {
                let g = gains[k.min(n - k)];
                slot.re *= g;
                slot.im *= g;
            }
            inverse.process(&mut buf);
            let scale = 1.0 / n as f32;
            for (i, slot) in buf.iter().enumerate() {
                acc[start + i] += slot.re * scale * window[i];
            }
            start += hop;
        }
        for i in 0..frames {
            out[i * ch + c] = acc[i + n];
        }
    }
    out
}

/// Render through the real chain, frame by frame.
pub fn render(
    samples: &[f32],
    sample_rate: u32,
    channels: usize,
    settings: &MasteringSettings,
) -> Vec<f32> {
    let ch = channels.max(1);
    let mut chain = MasteringChain::new(sample_rate, ch, settings);
    let mut out = samples.to_vec();
    for frame in out.chunks_exact_mut(ch) {
        chain.process_frame_inplace(frame);
    }
    out
}

/// A preset at default Intensity with the given resolved profile: the state
/// the chain builds from once analysis has landed.
pub fn preset_settings(preset: Preset, profile: Option<SourceProfile>) -> MasteringSettings {
    MasteringSettings {
        preset,
        intensity: 0.5,
        eq_sub_db: 0.0,
        eq_low_db: 0.0,
        eq_low_mid_db: 0.0,
        eq_mid_db: 0.0,
        eq_high_mid_db: 0.0,
        eq_high_db: 0.0,
        eq_sparkle_db: 0.0,
        eq_bands: EqBandFrequencies::default(),
        volume_match: false,
        source_lufs_integrated: None,
        input_gain_db: 0.0,
        output_gain_db: 0.0,
        delivery_profile: DeliveryProfile::Custom,
        album: None,
        advanced: AdvancedSettings {
            source_profile: profile,
            ..Default::default()
        },
    }
}

/// `settings` with a Smart Match reference, switch and amount.
pub fn with_match(
    mut settings: MasteringSettings,
    reference: SpectralBalance6,
    enabled: bool,
    amount: f32,
) -> MasteringSettings {
    settings.advanced.smart_match = Some(SmartMatchSettings {
        reference: Some(SmartMatchReference {
            name: "Reference".into(),
            spectral_6: reference,
        }),
        enabled,
        amount,
    });
    settings
}

/// A decoded song and its profile from the app's own analysis.
pub struct LoadedSong {
    pub spec: SongSpec,
    pub samples: Vec<f32>,
    pub sample_rate: u32,
    pub channels: usize,
    pub profile: SourceProfile,
}

pub fn load_song(spec: &SongSpec) -> CommandResult<LoadedSong> {
    let analysis = analyze_tracks_core_with_progress_sync(
        vec![AnalyzeRequest {
            id: TrackId(spec.label.clone()),
            path: spec.path.to_string_lossy().into_owned(),
        }],
        |_, _| {},
    )?
    .remove(0);
    let profile = SourceProfile::from_analysis(&analysis).ok_or_else(|| {
        CommandError::Other(format!("{}: analysis has no six-band profile", spec.label))
    })?;
    let pcm = crate::decode::decode_full(&spec.path)?;
    Ok(LoadedSong {
        spec: spec.clone(),
        samples: pcm.samples,
        sample_rate: pcm.sample_rate,
        channels: usize::from(pcm.channels.max(1)),
        profile,
    })
}

/// Shares in dB, normalized (as the decision reads them).
pub fn levels_db(shares: &SpectralBalance6) -> [f64; BAND_COUNT] {
    let s = shares_array(shares);
    let total: f64 = s.iter().sum();
    shares_to_db(&s.map(|v| v / total.max(1e-30)))
}

/// Measure rendered or filtered audio through the app's meter.
pub fn measure(
    samples: &[f32],
    sample_rate: u32,
    channels: usize,
) -> CommandResult<SpectralBalance6> {
    measure_shares(samples, sample_rate, channels)
        .ok_or_else(|| CommandError::Other("the meter found no energy".into()))
}

// ---------------------------------------------------------------------------
// G3 and G6: known-answer recovery and gap closed end to end
// ---------------------------------------------------------------------------

/// One known-answer case on one song.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaseResult {
    pub song: String,
    pub group: SongGroup,
    pub case: String,
    pub changed_bands: [bool; BAND_COUNT],
    /// The copy's measured shares (G7 reuses them as a track).
    pub copy_shares: Option<SpectralBalance6>,
    /// Reference (the original) minus track (the copy), offset removed.
    pub measured_gap_db: [f64; BAND_COUNT],
    pub correction_db: [f32; BAND_COUNT],
    pub capped: [bool; BAND_COUNT],
    /// The missing-band fade the copy's bands got (1 = none).
    pub fade: [f32; BAND_COUNT],
    /// G6: (gap − change the match made end to end) / gap, per changed band;
    /// `None` where the band is not eligible.
    pub remaining_ratio: [Option<f64>; BAND_COUNT],
    /// Context: the copy rendered with the match against the original
    /// rendered without it, as a fraction of the gap.
    pub remaining_ratio_vs_rendered_original: [Option<f64>; BAND_COUNT],
    /// Diagnostic: the same ratio as `remaining_ratio`, but through the EQ
    /// cascade alone (no compressor, saturation or limiter). It separates
    /// Smart Match's own accuracy on the real song from what the chain's
    /// dynamics do to any EQ change.
    #[serde(default)]
    pub eq_stage_ratio: [Option<f64>; BAND_COUNT],
}

/// The chain's EQ cascade alone (the biquads `process_frame_inplace` runs
/// before its dynamics), for G6's decomposition.
pub fn eq_stage_only(samples: &[f32], channels: usize, c: &crate::dsp::ChainCoeffs) -> Vec<f32> {
    let ch = channels.max(1);
    let mut filters = vec![
        c.sub_highpass,
        c.sub_highpass,
        c.sub,
        c.low,
        c.low_mid,
        c.mid,
        c.high_mid,
        c.high,
        c.sparkle,
        c.warmth,
        c.presence_air,
    ];
    if c.smart_match_active {
        filters.push(c.smart_match_bass);
    }
    let mut states = vec![vec![crate::dsp::BiquadState::default(); filters.len()]; ch];
    let mut out = samples.to_vec();
    for frame in out.chunks_exact_mut(ch) {
        for (k, sample) in frame.iter_mut().enumerate() {
            let mut y = *sample;
            for (f, st) in filters.iter().zip(states[k].iter_mut()) {
                y = st.process(f, y);
            }
            *sample = y;
        }
    }
    out
}

impl CaseResult {
    /// A band counts toward G3 and G6 when it was changed, its measured gap
    /// is at least 1 dB, and the missing-band rule did not engage there.
    pub fn eligible(&self, b: usize) -> bool {
        self.changed_bands[b]
            && self.measured_gap_db[b].abs() >= MIN_GAP_DB
            && (self.measured_gap_db[b] < 0.0 || self.fade[b] >= 1.0)
    }

    /// Changed bands with a gap of 1 dB or more that the fade kept from a
    /// full boost. Reported, not graded.
    pub fn faded(&self, b: usize) -> bool {
        self.changed_bands[b] && self.measured_gap_db[b] >= MIN_GAP_DB && self.fade[b] < 1.0
    }
}

/// Run G3 and G6 for one song: every standard case, decision and render.
pub fn known_answer_cases(
    song: &LoadedSong,
    params: &SmartMatchParams,
) -> CommandResult<Vec<CaseResult>> {
    let sr = song.sample_rate;
    let ch = song.channels;
    let original_shares = song.profile.spectral_6;
    let original_rendered = render(
        &song.samples,
        sr,
        ch,
        &preset_settings(Preset::Universal, Some(song.profile)),
    );
    let original_rendered_db = levels_db(&measure(&original_rendered, sr, ch)?);
    drop(original_rendered);

    let mut results = Vec::new();
    for change in standard_cases() {
        let copy = apply_tone_change(&song.samples, sr, ch, &change);
        let copy_shares = measure(&copy, sr, ch)?;
        let copy_profile = SourceProfile {
            spectral_6: copy_shares,
            ..song.profile
        };
        let decision = decide_with(
            params,
            Some(&copy_shares),
            Some(&original_shares),
            true,
            EVAL_AMOUNT,
        );
        let measured_gap = gap_db(&levels_db(&copy_shares), &levels_db(&original_shares));
        let below = below_typical_db(&levels_db(&copy_shares));
        let fade = below.map(|b| boost_fade(params, b));

        let plain = preset_settings(Preset::Universal, Some(copy_profile));
        let matched = with_match(plain.clone(), original_shares, true, EVAL_AMOUNT);
        let unmatched_db = levels_db(&measure(&render(&copy, sr, ch, &plain), sr, ch)?);
        let matched_db = levels_db(&measure(&render(&copy, sr, ch, &matched), sr, ch)?);
        let mut delta = [0.0; BAND_COUNT];
        for b in 0..BAND_COUNT {
            delta[b] = matched_db[b] - unmatched_db[b];
        }
        let delta = remove_offset(&delta);
        let vs_original = gap_db(&matched_db, &original_rendered_db);
        let eq_plain = crate::dsp::ChainCoeffs::from_settings(sr, &plain);
        let eq_matched = crate::dsp::ChainCoeffs::from_settings(sr, &matched);
        let eq_unmatched_db = levels_db(&measure(&eq_stage_only(&copy, ch, &eq_plain), sr, ch)?);
        let eq_matched_db = levels_db(&measure(&eq_stage_only(&copy, ch, &eq_matched), sr, ch)?);
        let mut eq_delta = [0.0; BAND_COUNT];
        for b in 0..BAND_COUNT {
            eq_delta[b] = eq_matched_db[b] - eq_unmatched_db[b];
        }
        let eq_delta = remove_offset(&eq_delta);

        let mut result = CaseResult {
            song: song.spec.label.clone(),
            group: song.spec.group,
            case: change.label(),
            changed_bands: change.changed_bands(),
            copy_shares: Some(copy_shares),
            measured_gap_db: measured_gap,
            correction_db: decision.correction_db,
            capped: decision.capped,
            fade,
            remaining_ratio: [None; BAND_COUNT],
            remaining_ratio_vs_rendered_original: [None; BAND_COUNT],
            eq_stage_ratio: [None; BAND_COUNT],
        };
        for b in 0..BAND_COUNT {
            if result.eligible(b) && !decision.capped[b] {
                let g = measured_gap[b];
                result.remaining_ratio[b] = Some((g - delta[b]) / g);
                result.remaining_ratio_vs_rendered_original[b] = Some(vs_original[b] / g);
                result.eq_stage_ratio[b] = Some((g - eq_delta[b]) / g);
            }
        }
        results.push(result);
    }
    Ok(results)
}

/// G3 and G6 tallies over a set of cases.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct KnownAnswerSummary {
    pub songs: usize,
    pub cases: usize,
    /// G3: changed bands with a gap of 1 dB or more and no missing-band fade.
    pub eligible_bands: usize,
    /// G3: eligible bands whose correction does not oppose the change.
    pub wrong_direction: usize,
    /// G3: eligible, uncapped bands where |correction − amount × gap| > 0.5.
    pub off_by_more_than_tolerance: usize,
    pub worst_amount_error_db: f64,
    /// G3: single- and two-band cases where every other band moved ≤ 0.5 dB.
    pub other_bands_quiet_cases: usize,
    pub other_bands_cases: usize,
    pub other_bands_quiet_share: f64,
    pub worst_other_band_move_db: f64,
    /// Reported only: boosts the missing-band rule reduced.
    pub faded_bands: usize,
    /// G6: eligible, uncapped bands evaluated, and those outside 35–65%.
    pub g6_bands: usize,
    pub g6_outside_range: usize,
    pub g6_min_ratio: f64,
    pub g6_max_ratio: f64,
    pub g6_median_ratio: f64,
    /// Context: ratio against the rendered original.
    pub context_vs_rendered_original_median: f64,
    pub context_vs_rendered_original_min: f64,
    pub context_vs_rendered_original_max: f64,
    pub context_outside_range: usize,
    /// Diagnostic: the EQ stage alone (see `CaseResult::eq_stage_ratio`).
    pub eq_stage_median_ratio: f64,
    pub eq_stage_min_ratio: f64,
    pub eq_stage_max_ratio: f64,
    pub eq_stage_outside_range: usize,
    /// G6 bands outside the range, per meter band.
    pub g6_outside_by_band: [usize; BAND_COUNT],
    pub g3_pass: bool,
    pub g6_pass: bool,
}

pub fn summarize_known_answers(cases: &[CaseResult]) -> KnownAnswerSummary {
    let mut s = KnownAnswerSummary {
        cases: cases.len(),
        g6_min_ratio: f64::INFINITY,
        g6_max_ratio: f64::NEG_INFINITY,
        context_vs_rendered_original_min: f64::INFINITY,
        context_vs_rendered_original_max: f64::NEG_INFINITY,
        eq_stage_min_ratio: f64::INFINITY,
        eq_stage_max_ratio: f64::NEG_INFINITY,
        ..Default::default()
    };
    let mut eq_stage = Vec::new();
    let mut songs: Vec<&str> = cases.iter().map(|c| c.song.as_str()).collect();
    songs.sort_unstable();
    songs.dedup();
    s.songs = songs.len();
    let mut ratios = Vec::new();
    let mut context = Vec::new();
    for c in cases {
        let amount = f64::from(EVAL_AMOUNT);
        for b in 0..BAND_COUNT {
            if c.faded(b) {
                s.faded_bands += 1;
            }
            if !c.eligible(b) {
                continue;
            }
            s.eligible_bands += 1;
            let corr = f64::from(c.correction_db[b]);
            // The copy moved the band by about −gap; the correction must have
            // the gap's sign (toward the reference).
            if corr == 0.0 || corr.signum() != c.measured_gap_db[b].signum() {
                s.wrong_direction += 1;
            }
            if !c.capped[b] {
                let err = (corr - amount * c.measured_gap_db[b]).abs();
                s.worst_amount_error_db = s.worst_amount_error_db.max(err);
                if err > G3_TOLERANCE_DB {
                    s.off_by_more_than_tolerance += 1;
                }
            }
            if let Some(r) = c.remaining_ratio[b] {
                s.g6_bands += 1;
                if r < G6_RANGE.0 || r > G6_RANGE.1 {
                    s.g6_outside_range += 1;
                    s.g6_outside_by_band[b] += 1;
                }
                s.g6_min_ratio = s.g6_min_ratio.min(r);
                s.g6_max_ratio = s.g6_max_ratio.max(r);
                ratios.push(r);
            }
            if let Some(r) = c.remaining_ratio_vs_rendered_original[b] {
                s.context_vs_rendered_original_min = s.context_vs_rendered_original_min.min(r);
                s.context_vs_rendered_original_max = s.context_vs_rendered_original_max.max(r);
                if r < G6_RANGE.0 || r > G6_RANGE.1 {
                    s.context_outside_range += 1;
                }
                context.push(r);
            }
            if let Some(r) = c.eq_stage_ratio[b] {
                s.eq_stage_min_ratio = s.eq_stage_min_ratio.min(r);
                s.eq_stage_max_ratio = s.eq_stage_max_ratio.max(r);
                if r < G6_RANGE.0 || r > G6_RANGE.1 {
                    s.eq_stage_outside_range += 1;
                }
                eq_stage.push(r);
            }
        }
        let changed = c.changed_bands.iter().filter(|x| **x).count();
        if changed < BAND_COUNT {
            s.other_bands_cases += 1;
            let worst = (0..BAND_COUNT)
                .filter(|&b| !c.changed_bands[b])
                .map(|b| f64::from(c.correction_db[b].abs()))
                .fold(0.0_f64, f64::max);
            s.worst_other_band_move_db = s.worst_other_band_move_db.max(worst);
            if worst <= G3_OTHER_BAND_DB {
                s.other_bands_quiet_cases += 1;
            }
        }
    }
    s.other_bands_quiet_share = if s.other_bands_cases > 0 {
        s.other_bands_quiet_cases as f64 / s.other_bands_cases as f64
    } else {
        1.0
    };
    s.g6_median_ratio = crate::smart_balance_reference::median(&ratios);
    s.context_vs_rendered_original_median = crate::smart_balance_reference::median(&context);
    s.eq_stage_median_ratio = crate::smart_balance_reference::median(&eq_stage);
    s.g3_pass = s.eligible_bands > 0
        && s.wrong_direction == 0
        && s.off_by_more_than_tolerance == 0
        && s.other_bands_quiet_share >= G3_OTHER_BAND_SHARE;
    s.g6_pass = s.g6_bands > 0 && s.g6_outside_range == 0;
    s
}

/// Run `work` over `items` on a bounded worker pool; results in input order.
pub fn parallel_map<T: Sync, R: Send>(
    items: &[T],
    workers: usize,
    work: impl Fn(&T) -> R + Sync,
) -> Vec<R> {
    let workers = workers.clamp(1, items.len().max(1));
    let next = AtomicUsize::new(0);
    let slots: Vec<Mutex<Option<R>>> = items.iter().map(|_| Mutex::new(None)).collect();
    std::thread::scope(|scope| {
        for _ in 0..workers {
            scope.spawn(|| loop {
                let i = next.fetch_add(1, Ordering::Relaxed);
                let Some(item) = items.get(i) else {
                    break;
                };
                let r = work(item);
                if let Ok(mut slot) = slots[i].lock() {
                    *slot = Some(r);
                }
            });
        }
    });
    slots
        .into_iter()
        .map(|s| s.into_inner().ok().flatten().expect("worker finished"))
        .collect()
}

/// The loudest contiguous `seconds` of a song (RMS of the channel mean over
/// windows stepped by one second), for the M22 report.
pub fn loudest_window(samples: &[f32], sample_rate: u32, channels: usize, seconds: f64) -> &[f32] {
    let ch = channels.max(1);
    let frames = samples.len() / ch;
    let win = (seconds * f64::from(sample_rate)) as usize;
    if win >= frames {
        return samples;
    }
    let step = sample_rate as usize;
    // Prefix sums of the per-frame mean-square.
    let mut prefix = vec![0.0_f64; frames + 1];
    for i in 0..frames {
        let mut m = 0.0_f64;
        for c in 0..ch {
            m += f64::from(samples[i * ch + c]);
        }
        m /= ch as f64;
        prefix[i + 1] = prefix[i] + m * m;
    }
    let mut best = (f64::NEG_INFINITY, 0);
    let mut start = 0;
    while start + win <= frames {
        let e = prefix[start + win] - prefix[start];
        if e > best.0 {
            best = (e, start);
        }
        start += step;
    }
    &samples[best.1 * ch..(best.1 + win) * ch]
}

/// Read a path list helper: every file under `root` (recursive), sorted.
pub fn files_under(root: &Path) -> CommandResult<Vec<PathBuf>> {
    let mut out = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let entries = std::fs::read_dir(&dir)
            .map_err(|e| CommandError::Io(format!("read {}: {e}", dir.display())))?;
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else {
                out.push(path);
            }
        }
    }
    out.sort();
    Ok(out)
}

// ---------------------------------------------------------------------------
// G1 and G2: identity on real songs
// ---------------------------------------------------------------------------

pub const ALL_PRESETS: [&str; 9] = [
    "universal",
    "clarity",
    "tape",
    "spatial",
    "oomph",
    "warmth",
    "punch",
    "loud",
    "custom",
];

pub fn preset_named(name: &str) -> Preset {
    match name {
        "universal" => Preset::Universal,
        "clarity" => Preset::Clarity,
        "tape" => Preset::Tape,
        "spatial" => Preset::Spatial,
        "oomph" => Preset::Oomph,
        "warmth" => Preset::Warmth,
        "punch" => Preset::Punch,
        "loud" => Preset::Loud,
        _ => Preset::Custom {
            id: "custom".into(),
        },
    }
}

/// Identity results for one song.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct IdentityResult {
    pub song: String,
    /// G1: preset × off-state checks, and how many matched bit for bit
    /// (coefficients equal and every rendered sample identical).
    pub g1_checks: usize,
    pub g1_identical: usize,
    /// G2: presets checked with a self-match, and how many were identical.
    pub g2_checks: usize,
    pub g2_identical: usize,
}

fn bit_identical(a: &[f32], b: &[f32]) -> bool {
    a.len() == b.len() && a.iter().zip(b).all(|(x, y)| x.to_bits() == y.to_bits())
}

/// G1 (no reference, switched off, amount zero) and G2 (self-match), every
/// preset, on `seconds` of a real song.
pub fn identity_checks(song: &LoadedSong, seconds: f64) -> IdentityResult {
    let sr = song.sample_rate;
    let ch = song.channels;
    let frames = ((seconds * f64::from(sr)) as usize).min(song.samples.len() / ch);
    let excerpt = &song.samples[..frames * ch];
    let other = SpectralBalance6 {
        sub: song.profile.spectral_6.sub * 0.5,
        ..song.profile.spectral_6
    };
    let mut r = IdentityResult {
        song: song.spec.label.clone(),
        ..Default::default()
    };
    for name in ALL_PRESETS {
        let plain = preset_settings(preset_named(name), Some(song.profile));
        let baseline_coeffs = crate::dsp::ChainCoeffs::from_settings(sr, &plain);
        let baseline = render(excerpt, sr, ch, &plain);
        let mut none = plain.clone();
        none.advanced.smart_match = Some(SmartMatchSettings::default());
        let off_states = [
            none,
            with_match(plain.clone(), other, false, 0.5),
            with_match(plain.clone(), other, true, 0.0),
        ];
        for s in &off_states {
            r.g1_checks += 1;
            if crate::dsp::ChainCoeffs::from_settings(sr, s) == baseline_coeffs
                && bit_identical(&render(excerpt, sr, ch, s), &baseline)
            {
                r.g1_identical += 1;
            }
        }
        let own = measure(&song.samples, sr, ch).unwrap_or(song.profile.spectral_6);
        let self_match = with_match(plain.clone(), own, true, 1.0);
        r.g2_checks += 1;
        if bit_identical(&render(excerpt, sr, ch, &self_match), &baseline) {
            r.g2_identical += 1;
        }
    }
    r
}

// ---------------------------------------------------------------------------
// G4: realization accuracy through the real coefficients
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RealizationSummary {
    pub cases: usize,
    pub worst_error_db: f64,
    pub over_half_db: usize,
    pub pass: bool,
}

fn eq_cascade(c: &crate::dsp::ChainCoeffs) -> Vec<crate::dsp::BiquadCoeffs> {
    let mut v = vec![
        c.sub_highpass,
        c.sub_highpass,
        c.sub,
        c.low,
        c.low_mid,
        c.mid,
        c.high_mid,
        c.high,
        c.sparkle,
        c.warmth,
        c.presence_air,
    ];
    if c.smart_match_active {
        v.push(c.smart_match_bass);
    }
    v
}

/// G4: every single-band cap case and 20 seeded mixed cases, at 44.1, 48 and
/// 96 kHz, in every place the corrective filter can run.
pub fn realization_accuracy() -> RealizationSummary {
    let typical = shares_from_levels(smart_match::TYPICAL_SHAPE_DB);
    let mut targets: Vec<[f64; BAND_COUNT]> = Vec::new();
    for band in 0..BAND_COUNT {
        for v in [
            f64::from(smart_match::BOOST_CAP_DB),
            -f64::from(smart_match::CUT_CAP_DB),
        ] {
            let mut t = [0.0; BAND_COUNT];
            t[band] = v;
            targets.push(t);
        }
    }
    let mut seed = 0x6A7E_u64;
    for _ in 0..20 {
        let mut t = [0.0; BAND_COUNT];
        for x in t.iter_mut() {
            seed = seed
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            *x = -4.0 + 7.0 * ((seed >> 11) as f64 / (1u64 << 53) as f64);
        }
        targets.push(t);
    }
    let mut s = RealizationSummary::default();
    for sr in [44_100, 48_000, 96_000] {
        for (i, t) in targets.iter().enumerate() {
            let mut levels = smart_match::TYPICAL_SHAPE_DB;
            for (l, x) in levels.iter_mut().zip(t) {
                *l += x;
            }
            for slot in 0..3 {
                let mut plain = preset_settings(
                    Preset::Universal,
                    Some(SourceProfile {
                        spectral_6: typical,
                        dynamic_range_p95_p10_db: 12.0,
                        dynamic_range_lu: 6.0,
                        stereo_correlation: Some(0.6),
                        stereo_width: 0.3,
                    }),
                );
                if slot >= 1 {
                    plain.advanced.warmth = Some(0.5);
                }
                if slot == 2 {
                    plain.advanced.presence_air = Some(0.25);
                }
                let matched = with_match(plain.clone(), shares_from_levels(levels), true, 1.0);
                let decision = smart_match::decide_for_settings(&matched);
                let change = smart_match::band_power_change_db(
                    sr,
                    &eq_cascade(&crate::dsp::ChainCoeffs::from_settings(sr, &plain)),
                    &eq_cascade(&crate::dsp::ChainCoeffs::from_settings(sr, &matched)),
                    240,
                );
                let _ = i;
                s.cases += 1;
                let worst = (0..BAND_COUNT)
                    .map(|b| (change[b] - f64::from(decision.correction_db[b])).abs())
                    .fold(0.0_f64, f64::max);
                s.worst_error_db = s.worst_error_db.max(worst);
                if worst > 0.5 {
                    s.over_half_db += 1;
                }
            }
        }
    }
    s.pass = s.over_half_db == 0;
    s
}

/// Shares from per-band levels in dB (normalized).
pub fn shares_from_levels(levels: [f64; BAND_COUNT]) -> SpectralBalance6 {
    let lin = levels.map(|d| 10f64.powf(d / 10.0));
    let total: f64 = lin.iter().sum();
    let s = lin.map(|v| (v / total) as f32);
    SpectralBalance6 {
        sub: s[0],
        low: s[1],
        low_mid: s[2],
        mid: s[3],
        presence: s[4],
        air: s[5],
    }
}

// ---------------------------------------------------------------------------
// G5: level invariance; G7: amount and caps
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LevelSummary {
    pub pairs: usize,
    pub worst_change_db: f64,
    pub pass: bool,
}

/// G5: each track at +12 and −12 dB, and each reference at ±12 dB, against
/// the unscaled correction.
pub fn level_invariance(
    tracks: &[&LoadedSong],
    references: &[SpectralBalance6],
    reference_songs: &[&LoadedSong],
    params: &SmartMatchParams,
) -> LevelSummary {
    let scaled = |song: &LoadedSong, db: f32| -> SpectralBalance6 {
        let g = 10f32.powf(db / 20.0);
        let s: Vec<f32> = song.samples.iter().map(|x| x * g).collect();
        measure(&s, song.sample_rate, song.channels).unwrap_or(song.profile.spectral_6)
    };
    let mut s = LevelSummary::default();
    for track in tracks {
        let base_track = measure(&track.samples, track.sample_rate, track.channels)
            .unwrap_or(track.profile.spectral_6);
        let up = scaled(track, 12.0);
        let down = scaled(track, -12.0);
        for reference in references {
            let base = decide_with(params, Some(&base_track), Some(reference), true, 1.0);
            for t in [&up, &down] {
                let d = decide_with(params, Some(t), Some(reference), true, 1.0);
                s.pairs += 1;
                for b in 0..BAND_COUNT {
                    s.worst_change_db = s.worst_change_db.max(f64::from(
                        (d.correction_db[b] - base.correction_db[b]).abs(),
                    ));
                }
            }
        }
        for reference_song in reference_songs {
            let base_reference = measure(
                &reference_song.samples,
                reference_song.sample_rate,
                reference_song.channels,
            )
            .unwrap_or(reference_song.profile.spectral_6);
            let base = decide_with(params, Some(&base_track), Some(&base_reference), true, 1.0);
            for db in [12.0, -12.0] {
                let r = scaled(reference_song, db);
                let d = decide_with(params, Some(&base_track), Some(&r), true, 1.0);
                s.pairs += 1;
                for b in 0..BAND_COUNT {
                    s.worst_change_db = s.worst_change_db.max(f64::from(
                        (d.correction_db[b] - base.correction_db[b]).abs(),
                    ));
                }
            }
        }
    }
    s.pass = s.pairs > 0 && s.worst_change_db <= 0.1;
    s
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AmountSummary {
    pub pairs: usize,
    pub amounts: Vec<f32>,
    /// Bands whose correction shrank as the amount grew.
    pub non_monotonic: usize,
    /// Corrections outside the caps at any amount.
    pub over_cap: usize,
    /// Decisions where the partial flag disagreed with an independent
    /// recomputation of cap activity.
    pub partial_mismatch: usize,
    pub pairs_with_a_cap_at_full_amount: usize,
    pub pass: bool,
}

/// G7 over (track, reference) share pairs.
pub fn amount_and_caps(
    pairs: &[(SpectralBalance6, SpectralBalance6)],
    params: &SmartMatchParams,
) -> AmountSummary {
    let amounts = vec![0.1_f32, 0.25, 0.5, 0.75, 1.0];
    let mut s = AmountSummary {
        amounts: amounts.clone(),
        ..Default::default()
    };
    for (track, reference) in pairs {
        s.pairs += 1;
        let mut previous = [0.0_f32; BAND_COUNT];
        let below = below_typical_db(&levels_db(track));
        for &amount in &amounts {
            let d = decide_with(params, Some(track), Some(reference), true, amount);
            let gap = d.gap_db.unwrap_or([0.0; BAND_COUNT]);
            let mut binds_any = false;
            for b in 0..BAND_COUNT {
                let c = d.correction_db[b];
                if c.abs() + 1e-6 < previous[b].abs()
                    || (previous[b] != 0.0 && c.signum() != previous[b].signum())
                {
                    s.non_monotonic += 1;
                }
                if c > params.boost_cap_db + 1e-6 || c < -params.cut_cap_db - 1e-6 {
                    s.over_cap += 1;
                }
                let mut v = f64::from(amount) * f64::from(gap[b]);
                if v > 0.0 {
                    v *= f64::from(boost_fade(params, below[b]));
                }
                binds_any |= v > f64::from(params.boost_cap_db) + 1e-4
                    || v < -f64::from(params.cut_cap_db) - 1e-4;
                previous[b] = c;
            }
            let near_edge = (0..BAND_COUNT).any(|b| {
                let mut v = f64::from(amount) * f64::from(gap[b]);
                if v > 0.0 {
                    v *= f64::from(boost_fade(params, below[b]));
                }
                (v - f64::from(params.boost_cap_db)).abs() < 1e-4
                    || (v + f64::from(params.cut_cap_db)).abs() < 1e-4
            });
            if !near_edge && d.partial != binds_any {
                s.partial_mismatch += 1;
            }
            if amount == 1.0 && d.partial {
                s.pairs_with_a_cap_at_full_amount += 1;
            }
        }
    }
    s.pass = s.non_monotonic == 0 && s.over_cap == 0 && s.partial_mismatch == 0;
    s
}

// ---------------------------------------------------------------------------
// G9: safety; G10: persistence
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SafetySummary {
    /// Each synthetic file and what the reference measurement returned.
    pub files: Vec<(String, String)>,
    pub refusals_correct: bool,
    /// Songs whose sub band was emptied, and how many got a sub boost.
    pub empty_band_tracks: usize,
    pub empty_band_boosted: usize,
    pub hostile_inputs_inert: bool,
    pub pass: bool,
}

fn write_wav(path: &Path, sample_rate: u32, channels: u16, samples: &[f32]) -> CommandResult<()> {
    let spec = hound::WavSpec {
        channels,
        sample_rate,
        bits_per_sample: 32,
        sample_format: hound::SampleFormat::Float,
    };
    let mut w = hound::WavWriter::create(path, spec)
        .map_err(|e| CommandError::Io(format!("create {}: {e}", path.display())))?;
    for s in samples {
        w.write_sample(*s)
            .map_err(|e| CommandError::Io(format!("write {}: {e}", path.display())))?;
    }
    w.finalize()
        .map_err(|e| CommandError::Io(format!("finalize {}: {e}", path.display())))
}

fn test_tone(sample_rate: u32, channels: u16, seconds: f64, amp: f32) -> Vec<f32> {
    let frames = (f64::from(sample_rate) * seconds) as usize;
    let mut out = Vec::with_capacity(frames * usize::from(channels));
    for n in 0..frames {
        let t = n as f32 / sample_rate as f32;
        let v = amp
            * ((std::f32::consts::TAU * 60.0 * t).sin()
                + 0.7 * (std::f32::consts::TAU * 160.0 * t).sin()
                + 0.5 * (std::f32::consts::TAU * 500.0 * t).sin()
                + 0.4 * (std::f32::consts::TAU * 1_500.0 * t).sin()
                + 0.3 * (std::f32::consts::TAU * 4_000.0 * t).sin()
                + 0.2 * (std::f32::consts::TAU * 10_000.0 * t).sin());
        for _ in 0..channels {
            out.push(v);
        }
    }
    out
}

/// G9 on synthetic files written to `scratch` and on `songs` with their sub
/// band removed by the independent equalizer.
pub fn safety_checks(
    scratch: &Path,
    songs: &[&LoadedSong],
    params: &SmartMatchParams,
) -> CommandResult<SafetySummary> {
    use crate::smart_match_reference::{
        measure_reference_file, SmartMatchReferenceMeasurement as M, SmartMatchRefusal as R,
    };
    std::fs::create_dir_all(scratch)
        .map_err(|e| CommandError::Io(format!("create {}: {e}", scratch.display())))?;
    struct Synthetic {
        name: &'static str,
        rate: u32,
        channels: u16,
        seconds: f64,
        amp: f32,
        expected: Option<R>,
    }
    let cases = [
        Synthetic {
            name: "silent.wav",
            rate: 44_100,
            channels: 2,
            seconds: 31.0,
            amp: 0.0,
            expected: Some(R::Silent),
        },
        Synthetic {
            name: "very-short.wav",
            rate: 44_100,
            channels: 2,
            seconds: 10.0,
            amp: 0.2,
            expected: Some(R::TooShort),
        },
        Synthetic {
            name: "low-rate.wav",
            rate: 22_050,
            channels: 2,
            seconds: 31.0,
            amp: 0.2,
            expected: Some(R::LowSampleRate),
        },
        // Mono can be measured, so plan M10 accepts it (deviation log).
        Synthetic {
            name: "mono.wav",
            rate: 48_000,
            channels: 1,
            seconds: 31.0,
            amp: 0.2,
            expected: None,
        },
    ];
    let mut s = SafetySummary {
        refusals_correct: true,
        ..Default::default()
    };
    for c in cases {
        let path = scratch.join(c.name);
        write_wav(
            &path,
            c.rate,
            c.channels,
            &test_tone(c.rate, c.channels, c.seconds, c.amp),
        )?;
        let got = measure_reference_file(&path);
        let ok = match (&got, c.expected) {
            (M::Refused { reason }, Some(want)) => *reason == want,
            (M::Measured { .. }, None) => true,
            _ => false,
        };
        s.refusals_correct &= ok;
        s.files.push((
            c.name.to_string(),
            format!("{got:?}").chars().take(80).collect(),
        ));
    }
    let garbage = scratch.join("not-audio.wav");
    std::fs::write(&garbage, b"definitely not audio")
        .map_err(|e| CommandError::Io(e.to_string()))?;
    let got = measure_reference_file(&garbage);
    s.refusals_correct &= got
        == M::Refused {
            reason: R::Unreadable,
        };
    s.files.push(("not-audio.wav".into(), format!("{got:?}")));

    for song in songs {
        let emptied = apply_tone_change(
            &song.samples,
            song.sample_rate,
            song.channels,
            &ToneChange::Band { band: 0, db: -90.0 },
        );
        let track = measure(&emptied, song.sample_rate, song.channels)?;
        let d = decide_with(
            params,
            Some(&track),
            Some(&song.profile.spectral_6),
            true,
            1.0,
        );
        s.empty_band_tracks += 1;
        if d.correction_db[0] > 0.0 {
            s.empty_band_boosted += 1;
        }
        // And through the chain: no panic, finite output.
        let settings = with_match(
            preset_settings(
                Preset::Universal,
                Some(SourceProfile {
                    spectral_6: track,
                    ..song.profile
                }),
            ),
            song.profile.spectral_6,
            true,
            1.0,
        );
        let frames = (song.sample_rate as usize * 10).min(emptied.len() / song.channels);
        let out = render(
            &emptied[..frames * song.channels],
            song.sample_rate,
            song.channels,
            &settings,
        );
        if out.iter().any(|x| !x.is_finite()) {
            s.empty_band_boosted += 1;
        }
    }

    let good = songs
        .first()
        .map(|s| s.profile.spectral_6)
        .unwrap_or(shares_from_levels(smart_match::TYPICAL_SHAPE_DB));
    let nan = SpectralBalance6 {
        sub: f32::NAN,
        ..good
    };
    let zero = SpectralBalance6 {
        sub: 0.0,
        low: 0.0,
        low_mid: 0.0,
        mid: 0.0,
        presence: 0.0,
        air: 0.0,
    };
    s.hostile_inputs_inert = [
        decide_with(params, Some(&nan), Some(&good), true, 1.0),
        decide_with(params, Some(&good), Some(&nan), true, 1.0),
        decide_with(params, Some(&zero), Some(&good), true, 1.0),
        decide_with(params, Some(&good), Some(&good), true, f32::NAN),
        decide_with(params, Some(&good), Some(&zero), true, f32::INFINITY),
    ]
    .iter()
    .all(|d| !d.changes_anything());
    s.pass = s.refusals_correct && s.empty_band_boosted == 0 && s.hostile_inputs_inert;
    Ok(s)
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PersistenceSummary {
    pub reopened_without_reference_file: bool,
    pub same_correction_after_reopen: bool,
    pub preset_carries_no_reference: bool,
    pub pass: bool,
}

/// G10: a project made with a real reference file reopens with the same match
/// after that file is deleted, and a saved preset carries no reference.
pub fn persistence_checks(
    scratch: &Path,
    track: &LoadedSong,
    reference_song: &LoadedSong,
) -> CommandResult<PersistenceSummary> {
    use crate::smart_match_reference::{
        measure_reference_file, SmartMatchReferenceMeasurement as M,
    };
    std::fs::create_dir_all(scratch).map_err(|e| CommandError::Io(e.to_string()))?;
    let ref_path = scratch.join("reference-copy.wav");
    write_wav(
        &ref_path,
        reference_song.sample_rate,
        reference_song.channels as u16,
        &reference_song.samples,
    )?;
    let M::Measured { reference } = measure_reference_file(&ref_path) else {
        return Err(CommandError::Other(
            "reference copy could not be measured".into(),
        ));
    };
    let mut settings = preset_settings(Preset::Universal, None);
    settings.advanced.smart_match = Some(SmartMatchSettings {
        reference: Some(reference.clone()),
        enabled: true,
        amount: 0.5,
    });
    let before = {
        let mut s = settings.clone();
        s.advanced.source_profile = Some(track.profile);
        smart_match::decide_for_settings(&s)
    };
    let project_path = scratch.join("song.ams.json");
    let id = TrackId("t".into());
    let mut track_settings = std::collections::HashMap::new();
    track_settings.insert(id.0.clone(), settings.clone());
    let state = crate::types::ProjectState {
        schema_version: 1,
        mode: crate::types::ProjectMode::Track,
        tracks: Vec::new(),
        track_order: vec![id],
        track_settings,
        album_intent: None,
        album_arc_kind: Default::default(),
        album_intensity: 1.0,
        album_title: String::new(),
        album_sample_rate: None,
        album_bit_depth: None,
        track_override_album: Vec::new(),
        selected_track_id: None,
        view_by_track_id: std::collections::HashMap::new(),
        last_saved_iso: None,
    };
    crate::project::write_session_atomic(&project_path, &state)?;
    std::fs::remove_file(&ref_path).map_err(|e| CommandError::Io(e.to_string()))?;
    let loaded = crate::project::read_session(&project_path)?;
    let mut reopened = loaded.track_settings["t"].clone();
    let reopened_ok = reopened.advanced.smart_match == settings.advanced.smart_match;
    reopened.advanced.source_profile = Some(track.profile);
    let after = smart_match::decide_for_settings(&reopened);

    let presets = scratch.join("user_presets.json");
    let _ = std::fs::remove_file(&presets);
    let saved = crate::settings::save_user_preset_at_path(
        &presets,
        "Mine",
        crate::types::PresetKind::Track,
        settings,
    )?;
    let listed = crate::settings::read_presets(&presets)?;
    let text = std::fs::read_to_string(&presets).map_err(|e| CommandError::Io(e.to_string()))?;
    let preset_ok = saved.settings.advanced.smart_match.is_none()
        && listed
            .iter()
            .all(|p| p.settings.advanced.smart_match.is_none())
        && !text.contains("smart_match");
    let s = PersistenceSummary {
        reopened_without_reference_file: reopened_ok,
        same_correction_after_reopen: after == before && before.changes_anything(),
        preset_carries_no_reference: preset_ok,
        pass: reopened_ok && after == before && preset_ok,
    };
    Ok(s)
}

// ---------------------------------------------------------------------------
// Ungated reports
// ---------------------------------------------------------------------------

/// One track matched to one reference (decision level).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchRow {
    pub track: String,
    pub reference: String,
    pub reference_genre: String,
    pub gap_db: [f32; BAND_COUNT],
    pub correction_db: [f32; BAND_COUNT],
    pub capped_bands: usize,
    pub partial: bool,
}

/// Cross-genre matches: each track against each of its references.
pub fn cross_genre(
    pairs: &[(&str, SpectralBalance6, &str, &str, SpectralBalance6)],
    params: &SmartMatchParams,
) -> Vec<MatchRow> {
    pairs
        .iter()
        .map(|(track, t, reference, genre, r)| {
            let d = decide_with(params, Some(t), Some(r), true, EVAL_AMOUNT);
            MatchRow {
                track: (*track).to_string(),
                reference: (*reference).to_string(),
                reference_genre: (*genre).to_string(),
                gap_db: d.gap_db.unwrap_or([0.0; BAND_COUNT]),
                correction_db: d.correction_db,
                capped_bands: d.capped.iter().filter(|c| **c).count(),
                partial: d.partial,
            }
        })
        .collect()
}

/// The owner's song matched to one LANDR master.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LandrRow {
    pub reference: String,
    /// What LANDR did: its master minus the source, per band, offset removed.
    pub landr_move_db: [f64; BAND_COUNT],
    /// Smart Match's decided correction at 50%.
    pub correction_db: [f32; BAND_COUNT],
    /// What the match changed end to end (Universal, default Intensity):
    /// matched render minus unmatched render, offset removed.
    pub rendered_move_db: [f64; BAND_COUNT],
    /// Bands where LANDR moved at least 0.5 dB, and how many of those the
    /// correction pushes the same way.
    pub bands_landr_moved: usize,
    pub same_direction: usize,
    pub partial: bool,
}

pub fn owner_vs_landr(
    source: &LoadedSong,
    masters: &[(String, SpectralBalance6)],
    params: &SmartMatchParams,
) -> CommandResult<Vec<LandrRow>> {
    let sr = source.sample_rate;
    let ch = source.channels;
    let plain = preset_settings(Preset::Universal, Some(source.profile));
    let unmatched = levels_db(&measure(&render(&source.samples, sr, ch, &plain), sr, ch)?);
    let mut rows = Vec::new();
    for (label, master) in masters {
        let landr_move = gap_db(&levels_db(&source.profile.spectral_6), &levels_db(master));
        let d = decide_with(
            params,
            Some(&source.profile.spectral_6),
            Some(master),
            true,
            EVAL_AMOUNT,
        );
        let matched = with_match(plain.clone(), *master, true, EVAL_AMOUNT);
        let matched_db = levels_db(&measure(
            &render(&source.samples, sr, ch, &matched),
            sr,
            ch,
        )?);
        let mut rendered = [0.0; BAND_COUNT];
        for b in 0..BAND_COUNT {
            rendered[b] = matched_db[b] - unmatched[b];
        }
        let rendered = remove_offset(&rendered);
        let mut moved = 0;
        let mut same = 0;
        for b in 0..BAND_COUNT {
            if landr_move[b].abs() >= 0.5 {
                moved += 1;
                if d.correction_db[b] != 0.0
                    && f64::from(d.correction_db[b]).signum() == landr_move[b].signum()
                {
                    same += 1;
                }
            }
        }
        rows.push(LandrRow {
            reference: label.clone(),
            landr_move_db: landr_move,
            correction_db: d.correction_db,
            rendered_move_db: rendered,
            bands_landr_moved: moved,
            same_direction: same,
            partial: d.partial,
        });
    }
    Ok(rows)
}

/// M22: the correction from the loudest 30 seconds against the whole song.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListenRow {
    pub song: String,
    pub group: SongGroup,
    /// Largest |correction(30 s) − correction(whole)| over three references
    /// and six bands.
    pub largest_difference_db: f32,
    /// Share of the 18 band corrections within 0.5 dB.
    pub share_within_half_db: f64,
}

pub fn thirty_second_listen(
    song: &LoadedSong,
    references: &[SpectralBalance6],
    params: &SmartMatchParams,
) -> CommandResult<ListenRow> {
    let window = loudest_window(&song.samples, song.sample_rate, song.channels, 30.0);
    let short = measure(window, song.sample_rate, song.channels)?;
    let whole = song.profile.spectral_6;
    let mut largest = 0.0_f32;
    let mut within = 0;
    let mut total = 0;
    for r in references {
        let a = decide_with(params, Some(&short), Some(r), true, EVAL_AMOUNT);
        let b = decide_with(params, Some(&whole), Some(r), true, EVAL_AMOUNT);
        for band in 0..BAND_COUNT {
            let diff = (a.correction_db[band] - b.correction_db[band]).abs();
            largest = largest.max(diff);
            total += 1;
            if diff <= 0.5 {
                within += 1;
            }
        }
    }
    Ok(ListenRow {
        song: song.spec.label.clone(),
        group: song.spec.group,
        largest_difference_db: largest,
        share_within_half_db: within as f64 / total.max(1) as f64,
    })
}
