//! Auto gain staging (Beta): two automatic gains, so a song is limited only
//! as much as the chosen loudness needs
//! (`docs/plans/2026-11-24-auto-gain-staging-execution-plan.md`).
//!
//! 1. **Level in**, before the Style's EQ, compressor and saturation, brings
//!    every song to the level the Styles were voiced at (contract rule 2).
//! 2. **Limiter drive**, just before the limiter, adds only as much gain as
//!    the chosen loudness needs (contract rules 3 and 4).
//!
//! The existing landing and final-rate peak protection stay the last, exact
//! step, unchanged (rule 10). The chain applies the gains through
//! `MasteringSettings::active_gain_staging`; this module resolves them. It
//! is never called on the audio thread.
//!
//! **How an export resolves the drive.** The Style stages run once and their
//! output (the exact signal that reaches the limiter) is kept. Each pass then
//! runs a fresh limiter, built as the chain builds it, over that signal times
//! the drive, followed by the chain's post-limiter stages, the export's
//! sample-rate conversion and its qualified finalizer. The drive is chosen on
//! the **delivered** loudness (rule 3). The last pass is the delivered audio;
//! a test pins that it equals a chain built from the resolved gains, bit for
//! bit.

use std::sync::atomic::{AtomicBool, Ordering};

use crate::analysis::SourceLevel;
use crate::dsp::{Limiter, MasteringChain};
use crate::output_protection::ProtectedPcm;
use crate::types::{
    CommandError, CommandResult, LoudnessPriority, MasteringSettings, ResolvedGainStaging,
    GAIN_STAGING_RANGE_DB,
};

/// The reference level (decision A6): the app's own analysis of the owner's
/// reference song, `It’s a coat-original-test.wav`. Its level in is exactly
/// 0.0 dB, so the Style stages meet that song exactly as they do today. The
/// slow fixture lane pins both values against the real file.
pub const NOMINAL_LUFS: f32 = -12.331_284_5;
pub const NOMINAL_PEAK_DBTP: f32 = -1.882_721;
/// The most the limiter may take off the loudest peak automatically (A5).
pub const CLEAN_LIMIT_DB: f32 = 6.0;
/// Target tolerance when resolving the drive.
pub const TARGET_TOLERANCE_LU: f32 = 0.2;
/// Maximum resolution passes. Frozen at 6 after Checkpoint A (the plan
/// allows 2 to 6) so the development material satisfies R4.
pub const MAX_PASSES: usize = 6;
/// Below this a source has no usable loudness measurement (A7).
const UNUSABLE_LUFS: f32 = -70.0;

/// Level in for a measured source (contract rule 2): `min(NOMINAL_LUFS −
/// loudness, NOMINAL_PEAK − true peak)`, limited to ±24 dB. `None` (inert)
/// without a usable measurement.
pub fn level_in_db(source_lufs: f32, source_peak_dbtp: f32) -> Option<f32> {
    if !source_lufs.is_finite() || source_lufs <= UNUSABLE_LUFS || !source_peak_dbtp.is_finite() {
        return None;
    }
    let raw = (NOMINAL_LUFS - source_lufs).min(NOMINAL_PEAK_DBTP - source_peak_dbtp);
    Some(raw.clamp(-GAIN_STAGING_RANGE_DB, GAIN_STAGING_RANGE_DB))
}

/// Level in from the same measurement the track analysis reports.
pub fn level_in_for(level: &SourceLevel) -> Option<f32> {
    level_in_db(level.lufs_integrated, level.true_peak_dbtp)
}

// ---------------------------------------------------------------------------
// Receipt (locked copy)
// ---------------------------------------------------------------------------

/// One decimal place and a real minus sign. A value that rounds to zero is
/// `0.0`. `signed` adds a `+` to positive values.
fn db_text(value: f32, signed: bool) -> String {
    let rounded = (value * 10.0).round() / 10.0;
    if rounded == 0.0 {
        "0.0 dB".to_string()
    } else if rounded < 0.0 {
        format!("\u{2212}{:.1} dB", -rounded)
    } else if signed {
        format!("+{rounded:.1} dB")
    } else {
        format!("{rounded:.1} dB")
    }
}

/// The export receipt's line, from the gains the render used (rule 9):
/// `None` when the song never used the feature (so other receipts keep their
/// shape), `Auto gain staging: off` when it is off or inert, else the gains.
pub fn receipt_digest(settings: &MasteringSettings) -> Option<String> {
    settings.advanced.auto_gain_staging?;
    Some(match settings.active_gain_staging() {
        Some(g) => format!(
            "Auto gain staging: level in {} \u{b7} limiter drive {} \u{b7} limiting {}{}",
            db_text(g.level_in_db, true),
            db_text(g.limiter_drive_db, true),
            db_text(g.limiter_load_db.max(0.0), false),
            if g.stopped_at_clean_limit {
                " \u{b7} stopped at the clean limit"
            } else {
                ""
            }
        ),
        None => "Auto gain staging: off".to_string(),
    })
}

/// The target, present only when the clean limit stopped the drive short of
/// it, so the receipt can show the delivered loudness against it.
pub fn receipt_target_lufs(settings: &MasteringSettings) -> Option<f32> {
    settings
        .active_gain_staging()
        .filter(|g| g.stopped_at_clean_limit)
        .and(settings.effective_target_lufs())
}

// ---------------------------------------------------------------------------
// Export resolution
// ---------------------------------------------------------------------------

/// What an export with auto gain staging delivers.
pub struct ExportOutcome {
    /// The delivered PCM: at the delivery rate, landed and protected.
    pub samples: Vec<f32>,
    pub measured: ProtectedPcm,
    /// Limiter passes run (each one a full delivery).
    pub passes: usize,
}

fn cancelled(cancel: Option<&AtomicBool>) -> bool {
    cancel.is_some_and(|c| c.load(Ordering::Relaxed))
}

fn cancel_error() -> CommandError {
    CommandError::Render("auto gain staging cancelled".into())
}

/// The signal that reaches the limiter, with the stages' ring-out on the
/// `lookahead` frames of silence the export's tail flush also feeds it.
struct StageCapture {
    pre: Vec<f32>,
    frames: usize,
    lookahead: usize,
    channels: usize,
}

const PROGRESS_CHUNK_FRAMES: usize = 4096;

fn capture_stages(
    chain: &mut MasteringChain,
    samples: &[f32],
    channels: usize,
    cancel: Option<&AtomicBool>,
    progress: &dyn Fn(f32),
) -> CommandResult<StageCapture> {
    let frames = samples.len() / channels;
    let lookahead = chain.latency_frames();
    let mut pre = Vec::with_capacity((frames + lookahead) * channels);
    pre.extend_from_slice(&samples[..frames * channels]);
    pre.resize((frames + lookahead) * channels, 0.0);
    let total = frames + lookahead;
    for (chunk_index, chunk) in pre.chunks_mut(PROGRESS_CHUNK_FRAMES * channels).enumerate() {
        if cancelled(cancel) {
            return Err(cancel_error());
        }
        for frame in chunk.chunks_exact_mut(channels) {
            chain.process_stages_inplace(frame);
        }
        let done = ((chunk_index + 1) * PROGRESS_CHUNK_FRAMES).min(total);
        progress(0.5 * done as f32 / total.max(1) as f32);
    }
    Ok(StageCapture {
        pre,
        frames,
        lookahead,
        channels,
    })
}

/// One limiter pass: the drive, a fresh limiter and the post-limiter stages,
/// aligned with the source as the export's tail flush aligns it. Returns the
/// audio and the limiter's load.
fn limiter_pass(
    chain: &MasteringChain,
    template: &Limiter,
    stage: &StageCapture,
    drive_db: f32,
) -> (Vec<f32>, f32) {
    let ch = stage.channels;
    let mut limiter = template.clone();
    // Exactly the gain `ChainCoeffs::from_settings` computes for this drive.
    let gain = (drive_db != 0.0).then(|| {
        10.0_f32.powf(drive_db.clamp(-GAIN_STAGING_RANGE_DB, GAIN_STAGING_RANGE_DB) / 20.0)
    });
    let mut out = vec![0.0_f32; stage.frames * ch];
    let mut frame = vec![0.0_f32; ch];
    let mut load = 0.0_f32;
    for i in 0..stage.frames + stage.lookahead {
        frame.copy_from_slice(&stage.pre[i * ch..(i + 1) * ch]);
        if let Some(g) = gain {
            for s in frame.iter_mut() {
                *s *= g;
            }
        }
        limiter.process_frame_inplace(&mut frame);
        load = load.max(limiter.gain_reduction_db());
        chain.apply_post_limiter(&mut frame);
        if i >= stage.lookahead {
            let j = i - stage.lookahead;
            out[j * ch..(j + 1) * ch].copy_from_slice(&frame);
        }
    }
    (out, load)
}

/// Resolve auto gain staging for a Track Master export and render it.
///
/// Returns `Ok(None)` when the feature is inert (switch off or absent, Album,
/// no loudness target, no usable source measurement): the caller renders as
/// today, and `settings.advanced.gain_staging_resolved` is left `None`.
/// Otherwise the resolved gains are written to
/// `settings.advanced.gain_staging_resolved` (for the receipt) and the
/// delivered PCM is returned.
#[allow(clippy::too_many_arguments)]
pub fn render_export(
    samples: &[f32],
    sample_rate: u32,
    channels: u16,
    settings: &mut MasteringSettings,
    delivery_rate: u32,
    bits: u16,
    cancel: Option<&AtomicBool>,
    progress: &dyn Fn(f32),
) -> CommandResult<Option<ExportOutcome>> {
    settings.advanced.gain_staging_resolved = None;
    let priority = match settings.advanced.auto_gain_staging {
        Some(a) if a.enabled => a.priority,
        _ => return Ok(None),
    };
    let Some(target) = settings.effective_target_lufs().filter(|t| t.is_finite()) else {
        return Ok(None);
    };
    if settings.album.is_some() || samples.is_empty() {
        return Ok(None);
    }
    let level = crate::analysis::measure_source_level(samples, sample_rate, channels)?;
    let Some(level_in) = level_in_for(&level) else {
        return Ok(None);
    };
    let ch = usize::from(channels.max(1));
    settings.advanced.gain_staging_resolved = Some(ResolvedGainStaging {
        level_in_db: level_in,
        ..Default::default()
    });
    let mut chain = MasteringChain::new(sample_rate, ch, settings);
    let template = MasteringChain::fresh_limiter(sample_rate, ch, settings);
    let stage = capture_stages(&mut chain, samples, ch, cancel, progress)?;

    let ceiling_dbtp = settings.effective_ceiling_dbtp();
    let ceiling_db = 20.0 * chain.coeffs.ceiling_lin.log10();
    let stage_lufs = crate::engine::measure_integrated_lufs(
        &stage.pre[..stage.frames * ch],
        sample_rate,
        channels,
    )
    .unwrap_or(f32::NEG_INFINITY);
    let detector = Limiter::detector_level(&stage.pre, ch);
    let cap = match priority {
        LoudnessPriority::Clean if detector > 0.0 => {
            ceiling_db + CLEAN_LIMIT_DB - 20.0 * detector.log10()
        }
        _ => f32::INFINITY,
    }
    .min(GAIN_STAGING_RANGE_DB);
    let mut drive = if stage_lufs.is_finite() && stage_lufs > UNUSABLE_LUFS {
        (target - stage_lufs).clamp(-GAIN_STAGING_RANGE_DB, cap)
    } else {
        0.0
    };

    let mut passes = 0;
    loop {
        if cancelled(cancel) {
            return Err(cancel_error());
        }
        let (rendered, load) = limiter_pass(&chain, &template, &stage, drive);
        let mut delivered = if delivery_rate != sample_rate {
            crate::sample_rate::convert_interleaved(
                &rendered,
                sample_rate,
                delivery_rate,
                channels,
            )?
        } else {
            rendered
        };
        let measured = crate::output_protection::finalize(
            &mut delivered,
            delivery_rate,
            channels,
            bits,
            Some(target),
            ceiling_dbtp,
            cancel,
        )?;
        passes += 1;
        progress(0.5 + 0.5 * (passes as f32 / MAX_PASSES as f32).min(1.0));
        let reached = measured.lufs >= target - TARGET_TOLERANCE_LU;
        let at_cap = drive >= cap;
        let short = !reached && stage_lufs.is_finite() && stage_lufs > UNUSABLE_LUFS;
        if !short || at_cap || passes >= MAX_PASSES || !measured.lufs.is_finite() {
            settings.advanced.gain_staging_resolved = Some(ResolvedGainStaging {
                level_in_db: level_in,
                limiter_drive_db: drive,
                limiter_load_db: load,
                stopped_at_clean_limit: short
                    && at_cap
                    && matches!(priority, LoudnessPriority::Clean)
                    && cap < GAIN_STAGING_RANGE_DB,
            });
            return Ok(Some(ExportOutcome {
                samples: delivered,
                measured,
                passes,
            }));
        }
        drive = (drive + (target - measured.lufs)).min(cap);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn level_in_follows_contract_rule_two() {
        assert_eq!(level_in_db(NOMINAL_LUFS, NOMINAL_PEAK_DBTP), Some(0.0));
        let loud = level_in_db(NOMINAL_LUFS + 3.0, NOMINAL_PEAK_DBTP + 1.0).unwrap();
        assert!((loud + 3.0).abs() < 1e-5, "{loud}");
        let quiet = level_in_db(NOMINAL_LUFS - 8.0, NOMINAL_PEAK_DBTP - 2.0).unwrap();
        assert!((quiet - 2.0).abs() < 1e-5, "the peak term binds: {quiet}");
        assert_eq!(level_in_db(-69.0, -60.0), Some(GAIN_STAGING_RANGE_DB));
        assert_eq!(level_in_db(-6.0, 30.0), Some(-GAIN_STAGING_RANGE_DB));
        assert_eq!(level_in_db(UNUSABLE_LUFS, -60.0), None);
        assert_eq!(level_in_db(f32::NAN, -3.0), None);
        assert_eq!(level_in_db(-14.0, f32::INFINITY), None);
    }

    #[test]
    fn db_text_uses_one_decimal_and_a_real_minus() {
        assert_eq!(db_text(-3.26, true), "\u{2212}3.3 dB");
        assert_eq!(db_text(-0.04, true), "0.0 dB");
        assert_eq!(db_text(0.0, true), "0.0 dB");
        assert_eq!(db_text(4.5, true), "+4.5 dB");
        assert_eq!(db_text(3.16, false), "3.2 dB");
    }
}
