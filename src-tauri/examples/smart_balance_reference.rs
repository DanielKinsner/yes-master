//! Smart Balance reference measurement (local-only evidence runner).
//!
//! Measures a folder of released songs (searched recursively) through the
//! app's own six-band meter and writes:
//!
//! - `smart-balance-reference-ledger.{json,csv}`: private, per file;
//! - `smart-balance-reference-stats.json`: numbers and anonymous IDs only,
//!   safe to commit.
//!
//! Usage (from `src-tauri`):
//!   cargo run --release --example smart_balance_reference -- \
//!     --references "../private-audio-fixtures/reference-masters" \
//!     --output ../test-output/smart-balance/reference [--seed 20261124]

use std::env;
use std::path::PathBuf;

use yes_master_lib::smart_balance_reference::{
    run_reference_measurement, ReferenceRunConfig, MIN_LIBRARY_SONGS,
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
            "--help" | "-h" => {
                println!("{}", usage());
                return Ok(());
            }
            other => return Err(format!("unknown argument: {other}\n\n{}", usage())),
        }
    }

    let reference_dir = reference_dir.ok_or_else(usage)?;
    let output_dir =
        output_dir.unwrap_or_else(|| PathBuf::from("../test-output/smart-balance/reference"));
    let result = run_reference_measurement(&reference_dir, &output_dir, &config)
        .map_err(|err| format!("smart balance reference measurement failed: {err}"))?;

    let counts = &result.stats.counts;
    println!("Smart Balance reference measurement complete.");
    println!(
        "Files seen: {} | accepted: {} (build {}, holdout {})",
        counts.files_seen, counts.accepted, counts.build, counts.holdout
    );
    println!(
        "Skipped: not audio {}, unreadable {}, undecodable {}, too short {}, low sample rate {}, silent {}, duplicate {}",
        counts.skipped_not_audio,
        counts.skipped_unreadable,
        counts.skipped_undecodable,
        counts.skipped_too_short,
        counts.skipped_low_sample_rate,
        counts.skipped_silent,
        counts.skipped_duplicate
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

fn usage() -> String {
    "Usage: cargo run --release --example smart_balance_reference -- --references <dir> \
     [--output <dir>] [--seed <u64>]"
        .to_string()
}
