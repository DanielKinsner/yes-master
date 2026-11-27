//! Slice 4: the Phase A measurements through the production export path.
//!
//! T and G are both rendered by `engine::mastering_render_to_path` (decode,
//! chain or auto gain staging, sample-rate conversion, the qualified
//! finalizer, the 24-bit WAV writer) to a scratch file, which is decoded
//! back and deleted. T's research render is kept beside it: the two must be
//! bit-identical (E1), and the research render supplies T's landing amount,
//! which a receipt does not carry (R3 reads it). G's gains come from the
//! export receipt's own line; level in is recomputed exactly with the same
//! measurement the export uses.

use super::research::*;
use super::{load, measure_delivered, row, Loaded, RenderRow, Song, SongResult};
use std::path::Path;
use std::time::Instant;
use yes_master_lib::types::{
    AutoGainStagingSettings, LoudnessPriority, MasteringSettings, RenderKind, RenderedMeasurements,
    SmartMatchReference, SmartMatchSettings, SpectralBalance6, TrackId,
};

/// The research render, written as the export's WAV writer writes it (its
/// deterministic TPDF dither replayed by `DeliveryPcm`), against the file the
/// production export wrote: equal 24-bit sample codes, every sample.
fn same_delivered_pcm(research: &[f32], exported: &[f32], channels: usize) -> bool {
    use yes_master_lib::peak_meter::pcm::PcmSource;
    if research.len() != exported.len() || channels == 0 {
        return false;
    }
    let Ok(written) =
        yes_master_lib::wav_writer::DeliveryPcm::new(research, channels as u16, 24, || false)
    else {
        return false;
    };
    let frames = research.len() / channels;
    let code = |v: f64| (v * 8_388_608.0).round() as i64;
    let mut buf = vec![0.0_f64; 4096];
    for start in (0..frames).step_by(4096) {
        let n = 4096.min(frames - start);
        for c in 0..channels {
            if written.read_channel(c, start, &mut buf[..n]).is_err() {
                return false;
            }
            for (i, v) in buf[..n].iter().enumerate() {
                if code(*v) != code(f64::from(exported[(start + i) * channels + c])) {
                    return false;
                }
            }
        }
    }
    true
}

/// One Track Master export through the production path, read back.
fn export_master(
    source: &Path,
    settings: &MasteringSettings,
    scratch: &Path,
    name: &str,
) -> Result<(Vec<f32>, RenderedMeasurements), String> {
    let out = scratch.join(format!("{name}.wav"));
    let _ = std::fs::remove_file(&out);
    let job = yes_master_lib::engine::mastering_render_to_path(
        TrackId(name.into()),
        source,
        settings,
        scratch,
        RenderKind::Master,
        &out,
    )
    .map_err(|e| format!("{name}: export: {e}"))?;
    let path = job
        .output_paths
        .first()
        .cloned()
        .ok_or_else(|| format!("{name}: no output"))?;
    let measurements = job
        .measurements
        .ok_or_else(|| format!("{name}: no measurements"))?;
    let pcm = yes_master_lib::decode::decode_full(Path::new(&path))
        .map_err(|e| format!("{name}: read back: {e}"))?;
    let _ = std::fs::remove_file(&path);
    if pcm.sample_rate != STANDARD_RATE {
        return Err(format!("{name}: delivered at {} Hz", pcm.sample_rate));
    }
    Ok((pcm.samples, measurements))
}

fn facts_from(m: &RenderedMeasurements, samples: &[f32]) -> DeliveryFacts {
    DeliveryFacts {
        lufs: m.lufs_integrated,
        true_peak_dbtp: m.true_peak_dbtp,
        lra: m.dynamic_range_lu,
        // A receipt does not carry the landing gain.
        landing_db: f32::NAN,
        all_finite: samples.iter().all(|s| s.is_finite()),
        under_ceiling: m.true_peak_dbtp <= STANDARD_CEILING_DBTP,
    }
}

/// E6 on a real song: with a Smart Match reference active, the decision and
/// the corrective filter are identical with auto gain staging on and off.
fn smart_match_unchanged(src: &Loaded) -> bool {
    let Some(profile) = src.profile else {
        return true;
    };
    let shift = |p: SpectralBalance6| SpectralBalance6 {
        sub: p.sub * 0.6,
        air: p.air * 1.6,
        ..p
    };
    let mut off = standard_export_settings(
        yes_master_lib::types::Preset::Universal,
        -9.0,
        Some(profile),
        Some(src.lufs),
    );
    off.advanced.smart_match = Some(SmartMatchSettings {
        reference: Some(SmartMatchReference {
            name: "Reference".into(),
            spectral_6: shift(profile.spectral_6),
        }),
        enabled: true,
        amount: 0.5,
    });
    let mut on = off.clone();
    on.advanced.auto_gain_staging = Some(AutoGainStagingSettings {
        enabled: true,
        priority: LoudnessPriority::Clean,
    });
    on.advanced.gain_staging_resolved = Some(yes_master_lib::types::ResolvedGainStaging {
        level_in_db: -2.0,
        limiter_drive_db: 3.0,
        ..Default::default()
    });
    let a = yes_master_lib::dsp::ChainCoeffs::from_settings(src.rate, &off);
    let b = yes_master_lib::dsp::ChainCoeffs::from_settings(src.rate, &on);
    yes_master_lib::smart_match::decide_for_settings(&off)
        == yes_master_lib::smart_match::decide_for_settings(&on)
        && a.smart_match_bass == b.smart_match_bass
        && a.smart_match_slot == b.smart_match_slot
        && a.sub == b.sub
        && a.low == b.low
        && a.presence_air == b.presence_air
        && a.warmth == b.warmth
}

/// E7: export time through the production path, switch off against switch
/// on (priority Clean), Universal at each Standard loudness, three runs each,
/// single-threaded. Decoding is part of both.
pub(super) fn timing_production(
    songs: &[Song],
    scratch: &Path,
    output: &Path,
) -> Result<(), String> {
    let mut rows = Vec::new();
    for song in songs {
        let src = load(song, None)?;
        let seconds_of_audio = (src.samples.len() / src.channels) as f64 / f64::from(src.rate);
        for target in STANDARD_TARGETS {
            let off = standard_export_settings(
                yes_master_lib::types::Preset::Universal,
                target,
                src.profile,
                Some(src.lufs),
            );
            let mut on = off.clone();
            on.advanced.auto_gain_staging = Some(AutoGainStagingSettings {
                enabled: true,
                priority: LoudnessPriority::Clean,
            });
            let mut off_s = Vec::new();
            let mut on_s = Vec::new();
            let mut digest = String::new();
            for run in 0..3 {
                let t0 = Instant::now();
                export_master(
                    &song.path,
                    &off,
                    scratch,
                    &format!("{}-off-{run}", song.anon),
                )?;
                off_s.push(t0.elapsed().as_secs_f64());
                let t0 = Instant::now();
                let (_, m) =
                    export_master(&song.path, &on, scratch, &format!("{}-on-{run}", song.anon))?;
                on_s.push(t0.elapsed().as_secs_f64());
                digest = m.auto_gain_staging_digest.unwrap_or_default();
            }
            let med = |v: &[f64]| median(v).unwrap_or(f64::NAN);
            println!(
                "{} {seconds_of_audio:.0}s target {target}: off {:.2}s, on {:.2}s ({digest})",
                song.label,
                med(&off_s),
                med(&on_s)
            );
            rows.push(serde_json::json!({
                "song": song.label, "seconds_of_audio": seconds_of_audio, "target": target,
                "export_off_s": med(&off_s), "export_on_s": med(&on_s), "runs": 3,
                "receipt_line": digest,
            }));
        }
    }
    let text = serde_json::to_string_pretty(&rows).map_err(|e| e.to_string())?;
    std::fs::write(output.join("timing-production.json"), text).map_err(|e| e.to_string())
}

/// Slice 8, F1: audition's fast estimate against the exact resolution (the
/// export's own, pinned bit-identical), for every Standard Style and loudness
/// with Clean, plus Reach target wherever Clean stopped at the clean limit
/// (the Slice 4 case set). Each estimate is run at every `margins` value.
pub(super) fn estimate_check_song(
    song: &Song,
    variants: &[(yes_master_lib::gain_staging::EstimatePeak, f32)],
    cached: Option<&[serde_json::Value]>,
) -> Result<Vec<serde_json::Value>, String> {
    use yes_master_lib::gain_staging::{estimate_preview_with, resolve_preview};
    let src = load(song, None)?;
    let ch = src.channels as u16;
    let level = yes_master_lib::analysis::measure_source_level(&src.samples, src.rate, ch)
        .map_err(|e| format!("{}: {e}", song.label))?;
    let mut rows = Vec::new();
    for (style, preset) in standard_styles() {
        for target in STANDARD_TARGETS {
            let base =
                standard_export_settings(preset.clone(), target, src.profile, Some(src.lufs));
            let mut priorities = vec![LoudnessPriority::Clean];
            let mut i = 0;
            while i < priorities.len() {
                let priority = priorities[i];
                i += 1;
                let mut s = base.clone();
                s.advanced.auto_gain_staging = Some(AutoGainStagingSettings {
                    enabled: true,
                    priority,
                });
                let exact_json = if let Some(cached) = cached {
                    let found = cached.iter().find(|r| {
                        r["song"] == song.label.as_str()
                            && r["style"] == style
                            && r["target"].as_f64() == Some(f64::from(target))
                            && r["priority"] == format!("{priority:?}").as_str()
                    });
                    let Some(found) = found else {
                        continue;
                    };
                    found["exact"].clone()
                } else {
                    let t0 = Instant::now();
                    let exact = resolve_preview(&src.samples, src.rate, ch, &s, Some(&level), None)
                        .map_err(|e| format!("{}: exact: {e}", song.label))?;
                    let exact_s = t0.elapsed().as_secs_f64();
                    let Some(exact) = exact else {
                        continue;
                    };
                    serde_json::json!({"drive_db": exact.gains.limiter_drive_db,
                        "level_in_db": exact.gains.level_in_db,
                        "load_db": exact.gains.limiter_load_db,
                        "delivered_lufs": exact.delivered_lufs,
                        "stopped_at_clean_limit": exact.gains.stopped_at_clean_limit,
                        "stopped_by_peak_check": exact.gains.stopped_by_peak_check,
                        "seconds": exact_s})
                };
                if priority == LoudnessPriority::Clean
                    && exact_json["stopped_at_clean_limit"] == true
                {
                    priorities.push(LoudnessPriority::ReachTarget);
                }
                let mut estimates = Vec::new();
                for &(peak, margin) in variants {
                    let t0 = Instant::now();
                    let est = estimate_preview_with(
                        &src.samples,
                        src.rate,
                        ch,
                        &s,
                        Some(&level),
                        peak,
                        margin,
                        None,
                    )
                    .map_err(|e| format!("{}: estimate: {e}", song.label))?;
                    let seconds = t0.elapsed().as_secs_f64();
                    if let Some((est, searched_again)) = est {
                        estimates.push(serde_json::json!({
                            "peak": format!("{peak:?}"),
                            "margin_db": margin, "drive_db": est.gains.limiter_drive_db,
                            "level_in_db": est.gains.level_in_db,
                            "load_db": est.gains.limiter_load_db,
                            "delivered_lufs": est.delivered_lufs,
                            "stopped_at_clean_limit": est.gains.stopped_at_clean_limit,
                            "stopped_by_peak_check": est.gains.stopped_by_peak_check,
                            "seconds": seconds, "searched_again": searched_again,
                        }));
                    }
                }
                rows.push(serde_json::json!({
                    "song": song.label, "group": song.group, "style": style, "target": target,
                    "priority": format!("{priority:?}"),
                    "exact": exact_json,
                    "estimates": estimates,
                }));
            }
        }
    }
    Ok(rows)
}

/// Slice 8, F4: the SHA-256 of every Track export, switch off and on (Clean
/// and Reach target), for each Standard Style and loudness, so two commits'
/// exports can be compared byte for byte. Songs run in parallel; each export
/// is single-threaded as in the app. Files are deleted after hashing.
pub(super) fn export_hashes(out: &Path, songs: &str) -> Result<(), String> {
    use sha2::{Digest, Sha256};
    let scratch = out.with_extension("scratch");
    std::fs::create_dir_all(&scratch).map_err(|e| e.to_string())?;
    let entries: Vec<(String, std::path::PathBuf)> = songs
        .split('|')
        .filter(|e| !e.trim().is_empty())
        .map(|e| {
            let (n, p) = e.split_once('=').unwrap_or((e, e));
            (n.to_string(), p.into())
        })
        .collect();
    let results = std::thread::scope(|scope| {
        let handles: Vec<_> = entries
            .iter()
            .map(|(name, path)| {
                let scratch = &scratch;
                scope.spawn(move || -> Result<Vec<serde_json::Value>, String> {
                    let song = Song {
                        label: name.clone(),
                        group: "owner".into(),
                        kind: String::new(),
                        anon: name.clone(),
                        reference: false,
                        path: path.clone(),
                    };
                    let src = load(&song, None)?;
                    let mut rows = Vec::new();
                    for (style, preset) in standard_styles() {
                        for target in STANDARD_TARGETS {
                            let off = standard_export_settings(
                                preset.clone(),
                                target,
                                src.profile,
                                Some(src.lufs),
                            );
                            for mode in ["off", "clean", "reach"] {
                                let mut s = off.clone();
                                if mode != "off" {
                                    s.advanced.auto_gain_staging = Some(AutoGainStagingSettings {
                                        enabled: true,
                                        priority: if mode == "clean" {
                                            LoudnessPriority::Clean
                                        } else {
                                            LoudnessPriority::ReachTarget
                                        },
                                    });
                                }
                                let tag = format!("{name}-{style}-{target}-{mode}");
                                let file = scratch.join(format!("{tag}.wav"));
                                let _ = std::fs::remove_file(&file);
                                let job = yes_master_lib::engine::mastering_render_to_path(
                                    TrackId(tag.clone()),
                                    &song.path,
                                    &s,
                                    scratch,
                                    RenderKind::Master,
                                    &file,
                                )
                                .map_err(|e| format!("{tag}: {e}"))?;
                                let bytes = std::fs::read(&file).map_err(|e| e.to_string())?;
                                let _ = std::fs::remove_file(&file);
                                let digest = format!("{:x}", Sha256::digest(&bytes));
                                let line = job
                                    .measurements
                                    .and_then(|m| m.auto_gain_staging_digest)
                                    .unwrap_or_default();
                                rows.push(serde_json::json!({"song": name, "style": style,
                                    "target": target, "mode": mode, "sha256": digest,
                                    "receipt_line": line}));
                            }
                        }
                    }
                    println!("hashed {name}");
                    Ok(rows)
                })
            })
            .collect();
        handles
            .into_iter()
            .map(|h| h.join().map_err(|_| "a song panicked".to_string())?)
            .collect::<Result<Vec<_>, String>>()
    })?;
    let rows: Vec<_> = results.into_iter().flatten().collect();
    let text = serde_json::to_string_pretty(&rows).map_err(|e| e.to_string())?;
    std::fs::write(out, text).map_err(|e| e.to_string())
}

/// Checkpoint C: the owner's listening pack. Universal at default Intensity,
/// Medium (−11) and High (−9), the switch off (A, today) and on at Clean (B),
/// through the production export, kept in `out` with a plain READ ME. These
/// are private renders of the owner's songs: never committed.
pub(super) fn listening_pack(out: &Path, songs: &str) -> Result<(), String> {
    std::fs::create_dir_all(out).map_err(|e| e.to_string())?;
    let mut lines = vec![
        "Auto gain staging: listening pack".to_string(),
        String::new(),
        "Universal at default Intensity (50%), Standard's export (WAV, 24-bit,".to_string(),
        "44.1 kHz, ceiling -1.0 dBTP), through the app's own export.".to_string(),
        "A = today (switch off). B = auto gain staging on, priority Clean.".to_string(),
        "PLR = peak-to-loudness ratio (true peak minus loudness): higher means".to_string(),
        "more punch kept; lower means more limited.".to_string(),
        String::new(),
    ];
    let mut unfair = Vec::new();
    for entry in songs.split('|').filter(|e| !e.trim().is_empty()) {
        let (name, path) = entry
            .split_once('=')
            .ok_or_else(|| format!("{entry}: expected \"Name=<path>\""))?;
        let song = Song {
            label: name.to_string(),
            group: "owner".into(),
            kind: String::new(),
            anon: name.to_string(),
            reference: false,
            path: path.into(),
        };
        let src = load(&song, None)?;
        for (level, target) in [("High", -9.0_f32), ("Medium", -11.0)] {
            let a = standard_export_settings(
                yes_master_lib::types::Preset::Universal,
                target,
                src.profile,
                Some(src.lufs),
            );
            let mut b = a.clone();
            b.advanced.auto_gain_staging = Some(AutoGainStagingSettings {
                enabled: true,
                priority: LoudnessPriority::Clean,
            });
            let mut facts = Vec::new();
            for (tag, settings) in [("A today", &a), ("B gain staging", &b)] {
                let file = out.join(format!("{name} - {level} - {tag}.wav"));
                let _ = std::fs::remove_file(&file);
                let job = yes_master_lib::engine::mastering_render_to_path(
                    TrackId(format!("pack-{name}-{level}-{tag}")),
                    &song.path,
                    settings,
                    out,
                    RenderKind::Master,
                    &file,
                )
                .map_err(|e| format!("{name} {level} {tag}: export: {e}"))?;
                let m = job
                    .measurements
                    .ok_or_else(|| format!("{name} {level} {tag}: no measurements"))?;
                println!(
                    "{} {level} {tag}: {:.2} LUFS, {:.2} dBTP",
                    name, m.lufs_integrated, m.true_peak_dbtp
                );
                facts.push(m);
            }
            let (fa, fb) = (&facts[0], &facts[1]);
            let plr = |m: &RenderedMeasurements| m.true_peak_dbtp - m.lufs_integrated;
            lines.push(format!("{name} - {level} (target {target:.0} LUFS)"));
            lines.push(format!(
                "  A today:          {:.1} LUFS, true peak {:.1} dBTP, PLR {:.1} dB",
                fa.lufs_integrated,
                fa.true_peak_dbtp,
                plr(fa)
            ));
            lines.push(format!(
                "  B gain staging:   {:.1} LUFS, true peak {:.1} dBTP, PLR {:.1} dB",
                fb.lufs_integrated,
                fb.true_peak_dbtp,
                plr(fb)
            ));
            lines.push(format!(
                "  B's line: {}",
                fb.auto_gain_staging_digest.clone().unwrap_or_default()
            ));
            let gap = fb.lufs_integrated - fa.lufs_integrated;
            if gap.abs() > 0.2 {
                let louder = if gap > 0.0 { "B" } else { "A" };
                lines.push(format!(
                    "  NOT A FAIR COMPARISON: {louder} is {:.1} dB louder. Turn {louder} down {:.1} dB, or match levels, before judging.",
                    gap.abs(),
                    gap.abs()
                ));
                unfair.push(format!("{name} - {level}"));
            } else {
                lines.push("  Same loudness (within 0.2 dB): a fair comparison.".into());
            }
            lines.push(String::new());
        }
    }
    lines.push(if unfair.is_empty() {
        "Every pair lands at the same loudness, so each A/B is fair.".into()
    } else {
        format!(
            "Pairs that do not land at the same loudness: {}.",
            unfair.join("; ")
        )
    });
    lines.push(String::new());
    lines.push("Private renders of your own songs: kept here only, never committed.".to_string());
    // Windows line endings, so Notepad shows the lines on any version.
    std::fs::write(out.join("READ ME.txt"), lines.join("\r\n")).map_err(|e| e.to_string())
}

/// Checkpoint B follow-up: the real export with Reach target, single thread,
/// against the switch off, with the export's progress trace (timestamped) so
/// the report can show progress keeps moving across resolution passes.
pub(super) fn timing_reach_production(
    songs: &[Song],
    scratch: &Path,
    output: &Path,
) -> Result<(), String> {
    let mut rows = Vec::new();
    for song in songs {
        let src = load(song, None)?;
        let seconds_of_audio = (src.samples.len() / src.channels) as f64 / f64::from(src.rate);
        for target in STANDARD_TARGETS {
            let off = standard_export_settings(
                yes_master_lib::types::Preset::Universal,
                target,
                src.profile,
                Some(src.lufs),
            );
            let mut reach = off.clone();
            reach.advanced.auto_gain_staging = Some(AutoGainStagingSettings {
                enabled: true,
                priority: LoudnessPriority::ReachTarget,
            });
            let mut off_s = Vec::new();
            let mut reach_s = Vec::new();
            let mut digest = String::new();
            let mut trace: Vec<(f64, f32)> = Vec::new();
            for run in 0..3 {
                let t0 = Instant::now();
                export_master(
                    &song.path,
                    &off,
                    scratch,
                    &format!("{}-off-{run}", song.anon),
                )?;
                off_s.push(t0.elapsed().as_secs_f64());
                let samples = std::cell::RefCell::new(Vec::new());
                let t0 = Instant::now();
                let name = format!("{}-reach-{run}", song.anon);
                let out = scratch.join(format!("{name}.wav"));
                let _ = std::fs::remove_file(&out);
                let progress = |f: f32| samples.borrow_mut().push((t0.elapsed().as_secs_f64(), f));
                let job = yes_master_lib::engine::mastering_render_with_progress(
                    TrackId(name.clone()),
                    &song.path,
                    &reach,
                    scratch,
                    RenderKind::Master,
                    Some(&progress),
                    Some(&out),
                )
                .map_err(|e| format!("{name}: export: {e}"))?;
                reach_s.push(t0.elapsed().as_secs_f64());
                for path in &job.output_paths {
                    let _ = std::fs::remove_file(path);
                }
                digest = job
                    .measurements
                    .and_then(|m| m.auto_gain_staging_digest)
                    .unwrap_or_default();
                if run == 0 {
                    trace = samples.into_inner();
                }
            }
            let total = reach_s[0];
            let monotonic = trace.windows(2).all(|w| w[1].1 >= w[0].1);
            // Between the first and the last update: the resolution passes.
            let max_gap = trace
                .windows(2)
                .map(|w| w[1].0 - w[0].0)
                .fold(0.0_f64, f64::max);
            // Before the first update (decode) and after the last (the write).
            let before_first = trace.first().map_or(total, |f| f.0);
            let after_last = trace.last().map_or(total, |l| total - l.0);
            let med = |v: &[f64]| median(v).unwrap_or(f64::NAN);
            println!(
                "{} {seconds_of_audio:.0}s target {target}: off {:.2}s, reach {:.2}s, progress updates {}, longest gap between updates {max_gap:.2}s, before first {before_first:.2}s, after last {after_last:.2}s ({digest})",
                song.label,
                med(&off_s),
                med(&reach_s),
                trace.len()
            );
            rows.push(serde_json::json!({
                "song": song.label, "seconds_of_audio": seconds_of_audio, "target": target,
                "export_off_s": med(&off_s), "export_reach_target_s": med(&reach_s), "runs": 3,
                "receipt_line": digest, "progress_updates": trace.len(),
                "progress_monotonic": monotonic,
                "progress_last": trace.last().map(|l| l.1),
                "longest_gap_between_updates_s": max_gap,
                "seconds_before_first_update": before_first,
                "seconds_after_last_update": after_last,
                "trace_s_fraction": trace,
            }));
        }
    }
    let text = serde_json::to_string_pretty(&rows).map_err(|e| e.to_string())?;
    std::fs::write(output.join("timing-reach-production.json"), text).map_err(|e| e.to_string())
}

/// Checkpoint B, E4 re-check: Reach target, through the export's own
/// resolver (`gain_staging::render_export`, pinned bit-identical to the
/// export), on every case where Clean stopped at the clean limit in a prior
/// production ledger. Records the passes and why the search stopped.
pub(super) fn reach_check_song(
    song: &Song,
    cases: &[(String, f32)],
) -> Result<Vec<serde_json::Value>, String> {
    let src = load(song, None)?;
    let mut rows = Vec::new();
    for (style, target) in cases {
        let preset = standard_styles()
            .into_iter()
            .find(|(name, _)| name == style)
            .map(|(_, p)| p)
            .ok_or_else(|| format!("unknown style {style}"))?;
        let mut s = standard_export_settings(preset, *target, src.profile, Some(src.lufs));
        s.advanced.auto_gain_staging = Some(AutoGainStagingSettings {
            enabled: true,
            priority: LoudnessPriority::ReachTarget,
        });
        let t0 = Instant::now();
        let outcome = yes_master_lib::gain_staging::render_export(
            &src.samples,
            src.rate,
            src.channels as u16,
            &mut s,
            STANDARD_RATE,
            STANDARD_BITS,
            None,
            &|_| {},
        )
        .map_err(|e| format!("{}: {e}", song.label))?
        .ok_or_else(|| format!("{}: inert", song.label))?;
        let r = s.advanced.gain_staging_resolved.unwrap_or_default();
        rows.push(serde_json::json!({
            "song": song.label, "style": style, "target": target,
            "lufs": outcome.measured.lufs, "true_peak_dbtp": outcome.measured.true_peak_dbtp,
            "passes": outcome.passes, "stop": format!("{:?}", outcome.stop),
            "level_in_db": r.level_in_db, "drive_db": r.limiter_drive_db,
            "limiter_load_db": r.limiter_load_db,
            "stopped_by_peak_check": r.stopped_by_peak_check,
            "receipt_line": yes_master_lib::gain_staging::receipt_digest(&s),
            "seconds": t0.elapsed().as_secs_f64(),
        }));
    }
    Ok(rows)
}

pub(super) fn process_song_production(song: &Song, scratch: &Path) -> SongResult {
    let start = Instant::now();
    let mut errors = Vec::new();
    let src = match load(song, None) {
        Ok(s) => s,
        Err(e) => return SongResult::failed(song, e, start),
    };
    let ch = src.channels;
    let x48 = to_analysis_rate(&src.samples, src.rate, ch);
    let anchors = source_anchors(&x48, ch);
    let source_shape = shape(&x48, ch, &anchors);
    drop(x48);
    // Level in exactly as the export computes it, from the same function on
    // the same decoded samples.
    let level_in =
        yes_master_lib::analysis::measure_source_level(&src.samples, src.rate, ch as u16)
            .ok()
            .and_then(|l| yes_master_lib::gain_staging::level_in_for(&l));
    let smart_match_ok = smart_match_unchanged(&src);
    let mut t_identical = true;
    let mut rows: Vec<RenderRow> = Vec::new();
    let tag = song.anon.clone();

    for (style, preset) in standard_styles() {
        let settings: Vec<MasteringSettings> = STANDARD_TARGETS
            .iter()
            .map(|t| standard_export_settings(preset.clone(), *t, src.profile, Some(src.lufs)))
            .collect();
        // T: the research render (for its landing amount) and the production
        // export, which must match it bit for bit.
        let t0 = Instant::now();
        let stage_t = stage_pass(&src.samples, src.rate, ch, &settings[0], 0.0);
        let limited_t = limiter_pass(&stage_t, STANDARD_CEILING_DBTP, 0.0);
        drop(stage_t);
        let t_chain_s = t0.elapsed().as_secs_f64();
        for (ti, target) in STANDARD_TARGETS.iter().enumerate() {
            let name = format!("{tag}-{style}-{}-T", -target);
            let research = deliver(limited_t.samples.clone(), src.rate, ch, *target);
            let production = export_master(&song.path, &settings[ti], scratch, &name);
            match (research, production) {
                (Ok((delivered, facts)), Ok((exported, m))) => {
                    let same = same_delivered_pcm(&delivered, &exported, ch)
                        && m.lufs_integrated == facts.lufs
                        && m.true_peak_dbtp == facts.true_peak_dbtp;
                    if !same {
                        errors.push(format!("{name}: production T differs from research T"));
                    }
                    t_identical &= same;
                    let mm = measure_delivered(delivered, facts, &src, &source_shape, &anchors);
                    rows.push(row(
                        style,
                        *target,
                        "T",
                        mm,
                        None,
                        None,
                        None,
                        limited_t.max_gain_reduction_db,
                        1,
                        false,
                        true,
                        false,
                        t_chain_s,
                        Vec::new(),
                    ));
                }
                (r, p) => {
                    t_identical = false;
                    errors.push(format!(
                        "{name}: {:?} / {:?}",
                        r.err(),
                        p.err().unwrap_or_default()
                    ));
                }
            }
        }

        // G through the production export, priority Clean; Reach target too
        // wherever Clean stopped at the clean limit (E4).
        for (ti, target) in STANDARD_TARGETS.iter().enumerate() {
            let mut s = settings[ti].clone();
            s.advanced.auto_gain_staging = Some(AutoGainStagingSettings {
                enabled: true,
                priority: LoudnessPriority::Clean,
            });
            let name = format!("{tag}-{style}-{}-G", -target);
            let t1 = Instant::now();
            let stopped = match export_master(&song.path, &s, scratch, &name) {
                Ok((exported, m)) => {
                    let digest = m.auto_gain_staging_digest.clone().unwrap_or_default();
                    let parsed = parse_receipt_digest(&digest);
                    let (drive, load, stopped) = parsed
                        .map(|(_, d, l, st)| (Some(d), l, st))
                        .unwrap_or((None, 0.0, false));
                    if parsed.is_none() && level_in.is_some() {
                        errors.push(format!("{name}: receipt line {digest:?}"));
                    }
                    let facts = facts_from(&m, &exported);
                    let within = (facts.lufs - target).abs() <= TARGET_TOLERANCE_LU;
                    let mm = measure_delivered(exported, facts, &src, &source_shape, &anchors);
                    rows.push(row(
                        style,
                        *target,
                        "G",
                        mm,
                        level_in,
                        drive,
                        None,
                        load,
                        0,
                        stopped,
                        within,
                        parsed.is_none(),
                        t1.elapsed().as_secs_f64(),
                        Vec::new(),
                    ));
                    stopped
                }
                Err(e) => {
                    errors.push(e);
                    false
                }
            };
            if stopped {
                s.advanced.auto_gain_staging = Some(AutoGainStagingSettings {
                    enabled: true,
                    priority: LoudnessPriority::ReachTarget,
                });
                let name = format!("{tag}-{style}-{}-R", -target);
                let t1 = Instant::now();
                match export_master(&song.path, &s, scratch, &name) {
                    Ok((exported, m)) => {
                        let digest = m.auto_gain_staging_digest.clone().unwrap_or_default();
                        let parsed = parse_receipt_digest(&digest);
                        let facts = facts_from(&m, &exported);
                        let within = (facts.lufs - target).abs() <= TARGET_TOLERANCE_LU;
                        let mm = measure_delivered(exported, facts, &src, &source_shape, &anchors);
                        rows.push(row(
                            style,
                            *target,
                            "G-reach",
                            mm,
                            level_in,
                            parsed.map(|p| p.1),
                            None,
                            parsed.map_or(0.0, |p| p.2),
                            0,
                            false,
                            within,
                            parsed.is_none(),
                            t1.elapsed().as_secs_f64(),
                            Vec::new(),
                        ));
                    }
                    Err(e) => errors.push(e),
                }
            }
        }
    }
    let mut result = SongResult::from_loaded(song, &src, level_in, rows, errors, start);
    result.production_t_identical = Some(t_identical);
    result.smart_match_unchanged = Some(smart_match_ok);
    result
}
