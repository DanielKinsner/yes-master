use std::env;
use std::path::PathBuf;

use yes_master_lib::reference_tuning::run_reference_tuning_dir;

fn main() {
    if let Err(err) = run() {
        eprintln!("{err}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let mut reference_dir: Option<PathBuf> = None;
    let mut output_dir: Option<PathBuf> = None;
    let mut args = env::args().skip(1);

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--references" | "-r" => {
                reference_dir = args.next().map(PathBuf::from);
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

    let reference_dir = reference_dir.ok_or_else(usage)?;
    let output_dir =
        output_dir.unwrap_or_else(|| PathBuf::from("../test-output/private-reference-tuning"));
    let result = run_reference_tuning_dir(&reference_dir, &output_dir)
        .map_err(|err| format!("private reference tuning failed: {err}"))?;

    println!("Private reference tuning comparison complete.");
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
    "Usage: cargo run --example private_reference_tuning -- --references <dir> [--output <dir>]"
        .to_string()
}
