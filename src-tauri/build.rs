fn main() {
    let target = std::env::var("TARGET").unwrap_or_default();
    let encoder_name = format!(
        "yes-master-encoder-{target}{}",
        if target.contains("windows") {
            ".exe"
        } else {
            ""
        }
    );
    println!("cargo:rustc-env=YES_MASTER_ENCODER_FILENAME={encoder_name}");
    let encoder_manifest = format!("binaries/manifest-{target}.json");
    println!("cargo:rerun-if-changed={encoder_manifest}");
    if let Ok(manifest) = std::fs::read_to_string(&encoder_manifest) {
        let manifest: serde_json::Value =
            serde_json::from_str(&manifest).expect("encoder manifest JSON");
        assert_eq!(
            manifest["target"].as_str(),
            Some(target.as_str()),
            "encoder architecture mismatch"
        );
        let hash = manifest["sha256"].as_str().expect("encoder SHA-256");
        assert!(
            hash.len() == 64 && hash.bytes().all(|c| c.is_ascii_hexdigit()),
            "invalid encoder SHA-256"
        );
        println!("cargo:rustc-env=YES_MASTER_ENCODER_SHA256={hash}");
    }
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
    // HEAD is usually a symbolic ref whose bytes do not change on commits.
    // Resolve git paths for both ordinary checkouts and linked worktrees.
    let mut git_paths = vec![
        "HEAD".to_string(),
        "index".to_string(),
        "packed-refs".to_string(),
    ];
    if let Ok(reference) = std::process::Command::new("git")
        .args(["symbolic-ref", "-q", "HEAD"])
        .output()
    {
        if reference.status.success() {
            git_paths.push(
                String::from_utf8_lossy(&reference.stdout)
                    .trim()
                    .to_string(),
            );
        }
    }
    for path in git_paths {
        if let Ok(resolved) = std::process::Command::new("git")
            .args(["rev-parse", "--path-format=absolute", "--git-path", &path])
            .output()
        {
            if resolved.status.success() {
                println!(
                    "cargo:rerun-if-changed={}",
                    String::from_utf8_lossy(&resolved.stdout).trim()
                );
            }
        }
    }
    println!("cargo:rerun-if-changed=src");
    println!("cargo:rerun-if-changed=../src");

    tauri_build::build()
}
