//! Auto gain staging research runner (local-only evidence runner; auto gain
//! staging plan `docs/plans/2026-11-24-auto-gain-staging-execution-plan.md`,
//! Phase A, Slices 0–1). It changes no product behavior.
//!
//! For every development song, each of the four Standard Styles at default
//! Intensity, and each Standard loudness (−14, −11, −9 LUFS), it renders and
//! measures T (today), S (turn-down first) and G (two gains, priority Clean),
//! then applies the plan's decision rule R1–R7 exactly as written.
//!
//! Development material only. The free library's 14 held-out songs are
//! identified by content hash from the committed attribution list and never
//! decoded.
//!
//! Usage (from `src-tauri`; pass the main checkout's private folders by
//! absolute path):
//!   cargo run --release --example auto_gain_staging_research -- \
//!     --library <free-cc folder> --library-sources <library-sources.json> \
//!     --corpus-json <corpus.json> --corpus-root <mastering-quality-20260915 folder> \
//!     --owner-dir <"tests for presets" folder> --owner-extra <private-audio-fixtures folder> \
//!     --suno <suno-set folder> --unmastered <unmastered-mixes folder> \
//!     --output <private dir> --evidence <committable dir> [--workers 8]
//!
//! G resolves its drive on the delivered loudness (contract rule 3) unless
//! `--g-resolve chain-output` is given, which reproduces the first
//! development run (the chain's output loudness before delivery).
//!
//! Optional: `--only <a|b|…>` keeps songs whose label contains any of the
//! texts (smoke runs; the decision needs the full set), `--excerpt-seconds
//! <s>` renders the first seconds only (smoke runs), `--timing 1` runs the
//! controlled single-thread timing on the `--only` songs instead of the
//! measurement, after checking their full-length split render bit for bit.

mod research;
#[allow(dead_code)]
#[path = "../../src/sample_rate.rs"]
mod sample_rate;

use research::*;
use serde::Serialize;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::time::Instant;
use yes_master_lib::engine::{analyze_tracks_core_with_progress_sync, AnalyzeRequest};
use yes_master_lib::smart_balance_reference::{
    anon_id_for_bytes, seeded_split, SplitSide, DEFAULT_SEED, HOLDOUT_FRACTION,
};
use yes_master_lib::smart_match_evaluation::{files_under, parallel_map};
use yes_master_lib::{types, SourceProfile, TrackId};

const REFERENCE_FILE: &str = "It\u{2019}s a coat-original-test.wav";
const AUDIO_EXTENSIONS: [&str; 6] = ["wav", "mp3", "flac", "aif", "aiff", "ogg"];

struct Args {
    library: PathBuf,
    library_sources: PathBuf,
    corpus_json: PathBuf,
    corpus_root: PathBuf,
    owner_dir: PathBuf,
    owner_extra: PathBuf,
    suno: PathBuf,
    unmastered: PathBuf,
    output: PathBuf,
    evidence: PathBuf,
    workers: usize,
    only: Option<String>,
    excerpt_seconds: Option<f64>,
    timing: bool,
    /// G resolves on the delivered loudness (contract rule 3; the default)
    /// or, with `--g-resolve chain-output`, on the chain's output before
    /// delivery (the first development run).
    g_on_delivered: bool,
}

fn parse_args() -> Result<Args, String> {
    let mut map: BTreeMap<String, String> = BTreeMap::new();
    let mut it = std::env::args().skip(1);
    while let Some(a) = it.next() {
        let Some(key) = a.strip_prefix("--") else {
            return Err(format!("unexpected argument {a}"));
        };
        let v = it.next().ok_or_else(|| format!("--{key} needs a value"))?;
        map.insert(key.to_string(), v);
    }
    let get = |k: &str| -> Result<PathBuf, String> {
        map.get(k)
            .map(PathBuf::from)
            .ok_or_else(|| format!("--{k} is required"))
    };
    Ok(Args {
        library: get("library")?,
        library_sources: get("library-sources")?,
        corpus_json: get("corpus-json")?,
        corpus_root: get("corpus-root")?,
        owner_dir: get("owner-dir")?,
        owner_extra: get("owner-extra")?,
        suno: get("suno")?,
        unmastered: get("unmastered")?,
        output: get("output")?,
        evidence: get("evidence")?,
        workers: map.get("workers").and_then(|w| w.parse().ok()).unwrap_or(8),
        only: map.get("only").cloned(),
        excerpt_seconds: map.get("excerpt-seconds").and_then(|s| s.parse().ok()),
        timing: map.get("timing").is_some_and(|v| v == "1"),
        g_on_delivered: match map.get("g-resolve").map(String::as_str) {
            None | Some("delivered") => true,
            Some("chain-output") => false,
            Some(other) => {
                return Err(format!(
                    "--g-resolve {other}: use delivered or chain-output"
                ))
            }
        },
    })
}

fn read_json(path: &Path) -> Result<serde_json::Value, String> {
    let text =
        std::fs::read_to_string(path).map_err(|e| format!("read {}: {e}", path.display()))?;
    serde_json::from_str(&text).map_err(|e| format!("parse {}: {e}", path.display()))
}

fn anon_of(path: &Path) -> Result<String, String> {
    let bytes = std::fs::read(path).map_err(|e| format!("read {}: {e}", path.display()))?;
    Ok(anon_id_for_bytes(&bytes))
}

fn is_audio(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| AUDIO_EXTENSIONS.contains(&e.to_ascii_lowercase().as_str()))
}

// ---------------------------------------------------------------------------
// Songs
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize)]
struct Song {
    /// What the committable summary shows: library/corpus titles, owner IDs.
    label: String,
    group: String,
    /// A plain description of an owner file (already public in the repo's
    /// docs); empty for library and corpus songs.
    kind: String,
    anon: String,
    reference: bool,
    #[serde(skip_serializing)]
    path: PathBuf,
}

/// The owner's own description of the files in `tests for presets`
/// (`docs/reviews/2026-09-14-landr-waves-dsp-assessment.md`, "Scope and
/// evidence"; Smart Balance plan deviation 3).
fn owner_kind(name: &str) -> &'static str {
    if name == REFERENCE_FILE {
        "reference song (unmastered source)"
    } else if name.starts_with("LANDR-") {
        "LANDR master of the reference song"
    } else if name.starts_with("It\u{2019}s a coat-") {
        "BandLab master of the reference song"
    } else if name.contains("YES") {
        "YES Master export of the reference song"
    } else {
        "owner song"
    }
}

struct Discovery {
    songs: Vec<Song>,
    library_build: usize,
    library_holdout_skipped: usize,
    suno_files: usize,
    suno_holdout: usize,
    unmastered_files: usize,
    unmastered_holdout: usize,
}

fn discover(args: &Args) -> Result<Discovery, String> {
    let mut songs: Vec<Song> = Vec::new();
    let mut seen: BTreeMap<String, String> = BTreeMap::new();
    let mut add = |songs: &mut Vec<Song>, song: Song| {
        if let Some(prior) = seen.get(&song.anon) {
            println!("skip duplicate {} (same bytes as {prior})", song.label);
            return;
        }
        seen.insert(song.anon.clone(), song.label.clone());
        songs.push(song);
    };

    // Free library: build split only. Held-out songs are hashed to identify
    // them and never decoded.
    let sources = read_json(&args.library_sources)?;
    let mut by_anon: BTreeMap<String, &serde_json::Value> = BTreeMap::new();
    for t in sources["tracks"].as_array().ok_or("no tracks")? {
        if let Some(id) = t["anon_id"].as_str() {
            by_anon.insert(id.to_string(), t);
        }
    }
    let (mut library_build, mut library_holdout_skipped) = (0, 0);
    let mut library = Vec::new();
    for path in files_under(&args.library).map_err(|e| e.to_string())? {
        let anon = anon_of(&path)?;
        let Some(t) = by_anon.get(&anon) else {
            continue;
        };
        match t["split"].as_str() {
            Some("build") => library_build += 1,
            Some("holdout") => {
                library_holdout_skipped += 1;
                continue;
            }
            _ => continue,
        }
        library.push(Song {
            label: format!(
                "{} \u{2013} {}",
                t["artist"].as_str().unwrap_or("?"),
                t["title"].as_str().unwrap_or("?")
            ),
            group: "library-build".into(),
            kind: String::new(),
            anon,
            reference: false,
            path,
        });
    }
    library.sort_by(|a, b| a.label.cmp(&b.label));

    // September corpus: all eight pieces are development (its holdout was
    // used up). "coat" is the owner's reference song.
    let reference_path = args.owner_dir.join(REFERENCE_FILE);
    let reference_anon = anon_of(&reference_path)?;
    let corpus = read_json(&args.corpus_json)?;
    let mut corpus_songs = Vec::new();
    for s in corpus["sources"].as_array().ok_or("no sources")? {
        let id = s["id"].as_str().unwrap_or_default();
        if id == "coat" {
            continue; // added with the owner's songs, as the reference
        }
        let path = args.corpus_root.join("sources").join(format!("{id}.wav"));
        corpus_songs.push(Song {
            label: s["title"].as_str().unwrap_or(id).to_string(),
            group: "corpus".into(),
            kind: s["kind"].as_str().unwrap_or_default().to_string(),
            anon: anon_of(&path)?,
            reference: false,
            path,
        });
    }

    // Owner's songs: `tests for presets` and the top-level fixtures.
    let mut owner = Vec::new();
    for dir in [&args.owner_dir, &args.owner_extra] {
        let mut entries: Vec<PathBuf> = std::fs::read_dir(dir)
            .map_err(|e| format!("read {}: {e}", dir.display()))?
            .flatten()
            .map(|e| e.path())
            .filter(|p| p.is_file() && is_audio(p))
            .collect();
        entries.sort();
        for path in entries {
            let anon = anon_of(&path)?;
            let name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or_default()
                .to_string();
            owner.push(Song {
                label: format!("owner {anon}"),
                group: "owner".into(),
                kind: owner_kind(&name).into(),
                reference: anon == reference_anon,
                anon,
                path,
            });
        }
    }

    // The owner's Suno songs and unmastered mixes: 75/25 by content hash.
    let mut split_folder = |dir: &Path, group: &str| -> Result<(usize, usize), String> {
        let files: Vec<PathBuf> = if dir.exists() {
            files_under(dir)
                .map_err(|e| e.to_string())?
                .into_iter()
                .filter(|p| is_audio(p))
                .collect()
        } else {
            Vec::new()
        };
        let anons: Vec<String> = files.iter().map(|p| anon_of(p)).collect::<Result<_, _>>()?;
        let hashes: Vec<u64> = anons
            .iter()
            .map(|a| u64::from_str_radix(a, 16).unwrap_or(0))
            .collect();
        let sides = seeded_split(&hashes, DEFAULT_SEED, HOLDOUT_FRACTION);
        let mut holdout = 0;
        for ((path, anon), side) in files.iter().zip(anons).zip(sides) {
            if side == SplitSide::Holdout {
                holdout += 1;
                continue;
            }
            owner.push(Song {
                label: format!("owner {anon}"),
                group: group.into(),
                kind: "owner song".into(),
                reference: false,
                anon,
                path: path.clone(),
            });
        }
        Ok((files.len(), holdout))
    };
    let (suno_files, suno_holdout) = split_folder(&args.suno, "owner-suno")?;
    let (unmastered_files, unmastered_holdout) =
        split_folder(&args.unmastered, "owner-unmastered")?;

    for s in owner.into_iter().chain(corpus_songs).chain(library) {
        add(&mut songs, s);
    }
    Ok(Discovery {
        songs,
        library_build,
        library_holdout_skipped,
        suno_files,
        suno_holdout,
        unmastered_files,
        unmastered_holdout,
    })
}

// ---------------------------------------------------------------------------
// Per-song work
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize)]
struct RenderRow {
    style: String,
    target: f32,
    design: &'static str,
    lufs: f32,
    true_peak_dbtp: f32,
    plr_db: f32,
    lra: f32,
    landing_db: f32,
    level_in_db: Option<f32>,
    drive_db: Option<f32>,
    turn_down_db: Option<f32>,
    /// The limiter's largest gain reduction in the delivered render, dB.
    limiter_load_db: f32,
    /// Whole-track passes to resolve: T 1; S full passes; G limiter passes
    /// after its one stage pass.
    passes: usize,
    stopped_at_clean_limit: bool,
    resolved: bool,
    inert: bool,
    all_finite: bool,
    under_ceiling: bool,
    change: ShapeChange,
    fingerprint: Fingerprint,
    seconds: f64,
    trace: Vec<(f32, f32)>,
}

#[derive(Debug, Clone, Serialize)]
struct SongResult {
    song: Song,
    rate: u32,
    channels: usize,
    seconds_of_audio: f64,
    source_lufs: f32,
    source_true_peak_dbtp: f32,
    source_lra: f32,
    level_in_db: Option<f32>,
    decomposition_bit_identical: bool,
    coefficients_target_independent: bool,
    transient_anchors: usize,
    section_pair: Option<(usize, usize)>,
    rows: Vec<RenderRow>,
    errors: Vec<String>,
    wall_seconds: f64,
}

struct Loaded {
    samples: Vec<f32>,
    rate: u32,
    channels: usize,
    lufs: f32,
    true_peak: f32,
    lra: f32,
    profile: Option<SourceProfile>,
}

fn load(song: &Song, excerpt: Option<f64>) -> Result<Loaded, String> {
    let analysis = analyze_tracks_core_with_progress_sync(
        vec![AnalyzeRequest {
            id: TrackId(song.anon.clone()),
            path: song.path.to_string_lossy().into_owned(),
        }],
        |_, _| {},
    )
    .map_err(|e| format!("{}: analysis: {e}", song.label))?
    .pop()
    .ok_or_else(|| format!("{}: no analysis", song.label))?;
    let pcm = yes_master_lib::decode::decode_full(&song.path)
        .map_err(|e| format!("{}: decode: {e}", song.label))?;
    let channels = usize::from(pcm.channels.max(1));
    let mut samples = pcm.samples;
    if let Some(s) = excerpt {
        let keep = ((s * f64::from(pcm.sample_rate)) as usize) * channels;
        samples.truncate(keep.min(samples.len()));
    }
    Ok(Loaded {
        samples,
        rate: pcm.sample_rate,
        channels,
        lufs: analysis.lufs_integrated,
        true_peak: analysis.true_peak_dbtp,
        lra: analysis.dynamic_range_lu,
        profile: SourceProfile::from_analysis(&analysis),
    })
}

struct Measured {
    facts: DeliveryFacts,
    change: ShapeChange,
    fingerprint: Fingerprint,
}

fn measure_delivery(
    render: Vec<f32>,
    src: &Loaded,
    target: f32,
    source_shape: &Shape,
    anchors: &Anchors,
) -> Result<Measured, String> {
    let (delivered, facts) = deliver(render, src.rate, src.channels, target)?;
    Ok(measure_delivered(
        delivered,
        facts,
        src,
        source_shape,
        anchors,
    ))
}

fn measure_delivered(
    delivered: Vec<f32>,
    facts: DeliveryFacts,
    src: &Loaded,
    source_shape: &Shape,
    anchors: &Anchors,
) -> Measured {
    let sample_peak_db = f64::from(lin_to_db(
        delivered.iter().map(|s| s.abs()).fold(0.0_f32, f32::max),
    ));
    let x48 = to_analysis_rate(&delivered, STANDARD_RATE, src.channels);
    drop(delivered);
    let render_shape = shape(&x48, src.channels, anchors);
    Measured {
        change: compare(source_shape, anchors, &render_shape),
        fingerprint: fingerprint(
            source_shape,
            f64::from(src.lufs),
            &render_shape,
            f64::from(facts.lufs),
            sample_peak_db,
        ),
        facts,
    }
}

#[allow(clippy::too_many_arguments)]
fn row(
    style: &str,
    target: f32,
    design: &'static str,
    m: Measured,
    level_in_db: Option<f32>,
    drive_db: Option<f32>,
    turn_down_db: Option<f32>,
    limiter_load_db: f32,
    passes: usize,
    stopped: bool,
    resolved: bool,
    inert: bool,
    seconds: f64,
    trace: Vec<(f32, f32)>,
) -> RenderRow {
    RenderRow {
        style: style.into(),
        target,
        design,
        lufs: m.facts.lufs,
        true_peak_dbtp: m.facts.true_peak_dbtp,
        plr_db: m.facts.true_peak_dbtp - m.facts.lufs,
        lra: m.facts.lra,
        landing_db: m.facts.landing_db,
        level_in_db,
        drive_db,
        turn_down_db,
        limiter_load_db,
        passes,
        stopped_at_clean_limit: stopped,
        resolved,
        inert,
        all_finite: m.facts.all_finite,
        under_ceiling: m.facts.under_ceiling,
        change: m.change,
        fingerprint: m.fingerprint,
        seconds,
        trace,
    }
}

fn process_song(
    song: &Song,
    nominal: Nominal,
    excerpt: Option<f64>,
    g_on_delivered: bool,
) -> SongResult {
    let start = Instant::now();
    let mut errors = Vec::new();
    let src = match load(song, excerpt) {
        Ok(s) => s,
        Err(e) => {
            return SongResult {
                song: song.clone(),
                rate: 0,
                channels: 0,
                seconds_of_audio: 0.0,
                source_lufs: f32::NAN,
                source_true_peak_dbtp: f32::NAN,
                source_lra: f32::NAN,
                level_in_db: None,
                decomposition_bit_identical: false,
                coefficients_target_independent: false,
                transient_anchors: 0,
                section_pair: None,
                rows: Vec::new(),
                errors: vec![e],
                wall_seconds: start.elapsed().as_secs_f64(),
            }
        }
    };
    let ch = src.channels;
    let x48 = to_analysis_rate(&src.samples, src.rate, ch);
    let anchors = source_anchors(&x48, ch);
    let source_shape = shape(&x48, ch, &anchors);
    drop(x48);
    let level_in = level_in_db(nominal, src.lufs, src.true_peak);
    let mut rows = Vec::new();
    let mut decomposition_ok = true;
    let mut coeffs_ok = true;
    let excerpt_frames = (20 * src.rate as usize).min(src.samples.len() / ch);

    for (style, preset) in standard_styles() {
        let settings: Vec<_> = STANDARD_TARGETS
            .iter()
            .map(|t| standard_export_settings(preset.clone(), *t, src.profile, Some(src.lufs)))
            .collect();
        // The Style stages do not depend on the loudness target.
        let coeffs: Vec<_> = settings
            .iter()
            .map(|s| yes_master_lib::dsp::ChainCoeffs::from_settings(src.rate, s))
            .collect();
        coeffs_ok &= coeffs.windows(2).all(|w| w[0] == w[1]);
        // The split render is bit-identical to the production chain.
        decomposition_ok &= decomposition_matches_chain(
            &src.samples[..excerpt_frames * ch],
            src.rate,
            ch,
            &settings[0],
        );

        // ---- T: today -------------------------------------------------
        let t0 = Instant::now();
        let stage_t = stage_pass(&src.samples, src.rate, ch, &settings[0], 0.0);
        let limited_t = limiter_pass(&stage_t, STANDARD_CEILING_DBTP, 0.0);
        let t_chain_s = t0.elapsed().as_secs_f64();
        let t_lufs = loudness(&limited_t.samples, src.rate, ch);
        let mut t_rows = Vec::new();
        for target in STANDARD_TARGETS.iter() {
            let t1 = Instant::now();
            match measure_delivery(
                limited_t.samples.clone(),
                &src,
                *target,
                &source_shape,
                &anchors,
            ) {
                Ok(m) => {
                    let r = row(
                        style,
                        *target,
                        "T",
                        m,
                        None,
                        None,
                        None,
                        limited_t.max_gain_reduction_db,
                        1,
                        false,
                        true,
                        false,
                        t_chain_s + t1.elapsed().as_secs_f64(),
                        vec![(0.0, t_lufs)],
                    );
                    t_rows.push(Some(r.clone()));
                    rows.push(r);
                }
                Err(e) => {
                    errors.push(format!("{style} {target} T: {e}"));
                    t_rows.push(None);
                }
            }
        }

        // ---- S: turn-down first ----------------------------------------
        for (ti, target) in STANDARD_TARGETS.iter().enumerate() {
            let t1 = Instant::now();
            let (res, render) = resolve_turn_down(
                &src.samples,
                src.rate,
                ch,
                &settings[ti],
                t_lufs,
                *target,
                TARGET_TOLERANCE_LU,
                MAX_PASSES,
            );
            match render {
                None => {
                    // No overshoot to move: S renders exactly as T.
                    if let Some(Some(t)) = t_rows.get(ti) {
                        let mut s = t.clone();
                        s.design = "S";
                        s.turn_down_db = Some(0.0);
                        s.passes = res.passes;
                        s.resolved = res.resolved;
                        s.trace = res.trace.clone();
                        rows.push(s);
                    }
                }
                Some(limited) => {
                    let load = limited.max_gain_reduction_db;
                    match measure_delivery(limited.samples, &src, *target, &source_shape, &anchors)
                    {
                        Ok(m) => rows.push(row(
                            style,
                            *target,
                            "S",
                            m,
                            None,
                            None,
                            Some(res.pre_gain_db),
                            load,
                            res.passes,
                            false,
                            res.resolved,
                            false,
                            t1.elapsed().as_secs_f64(),
                            res.trace,
                        )),
                        Err(e) => errors.push(format!("{style} {target} S: {e}")),
                    }
                }
            }
        }

        // ---- G: two gains, priority Clean ------------------------------
        let t0 = Instant::now();
        let stage_g = match level_in {
            Some(li) if li != 0.0 => Some(stage_pass(&src.samples, src.rate, ch, &settings[0], li)),
            _ => None,
        };
        let g_stage_s = t0.elapsed().as_secs_f64();
        let stage_for_g = stage_g.as_ref().unwrap_or(&stage_t);
        for target in STANDARD_TARGETS {
            let t1 = Instant::now();
            if level_in.is_none() {
                // Inert (A7): G renders exactly as T.
                if let Some(t) = rows
                    .iter()
                    .find(|r| r.design == "T" && r.style == style && r.target == target)
                {
                    let mut g = t.clone();
                    g.design = "G";
                    g.inert = true;
                    rows.push(g);
                }
                continue;
            }
            let resolved = if g_on_delivered {
                resolve_drive_delivered(
                    stage_for_g,
                    target,
                    Some(CLEAN_LIMIT_DB),
                    TARGET_TOLERANCE_LU,
                    MAX_PASSES,
                )
                .map(|(res, load, samples, facts)| {
                    let m = measure_delivered(samples, facts, &src, &source_shape, &anchors);
                    (res, load, m)
                })
            } else {
                let (res, limited) = resolve_drive(
                    stage_for_g,
                    target,
                    STANDARD_CEILING_DBTP,
                    Some(CLEAN_LIMIT_DB),
                    TARGET_TOLERANCE_LU,
                    MAX_PASSES,
                );
                let load = limited.max_gain_reduction_db;
                measure_delivery(limited.samples, &src, target, &source_shape, &anchors)
                    .map(|m| (res, load, m))
            };
            match resolved {
                Ok((res, load, m)) => rows.push(row(
                    style,
                    target,
                    "G",
                    m,
                    level_in,
                    Some(res.drive_db),
                    None,
                    load,
                    res.limiter_passes,
                    res.stopped_at_clean_limit,
                    res.resolved,
                    res.inert,
                    g_stage_s + t1.elapsed().as_secs_f64(),
                    res.trace,
                )),
                Err(e) => errors.push(format!("{style} {target} G: {e}")),
            }
        }
    }
    SongResult {
        song: song.clone(),
        rate: src.rate,
        channels: ch,
        seconds_of_audio: (src.samples.len() / ch) as f64 / f64::from(src.rate),
        source_lufs: src.lufs,
        source_true_peak_dbtp: src.true_peak,
        source_lra: src.lra,
        level_in_db: level_in,
        decomposition_bit_identical: decomposition_ok,
        coefficients_target_independent: coeffs_ok,
        transient_anchors: anchors.transient_times.len(),
        section_pair: anchors.section_pair,
        rows,
        errors,
        wall_seconds: start.elapsed().as_secs_f64(),
    }
}

// ---------------------------------------------------------------------------
// Aggregation
// ---------------------------------------------------------------------------

fn outcome(r: &RenderRow) -> DesignOutcome {
    DesignOutcome {
        lufs: r.lufs,
        true_peak_dbtp: r.true_peak_dbtp,
        plr_db: r.plr_db,
        landing_db: r.landing_db,
        all_finite: r.all_finite,
        under_ceiling: r.under_ceiling,
        stopped_at_clean_limit: r.stopped_at_clean_limit,
        tone_band_delta_db: r.change.tone_band_delta_db,
    }
}

fn rule_inputs(results: &[SongResult]) -> (Vec<RuleCase>, Vec<DistinctCase>) {
    let mut cases = Vec::new();
    let mut distinct = Vec::new();
    for s in results {
        for target in STANDARD_TARGETS {
            for (style, _) in standard_styles() {
                let find = |d: &str| {
                    s.rows
                        .iter()
                        .find(|r| r.design == d && r.style == style && r.target == target)
                        .map(outcome)
                };
                cases.push(RuleCase {
                    song: s.song.label.clone(),
                    reference_song: s.song.reference,
                    source_lufs: s.source_lufs,
                    style: style.into(),
                    target,
                    level_in_db: s.level_in_db,
                    t: find("T"),
                    s: find("S"),
                    g: find("G"),
                });
            }
            let dist = |d: &str| {
                let prints: Vec<Fingerprint> = standard_styles()
                    .iter()
                    .filter_map(|(style, _)| {
                        s.rows
                            .iter()
                            .find(|r| r.design == d && r.style == *style && r.target == target)
                            .map(|r| r.fingerprint)
                    })
                    .collect();
                (prints.len() == 4).then(|| distinctness(&prints).0)
            };
            distinct.push(DistinctCase {
                song: s.song.label.clone(),
                source_lufs: s.source_lufs,
                target,
                t: dist("T"),
                s: dist("S"),
                g: dist("G"),
            });
        }
    }
    (cases, distinct)
}

fn git_commit() -> String {
    std::process::Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "unknown".into())
}

fn write_json(path: &Path, value: &impl Serialize) -> Result<(), String> {
    let text = serde_json::to_string_pretty(value).map_err(|e| e.to_string())?;
    std::fs::write(path, text).map_err(|e| format!("write {}: {e}", path.display()))
}

fn csv_ledger(results: &[SongResult]) -> String {
    let mut out = String::from(
        "song,group,kind,source_lufs,source_tp,level_in,style,target,design,lufs,tp,plr,lra,landing,drive,turn_down,limiter_load,passes,stopped,resolved,section_contrast_delta,attack_median_delta,attack_p10_delta,side_mid_delta,correlation_delta,max_abs_tone_delta\n",
    );
    let f = |v: Option<f64>| v.map_or(String::new(), |v| format!("{v:.4}"));
    for s in results {
        for r in &s.rows {
            let max_tone = r
                .change
                .tone_band_delta_db
                .iter()
                .map(|v| v.abs())
                .fold(0.0, f64::max);
            out.push_str(&format!(
                "\"{}\",{},\"{}\",{:.3},{:.3},{},{},{},{},{:.3},{:.3},{:.3},{:.3},{:.3},{},{},{:.3},{},{},{},{},{},{},{},{},{:.3}\n",
                s.song.label,
                s.song.group,
                s.song.kind,
                s.source_lufs,
                s.source_true_peak_dbtp,
                f(s.level_in_db.map(f64::from)),
                r.style,
                r.target,
                r.design,
                r.lufs,
                r.true_peak_dbtp,
                r.plr_db,
                r.lra,
                r.landing_db,
                f(r.drive_db.map(f64::from)),
                f(r.turn_down_db.map(f64::from)),
                r.limiter_load_db,
                r.passes,
                r.stopped_at_clean_limit,
                r.resolved,
                f(r.change.section_contrast_delta_db),
                f(r.change.attack_median_delta_db),
                f(r.change.attack_p10_delta_db),
                f(r.change.side_mid_delta_db),
                f(r.change.correlation_delta),
                max_tone,
            ));
        }
    }
    out
}

#[derive(Serialize)]
struct Summary<'a> {
    schema: &'static str,
    commit: String,
    g_resolves_on: &'static str,
    excerpt_seconds: Option<f64>,
    parameters: BTreeMap<&'static str, f64>,
    nominal: Nominal,
    material: BTreeMap<&'static str, usize>,
    quiet_dynamic_sources_below_minus_18: Vec<String>,
    gates_off: bool,
    every_decomposition_bit_identical: bool,
    every_coefficient_set_target_independent: bool,
    conditions_g: Vec<Condition>,
    conditions_s: Vec<Condition>,
    selected: &'static str,
    songs: Vec<SongSummary<'a>>,
}

#[derive(Serialize)]
struct SongSummary<'a> {
    label: &'a str,
    group: &'a str,
    kind: &'a str,
    reference: bool,
    seconds_of_audio: f64,
    source_lufs: f32,
    source_true_peak_dbtp: f32,
    source_lra: f32,
    level_in_db: Option<f32>,
    errors: &'a [String],
    rows: Vec<serde_json::Value>,
}

fn summary_rows(s: &SongResult) -> Vec<serde_json::Value> {
    s.rows
        .iter()
        .map(|r| {
            serde_json::json!({
                "style": r.style, "target": r.target, "design": r.design,
                "lufs": r.lufs, "true_peak_dbtp": r.true_peak_dbtp, "plr_db": r.plr_db,
                "lra": r.lra, "landing_db": r.landing_db, "drive_db": r.drive_db,
                "turn_down_db": r.turn_down_db, "limiter_load_db": r.limiter_load_db,
                "passes": r.passes, "stopped_at_clean_limit": r.stopped_at_clean_limit,
                "resolved": r.resolved, "inert": r.inert,
                "all_finite": r.all_finite, "under_ceiling": r.under_ceiling,
                "section_contrast_delta_db": r.change.section_contrast_delta_db,
                "attack_median_delta_db": r.change.attack_median_delta_db,
                "attack_p10_delta_db": r.change.attack_p10_delta_db,
                "tone_band_delta_db": r.change.tone_band_delta_db,
                "side_mid_delta_db": r.change.side_mid_delta_db,
                "correlation_delta": r.change.correlation_delta,
                "fingerprint": r.fingerprint,
            })
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Controlled timing (single thread, sequential)
// ---------------------------------------------------------------------------

fn timing(songs: &[Song], nominal: Nominal, output: &Path) -> Result<(), String> {
    let mut rows = Vec::new();
    for song in songs {
        let src = load(song, None)?;
        let ch = src.channels;
        let level_in = level_in_db(nominal, src.lufs, src.true_peak).unwrap_or(0.0);
        let seconds_of_audio = (src.samples.len() / ch) as f64 / f64::from(src.rate);
        for target in STANDARD_TARGETS {
            let settings = standard_export_settings(
                yes_master_lib::Preset::Universal,
                target,
                src.profile,
                Some(src.lufs),
            );
            // Untimed: the full-length split render equals the production one.
            let production = production_chain_render(&src.samples, src.rate, ch, &settings);
            let split = limiter_pass(
                &stage_pass(&src.samples, src.rate, ch, &settings, 0.0),
                STANDARD_CEILING_DBTP,
                0.0,
            );
            let full_length_identical = production.len() == split.samples.len()
                && production
                    .iter()
                    .zip(&split.samples)
                    .all(|(a, b)| a.to_bits() == b.to_bits());
            drop((production, split));
            if !full_length_identical {
                return Err(format!("{}: full-length split render differs", song.label));
            }
            let mut t_s = Vec::new();
            let mut g_s = Vec::new();
            let mut p_s = Vec::new();
            let mut g_passes = 0;
            for _ in 0..3 {
                let t0 = Instant::now();
                let out = production_chain_render(&src.samples, src.rate, ch, &settings);
                deliver(out, src.rate, ch, target)?;
                p_s.push(t0.elapsed().as_secs_f64());

                let t0 = Instant::now();
                let stage = stage_pass(&src.samples, src.rate, ch, &settings, 0.0);
                let lim = limiter_pass(&stage, STANDARD_CEILING_DBTP, 0.0);
                drop(stage);
                deliver(lim.samples, src.rate, ch, target)?;
                t_s.push(t0.elapsed().as_secs_f64());

                let t0 = Instant::now();
                let stage = stage_pass(&src.samples, src.rate, ch, &settings, level_in);
                let (res, _, _, _) = resolve_drive_delivered(
                    &stage,
                    target,
                    Some(CLEAN_LIMIT_DB),
                    TARGET_TOLERANCE_LU,
                    MAX_PASSES,
                )?;
                drop(stage);
                g_s.push(t0.elapsed().as_secs_f64());
                g_passes = res.limiter_passes;
            }
            let med = |v: &[f64]| median(v).unwrap_or(f64::NAN);
            println!(
                "{} {:.0}s target {target}: production {:.2}s, T split {:.2}s, G {:.2}s ({} limiter passes)",
                song.label,
                seconds_of_audio,
                med(&p_s),
                med(&t_s),
                med(&g_s),
                g_passes
            );
            rows.push(serde_json::json!({
                "song": song.label, "seconds_of_audio": seconds_of_audio, "target": target,
                "production_render_and_delivery_s": med(&p_s),
                "t_split_render_and_delivery_s": med(&t_s),
                "g_resolve_and_delivery_s": med(&g_s),
                "g_limiter_passes": g_passes, "runs": 3,
                "full_length_split_bit_identical": full_length_identical,
            }));
        }
    }
    write_json(&output.join("timing.json"), &rows)
}

// ---------------------------------------------------------------------------

fn main() {
    if let Err(e) = run() {
        eprintln!("{e}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let args = parse_args()?;
    // A15: the gated systems stay off.
    let gates_off = !yes_master_lib::guardrails::is_adaptive_compression_enabled()
        && !yes_master_lib::confidence::is_confidence_gating_enabled()
        && !yes_master_lib::album::is_album_character_enabled();
    if !gates_off {
        return Err("a gated system is switched on; this research needs them off".into());
    }
    std::fs::create_dir_all(&args.output).map_err(|e| e.to_string())?;
    std::fs::create_dir_all(&args.evidence).map_err(|e| e.to_string())?;

    let discovery = discover(&args)?;
    let reference = discovery
        .songs
        .iter()
        .find(|s| s.reference)
        .ok_or("the reference song was not found")?
        .clone();
    let reference_loaded = load(&reference, None)?;
    let nominal = Nominal {
        lufs: reference_loaded.lufs,
        peak_dbtp: reference_loaded.true_peak,
    };
    drop(reference_loaded);
    println!(
        "nominal (reference song analysis): {:.4} LUFS, {:.4} dBTP",
        nominal.lufs, nominal.peak_dbtp
    );
    let mut songs = discovery.songs.clone();
    if let Some(only) = &args.only {
        let wanted: Vec<&str> = only.split('|').collect();
        songs.retain(|s| {
            wanted.iter().any(|w| s.label.contains(w)) || (s.reference && !args.timing)
        });
    }
    println!(
        "songs: {} (library build {}, library holdout skipped {}, suno {} with {} held out, unmastered {} with {} held out)",
        songs.len(),
        discovery.library_build,
        discovery.library_holdout_skipped,
        discovery.suno_files,
        discovery.suno_holdout,
        discovery.unmastered_files,
        discovery.unmastered_holdout
    );
    if args.timing {
        return timing(&songs, nominal, &args.output);
    }

    let started = Instant::now();
    let results = parallel_map(&songs, args.workers, |song| {
        let r = process_song(song, nominal, args.excerpt_seconds, args.g_on_delivered);
        println!(
            "done {} ({:.0}s audio, {:.0}s wall, {} rows, {} errors)",
            song.label,
            r.seconds_of_audio,
            r.wall_seconds,
            r.rows.len(),
            r.errors.len()
        );
        r
    });
    let wall = started.elapsed().as_secs_f64();

    let (cases, distinct) = rule_inputs(&results);
    let g = evaluate_rule(&cases, &distinct, Design::G);
    let s = evaluate_rule(&cases, &distinct, Design::S);
    let selected = select_design(&g, &s);
    for c in g.iter() {
        println!(
            "G {} {} ({} cases): {}",
            c.id,
            if c.pass { "PASS" } else { "FAIL" },
            c.cases,
            c.detail
        );
    }
    for c in s.iter() {
        println!(
            "S {} {} ({} cases): {}",
            c.id,
            if c.pass { "PASS" } else { "FAIL" },
            c.cases,
            c.detail
        );
    }
    println!("selected: {selected}; wall {wall:.0}s");

    // Private ledger: names files.
    let private = serde_json::json!({
        "commit": git_commit(),
        "wall_seconds": wall,
        "paths": songs.iter().map(|s| (s.label.clone(), s.path.to_string_lossy().into_owned())).collect::<BTreeMap<_, _>>(),
        "results": results,
        "cases": cases,
        "distinct": distinct,
    });
    write_json(&args.output.join("ledger.json"), &private)?;
    std::fs::write(args.output.join("ledger.csv"), csv_ledger(&results))
        .map_err(|e| e.to_string())?;

    // Committable summary: numbers, library/corpus titles, owner IDs.
    let mut material = BTreeMap::new();
    material.insert("songs", results.len());
    material.insert("library_build", discovery.library_build);
    material.insert(
        "library_holdout_never_decoded",
        discovery.library_holdout_skipped,
    );
    material.insert(
        "corpus",
        results.iter().filter(|r| r.song.group == "corpus").count(),
    );
    material.insert(
        "owner",
        results
            .iter()
            .filter(|r| r.song.group.starts_with("owner"))
            .count(),
    );
    material.insert("suno_files", discovery.suno_files);
    material.insert("unmastered_files", discovery.unmastered_files);
    material.insert("cases", cases.len());
    let mut parameters = BTreeMap::new();
    parameters.insert("clean_limit_db", f64::from(CLEAN_LIMIT_DB));
    parameters.insert("target_tolerance_lu", f64::from(TARGET_TOLERANCE_LU));
    parameters.insert("max_passes", MAX_PASSES as f64);
    parameters.insert("level_in_range_db", f64::from(LEVEL_IN_RANGE_DB));
    let summary = Summary {
        schema: "auto-gain-staging-phase-a-v1",
        commit: git_commit(),
        g_resolves_on: if args.g_on_delivered {
            "delivered loudness"
        } else {
            "chain output before delivery"
        },
        excerpt_seconds: args.excerpt_seconds,
        parameters,
        nominal,
        material,
        quiet_dynamic_sources_below_minus_18: results
            .iter()
            .filter(|r| r.source_lufs < -18.0)
            .map(|r| r.song.label.clone())
            .collect(),
        gates_off,
        every_decomposition_bit_identical: results.iter().all(|r| r.decomposition_bit_identical),
        every_coefficient_set_target_independent: results
            .iter()
            .all(|r| r.coefficients_target_independent),
        conditions_g: g,
        conditions_s: s,
        selected,
        songs: results
            .iter()
            .map(|r| SongSummary {
                label: &r.song.label,
                group: &r.song.group,
                kind: &r.song.kind,
                reference: r.song.reference,
                seconds_of_audio: r.seconds_of_audio,
                source_lufs: r.source_lufs,
                source_true_peak_dbtp: r.source_true_peak_dbtp,
                source_lra: r.source_lra,
                level_in_db: r.level_in_db,
                errors: &r.errors,
                rows: summary_rows(r),
            })
            .collect(),
    };
    write_json(&args.evidence.join("phase-a-summary.json"), &summary)?;
    if results.iter().any(|r| !r.decomposition_bit_identical) {
        return Err("the split render did not match the production chain; results invalid".into());
    }
    Ok(())
}
