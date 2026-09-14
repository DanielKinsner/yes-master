# Crossover refactor evidence

Branch: `codex/dsp-crossover-equivalence`, based on `a001db96`.

Owner decision, 2026-09-14: defer this refactor until after launch. Keep it on
this local branch; it is not part of launch integration.

The production compressor and the existing crossover test now call one private
`LR4State::split` routine. The eight filter calls, coefficients, arithmetic order,
state layout and processing position are unchanged. No preset or timing constants
changed. The test retains its original 44.1 kHz frequencies, warmup, measurement
length and 0.12 dB tolerance. Coefficients are prepared once outside its sample loop.

## Before/after evidence on this Windows machine

- A temporary unit-test probe captured the original and refactored full-chain
  output samples plus all crossover-filter and compressor-envelope state bits
  after every frame. It covered 96 cases, each 8192 frames: eight rates from
  32 to 768 kHz, mono/stereo, Universal/Tape/Oomph, manual linked/unlinked and
  compressor Off. Inputs included impulses, deterministic noise, silence,
  inherited-state coefficient updates and resets.
- Direct buffer comparison found all **94,371,840 bytes identical**. SHA-256:
  `c732aae09e406d6a03964190265fc73c6f4f7006a7103c0606c4c5bca9241fbe`.
- The temporary probe is not part of the application or committed test suite.
  The production helper contains no allocation; its return value is a fixed
  three-float array. Filter sequence and coefficient mapping were checked against
  the baseline. The local comparison establishes bit identity for these inputs
  and this compiler/platform, not every possible input or platform.
- Final Rust suite: **643 passed, 0 failed, 18 existing opt-in tests ignored**,
  with all four real-fixture contracts explicitly enabled and passing. The
  ignored tests include separate encoder qualification and diagnostic/report
  lanes; they are not counted as validated by this run.
- `cargo fmt --check`, strict `cargo clippy --all-targets -- -D warnings`, and
  `git diff --check` passed. No frontend, public command/type or bridge contract
  changed; no frontend or native bridge build is claimed.
- The existing `listening_chain_probe` processed the same representative
  eight-second private-fixture window before and after. All three output hashes
  matched. Full-chain mean over five runs was 0.117726 s before / 0.119771 s after
  at 48 kHz. This small timing difference is not a statistically established
  performance change; no speedup or audio-device deadline claim is made.

The final suite was run from the repository root with `CARGO_BUILD_JOBS=2`,
`AMS_RUN_REAL_FIXTURE=1`, and `AMS_REAL_FIXTURE_PATH` set to the file below:

```text
cargo test --manifest-path src-tauri/Cargo.toml --target-dir src-tauri/target/codex-rc -- --test-threads=1
```

These environment variables were scoped to the test command's shell process.

## Additional high-rate finding: pre-existing, unresolved

Experimentally extending the old flatness assertion beyond 44.1 kHz exposed a
60 Hz deviation of **-0.349635 dB at 384 kHz** and **+0.850603 dB at 768 kHz**.
The same diagnostic was then run against the original production compressor
from `a001db96`, with unity ratios/makeup to isolate its crossover. It reproduced
both failures. The underlying numerical cause has not been established.

This is not a refactor-induced output difference. The unsuccessful expanded
flatness assertion is retained as a local diagnostic and reported here; it is
not claimed as passing coverage. The committed test retains its original scope
and tolerance. No numerical correction, relaxed tolerance, snapshot update or
new permanent ignored test is included in this behavior-preserving change.
Investigate the high-rate behavior separately before claiming flatness there.

## Local evidence and boundaries

Ignored `test-output/dsp-crossover/` contains the temporary probe, original and
refactored captures, direct comparison JSON, original-code diagnostic log and
verification logs. These files do not travel with the branch.

The fixture lane uses the existing `private-audio-fixtures/So Ouch....wav` in
place; source SHA-256:
`d4087931e60c08e6e52f3fbd565b6e98c8bd1d89c5805fcd345c8b4408c1ec9f`.
No private audio is added to git. No installed app, audio device, device settings
or other agent's app is controlled. This work does not establish Mac, installed
playback, listening, exact-commit remote CI or release evidence.
