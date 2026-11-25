//! Smart Balance reference measurement (local-only evidence runner).
//!
//! **Measure a library.** Measures a folder of released songs (searched
//! recursively) through the app's own six-band meter and writes:
//!
//! - `smart-balance-reference-ledger.{json,csv}`: private, per file;
//! - `smart-balance-reference-stats.json`: numbers and anonymous IDs only,
//!   safe to commit.
//!
//! **Place songs.** With `--stats` and one or more `--place` folders, measures
//! those songs through the same meter and writes where each sits against the
//! range (`smart-balance-placement.{json,csv}`, private: it names files).
//!
//! Usage (from `src-tauri`):
//!   cargo run --release --example smart_balance_reference -- \
//!     --references "../private-audio-fixtures/reference-masters" \
//!     --output ../test-output/smart-balance/reference \
//!     [--exclude-matching <dir>]... [--provisional] [--seed 20261124]
//!
//!   cargo run --release --example smart_balance_reference -- \
//!     --stats <smart-balance-reference-stats.json> --place <dir> [--place <dir>]... \
//!     --output ../test-output/smart-balance/placement
//!
//! **Emit Smart Match's typical shape.** With `--emit-typical-shape`, writes
//! the Rust table Smart Match's missing-band rule reads
//! (`src/smart_match_typical_shape.rs` unless `--rust-out` says otherwise), so
//! the numbers are never copied by hand:
//!   cargo run --release --example smart_balance_reference -- \
//!     --emit-typical-shape <smart-balance-reference-stats.json>

use std::env;
use std::path::PathBuf;

use yes_master_lib::smart_balance_reference::{
    read_stats, run_placement, run_reference_measurement, typical_shape_rust_source,
    ReferenceRunConfig, MIN_LIBRARY_SONGS, TYPICAL_SHAPE_RUST_FILE,
};

fn main() {
    if let Err(err) = run() {
        eprintln!("{err}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let mut reference_dir: Option<PathBuf> = None;
    let mut output_dir: Option<PathBuf> = None;
    let mut stats_path: Option<PathBuf> = None;
    let mut place_dirs: Vec<PathBuf> = Vec::new();
    let mut emit_typical_shape: Option<PathBuf> = None;
    let mut rust_out: Option<PathBuf> = None;
    let mut config = ReferenceRunConfig::default();
    let mut args = env::args().skip(1);

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--references" | "-r" => reference_dir = args.next().map(PathBuf::from),
            "--output" | "-o" => output_dir = args.next().map(PathBuf::from),
            "--seed" => {
                let raw = args.next().ok_or_else(usage)?;
                config.seed = raw
                    .parse()
                    .map_err(|e| format!("--seed must be an unsigned integer: {e}"))?;
            }
            "--provisional" => config.provisional = true,
            "--exclude-matching" => {
                let dir = args.next().ok_or_else(usage)?;
                config.exclude_matching.push(PathBuf::from(dir));
            }
            "--stats" => stats_path = args.next().map(PathBuf::from),
            "--place" => {
                let dir = args.next().ok_or_else(usage)?;
                place_dirs.push(PathBuf::from(dir));
            }
            "--emit-typical-shape" => emit_typical_shape = args.next().map(PathBuf::from),
            "--rust-out" => rust_out = args.next().map(PathBuf::from),
            "--help" | "-h" => {
                println!("{}", usage());
                return Ok(());
            }
            other => return Err(format!("unknown argument: {other}\n\n{}", usage())),
        }
    }

    if let Some(stats_path) = emit_typical_shape {
        if reference_dir.is_some() || !place_dirs.is_empty() {
            return Err(format!(
                "--emit-typical-shape runs on its own\n\n{}",
                usage()
            ));
        }
        let out = rust_out.unwrap_or_else(|| PathBuf::from(TYPICAL_SHAPE_RUST_FILE));
        return emit(&stats_path, &out);
    }

    match (reference_dir, stats_path) {
        (Some(reference_dir), None) => {
            if !place_dirs.is_empty() {
                return Err(format!("--place needs --stats\n\n{}", usage()));
            }
            let output_dir = output_dir
                .unwrap_or_else(|| PathBuf::from("../test-output/smart-balance/reference"));
            measure(&reference_dir, &output_dir, &config)
        }
        (None, Some(stats_path)) => {
            if place_dirs.is_empty() {
                return Err(format!("--stats needs at least one --place\n\n{}", usage()));
            }
            let output_dir = output_dir
                .unwrap_or_else(|| PathBuf::from("../test-output/smart-balance/placement"));
            place(&stats_path, &place_dirs, &output_dir, &config)
        }
        _ => Err(usage()),
    }
}

fn measure(
    reference_dir: &std::path::Path,
    output_dir: &std::path::Path,
    config: &ReferenceRunConfig,
) -> Result<(), String> {
    let result = run_reference_measurement(reference_dir, output_dir, config)
        .map_err(|err| format!("smart balance reference measurement failed: {err}"))?;

    let counts = &result.stats.counts;
    println!("Smart Balance reference measurement complete.");
    if result.stats.provisional {
        println!("Labelled PROVISIONAL.");
    }
    println!(
        "Files seen: {} | accepted: {} (build {}, holdout {})",
        counts.files_seen, counts.accepted, counts.build, counts.holdout
    );
    println!(
        "Skipped: not audio {}, unreadable {}, undecodable {}, too short {}, low sample rate {}, silent {}, duplicate {}, matches excluded {}",
        counts.skipped_not_audio,
        counts.skipped_unreadable,
        counts.skipped_undecodable,
        counts.skipped_too_short,
        counts.skipped_low_sample_rate,
        counts.skipped_silent,
        counts.skipped_duplicate,
        counts.skipped_matches_excluded
    );
    if !result.stats.meets_minimum_library_size {
        println!(
            "WARNING: {} accepted songs is below the plan's minimum of {MIN_LIBRARY_SONGS}. \
             Do not use these statistics as the healthy range.",
            counts.accepted
        );
    }
    println!("Stats (committable): {}", result.stats_path.display());
    println!("Ledger (private): {}", result.ledger_json_path.display());
    println!("Ledger CSV (private): {}", result.ledger_csv_path.display());
    Ok(())
}

fn place(
    stats_path: &std::path::Path,
    place_dirs: &[PathBuf],
    output_dir: &std::path::Path,
    config: &ReferenceRunConfig,
) -> Result<(), String> {
    let stats = read_stats(stats_path).map_err(|err| format!("cannot read statistics: {err}"))?;
    let result = run_placement(place_dirs, &stats, output_dir, config)
        .map_err(|err| format!("smart balance placement failed: {err}"))?;
    let report = &result.report;
    println!(
        "Placed {} songs against a range from {} build songs{}; skipped {}.",
        report.placed.len(),
        report.library_build_songs,
        if report.provisional {
            " (PROVISIONAL)"
        } else {
            ""
        },
        report.skipped.len()
    );
    println!("Placement (private): {}", result.report_json_path.display());
    println!(
        "Placement CSV (private): {}",
        result.report_csv_path.display()
    );
    Ok(())
}

fn emit(stats_path: &std::path::Path, out: &std::path::Path) -> Result<(), String> {
    let stats = read_stats(stats_path).map_err(|err| format!("cannot read statistics: {err}"))?;
    let source = typical_shape_rust_source(&stats).map_err(|err| err.to_string())?;
    std::fs::write(out, source).map_err(|e| format!("write {}: {e}", out.display()))?;
    println!(
        "Wrote Smart Match's typical shape ({} build songs{}) to {}",
        stats.counts.build,
        if stats.provisional {
            ", PROVISIONAL"
        } else {
            ""
        },
        out.display()
    );
    Ok(())
}

fn usage() -> String {
    "Usage:\n  smart_balance_reference --references <dir> [--output <dir>] [--seed <u64>] \
     [--provisional] [--exclude-matching <dir>]...\n  \
     smart_balance_reference --stats <stats.json> --place <dir> [--place <dir>]... \
     [--output <dir>]\n  \
     smart_balance_reference --emit-typical-shape <stats.json> [--rust-out <file>]"
        .to_string()
}
