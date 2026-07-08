fn main() {
    // Owner finding 2026-07-08: every dev build reports "0.9.0", so during
    // hand-testing there is no way to tell WHICH build is installed — stale
    // builds have burned whole test sessions. Stamp each binary with the git
    // hash + build time; `build_info` (lib.rs) surfaces it in the Help dialog
    // and the startup log line.
    let git_hash = std::process::Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_else(|| "unknown".to_string());
    let dirty = std::process::Command::new("git")
        .args(["status", "--porcelain"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| !o.stdout.is_empty())
        .unwrap_or(false);
    let now = chrono::Local::now().format("%Y-%m-%d %H:%M");
    let stamp = format!("{git_hash}{} · {now}", if dirty { "+" } else { "" });
    println!("cargo:rustc-env=YES_BUILD_STAMP={stamp}");
    // Rebuild whenever HEAD moves so the hash never goes stale in-place.
    println!("cargo:rerun-if-changed=../.git/HEAD");

    tauri_build::build()
}
