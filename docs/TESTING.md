# Testing

## Fast Lane

Run from repo root:

```powershell
npm test
npm run build
npm run build:windows
```

Run from `src-tauri`:

```powershell
cargo fmt --check
cargo clippy --target-dir target\codex-rc --all-targets -- -D warnings
cargo test --lib --target-dir target\codex-rc
cargo test --target-dir target\codex-rc
```

Use this lane for normal UI, state, packaging-script, and backend contract work.
The explicit `target\codex-rc` directory avoids collisions with a running debug
app that may lock the default target executable on Windows.

## iPhone Native Bridge Lane

When changing shared Rust types, `yes_master_lib` behavior, adaptive/profile
resolution, or `#[tauri::command]` signatures that the phone bridge may depend
on, also run:

```powershell
cd apps/iphone-native/rust
cargo check --all-targets
cargo test
```

The desktop lanes do not compile this bridge. A shared struct or signature can
drift here while all desktop checks stay green.

## Slow Fixture Lane

Private audio must live under:

```text
private-audio-fixtures/
```

Run from `src-tauri`:

```powershell
$env:AMS_RUN_REAL_FIXTURE = "1"
cargo test --target-dir target\codex-rc
Remove-Item Env:\AMS_RUN_REAL_FIXTURE
```

Use this lane before merging changes to:

- DSP chain behavior.
- Compressor mode.
- Limiter/ceiling behavior.
- LUFS landing.
- Export checks.
- WAV writing.
- Source/master parity.
- Decode/playback paths that affect audition trust.

## Manual Listening Gate

Automated tests cannot approve taste.

Before calling Track Master private-solid, manually verify with audio playing:

- Intensity sweeps.
- EQ/tone sweeps.
- Output gain sweeps.
- Compressor threshold/density sweeps.
- Preview LUFS off and on.
- Original/Mastered switching.
- Volume Match off and on.
- Export and open output.

## Already-Mastered Regression Matrix

Use the local-only runner documented in `docs/PRIVATE_AUDIO_FIXTURES.md`:

```powershell
cd src-tauri
cargo run --example private_fixture_matrix -- --manifest ..\private-audio-fixtures\manifest.json --output ..\test-output\private-fixture-matrix
```

To validate the Phase B confidence gate in the same headless lane:

```powershell
cd src-tauri
$env:YES_MASTER_CONFIDENCE_GATING = "1"
cargo run --example private_fixture_matrix -- --manifest ..\private-audio-fixtures\manifest.json --output ..\test-output\private-fixture-matrix-confidence
Remove-Item Env:\YES_MASTER_CONFIDENCE_GATING
```

If the full private manifest is too slow for an interactive run, create a
local-only subset manifest under ignored private fixture storage and record that
the evidence is representative rather than complete.

Required coverage:

| Case | Preset | Compressor | Expected Evidence |
| --- | --- | --- | --- |
| Already-processed source | Universal | Preset | Source/render LUFS, TP, LRA, warning codes |
| Already-processed source | Universal | Off | Source/render LUFS, TP, LRA, warning codes |
| Already-processed source | Loud | Preset | Source/render LUFS, TP, LRA, warning codes |
| Already-processed source | Loud | Off | Source/render LUFS, TP, LRA, warning codes |
| Already-processed source | Clarity | Preset | Source/render LUFS, TP, LRA, warning codes |
| Already-processed source | Clarity | Off | Source/render LUFS, TP, LRA, warning codes |

The goal is not to prevent bold masters. The goal is to catch cases where the
app makes the output objectively flatter/hotter and then fails to make the user
review that fact.

## Private Reference Tuning

Use the local-only reference runner after preset/DSP retunes:

```powershell
cd src-tauri
cargo run --example private_reference_tuning -- --references "..\tests for presets" --output ..\test-output\private-reference-tuning
```

To compare reference gaps with Phase B confidence gating enabled:

```powershell
cd src-tauri
$env:YES_MASTER_CONFIDENCE_GATING = "1"
cargo run --example private_reference_tuning -- --references "..\tests for presets" --output ..\test-output\private-reference-tuning-confidence
Remove-Item Env:\YES_MASTER_CONFIDENCE_GATING
```

Use the ledger as evidence, but do not treat it as a listening substitute. The
runner output and rendered WAVs are private/ignored and must not be committed.

## Tooling Notes

- Clippy is part of the hard local gate. If it is missing on a fresh toolchain,
  install it with `rustup component add clippy`.
- Windows packaging should produce MSI and NSIS artifacts under ignored
  `src-tauri/target/release/bundle/` and should not leave
  `src-tauri/target/release/produce_dialog_smoke.exe` registered as an app
  binary.
