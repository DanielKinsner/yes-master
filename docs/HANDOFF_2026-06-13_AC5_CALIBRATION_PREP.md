# AC-5 Adaptive Compressor Calibration Prep

Status: prep only. AC-5 has not started. The adaptive-compression default
gate remains OFF, no private owner fixtures were run, and no DSP snapshots
were updated.

## Runtime-Togglable Build Plan / Status

Current code status:

- Default gate is OFF: `ADAPTIVE_COMPRESSION` is initialized to `false` in
  `src-tauri/src/guardrails.rs`.
- Startup seed exists: `YES_MASTER_ADAPTIVE_COMPRESSION=1|true|on|yes`.
- Runtime command exists: `set_adaptive_compression(enabled)` and
  `adaptive_compression_enabled()`.
- Frontend API wrappers exist: `api.setAdaptiveCompression(enabled)` and
  `api.adaptiveCompressionEnabled()`.
- AC-4 proved mobile inheritance with the gate forced ON in synthetic dense
  bridge fixtures, matching desktop bit-for-bit.
- No owner-facing toggle exists. For the listening session, use an operator
  dev/diagnostic path to call the command at runtime. If that is not available
  in the packaged app, use two launch scripts with the env seed as a fallback,
  but treat that as a weaker A/B workflow and confirm before the owner sits down.

Session build plan:

1. Build from the same commit on all devices.
2. Confirm gate OFF at app start.
3. Confirm the session operator can switch OFF -> ON -> OFF without changing
   track, preset, intensity, loudness target, or transport state.
4. Keep Volume Match and export level behavior unchanged; judge only the
   compressor adaptation and documented loudness landing behavior.
5. Do not flip the default, lock constants, regenerate snapshots, or update
   PRODUCT/APP_BEHAVIOR until the owner listening notes exist.

## Fixture Queue

Private audio stays outside git. Use `private-audio-fixtures/` or another
local-only folder; do not commit filenames or rendered private masters.

Minimum queue:

| Bucket | Count | Purpose | Notes |
| --- | ---: | --- | --- |
| Already-mastered private tracks | 3-5 | Prove ON is more transparent on hot/limited/low-LRA sources | Include different genres if available. These are the critical AC-5 decision tracks. |
| Normal dynamic tracks | 2-3 | Prove ON is effectively identity on healthy dynamic sources | Prefer clean mixes with real crest and no heavy limiting. |
| Quiet/acoustic track | 1 | Prove the classifier does not overreact to sparse/quiet material | Keep one vocal/guitar or similarly exposed track. |
| Transient-forward / Punch case | 1 | Prove ON does not soften impact more than OFF | Can overlap with a normal dynamic track if it has clear drum/transient content. |
| iPhone spot-check subset | 2 | Prove mobile listening inherits the same behavior | One already-mastered case, one dynamic/transient case. |

Matrix per source:

- Presets: Universal, Loud, Clarity, Oomph.
- Intensities: 0.5 and 1.0.
- Gate states: OFF and ON.
- Record notes as `keep`, `adjust`, or `reject` against the specific constant
  likely responsible.

## Proposed Constant Sheet

All values below are current code proposals marked `TBD-CALIBRATION` in
`src-tauri/src/guardrails.rs`. They are not locked.

| Constant | Proposed value | What owner is judging | Adjustment note |
| --- | ---: | --- | --- |
| `ALREADY_MASTERED_HOT_LUFS` | `-10.0` LUFS | How hot a source must be before stand-down can trigger | Raise if normal loud mixes false-trigger; lower if obvious masters miss. |
| `ALREADY_MASTERED_TRUE_PEAK_DBBTP` | `-1.2` dBTP | How close to ceiling a source must be | Raise toward ceiling if false-triggering; lower if limited masters miss. |
| `ALREADY_MASTERED_LRA_LU` | `6.0` LU | Maximum LRA for already-mastered stand-down | Lower if dynamic tracks false-trigger; raise if compressed masters miss. |
| `ALREADY_MASTERED_BAND_PSR_DB` | `8.0` dB | Uniform low per-band PSR requirement for stand-down | Lower for stricter stand-down; raise if known masters miss. |
| `BAND_PSR_SOFT_DB` | `12.0` dB | Per-band density value where easing begins | Raise to ease sooner; lower to leave more sources at preset baseline. |
| `BAND_PSR_FULL_DB` | `8.0` dB | Per-band density value where easing reaches full amount | Raise for stronger easing on moderately dense bands; lower for less intervention. |
| `BAND_COMPRESSION_DENSITY_CAP` | `0.45` | Max density reduction at full adaptation | Lower if ON sounds too light; raise only if already-mastered sources still pump. |
| `BAND_THRESHOLD_LIFT_MAX_DB` | `4.0` dB | Max threshold lift at full adaptation | Lower if too transparent/under-compressed; raise if dense bands still clamp. |
| `BAND_RATIO_EASE_CAP` | `0.35` | Max ratio easing; floor ratio multiplier is `0.65` | Lower if punch is softened; raise if already-mastered sources still feel worked. |

Do not reopen Tier-1 adaptive voicing constants in this session; those were
owner-listened and accepted on 2026-06-11.

## Session Script

For each source x preset x intensity:

1. Start gate OFF and listen to the current validated preset baseline.
2. Switch gate ON without changing any other setting.
3. Already-mastered sources: ON should be audibly more transparent, with less
   pumping/density and no surprising loudness penalty beyond landing behavior.
4. Dynamic sources: ON should be indistinguishable from OFF.
5. Transient-forward sources: ON must not soften impact more than OFF.
6. Capture notes as constant-specific actions, not vague taste notes.

Owner decisions to capture before AC-5 code work:

- Keep or adjust each TBD constant above.
- Mode-pill label decision: keep `Preset`, or use an `Adaptive` label in the
  calibrated UI copy.
- Whether Phase-B `CONFIDENCE_GATING` default flips in the same sitting. If yes,
  do that in a separate commit from AC-5.

## Ask-The-Owner Message

Use this message before scheduling the calibration sitting:

> I have AC-1 through AC-4 landed with the adaptive compressor still gate-OFF
> by default. For AC-5 I need your listening session before I flip anything.
> Please queue private, local-only audio: 3-5 already-mastered/hot-limited
> tracks, 2-3 healthy dynamic tracks, one quiet/acoustic track, and one
> transient-forward track for Punch impact. We will A/B gate OFF vs ON across
> Universal, Loud, Clarity, and Oomph at intensities 0.5 and 1.0, then spot-check
> two cases on iPhone. I also need three owner decisions during the session:
> final constant adjustments, whether the UI mode pill should still say `Preset`
> or say `Adaptive`, and whether Phase-B `CONFIDENCE_GATING` should flip in the
> same sitting as a separate commit. Please do not put any private audio or
> rendered masters in git; keep them local and tell me the fixture folder when
> ready.

## AC-5 Commit Checklist After Owner Signoff

- Replace `TBD-CALIBRATION` markers with
  `LOCKED-BY-LISTENING(YYYY-MM-DD)` and the owner evidence note.
- Flip adaptive-compression default ON only if owner explicitly signs off.
- Regenerate DSP snapshots only in the AC-5 flip commit.
- Add gate-ON cases to the already-mastered matrix and re-run it.
- Run the slow fixture lane with `AMS_RUN_REAL_FIXTURE=1`.
- Update PRODUCT.md / APP_BEHAVIOR.md compressor canon.
- Update parity fixture/docs if any shared constant is consumed cross-language.
- Re-run the required desktop and mobile lanes, then commit AC-5 with the full
  lane output and listening evidence.
