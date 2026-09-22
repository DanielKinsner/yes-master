//! Stamp the binary with a hash of the desktop sources it was built from, so
//! `version()` can say which chain it carries. `scripts/build-tryit-wasm.mjs`
//! computes the same hash (same file list, same algorithm) into
//! `src/tryit/engine/sources.stamp.json`, and `src/tryit/engine-stamp.test.ts`
//! fails when the desktop sources move on without a rebuild.
use sha2::{Digest, Sha256};
use std::path::Path;

/// Keep in lockstep with `TRACKED_SOURCES` in `scripts/build-tryit-wasm.mjs`.
const SOURCES: &[&str] = &[
    "src-tauri/src/types.rs",
    "src-tauri/src/dsp.rs",
    "src-tauri/src/export_format.rs",
    "src-tauri/src/analysis.rs",
    "src-tauri/src/deep_analysis.rs",
    "src-tauri/src/confidence.rs",
    "src-tauri/src/guardrails.rs",
    "web/tryit/wasm/src/lib.rs",
    "web/tryit/wasm/src/stubs.rs",
    "web/tryit/wasm/Cargo.toml",
];

fn main() {
    let repo = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let mut joined = String::new();
    for rel in SOURCES {
        let path = repo.join(rel);
        println!("cargo:rerun-if-changed={}", path.display());
        let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
        // Normalize line endings so Windows checkouts and CI agree.
        let normalized: Vec<u8> = bytes.into_iter().filter(|b| *b != b'\r').collect();
        joined.push_str(&format!("{rel}:{:x}\n", Sha256::digest(&normalized)));
    }
    let stamp = format!("{:x}", Sha256::digest(joined.as_bytes()));
    println!("cargo:rustc-env=YES_TRYIT_SOURCE_STAMP={}", &stamp[..12]);
}
