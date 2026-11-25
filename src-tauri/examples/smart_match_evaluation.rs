//! Smart Match evaluation (local-only evidence runner; Smart Match plan,
//! Slice 6).
//!
//! Two phases, run in this order:
//!
//! - `--phase development`: every gate on development material (the
//!   library's build split and the five development corpus pieces), plus the
//!   three ungated reports. Use it for any tuning.
//! - `--phase holdout --frozen-commit <sha>`: G3 and G6 on the library's
//!   held-out songs and the three holdout corpus pieces, once, with frozen
//!   parameters; plus the 30-second report for the holdout pieces.
//!
//! Usage (from `src-tauri`; every path is required and absolute):
//!   cargo run --release --example smart_match_evaluation -- --phase development \
//!     --library <free-cc folder> --library-sources <library-sources.json> \
//!     --corpus-json <corpus.json> --corpus-root <mastering-quality-20260915 folder> \
//!     --owner-dir <"tests for presets" folder> --output <private dir> --evidence <committable dir>

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::Serialize;
use yes_master_lib::smart_balance_reference::anon_id_for_bytes;
use yes_master_lib::smart_match::{SmartMatchParams, TYPICAL_SHAPE_PROVISIONAL};
use yes_master_lib::smart_match_evaluation::*;
use yes_master_lib::SpectralBalance6;

/// Corpus genres for the cross-genre report. The corpus file has no genre
/// field; these are the executor's labels from titles, stems and sources.
const CORPUS_GENRES: [(&str, &str); 8] = [
    ("coat", "unlabelled"),
    ("piano", "cinematic/orchestral/classical"),
    ("funk", "jazz/funk/soul"),
    ("metal", "metal"),
    ("baby", "rock"),
    ("aphelion", "cinematic/orchestral/classical"),
    ("rich", "hip-hop/beats"),
    ("imaginal", "other"),
];

struct Args {
    phase: String,
    frozen_commit: Option<String>,
    library: PathBuf,
    library_sources: PathBuf,
    corpus_json: PathBuf,
    corpus_root: PathBuf,
    owner_dir: PathBuf,
    output: PathBuf,
    evidence: PathBuf,
    workers: usize,
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
    let phase = map.get("phase").cloned().unwrap_or_default();
    if phase != "development" && phase != "holdout" {
        return Err("--phase must be development or holdout".into());
    }
    let frozen_commit = map.get("frozen-commit").cloned();
    if phase == "holdout" && frozen_commit.is_none() {
        return Err("the holdout phase needs --frozen-commit <sha>".into());
    }
    Ok(Args {
        phase,
        frozen_commit,
        library: get("library")?,
        library_sources: get("library-sources")?,
        corpus_json: get("corpus-json")?,
        corpus_root: get("corpus-root")?,
        owner_dir: get("owner-dir")?,
        output: get("output")?,
        evidence: get("evidence")?,
        workers: map.get("workers").and_then(|w| w.parse().ok()).unwrap_or(8),
    })
}

fn read_json(path: &Path) -> Result<serde_json::Value, String> {
    let text =
        std::fs::read_to_string(path).map_err(|e| format!("read {}: {e}", path.display()))?;
    serde_json::from_str(&text).map_err(|e| format!("parse {}: {e}", path.display()))
}

/// Library songs: every accepted file, labelled "Artist – Title", with its
/// split and genre from the committed attribution list (matched by content).
fn library_specs(args: &Args) -> Result<Vec<SongSpec>, String> {
    let sources = read_json(&args.library_sources)?;
    let mut by_anon: BTreeMap<String, &serde_json::Value> = BTreeMap::new();
    for t in sources["tracks"].as_array().ok_or("no tracks")? {
        if let Some(id) = t["anon_id"].as_str() {
            by_anon.insert(id.to_string(), t);
        }
    }
    let mut specs = Vec::new();
    for path in files_under(&args.library).map_err(|e| e.to_string())? {
        let Ok(bytes) = std::fs::read(&path) else {
            continue;
        };
        let anon = anon_id_for_bytes(&bytes);
        let Some(t) = by_anon.get(&anon) else {
            continue;
        };
        let group = match t["split"].as_str() {
            Some("build") => SongGroup::LibraryBuild,
            Some("holdout") => SongGroup::LibraryHoldout,
            _ => continue,
        };
        specs.push(SongSpec {
            label: format!(
                "{} – {}",
                t["artist"].as_str().unwrap_or("?"),
                t["title"].as_str().unwrap_or("?")
            ),
            group,
            genre: t["genre_tag"].as_str().unwrap_or("other").to_string(),
            path,
        });
    }
    specs.sort_by(|a, b| a.label.cmp(&b.label));
    Ok(specs)
}

/// Corpus pieces from the committed corpus list. The owner's song is shown
/// by an anonymous content ID; the openly licensed pieces by title.
fn corpus_specs(args: &Args) -> Result<Vec<SongSpec>, String> {
    let corpus = read_json(&args.corpus_json)?;
    let mut specs = Vec::new();
    for s in corpus["sources"].as_array().ok_or("no sources")? {
        let id = s["id"].as_str().unwrap_or_default();
        let group = match s["split"].as_str() {
            Some("development") => SongGroup::CorpusDevelopment,
            Some("holdout") => SongGroup::CorpusHoldout,
            _ => continue,
        };
        let path = if id == "coat" {
            args.owner_dir.join("It\u{2019}s a coat-original-test.wav")
        } else {
            args.corpus_root.join("sources").join(format!("{id}.wav"))
        };
        let label = if id == "coat" {
            let bytes =
                std::fs::read(&path).map_err(|e| format!("read {}: {e}", path.display()))?;
            format!("owner {}", anon_id_for_bytes(&bytes))
        } else {
            s["title"].as_str().unwrap_or(id).to_string()
        };
        let genre = CORPUS_GENRES
            .iter()
            .find(|(k, _)| *k == id)
            .map(|(_, g)| (*g).to_string())
            .unwrap_or_else(|| "unlabelled".into());
        specs.push(SongSpec {
            label,
            group,
            genre,
            path,
        });
    }
    Ok(specs)
}

#[derive(Serialize)]
struct GateRow {
    gate: &'static str,
    pass: Option<bool>,
    number: String,
}

fn main() {
    if let Err(e) = run() {
        eprintln!("{e}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let args = parse_args()?;
    let params = SmartMatchParams::DEFAULT;
    if !params.within_plan_bounds() {
        return Err("product parameters are outside the plan's bounds".into());
    }
    let library = library_specs(&args)?;
    let corpus = corpus_specs(&args)?;
    let holdout = args.phase == "holdout";
    let in_phase = |s: &&SongSpec| s.group.is_holdout() == holdout;
    let songs: Vec<SongSpec> = library
        .iter()
        .chain(corpus.iter())
        .filter(in_phase)
        .cloned()
        .collect();
    eprintln!(
        "phase {}: {} songs ({} library, {} corpus)",
        args.phase,
        songs.len(),
        songs
            .iter()
            .filter(|s| matches!(s.group, SongGroup::LibraryBuild | SongGroup::LibraryHoldout))
            .count(),
        songs
            .iter()
            .filter(|s| matches!(
                s.group,
                SongGroup::CorpusDevelopment | SongGroup::CorpusHoldout
            ))
            .count()
    );

    // G3 and G6 on every song in this phase.
    let per_song = parallel_map(&songs, args.workers, |spec| {
        let started = std::time::Instant::now();
        let r = load_song(spec).and_then(|song| {
            let cases = known_answer_cases(&song, &params)?;
            Ok((song.profile.spectral_6, cases))
        });
        eprintln!(
            "  {} in {:.0} s{}",
            spec.label,
            started.elapsed().as_secs_f64(),
            if r.is_err() { " (FAILED)" } else { "" }
        );
        r.map_err(|e| format!("{}: {e}", spec.label))
    });
    let mut shares: BTreeMap<String, SpectralBalance6> = BTreeMap::new();
    let mut cases = Vec::new();
    let mut failures = Vec::new();
    for (spec, r) in songs.iter().zip(per_song) {
        match r {
            Ok((s, c)) => {
                shares.insert(spec.label.clone(), s);
                cases.extend(c);
            }
            Err(e) => failures.push(e),
        }
    }
    let known = summarize_known_answers(&cases);

    let mut gates = vec![
        GateRow {
            gate: "G3 known-answer recovery",
            pass: Some(known.g3_pass),
            number: format!(
                "{} eligible bands in {} cases over {} songs; wrong direction {}; worst |correction − 0.5 × gap| {:.3} dB ({} over 0.5); other bands quiet in {:.1}% of {} cases (worst {:.2} dB); {} boosts faded by the missing-band rule (reported, not graded)",
                known.eligible_bands, known.cases, known.songs, known.wrong_direction,
                known.worst_amount_error_db, known.off_by_more_than_tolerance,
                known.other_bands_quiet_share * 100.0, known.other_bands_cases,
                known.worst_other_band_move_db, known.faded_bands
            ),
        },
        GateRow {
            gate: "G6 gap closed end to end",
            pass: Some(known.g6_pass),
            number: format!(
                "{} bands; remaining gap {:.3}–{:.3} of the original (median {:.3}); {} outside 0.35–0.65 (by band sub..air {:?}). Diagnostic, EQ stage alone: median {:.3}, range {:.3}–{:.3}, {} outside. Context against the rendered original: median {:.3}, range {:.3}–{:.3}, {} outside",
                known.g6_bands, known.g6_min_ratio, known.g6_max_ratio, known.g6_median_ratio,
                known.g6_outside_range, known.g6_outside_by_band,
                known.eq_stage_median_ratio, known.eq_stage_min_ratio, known.eq_stage_max_ratio,
                known.eq_stage_outside_range,
                known.context_vs_rendered_original_median,
                known.context_vs_rendered_original_min, known.context_vs_rendered_original_max,
                known.context_outside_range
            ),
        },
    ];

    let mut report = serde_json::json!({
        "schema": "yes-master.smart-match.evaluation.v1",
        "phase": args.phase,
        "frozen_commit": args.frozen_commit,
        "parameters": params,
        "typical_shape_provisional": TYPICAL_SHAPE_PROVISIONAL,
        "amount": EVAL_AMOUNT,
        "songs": songs.iter().map(|s| serde_json::json!({"label": s.label, "group": s.group, "genre": s.genre})).collect::<Vec<_>>(),
        "load_failures": failures,
        "known_answers": known,
    });

    if !holdout {
        // Development-only gates and reports. Songs are loaded again as needed.
        let dev_corpus: Vec<&SongSpec> = corpus
            .iter()
            .filter(|s| s.group == SongGroup::CorpusDevelopment)
            .collect();
        let build: Vec<&SongSpec> = library
            .iter()
            .filter(|s| s.group == SongGroup::LibraryBuild)
            .collect();
        let load = |s: &SongSpec| load_song(s).map_err(|e| format!("{}: {e}", s.label));
        let corpus_loaded: Vec<LoadedSong> = dev_corpus
            .iter()
            .map(|s| load(s))
            .collect::<Result<_, _>>()?;
        let coat = corpus_loaded
            .iter()
            .find(|s| s.spec.label.starts_with("owner "))
            .ok_or("the owner's song is missing")?;
        let first_build = load(build.first().ok_or("no build songs")?)?;

        // G1 and G2 on the owner's song and one library song (30 s each).
        let identity: Vec<IdentityResult> = [coat, &first_build]
            .iter()
            .map(|s| identity_checks(s, 30.0))
            .collect();
        let g1_checks: usize = identity.iter().map(|r| r.g1_checks).sum();
        let g1_same: usize = identity.iter().map(|r| r.g1_identical).sum();
        let g2_checks: usize = identity.iter().map(|r| r.g2_checks).sum();
        let g2_same: usize = identity.iter().map(|r| r.g2_identical).sum();
        gates.insert(0, GateRow {
            gate: "G1 off identity",
            pass: Some(g1_same == g1_checks && g1_checks > 0),
            number: format!("{g1_same}/{g1_checks} preset × off-state renders bit-identical (coefficients equal), 2 real songs × 9 presets × 3 states"),
        });
        gates.insert(
            1,
            GateRow {
                gate: "G2 self-match identity",
                pass: Some(g2_same == g2_checks && g2_checks > 0),
                number: format!(
                    "{g2_same}/{g2_checks} self-matched renders bit-identical to match off"
                ),
            },
        );

        let g4 = realization_accuracy();
        gates.push(GateRow {
            gate: "G4 realization accuracy",
            pass: Some(g4.pass),
            number: format!(
                "{} cases; worst error {:.4} dB; {} over 0.5 dB",
                g4.cases, g4.worst_error_db, g4.over_half_db
            ),
        });

        // Fixed references for G5 and the 30-second report: the first build
        // song (by label) in three different genres.
        let mut fixed: Vec<&SongSpec> = Vec::new();
        for spec in &build {
            if fixed.len() < 3 && !fixed.iter().any(|f| f.genre == spec.genre) {
                fixed.push(spec);
            }
        }
        let fixed_loaded: Vec<LoadedSong> =
            fixed.iter().map(|s| load(s)).collect::<Result<_, _>>()?;
        let fixed_shares: Vec<SpectralBalance6> =
            fixed_loaded.iter().map(|s| s.profile.spectral_6).collect();
        let tracks: Vec<&LoadedSong> = corpus_loaded.iter().collect();
        let refs: Vec<&LoadedSong> = fixed_loaded.iter().collect();
        let g5 = level_invariance(&tracks, &fixed_shares, &refs, &params);
        gates.push(GateRow {
            gate: "G5 level invariance",
            pass: Some(g5.pass),
            number: format!(
                "{} comparisons at ±12 dB; worst change {:.5} dB",
                g5.pairs, g5.worst_change_db
            ),
        });

        // Cross-genre pairs: each development corpus piece against five build
        // songs of other genres, one per genre where possible.
        let mut cross_pairs = Vec::new();
        for piece in &corpus_loaded {
            let mut used_genres: Vec<&str> = Vec::new();
            let mut picked = 0;
            for spec in &build {
                if picked == 5 {
                    break;
                }
                if spec.genre == piece.spec.genre || used_genres.contains(&spec.genre.as_str()) {
                    continue;
                }
                if let Some(s) = shares.get(&spec.label) {
                    used_genres.push(spec.genre.as_str());
                    cross_pairs.push((
                        piece.spec.label.as_str(),
                        piece.profile.spectral_6,
                        spec.label.as_str(),
                        spec.genre.as_str(),
                        *s,
                    ));
                    picked += 1;
                }
            }
        }
        let cross = cross_genre(&cross_pairs, &params);

        // G7 over every known-answer pair and every cross-genre pair.
        let mut g7_pairs: Vec<(SpectralBalance6, SpectralBalance6)> =
            cross_pairs.iter().map(|p| (p.1, p.4)).collect();
        for c in &cases {
            if let (Some(t), Some(r)) = (c.copy_shares, shares.get(&c.song)) {
                g7_pairs.push((t, *r));
            }
        }
        let g7 = amount_and_caps(&g7_pairs, &params);
        gates.push(GateRow {
            gate: "G7 amount and caps",
            pass: Some(g7.pass),
            number: format!(
                "{} pairs × amounts {:?}: {} non-monotonic, {} over a cap, {} partial-flag mismatches; {} pairs hit a cap at 100%",
                g7.pairs, g7.amounts, g7.non_monotonic, g7.over_cap, g7.partial_mismatch, g7.pairs_with_a_cap_at_full_amount
            ),
        });
        gates.push(GateRow {
            gate: "G8 performance",
            pass: None,
            number: "measured with chain_bench, outside this runner (see the evaluation note)"
                .into(),
        });

        let scratch = args.output.join("scratch");
        let g9 = safety_checks(&scratch, &tracks, &params).map_err(|e| e.to_string())?;
        gates.push(GateRow {
            gate: "G9 safety",
            pass: Some(g9.pass),
            number: format!(
                "refusals correct: {}; {} songs with the sub band emptied, {} boosted or non-finite; hostile inputs inert: {}",
                g9.refusals_correct, g9.empty_band_tracks, g9.empty_band_boosted, g9.hostile_inputs_inert
            ),
        });
        let g10 = persistence_checks(&scratch, coat, &first_build).map_err(|e| e.to_string())?;
        gates.push(GateRow {
            gate: "G10 persistence",
            pass: Some(g10.pass),
            number: format!(
                "reopened without the reference file: {}; same correction: {}; preset carries no reference: {}",
                g10.reopened_without_reference_file, g10.same_correction_after_reopen, g10.preset_carries_no_reference
            ),
        });
        let _ = std::fs::remove_dir_all(&scratch);

        // The owner's song against the three LANDR masters.
        let mut masters = Vec::new();
        for path in files_under(&args.owner_dir).map_err(|e| e.to_string())? {
            let name = path
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_default();
            if name.starts_with("LANDR-") {
                let pcm = yes_master_lib::decode::decode_full(&path).map_err(|e| e.to_string())?;
                let s = measure(&pcm.samples, pcm.sample_rate, usize::from(pcm.channels))
                    .map_err(|e| e.to_string())?;
                let bytes = std::fs::read(&path).map_err(|e| e.to_string())?;
                masters.push((
                    format!("owner LANDR master {}", anon_id_for_bytes(&bytes)),
                    s,
                ));
            }
        }
        let landr = owner_vs_landr(coat, &masters, &params).map_err(|e| e.to_string())?;

        let listen: Vec<ListenRow> = corpus_loaded
            .iter()
            .map(|s| thirty_second_listen(s, &fixed_shares, &params))
            .collect::<Result<_, _>>()
            .map_err(|e| e.to_string())?;

        report["identity"] = serde_json::json!(identity);
        report["realization"] = serde_json::json!(g4);
        report["level_invariance"] = serde_json::json!(g5);
        report["amount_and_caps"] = serde_json::json!(g7);
        report["safety"] = serde_json::json!(g9);
        report["persistence"] = serde_json::json!(g10);
        report["fixed_references"] = serde_json::json!(fixed
            .iter()
            .map(|s| (&s.label, &s.genre))
            .collect::<Vec<_>>());
        report["cross_genre"] = serde_json::json!(cross);
        report["owner_vs_landr"] = serde_json::json!(landr);
        report["thirty_second_listen"] = serde_json::json!(listen);
    } else {
        // The 30-second report for the holdout corpus pieces (reporting only).
        let build: Vec<&SongSpec> = library
            .iter()
            .filter(|s| s.group == SongGroup::LibraryBuild)
            .collect();
        let mut fixed: Vec<&SongSpec> = Vec::new();
        for spec in &build {
            if fixed.len() < 3 && !fixed.iter().any(|f| f.genre == spec.genre) {
                fixed.push(spec);
            }
        }
        let fixed_shares: Vec<SpectralBalance6> = fixed
            .iter()
            .map(|s| {
                load_song(s)
                    .map(|l| l.profile.spectral_6)
                    .map_err(|e| e.to_string())
            })
            .collect::<Result<_, _>>()?;
        let mut listen = Vec::new();
        for spec in corpus
            .iter()
            .filter(|s| s.group == SongGroup::CorpusHoldout)
        {
            let song = load_song(spec).map_err(|e| e.to_string())?;
            listen.push(
                thirty_second_listen(&song, &fixed_shares, &params).map_err(|e| e.to_string())?,
            );
        }
        report["thirty_second_listen"] = serde_json::json!(listen);
    }

    report["gates"] = serde_json::json!(gates);
    report["cases"] = serde_json::json!(cases);

    std::fs::create_dir_all(&args.output).map_err(|e| e.to_string())?;
    std::fs::create_dir_all(&args.evidence).map_err(|e| e.to_string())?;
    let private = args
        .output
        .join(format!("smart-match-evaluation-{}.json", args.phase));
    let mut private_report = report.clone();
    private_report["song_paths"] = serde_json::json!(songs
        .iter()
        .map(|s| (&s.label, s.path.display().to_string()))
        .collect::<Vec<_>>());
    std::fs::write(
        &private,
        serde_json::to_string_pretty(&private_report).unwrap() + "\n",
    )
    .map_err(|e| e.to_string())?;
    let public = args
        .evidence
        .join(format!("smart-match-evaluation-{}.json", args.phase));
    std::fs::write(
        &public,
        serde_json::to_string_pretty(&report).unwrap() + "\n",
    )
    .map_err(|e| e.to_string())?;

    println!("Smart Match evaluation, {} phase:", args.phase);
    for g in &gates {
        let verdict = match g.pass {
            Some(true) => "PASS",
            Some(false) => "FAIL",
            None => "n/a",
        };
        println!("  {verdict:4}  {}: {}", g.gate, g.number);
    }
    if !failures.is_empty() {
        println!("  Songs that failed to load: {failures:?}");
    }
    println!("Private: {}", private.display());
    println!("Committable: {}", public.display());
    Ok(())
}
