use std::env;
use std::path::PathBuf;

use yes_master_lib::fixture_matrix::run_manifest_path;

fn main() {
    if let Err(err) = run() {
        eprintln!("{err}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let mut manifest_path: Option<PathBuf> = None;
    let mut output_dir: Option<PathBuf> = None;
    let mut args = env::args().skip(1);

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--manifest" | "-m" => {
                manifest_path = args.next().map(PathBuf::from);
            }
            "--output" | "-o" => {
                output_dir = args.next().map(PathBuf::from);
            }
            "--help" | "-h" => {
                print_usage();
                return Ok(());
            }
            other => {
                return Err(format!("unknown argument: {other}\n\n{}", usage()));
            }
        }
    }

    let manifest_path = manifest_path.ok_or_else(usage)?;
    let output_dir =
        output_dir.unwrap_or_else(|| PathBuf::from("../test-output/private-fixture-matrix"));
    let result = run_manifest_path(&manifest_path, &output_dir)
        .map_err(|err| format!("private fixture matrix failed: {err}"))?;

    println!("Private fixture matrix complete.");
    println!("Rows: {}", result.rows.len());
    println!("JSON: {}", result.report_path.display());
    println!("CSV: {}", result.csv_path.display());
    println!("Renders: {}", result.render_dir.display());
    Ok(())
}

fn print_usage() {
    println!("{}", usage());
}

fn usage() -> String {
    "Usage: cargo run --example private_fixture_matrix -- --manifest <manifest.json> [--output <dir>]"
        .to_string()
}
