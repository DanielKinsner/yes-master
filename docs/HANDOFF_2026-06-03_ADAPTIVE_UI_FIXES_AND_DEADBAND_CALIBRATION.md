# Handoff — Adaptive DSP UI fixes + the deadband-calibration decision (2026-06-03)

Audience: the next agent/model (or Dan on a different machine). This is written to
be read COLD and acted on WITHOUT re-investigating. Everything below was verified
against the tree this session. **Do not go hunting for other root causes for the
three reports in §1 — they are fully diagnosed here.** The only open item is a
*taste/calibration* decision that is **Dan's to make by ear** (§4); do not change
those constants without him.

> Branch: `main`. All work below is committed + pushed. Working tree clean.
> This session's commits (newest last):
> - `9529e0f` fix(adaptive-dsp): Reset restores Adapt Strength to default (0.6)
> - `292f3fa` test(adaptive-dsp): update reset assertion for Adapt Strength default
> - `d26cd30` feat(adaptive-dsp): surface deadband thresholds on GuardrailReadout
> - `2cb097f` feat(adaptive-dsp): group Adapt Strength + a legible per-axis trim readout
>
> Earlier this same day, B2 (backend-owned source_profile) and B5 (export-receipt
> traceability) landed and were fully validated — see
> `docs/HANDOFF_2026-06-03_ADAPTIVE_DSP_TIER2.md` §6/§7 and
> `docs/ADAPTIVE_DSP_NEXT_STEPS.md`. Those are DONE; do not redo them.

---

## 1. The three reports and their exact disposition

Dan reported (on a Mac build of the app, mastering "The Keeper"):

1. **"Hitting reset doesn't change the Adapt Strength."**
   → **REAL UI BUG. FIXED this session** (commit `9529e0f`).

2. **"As you slide Adapt Strength it's not modifying the trim percentages shown in
   the EQ (Highs/Lows); same with Width."**
   → **NOT a wiring bug.** The slider→readout path is live and correct (traced
   end-to-end, see §3). The EQ/Width percentages don't move because **the source
   sits inside the deadbands**, so those axes legitimately read `-0%` at *every*
   strength. Only the axes the source actually crosses (here: density/Comp) move
   with the slider. The fix for "feels inert" is **legibility** (done this session,
   §2) plus an optional **deadband calibration** that is **taste-gated** (§4).

3. **"Loud preset shows nothing — at full Adapt Strength it's not adding or
   subtracting anything."** (Readout showed `Highs -0% Lows -0% Comp -10% Width -0%`,
   same as other presets.)
   → **Same root cause as #2** (source inside deadbands) **plus two structural
   facts about the Loud preset** (§4.3): Loud's EQ boosts are tiny, and its
   low-shelf boost is *below the character floor* so the Lows axis can **never**
   trim on Loud regardless of source. Not a bug; documented for calibration.

Dan also asked: **"Can we get the text metadata for the trimming side-by-side with
the slider."** → **DONE this session** (commit `2cb097f`, §2).

**Bottom line:** one real bug (fixed), one UX/legibility gap (fixed), and one
taste-gated calibration question (left for Dan, fully specified in §4).

---

## 2. What changed this session (exact, file:line)

### 2.1 Reset now restores Adapt Strength to its default — `9529e0f`
- `src/App.tsx` → `resetAdvancedControls` (inside `AdvancedControlsCard`, ~L2061).
  Previously it spread `...a` and null-swept every Advanced field **except**
  `adaptive_strength` (a deliberate B4 omission with an in-code comment). That was
  the bug: every other control reset to default, Adapt Strength did not.
- Now the reset payload includes `adaptive_strength: ADAPTIVE_STRENGTH_DEFAULT`
  (= `0.6`, from `src/bindings.ts:277`). We write the **explicit** default, **not
  `null`** — `null` also resolves to the default downstream but displays
  ambiguously, and the durable "off" is `0.0`. (B4's real hazard was "writing null
  silently re-arms adaptation," not "reset must never touch strength.")
- Tests: `src/App.adaptive-strength.test.tsx` ("resets Adapt strength to the
  explicit default (0.6) on Reset") and the exact-payload assertion in
  `src/App.compressor-mode.test.tsx` ("resets the advanced controls section…",
  ~L260) now include `adaptive_strength: ADAPTIVE_STRENGTH_DEFAULT`.
  - ⚠️ Process note: `9529e0f` was pushed after running only the *targeted*
    adaptive-strength test, which let the compressor-mode exact-payload assertion
    break on `main` for one commit; `292f3fa` restored green. **Always run the full
    `npm test` before pushing any FE change** — several tests assert exact reset
    payloads.

### 2.2 Deadband thresholds surfaced on the readout — `d26cd30`
- `src-tauri/src/guardrails.rs` → `GuardrailReadout` gained
  `bright_deadband`, `low_deadband`, `width_corr_deadband` (all `f32`,
  `#[serde(default)]`). `readout_for` fills them from the module constants
  `BRIGHT_DEADBAND` / `LOW_DEADBAND` / `WIDTH_CORR_DEADBAND` in **both** arms
  (active and inactive). This is pure display plumbing — **no DSP/calibration
  change**; it just exposes existing constants so the UI never hardcodes them.
- `src/bindings.ts` → `GuardrailReadout` mirrors the three new optional fields.
- Tests: `guardrails.rs` `readout_carries_deadbands_even_when_inactive` +
  assertions appended to `readout_reports_trims_and_context_when_active`.

### 2.3 Adapt Strength + legible per-axis readout, grouped — `2cb097f`
- `src/App.tsx`:
  - The **Adapt Strength** `NumberField` was moved OUT of the `advanced-grid` into
    its own block (`<div className="adaptive-block …">`) immediately followed by
    the readout, so the slider and "what it's doing" are one visual unit (Dan's
    "side-by-side" request). The grid above now holds 8 fields (still even / 2-col).
  - New `AdaptiveReadout` component (defined just above `AdvancedControlsCard`).
    Per axis it shows the **realized trim** AND the **source context vs that axis's
    deadband**, e.g.:
    ```
    Adaptive trims (chain, pre-landing)
    Highs -0%    presence+air 0.27 / 0.30 · in range
    Lows  -0%    sub+low 0.38 / 0.42 · in range
    Comp -10%    DR 7.2 dB
    Width -0%    corr 0.85 / 0.50 · in range
    ```
    - bright/low show `share / deadband`; they trim when the share **exceeds** the
      deadband. width shows `corr / deadband`; it trims when correlation is
      **below** the deadband (lower correlation = wider). Comp shows the DR (the
      `>= 100 dB` "no-DR" sentinel renders as `DR n/a`; mono width renders `mono`).
    - `· in range` is appended when an axis trimmed `0%` AND the source is on the
      no-trim side of its deadband — so `-0%` reads as "source already neutral on
      this axis," not "the control is broken."
  - The album note ("Adaptive applies to Track Master export, not Album renders.")
    moved into the same block; it shows instead of the readout in album mode.
- Tests: `src/App.adaptive-strength.test.tsx` readout test now asserts the context
  lines (`presence+air 0.34 / 0.30`, `sub+low 0.30 / 0.42`, `DR 4.0 dB`,
  `corr 0.30 / 0.50`) and the `· in range` flag.

---

## 3. Why bug #2 is NOT a wiring bug (so nobody re-investigates it)

The slider→readout path was traced end-to-end and is **live and correct**:

1. Slider `onChange` → `update("adaptive_strength", v)`
   (`src/App.tsx` `Adaptive... NumberField`, in the new adaptive block).
2. `update` (`src/App.tsx` `AdvancedPanel`, ~L1960) → `onAdvanced({ ...a, [field]: value })`.
3. `onAdvanced` is wired to `tm.setAdvanced` (`src/App.tsx:119`).
4. `setAdvanced` (`src/hooks/useTrackMaster.ts:1052`) → `updateSettings(... applyAdvancedWithProfileFlip ...)`.
5. `applyAdvancedWithProfileFlip` (`src/lib/settings-transitions.ts:68`) returns a
   **new** `MasteringSettings` (`adaptive_strength` is not in `SHADOWED_ADVANCED_KEYS`,
   so it passes through with no profile flip).
6. `updateSettings` (`src/hooks/useTrackMaster.ts:744-746`) does
   `setSettingsMap(prev => ({ ...prev, [id]: nextSettings }))` → new map entry.
7. `selectedSettings` (`src/hooks/useTrackMaster.ts:556-558`) reads
   `settingsMap[selectedTrackId]` directly (no `useMemo`) → new identity.
8. The readout effect (`src/hooks/useTrackMaster.ts:573-589`) lists
   `selectedSettings` in its dep array, has **no debounce**, and uses a latest-wins
   `guardrailReadoutReq` ref (not a cache) → it re-invokes `guardrail_readout` on
   every strength edit.
9. `guardrail_readout` (`src-tauri/src/guardrails.rs`, the `#[tauri::command]`)
   runs `apply_resolved_profile(&mut settings, store.get(track_id), album)` then
   `readout_for(&settings)`, which reads
   `settings.advanced.adaptive_strength.unwrap_or(ADAPTIVE_STRENGTH_DEFAULT)`.
10. `SourceGuardrails::compute` multiplies each axis's `*_raw` trigger by
    `strength` (`src-tauri/src/guardrails.rs:96-143`).

**The math is why EQ/Width look frozen:** each axis multiplier is
`1.0 - (raw * strength).min(CAP)`. When the source sits in a deadband, `raw = 0`,
so `(0 * strength) = 0` for **any** strength → multiplier `1.0` → trim `0%`. The
slider only modulates axes whose source value crossed the trigger. For "The
Keeper," that is the density (Comp) axis only — which is exactly what Dan saw
(`Comp -10%`, the rest `-0%`). `Comp -10%` proves `active = true` (a profile is
resolved), which also rules out any store-miss / TrackId-mismatch theory.

**Do not "fix" this in the FE.** The FE is correct. The lever is the deadbands (§4).

---

## 4. THE DECISION FOR DAN — deadband calibration (TASTE-GATED, by ear)

This is the substantive item behind reports #2 and #3. **It is a DSP calibration
decision and must be made by ear with the private fixture set. Do not change these
constants without Dan's explicit go-ahead and a listening pass.** They were
*deliberately* set to their current values during a listening session earlier
(see the in-code comments), so changing them blindly would undo that work.

### 4.1 The exact constants (the ONLY place to tune) — `src-tauri/src/guardrails.rs`
```
L18  ADAPTIVE_STRENGTH_DEFAULT = 0.6
L36  BRIGHT_DEADBAND      = 0.30   // presence+air must EXCEED this to trim Highs
L38  BRIGHT_EXCESS_FULL   = 0.12   // share above deadband that maps to full trim
L41  LOW_DEADBAND         = 0.42   // sub+low must EXCEED this to trim Lows
L43  LOW_EXCESS_FULL      = 0.15
L46  DENSITY_DR_SOFT_DB   = 8.0    // P95-P10 DR (dB) where density trim begins
L47  DENSITY_DR_FULL_DB   = 3.0    // …reaches full
L49  DENSITY_LRA_SOFT_LU  = 6.0    // LRA (LU) where density trim begins
L50  DENSITY_LRA_FULL_LU  = 3.0
L54  WIDTH_CORR_DEADBAND  = 0.50   // correlation must be BELOW this to trim Width
L55  WIDTH_CORR_FULL      = 0.20
L60  EQ_CAP      = 0.50            // max fraction of an EQ move removed
L62  DENSITY_CAP = 0.60
L62  WIDTH_CAP   = 0.70
L66  EQ_BOOST_FLOOR_DB = 0.5       // positive EQ boosts never trimmed below this
```
(Line numbers as of `2cb097f`; if they drift, the constants are the
`const … : f32 = …;` block at the top of `guardrails.rs`.)

### 4.2 The diagnosis for "The Keeper" — confirm it in 5 seconds in the UI
The new readout now prints the source shares vs deadbands. **Have Dan look at the
Adapt Strength block in the app:**
- If `presence+air` reads **below 0.30** → the source is genuinely not bright; the
  `-0%` on Highs is correct. To make the feature act, **lower `BRIGHT_DEADBAND`**.
- If `sub+low` reads **below 0.42** → not boomy; `-0%` on Lows is correct.
- If `corr` reads **above 0.50** → not wide; `-0%` on Width is correct.
- `Comp` shows `DR x.x dB`; density is the one axis firing (the screenshot showed
  source dynamic range ~5.8 LU, which crosses `DENSITY_LRA_SOFT_LU = 6.0`).

In other words: with the current numbers, the adaptive system is **working as
designed** — it is *defensive* and does little on already-well-balanced material.
Whether it should do *more* is a taste call.

### 4.3 Two structural facts about the Loud preset (report #3)
`PRESET_LOUD` (`src-tauri/src/dsp.rs:542-553`): `sub_db=0.0`, `low_shelf_db=0.4`,
`low_mid_db=-1.6`, `presence_db=1.8`, `high_mid_db=0.0`, `air_db=1.2`,
`sparkle_db=0.0`, `stereo_width=1.03`.
- The readout's **bright pool** is `[high_mid_db, air_db, sparkle_db] = [0, 1.2, 0]`
  → only the 1.2 dB air boost can ever be trimmed (and only if the source is
  bright). The **low pool** is `[sub_db, low_shelf_db] = [0, 0.4]`.
- **`low_shelf_db = 0.4 < EQ_BOOST_FLOOR_DB = 0.5`**, so `floor_boost`
  (`guardrails.rs:186-195`) returns it unchanged → **the Lows axis can NEVER trim
  on Loud, for any source.** This is by-design (don't trim a boost that's already
  below the character floor), but it means `Lows -0%` on Loud is structural, not
  the source. If Dan wants Loud's low axis to be able to engage on a boomy source,
  the (taste-gated) options are: raise `PRESET_LOUD.low_shelf_db` to `>= 0.6`, or
  lower `EQ_BOOST_FLOOR_DB` — both are preset/character calibration.
- `stereo_width = 1.03` is only 0.03 above neutral, so even a fully-wide source
  yields a microscopic width trim on Loud.

### 4.4 Calibration options (in order of preference) — DO NOT apply without Dan
- **(Preferred, principled) Tilt-vs-reference brightness metric (B6).** Replace the
  absolute presence+air deadband with a spectral-slope-vs-pink comparison, so a
  flat spectrum reads as zero excess *by construction* and the deadband number
  becomes ~irrelevant. This is the planned fix; see
  `docs/plans/2026-06-02-002-adaptive-dsp-tier1-finish-and-tier2.md` §1 and
  `docs/ADAPTIVE_DSP_NEXT_STEPS.md` ("Tilt-vs-reference brightness metric"). Bigger
  change; needs measured-neutral data from Dan's references.
- **(Quick, blunt) Lower the deadbands** so realistic masters trip the triggers:
  candidate ranges (NOT decisions — for Dan to A/B):
  `BRIGHT_DEADBAND 0.30 → ~0.26`, `LOW_DEADBAND 0.42 → ~0.38`,
  `WIDTH_CORR_DEADBAND 0.50 → ~0.55–0.60`, optionally `DENSITY_DR_SOFT_DB 8.0 → ~10.0`.
  ⚠️ The comment at `guardrails.rs:30-36` records that `BRIGHT_DEADBAND` was raised
  **0.20 → 0.30** earlier today specifically because 0.20 over-trimmed neutral
  masters by ~39%. Reversing toward 0.20 re-introduces that. This is precisely the
  by-ear tradeoff only Dan can settle.

### 4.5 How to validate any deadband change (mandatory process)
1. Edit only the constants in `src-tauri/src/guardrails.rs` (the single tuning file).
2. `cd src-tauri && cargo test --lib --target-dir target/codex-rc` — the unit tests
   at `guardrails.rs` (e.g. `bright_source_trims_only_air`, `neutral_source_is_identity`)
   use contrived sources and should stay green; if one flips, you changed behavior
   the test pins — update intentionally, don't paper over.
3. The REAL gate is the listening + slow-fixture lane (private audio, not in git):
   ```powershell
   cd src-tauri
   $env:AMS_RUN_REAL_FIXTURE = "1"
   cargo test
   Remove-Item Env:\AMS_RUN_REAL_FIXTURE
   ```
   plus A/B in the app: Adapt 0 vs 60% on already-mastered / bright / dense / wide
   AND neutral sources; confirm neutral still does ~nothing.
4. Do not ship a deadband change without Dan signing off by ear.

---

## 5. What is verified GREEN right now (as of `2cb097f`, then full validation)

Run from repo root `C:\Users\Daniel Kinsner\OneDrive\Documents\GitHub\yes-master`:
- `npm test` → **158 passed** (19 files).
- `npm run build` → clean (tsc + vite).
- `cd src-tauri && cargo test --lib --target-dir target/codex-rc` → **248 passed**;
  all `preset_byte_identity` snapshots green.
- `cargo test --target-dir target/codex-rc` (full incl. integration) → all green;
  2 ignored = the private-fixture slow lane (needs `AMS_RUN_REAL_FIXTURE=1`).
- `cargo clippy --all-targets --target-dir target/codex-rc -- -D warnings` → clean.
- `npm run build:windows` → MSI + NSIS bundles built.
  - GOTCHA: after `build:windows`, `git status` shows `src-tauri/Cargo.toml`
    modified — it is **EOL-only** (CRLF). Confirm with
    `git -c core.autocrlf=false diff --quiet src-tauri/Cargo.toml` (no output =
    EOL-only) then `git restore src-tauri/Cargo.toml`. (`--ignore-all-space` does
    NOT detect this; use the autocrlf form above.)

Working tree is clean and everything is pushed to `main`.

---

## 6. Precise next actions (ordered; each scoped — do exactly these, nothing else)

1. **Dan: look at the new Adapt Strength readout in the app** on "The Keeper" and
   read the `presence+air X.XX / 0.30`, `sub+low X.XX / 0.42`, `corr X.XX / 0.50`
   lines. This tells you definitively whether the `-0%` axes are correct (source in
   range) or whether the deadbands are mis-sized for material you hear as
   bright/wide. (Answers the one open question from the investigation.)
2. **If (and only if) Dan decides the feature should act more** on typical
   material: pick a path in §4.4 and follow the validation process in §4.5. Prefer
   the tilt-vs-reference metric (B6) over blunt deadband lowering. **A model must
   not pick these numbers; bring options to Dan.**
3. **If Dan wants Loud's Lows to be able to engage**: §4.3 (raise
   `PRESET_LOUD.low_shelf_db` to ≥0.6 OR lower `EQ_BOOST_FLOOR_DB`) — preset/character
   calibration, by ear, same validation process.
4. Everything else on the adaptive roadmap (Tier-2: measured-neutral, PSR closed
   loop, total-loudness-loss budget B3, stereo_width co-trigger B7) remains as
   documented in `docs/ADAPTIVE_DSP_NEXT_STEPS.md`. Not touched this session.

### Do NOT do
- Do NOT re-investigate the slider→readout wiring (§3 — it is confirmed correct).
- Do NOT change any constant in `guardrails.rs` or `PRESET_*` in `dsp.rs` without
  Dan's by-ear signoff (taste-gated).
- Do NOT revert the Reset behavior (§2.1) — Reset restoring the 0.6 default is the
  intended fix; the B4 concern is satisfied by writing the explicit value, not null.
- Do NOT add adaptive profile injection for Album — album is intentionally flat
  (backend strips + `album` gate); this is unchanged.

---

## 7. Pointers
- This session's UI code: `src/App.tsx` (`AdaptiveReadout`, `AdvancedControlsCard`,
  `resetAdvancedControls`), `src/hooks/useTrackMaster.ts` (readout effect L573-589),
  `src/bindings.ts` (`GuardrailReadout`).
- DSP + the ONLY tuning file: `src-tauri/src/guardrails.rs`. Preset tables:
  `src-tauri/src/dsp.rs` (`preset_calibration` / `PRESET_*`).
- Backend-owned profile (B2) + receipt traceability (B5):
  `docs/HANDOFF_2026-06-03_ADAPTIVE_DSP_TIER2.md`, `docs/ADAPTIVE_DSP_NEXT_STEPS.md`.
- Tier-1 finish / Tier-2 options (incl. tilt-vs-reference):
  `docs/plans/2026-06-02-002-adaptive-dsp-tier1-finish-and-tier2.md`.
- Build/test contract: `CLAUDE.md` + §5 above.
