# 2026-07-03 Hardening Plan — "Hostile-Input Solid"

Owner-interviewed plan (grill session, 2026-07-03). This document is both the
execution plan and the handoff artifact: any agent (Fable or Opus) picking up
mid-push starts here. Branch: `harden/2026-07-03-hostile-input`.

## Owner decisions locked in this session

These were answered by the owner on 2026-07-03 and are **binding**. Several
resolve entries in `docs/OPEN_THREADS_AND_DECISIONS.md` (noted inline).

| # | Decision |
|---|---|
| D1 | **North star: hostile-input solid.** The app must survive anything a real user throws at it — corrupt/truncated/weird sources, absurd formats, I/O failure, UI abuse — with graceful, honest errors. DSP math gets a *verification pass* (owner has independent multi-stage output checks and believes the math is sound), not a re-derivation. |
| D2 | **Two-tier fix policy for DSP findings.** Objective correctness bugs (wrong formula, NaN path, missed inter-sample peak, truncated tail) are fixed immediately: each fix carries a mechanical test proving the old behavior wrong, snapshot regen in the same commit, and a line in the Spot-Listen Queue (below). Taste-shaped findings become listening notes only — zero code change. |
| D3 | **Scope: engine first; capture endpoint gets one security pass.** The Supabase beta-capture backend gets a focused one-time security review (RLS, rate limiting, validation, client-bundle secrets). This resolves ledger Part B Q11 in substance: the landing/web surface is in agent scope for security & verification purposes. |
| D4 | **Album Master promise (resolves ledger Part B Q8):** *"One coherent record — consistent loudness, one delivery format, honest per-track receipts, and nothing silently altered."* The user-chosen arc is expressive bonus, not core promise. DDP/cue/ISRC/gapless are explicitly out of v1. |
| D5 | **Riders approved (all four):** dead-code tail (ledger 11b), thread #13 verification, doc-accuracy checks (ledger 20/21), CSS styling-debt batch (ledger 11a — feasible now that a live browser preview exists). Sequenced last; droppable if usage runs out. |
| D6 | **Ruin taxonomy (owner's fears, in priority order):** (1) presets lacking distinct character while staying safe → build the preset-fingerprint harness; (2) "adaptive ruin" — the engine burying intentional content (e.g. deliberate high end) → mechanical bounded-adaptation proofs; (3) loudness landing inaccuracy → landing-accuracy verification. |
| D7 | **Album character system gated OFF by default.** The silent genre-inference system (`AcousticFolk`/`Transition`/`HeavyDjent`/`ReturnAcoustic` → per-track loudness pulls −1.25..+0.82 dB + EQ/width/warmth/intensity biases) gets the Adaptive Compressor treatment: default-OFF gate, code preserved, flips ON only after an owner listening session. The filename-keyword override dies (or is demoted to a visible suggestion) in any future flip commit. |
| D8 | **Per-track treatment already exists** (the album Override toggle) — harden it rather than build anything new. Richer per-track UI stays backlog. |
| D9 | **Override semantics = full sound exemption.** An overridden track renders with its own settings and its own loudness target — no arc offset, no character bias — but still in the album's delivery format (rate/depth/channels), and the manifest marks it honestly. |
| D10 | **Doc hygiene is in scope.** Canon docs (PRODUCT.md, APP_BEHAVIOR.md), the ledger, and stale comments get minimal-truth updates in the same commits as the behavior they describe. Full S5.4 canon rewrite stays owner-gated. |
| D11 | **Fable implements** after this plan lands, small commits, riders last. If the session dies, an Opus session resumes from this doc + the commit ledger at the bottom. |

## Findings so far (pre-audit, discovered during the grill)

- **F1 — Override promise broken (fix: A1).** The UI banner says
  "its own settings will be applied at export," and the frontend does send the
  overridden track's settings (`useTrackMaster.ts:1656`). But
  `track_override_album` never crosses into Rust — the album render layer
  (`album_render.rs:294-323`) unconditionally stacks the arc LUFS offset and
  the character bias (EQ/width/warmth/intensity) on top of whatever settings
  arrive. The render path cannot even know a track is overridden.
- **F2 — Silent album alteration (fix: A2).** The character system alters
  per-track sound with no UI, no receipt line, no listening signoff, from a
  taxonomy tuned on one specific record; filename keywords can force a label.
  Contradicts D4. Gate OFF per D7.
- **F3 — Stale comment (fix: rides A2).** `AlbumPanel.tsx:11` claims the album
  layer "only modulates the per-track LUFS target via arc + character" — it
  also applies 4-band EQ, width, warmth, and intensity biases.
- **F4 — Empty-path fallback (fix: A5).** `exportAlbumPlan` sends
  `source_path: sourceTrack?.path ?? ""` when a plan entry doesn't match a
  track. Verify backend behavior on empty path; make the failure honest.
- **F5 — Decode surface wider than documented (informs D-workstream).**
  Import uses symphonia's generic probe (`decode.rs`), so the accepted-input
  surface is every container/codec enabled in Cargo features — not just WAV.
  Enumerate features and cover each entry point (three separate probe sites
  in `decode.rs`).

### Findings from the 2026-07-03 DSP math audit (5 dimensions, adversarially verified)

- **F6 — RS-09 CONFIRMED objective bug → FIXED.** Every export dropped its
  final ~3 ms (limiter lookahead ring never flushed) and carried a ~3 ms
  silent lead-in. Fixed via `MasteringChain::flush_render_tail` on both
  export paths; ledger Part B Q17 closed as a fix.
- **F7 — `.aac` UI/backend mismatch — REFUTED by mechanical test.** The
  recon claimed raw ADTS `.aac` couldn't decode without an `adts` cargo
  feature; `tests/decode_surface.rs` generates a real ADTS file with ffmpeg
  and proves it decodes with stock features (symphonia 0.5's `aac` feature
  ships the AdtsReader — there is no separate `adts` feature in 0.5). The
  test stays as the permanent pin that every UI-advertised extension
  (wav/mp3/m4a/aac/flac/ogg) decodes end-to-end.
- **F8 — Analysis "air" band reads zero at 8 kHz / 11.025 kHz sources
  (uncertain, investigate).** `analysis.rs` 6-band edges put the air band
  above Nyquist for those rates. Reduce-only adaptation makes this SAFE
  (brightness underestimated → less trim), but analysis displays/fingerprints
  for such sources are skewed. Verify + decide whether to renormalize.
- **Everything else verified OK** (owner's "the math is there" confirmed):
  RBJ biquads exact; Nyquist clamp at 0.45·fs; K-weighting re-derived per
  sample rate (pinned vs published 48 kHz coefficients); BS.1770 two-pass
  gating correct; ebur128 crate for file-level LUFS; landing math correct in
  dB domain; Volume Match forced off on all export paths; rubato FFT
  resampler (anti-aliased both directions); BS.775-style downmix (LFE
  excluded); ceiling checks run post-resample on both paths; M/S width
  textbook + mono-safe; saturation odd (no DC); LR4 crossover sums flat;
  Compressor Off bypasses exactly the creative block. Minor taste-notes
  (ISP skip margin tightness, comment nits) recorded by the audit, no action.

## Spot-Listen Queue

Every commit that changes rendered bytes adds a line here. The owner runs
this queue at the next listening sitting. (Two-tier policy, D2.)

| Date | Commit | What changed | What to listen for |
|---|---|---|---|
| 2026-07-03 | A2 gate | Album renders no longer receive character-label loudness pulls (−1.25..+0.82 dB) or bias EQ/width/warmth/intensity moves — arc + source compensation only. Track Master unaffected. | Render a familiar album with tracks that used to trigger labels (heavy + acoustic mix); confirm the arc-only album still feels coherent and no track feels wrongly loud/quiet vs before. |
| 2026-07-03 | RS-09 flush | Every export (track + album) now contains its true final ~3 ms (previously silently dropped) and no longer has a ~3 ms silent lead-in — output is sample-aligned with the source. | Render a track with a hard-cut ending or a long reverb/fade tail; listen to the very end and confirm the tail completes naturally. Nothing else should sound different. |

---

## Workstream A — Album Master truth (first: has live findings)

Goal: make Album Master exactly match D4's promise, with proof.

- **A1 — Plumb override into the render path.** Add an `override_album: bool`
  (or equivalent) to `AlbumTrackRenderInput` (`types.rs`, `bindings.ts`,
  `api.ts`) so `album_render.rs` exempts overridden tracks from arc offset +
  character bias and keeps their own LUFS target, while still delivering in
  the album format. Manifest/receipt marks the track "override: own settings."
  Tests: `album_render.rs` unit tests (overridden entry gets zero shadowing),
  `contracts.rs`, `App.album-export.test.tsx` (override round-trips to the
  backend call), integration test in `useTrackMaster.integration.test.tsx`.
  **Touches shared types → run iPhone + Android lanes (CLAUDE.md rule).**
- **A2 — Gate the character system OFF by default.** Mirror the
  `ADAPTIVE_COMPRESSION` AtomicBool pattern from `guardrails.rs:72-106`
  (default OFF, `is_/set_` accessors, test lock, dev/env re-enable for the
  future listening session). Gate at the `infer_album_characters` call in
  `build_album_plan_with_names` → all-`None` characters → zero char-offsets
  and zero biases by construction. Gate-OFF identity test (plan with gate OFF
  == plan with characters stripped); gate-ON tests keep exercising the
  existing inference/bias unit tests directly. Fix the stale `AlbumPanel.tsx`
  comment (F3). Regen affected snapshots in the same commit; add Spot-Listen
  line ("album renders lose character biases — arc-only now").
- **A3 — Album promise proofs.** Extend `src-tauri/tests/album_sample_rate.rs`
  (or a new `album_promise.rs`): (1) loudness consistency — non-override,
  non-arc tracks land within tolerance of the album target; (2) format parity —
  every rendered track + the continuous file at album rate/depth/channels;
  (3) manifest truthfulness — per-track reported source/rendered rates,
  channels, LUFS match measured reality; (4) fold-down/upmix correctness
  (mono→stereo upmix gain, >stereo fold-down: verify the gain law used and pin
  it); (5) arc honesty — arc offsets applied exactly as planned, and visible
  in the manifest.
- **A4 — Canon minimal-truth updates.** PRODUCT.md album section gets D4's
  promise (replacing "pending owner definition"); APP_BEHAVIOR.md album
  section describes: arc (user-chosen), character system (built, gated OFF
  pending owner listening — same wording pattern as the Adaptive Compressor),
  override semantics (D9). Ledger updates: Q8 answered, Q11 answered, new
  owner-gated thread "album character listening + flip decision."
- **A5 — Hostile album inputs.** Tests for: empty/missing source path (F4),
  plan/track list mismatch, zero tracks, single track, duplicate track ids,
  missing analysis on one track, zero-duration track, absurd track count.
  Assert graceful `CommandError`s, no panic, no partial files left behind.

## Workstream B — Adaptive-ruin proofs (owner fear #2)

Goal: mechanical proof the adaptive layer can only *soften the chosen preset*,
never bury intent. Files: `guardrails.rs`, `profile_store.rs`, `confidence.rs`.

- **B1 — Bounded-adaptation property tests.** For randomized source profiles
  (proptest or seeded loops): trims are reduce-only relative to preset
  baseline (never flip sign, never exceed per-axis caps), zero confidence →
  no-op, Adapt Strength 0 → identity, trims monotone in confidence (the
  existing `compute_with_confidence` test at `guardrails.rs:1017` asserts one
  point of this — generalize).
- **B2 — Landing accuracy proofs.** Synthetic committable sources (generated
  in-test: shaped noise beds at several crest factors / sample rates) ×
  presets × targets: assert |rendered integrated LUFS − effective target| ≤
  tolerance, and true peak ≤ ceiling. This is the mechanical answer to ruin
  type "loudness inaccurate."
- **B3 — HF-burial proof (the owner's named nightmare).** Synthetic track with
  deliberate bright content (e.g. shimmer band at 12–16 kHz over a dark bed):
  assert the adaptive path can reduce the *preset's* brightness boost to zero
  but never cut below the preset-off baseline, and that Adapt Strength 0
  renders byte-close to non-adaptive.

## Workstream C — DSP math verification pass (two-tier, D2)

Read-first; fixes only for objective wrongness. Order by blast radius:

- **C1 — Limiter/true-peak** (`dsp.rs`): lookahead window vs attack
  correctness; oversampling factor for inter-sample peak detection (4× is the
  BS.1770/EBU floor — verify); ceiling honored on hostile signals (full-scale
  square, Nyquist-adjacent tones, DC-offset material) — add a "never exceeds
  ceiling + ε" battery. **RS-09**: classify the ~3 ms flush/tail question —
  if export truncates real audio in the lookahead buffer, that's tier-1
  (fix + snapshot regen + spot-listen); if benign, document and close ledger Q17.
- **C2 — EQ filters** (`dsp.rs`): coefficient formulas vs RBJ cookbook for
  each band type; stability at extreme sample rates (8 kHz, 11.025 kHz — the
  Nyquist-clamp cases the ledger names); denormal protection.
- **C3 — Loudness** (`analysis.rs`/`engine.rs`): BS.1770 K-weighting +
  gating spot-check against synthetic known-LUFS vectors (constructable:
  −23 LUFS 997 Hz sine per EBU tech 3341 cases).
- **C4 — Resampling + channel math** (`sample_rate.rs`, album fold-down):
  aliasing floor on a sweep through Nyquist; fold-down/upmix gain laws
  (center/LFE handling if any; mono upmix level) — pin whatever is correct.
- **C5 — Width/saturation** (`dsp.rs`): M/S encode/decode unity, mono
  compatibility at width extremes, saturation DC/asymmetry behavior.

Output: findings table appended here; tier-1 items fixed immediately.

## Workstream D — Hostile-input battery (owner fear #0)

- **D1 — Decode corpus.** In-test generated adversarial files (no binaries in
  git): truncated header/data, `fmt` lies (0 channels, 65535 channels, 0 Hz,
  9.6 MHz rates), data-chunk length lies, zero-length, 1-sample, NaN/Inf in
  float WAVs, denormal floods, DC, full-scale squares, huge-duration claims.
  Assert: no panic, no hang (bounded time), typed `CommandError`, no partial
  state poisoning the session. Cover all three probe sites in `decode.rs`.
- **D2 — Enumerate the real decode surface.** Audit `Cargo.toml` symphonia
  features; document exactly which formats import accepts in APP_BEHAVIOR.md;
  make sure each enabled codec path is behind the same error discipline.
- **D3 — Session/project JSON.** Extend the existing malformed-JSON coverage
  (`baee195` precedent): wrong types, huge files, unknown fields, path
  traversal in stored paths, non-UTF8. Restore must never crash or half-load.
- **D4 — Export I/O failure.** Read-only target dir, path collisions (the
  never-overwrite guarantee under rapid double-export), Windows path-length
  limits, unicode/emoji filenames, file locked by another process
  (Windows share-mode), disk-full where simulable. Assert honest errors and
  no clobbered/corrupt output.
- **D5 — Runtime abuse.** Targeted review + tests around `audio.rs` chain
  swaps (recent `800e6ad` reuse path): rapid track switching during
  audition/render, cancel mid-export, double-export race, seek storms on long
  sources. Thread #13's `play_track` 5 s vs `play_master` 15 s timeout
  asymmetry gets verified here (rider, D5).

## Workstream E — Preset fingerprint harness (owner fear #1)

Goal: turn "do presets have enough character while staying safe" into numbers
so the Wave-10 listening sitting has instruments. Ledger thread #5 pairing.

- **E1 —** Committable synthetic fixture set (generated, deterministic: pink
  bed, drum-ish transient loop, tonal pad — no private audio in git).
- **E2 —** Fingerprint per preset: 6-band tilt delta vs Universal, DR/PSR,
  width, saturation THD proxy, landed LUFS. Emit distance matrix; assert
  (a) min pairwise character distance ≥ floor (presets stay distinct),
  (b) safety bounds (max tilt/DR deviation). Pin with tolerance goldens
  (Batch-F pattern — OS/arch independent).
- **E3 —** Owner-readable report (MD/CSV, git-ignored output dir) for
  listening sittings; re-runnable after any retune.

## Workstream F — Capture endpoint security pass (D3)

Read-only inventory via Supabase MCP: tables, RLS policies, edge functions,
anon-role grants. Check: RLS on + insert-only anon policy for capture table,
no select leak of collected emails, rate limiting / abuse posture, email
validation, no service-role key or secrets in the client bundle (grep landing
code + built assets), CORS. Fix repo-side issues; document Supabase-side
changes and apply the minimal ones (with owner-visible migration note).

## Workstream G — Riders (last; droppable in order)

1. **G1 — Thread #13 verification** (with D5) — report + ledger close.
2. **G2 — Dead-code tail (11b)** — six items, each with its test callers
   updated; mobile-entangled ones get the mobile lanes run.
3. **G3 — Doc-accuracy (20/21)** — IPHONE_APP_OVERVIEW preset names;
   ENGINE_REFERENCE stale pre-85%-lean numbers.
4. **G4 — CSS batch (11a)** — 10 items, with live browser-preview visual
   verification; the imprecise recs from the audit get re-derived from
   current CSS, not applied blind.

## Sequencing & verification

Order: **A → B → D → C (interleaved while in dsp.rs) → E → F → G.**
Small commits, one slice per commit where possible.

Per-slice lanes (CLAUDE.md): `npm test`, `npm run build`,
`npm run build:windows`, and in `src-tauri`: `cargo fmt --check`,
`cargo clippy --target-dir target\codex-rc --all-targets -- -D warnings`,
`cargo test --lib --target-dir target\codex-rc`,
`cargo test --target-dir target\codex-rc`.
Shared-type changes (A1 at minimum) additionally run the iPhone
(`apps/iphone-native/rust`: `cargo check --all-targets` + `cargo test`) and
Android (`apps/android-native/rust`: `cargo test` + `cargo ndk -t arm64-v8a
--platform 29 check`) lanes. Any DSP/export-affecting merge runs the slow
fixture lane (`AMS_RUN_REAL_FIXTURE=1 cargo test`).

## Commit ledger (updated as slices land)

| Slice | Commit | Status |
|---|---|---|
| Plan + ledger updates | `06c9ab3` | done |
| A2 — album character gate OFF | `0d4df90` | done — all lanes green |
| A1 — override full sound exemption | `f5c2ee2` | done — F1 fixed end-to-end (proof: `tests/album_override.rs`); mobile lanes green |
| A4 — canon doc truth | `4d854df` | done |
| A3a — loudness consistency proof | `31c2b5c` | done |
| A3b — manifest truthfulness proof | `9fad14e` | done |
| A5a — hostile error paths (fix + pins) | `11338c0`, `f222d0f` | done — missing-source error now names the file |
| A5b — no partial output on failure (fix + pin) | `c60fc64`, `4766edf` | done |
| B1/B3 — adaptive-ruin property proofs | `99592a5` | done — 6 proofs incl. the HF-burial nightmare |
| C — DSP math audit (workflow, 5 dims + refuters) | n/a (read-only) | done — findings F6–F8 above; 1 confirmed bug |
| RS-09 — limiter flush (F6) | `081e508`, `50e16ee` | done — all lanes + slow fixture lane green |
| Ledger/plan paperwork | `3501977` | done — Q17 + thread #13 closed |
| D2 — decode-surface pin (F7 refuted) | `1761d20` | done — every advertised format proven to decode (ffmpeg-generated, graceful skip) |
| D1 — decode boundary hardening (fix + corpus) | `1477766`, `8ef0692` | done — symphonia panic caught (0 Hz fmt crashed the parser!), 0-rate rejected, NaN/Inf sanitized |

**Next up (not yet started):** E — preset fingerprint harness (owner fear #1);
F — Supabase capture security pass; D3 session-JSON extension; D4 export-I/O
failure battery; D5 runtime-abuse review (`audio.rs` chain swaps); B2
track-master landing matrix; G riders (dead-code 11b, doc-accuracy 20/21,
CSS 11a with browser preview). F8 (low-rate air-band) awaits investigation.
