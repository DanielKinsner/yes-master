# Windows E2E Follow-up Plan — 2026-07-13

Source report:
`docs/reviews/2026-07-13-windows-e2e-report.md`

## Status

The Windows E2E run found **no new objective code defect**. There is no immediate
patch slice. This plan is deliberately conditional so a successor does not turn
an owner-ear acceptance question into speculative audio-engine surgery.

## Step 1 — owner recheck on the stamped build

Use installed YES Master `0.9.0`, receipt/build stamp `0a16d7a+`.

1. Play the same representative track in Mastered mode.
2. Toggle Original/Mastered at a steady rhythm with Volume Match OFF.
3. Repeat with Volume Match ON.
4. Rapid-toggle six times and confirm no timeout/error toast.
5. Record one of: `PASS — inaudible/acceptable`, `FAIL — audible full dip`, or
   `FAIL — transport/error regression`, with Volume Match direction and rough
   duration.

If PASS, update the owner listening note / go-no-go ledger only. No code change.

## Step 2 — conditional engineering slice only after a FAIL

Before editing:

1. `git pull --ff-only` on `main`.
2. Re-read `docs/PRODUCT.md`, `docs/APP_BEHAVIOR.md`, `docs/ARCHITECTURE.md`,
   `docs/TESTING.md`, `docs/RELEASE_STABILIZATION.md`, the beta execution plan,
   this plan, and the source report.
3. Reproduce the exact failed direction on the stamped/current build.
4. Add measurement instrumentation behind a dev/test-only seam; do not restore
   production diagnostics.

Implementation boundary if the audible dip is proven:

- Plan a single-stream/sample-synchronous Original/Mastered switch, or a
  measured latency-compensated handoff, at the playback seam.
- Keep the playhead authoritative and monotonic.
- Preserve the working Volume Match gain cache and latest-request guard.
- Do not touch render/export DSP, LUFS landing, preset calibration,
  `TBD-CALIBRATION` values, or owner-gated defaults.
- Add a regression oracle that proves first-frame audibility, no near-zero
  overlap at the device handoff, unity behavior for correlated same-track
  signals, and no stale error under overlapping requests.

## Step 3 — verification for a proven patch

Run:

```powershell
npm test
npm run build
npm run build:windows
npm run verify:rust
```

Because this is audition-path work, also run the private-fixture slow lane from
`src-tauri` and repeat Step 1 by ear on the newly stamped installed build.

Commit in very small green chunks and push directly to `main` only after the
mechanical lane and owner-ear acceptance both pass.

## Stop rule

If the owner does not report a FAIL, or the successor cannot reproduce an
objective transport/error defect, stop with a no-op. Do not infer permission to
redesign the crossfade or retune sound.
