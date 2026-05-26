# Private Audio Fixtures

Use this convention when testing Album Mastering Studio with real user audio.

Private audio must never be committed. The repo ignores:

```text
private-audio-fixtures/
```

## Folder Shape

```text
private-audio-fixtures/
  manifest.json
  clean-full-mix.wav
  rough-problem-track.wav
  acoustic-quiet-track.wav
  heavy-dense-track.wav
  album-sequence-01.wav
  album-sequence-02.wav
```

The filenames above are examples. Use whatever local names are convenient.

## Manifest Template

```json
{
  "version": 1,
  "notes": "Private local fixtures for Album Mastering Studio. Do not commit audio.",
  "fixtures": [
    {
      "id": "clean-full-mix",
      "path": "clean-full-mix.wav",
      "purpose": "A finished mix that already sounds decent.",
      "mode": ["track"],
      "quick_test": true,
      "slow_test": true,
      "listening_focus": ["overall polish", "source/master A-B", "export checks"],
      "known_issues": []
    },
    {
      "id": "rough-problem-track",
      "path": "rough-problem-track.wav",
      "purpose": "A mix with harshness, mud, clipping, dullness, or other known problems.",
      "mode": ["track"],
      "quick_test": true,
      "slow_test": true,
      "listening_focus": ["quality warnings", "safe Universal behavior", "advanced controls"],
      "known_issues": ["describe what bothers you before the app touches it"]
    },
    {
      "id": "album-sequence",
      "paths": ["album-sequence-01.wav", "album-sequence-02.wav"],
      "purpose": "Adjacent album tracks for sequence, role, boundary, and consistency testing.",
      "mode": ["album"],
      "quick_test": false,
      "slow_test": true,
      "listening_focus": ["track-to-track loudness", "boundaries", "album cohesion"],
      "known_issues": []
    }
  ]
}
```

## Rules For Agents

- Use real fixtures for local automated analysis/render checks when available.
- Use real fixtures for manual listening notes when judging product quality.
- Do not commit private audio, rendered masters from private audio, waveform images derived from private audio, or fixture-specific generated artifacts.
- Do not assume fixture files exist. If missing, fall back to synthetic tests and say that real-audio verification is still pending.
- Prefer short clips for quick loops and full tracks/albums for slow verification.

## Already-Mastered Matrix

Use this local-only runner to measure already-mastered source material across
the release-stabilization preset/compressor matrix:

```powershell
cd src-tauri
cargo run --example private_fixture_matrix -- --manifest ..\private-audio-fixtures\manifest.json --output ..\test-output\private-fixture-matrix
```

The runner uses only fixtures with a singular `path` field and `track` mode.
It renders:

- Universal / Preset compressor.
- Universal / Compressor Off.
- Loud / Preset compressor.
- Loud / Compressor Off.
- Clarity / Preset compressor.
- Clarity / Compressor Off.

It writes:

- `already-mastered-matrix.json`
- `already-mastered-matrix.csv`
- rendered WAVs under `renders/`

All of those output locations are ignored by git. Commit the harness and
aggregate interpretation only; do not commit private audio, rendered private
masters, or fixture-specific output ledgers.

## Private Reference Tuning

Use this local-only runner when comparing YES Master presets against private
reference masters from an external mastering service:

```powershell
cd src-tauri
cargo run --example private_reference_tuning -- --references "..\tests for presets" --output ..\test-output\private-reference-tuning
```

The reference directory should contain one source file named
`<track>-original-test.wav` and matching external reference masters named:

- `<track>-universal-test.wav`
- `<track>-clarity-test.wav`
- `<track>-oomph-test.wav`
- `<track>-tape-test.wav`

The runner renders the source through YES Master using the same preset names
with the preset compressor active, then writes:

- `reference-tuning-report.json`
- `reference-tuning-report.csv`
- rendered YES Master WAVs under `renders/`

The CSV reports the source-to-reference move, the source-to-YES move, and the
remaining YES-minus-reference gap for loudness, dynamic range, spectral balance,
transient density, stereo width, and optional density/correlation metrics when
analysis provides them.

All of those output locations are ignored by git. Commit code, docs, and any
non-private aggregate conclusions only; do not commit private audio, rendered
private masters, or private ledgers.
