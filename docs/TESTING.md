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
cargo test --lib
cargo test
```

Use this lane for normal UI, state, packaging-script, and backend contract work.

## Slow Fixture Lane

Private audio must live under:

```text
private-audio-fixtures/
```

Run from `src-tauri`:

```powershell
$env:AMS_RUN_REAL_FIXTURE = "1"
cargo test
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

Add this as a first stabilization fixture protocol:

| Case | Preset | Compressor | Expected Evidence |
| --- | --- | --- | --- |
| Already-processed source | Universal | Preset | Source/render LUFS, TP, LRA, warning codes |
| Already-processed source | Universal | Off | Source/render LUFS, TP, LRA, warning codes |
| Already-processed source | Loud | Preset | Source/render LUFS, TP, LRA, warning codes |
| Already-processed source | Clarity | Preset | Source/render LUFS, TP, LRA, warning codes |

The goal is not to prevent bold masters. The goal is to catch cases where the
app makes the output objectively flatter/hotter and then fails to make the user
review that fact.

## Known Tooling Gaps

- `cargo fmt --check` currently reports pre-existing formatting drift.
- Clippy was not installed during migration recon. Install with
  `rustup component add clippy` before making it a hard gate.
