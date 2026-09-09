# Windows source rebuild and LAME relink proof

This describes mechanical reproducibility, not a grant of distribution rights.
The [permission draft](lgpl-relink-exception-DRAFT.md) remains unapproved.

The Windows candidate source archive is exact commit
`7cd36ab641eb35fa737ded13a06b826d2b4cfdb7`. The redistribution manifest hashes
the application ZIP, `mp3lame-encoder-0.2.5.crate`, `mp3lame-sys-0.1.11.crate`,
and the separate encoder's source, configuration and licenses. Verify these
hashes before extracting. The qualified Windows encoder package is a separate
companion archive with its own sealed manifest.

Prerequisites: Windows x64, MSVC C++ build tools/SDK, Rust and Cargo, Node/npm,
and the dependency versions in Cargo.lock/package-lock.json. A first build needs
dependency access (`npm ci`, `cargo fetch --locked`); subsequent Cargo builds can
use `--offline`. The test used the existing npm dependency installation and
Cargo cache, so it is not an air-gapped dependency-vendoring proof.

1. Extract the application ZIP into `source/`. Extract both `.crate` archives
   with `tar -xf` into sibling `vendor/`. Keep the candidate install/build and
   original archives untouched.
2. Add `source/.cargo/config.toml` with the following local source overrides:

   ```toml
   [patch.crates-io]
   mp3lame-sys = { path = "../vendor/mp3lame-sys-0.1.11" }
   mp3lame-encoder = { path = "../vendor/mp3lame-encoder-0.2.5" }
   ```

3. Modify the library in that private tree. The recorded proof changed only
   `LAME_MINOR_VERSION` from 100 to 101 in
   `vendor/mp3lame-sys-0.1.11/lame-3.100/libmp3lame/version.h`. This deliberate
   version marker is a test modification, not a proposed product update.
4. From `source/`, install npm dependencies, stage the qualified encoder, build
   the frontend, and build the complete app into a fresh target:

   ```powershell
   npm ci
   node scripts/stage-audio-encoder.mjs C:/path/to/qualified-package x86_64-pc-windows-msvc
   npm run build
   cargo build --manifest-path src-tauri/Cargo.toml --target-dir ../target --release --offline
   cargo test --manifest-path src-tauri/Cargo.toml --target-dir ../target --release --offline --test mp3_export
   ```

   Do not use `--locked` after introducing deliberate local dependency patches:
   Cargo must update the copied lockfile's source entries. This affects only
   the private rebuild tree. Do not change the release archive or original lock.
5. Verify a complete executable is produced and the altered library is actually
   linked. The recorded test found `LAME3.101` in the full executable, encoded a
   synthetic stereo signal with `yes_master_lib::mp3::write`, asserted the same
   marker in those MP3 bytes, and decoded finite non-silent 48 kHz stereo audio.
   Existing MP3 Track/Album tests also passed. The proof source, modified header,
   local Cargo override, command logs and hashes accompany the local candidate
   as a separate proof folder; none replace the unmodified source archives.

Local evidence: `test-output/candidate-7cd36ab6/relink-proof/`. Full release build
passed in 2m14s; the three existing MP3 tests and one modified-library proof passed.
The local-path wrapper emitted an upstream unused-import warning; it was not
edited to silence that warning. The unmodified application passed strict Clippy
separately. No modified library/app was installed or substituted for the candidate.

Mac source rebuild, universal sidecar execution and installed hashing remain
unexecuted. A recipient must also have the applicable modification/relinking
permissions; supplying these files alone does not resolve that owner decision.
