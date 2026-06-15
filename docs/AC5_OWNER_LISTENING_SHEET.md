# AC-5 Listening Sheet (for Dan)

Plain-language companion to `HANDOFF_2026-06-13_AC5_CALIBRATION_PREP.md`.
That doc is the full spec; this one is "what you actually do." Nothing here
changes the app — the adaptive compressor stays OFF by default until you sign off.

## The one-sentence why

The adaptive compressor is built and gated OFF. Your ears decide two things:
(1) does turning it **ON** make already-mastered/hot tracks sound *more
transparent* (less pumping) without hurting healthy dynamic tracks, and (2) what
the nine tuning numbers should be. Until you do this, it ships OFF and sounds
exactly like today.

## Before you sit down (I handle this — ~15 min pre-flight)

- Build every device (Mac, Windows, iPhone) from the **same commit**.
- Confirm the gate reads **OFF** at startup.
- Confirm I can flip **OFF → ON → OFF at runtime without changing the track,
  preset, intensity, loudness, or playhead** (see "How we toggle" below). If the
  clean runtime path isn't available in your build, I fall back to two launch
  modes (env seed) — but I'll confirm the instant-toggle path works first, because
  same-playhead A/B is the whole point.
- Confirm your private fixture folder is staged locally (never committed).
- Print the constant sheet (below) so notes map to a specific knob.

## How we toggle (verified in code)

- Runtime, no rebuild: `api.setAdaptiveCompression(true|false)` →
  `set_adaptive_compression` command flips the gate live. This is the path that
  lets us A/B the *same moment* of a song.
- Fallback only: launch with `YES_MASTER_ADAPTIVE_COMPRESSION=1` for ON, without
  it for OFF. Works anywhere but needs a relaunch between A and B (weaker).

You won't type commands — I drive the toggle while you listen and call it.

## What to queue (private audio, local only — never in git)

| Bucket | How many | Why it matters |
| --- | ---: | --- |
| Already-mastered / hot / limited tracks | 3–5 | **The decision tracks.** ON should sound more open, less pumped. |
| Healthy dynamic mixes | 2–3 | ON should be basically *identical* — proof it doesn't touch good material. |
| Quiet / acoustic (1 exposed vocal or guitar) | 1 | Proof the detector doesn't overreact to sparse material. |
| Transient-forward / "Punch" track | 1 | ON must **not** soften the hits more than OFF. |
| iPhone spot-checks | 2 | One mastered, one dynamic — proof mobile matches desktop. |

## The routine (per track)

For each track, walk: **Universal / Loud / Clarity / Oomph**, each at **intensity
0.5 and 1.0**:

1. Play with gate **OFF** (today's sound).
2. I flip **ON** at the same spot — nothing else changes.
3. Call it: **keep / adjust / reject**, and if "adjust," roughly which way.

Listen for:
- **Already-mastered:** ON = more transparent, less density/pumping, no weird
  loudness drop. (This is the win we're chasing.)
- **Dynamic:** ON ≈ OFF. If you can clearly hear it working, that's a flag.
- **Punch:** transients stay as punchy as OFF.

## The nine knobs (so "adjust" maps to a number)

| If you hear… | The knob | Current proposal |
| --- | --- | ---: |
| Normal-loud mixes wrongly treated as "mastered" | `ALREADY_MASTERED_HOT_LUFS` | −10.0 LUFS |
| …same, on near-ceiling tracks | `ALREADY_MASTERED_TRUE_PEAK_DBBTP` | −1.2 dBTP |
| Dynamic tracks wrongly treated as "mastered" | `ALREADY_MASTERED_LRA_LU` | 6.0 LU |
| Stand-down triggering too easily/rarely (per-band) | `ALREADY_MASTERED_BAND_PSR_DB` | 8.0 dB |
| Easing starts too late / too early | `BAND_PSR_SOFT_DB` | 12.0 dB |
| Easing reaches full too weakly / strongly | `BAND_PSR_FULL_DB` | 8.0 dB |
| ON sounds too light → lower; still pumps → raise | `BAND_COMPRESSION_DENSITY_CAP` | 0.45 |
| Too transparent → lower; still clamps → raise | `BAND_THRESHOLD_LIFT_MAX_DB` | 4.0 dB |
| Punch softened → lower; still feels worked → raise | `BAND_RATIO_EASE_CAP` | 0.35 |

(Do **not** reopen the Tier-1 voicing constants — you already accepted those 2026-06-11.)

## Three decisions to capture (these are what unblock the commit)

1. **The nine numbers** — keep each, or adjust (roughly which way).
2. **Mode-pill label** — should the UI still say **"Preset"**, or say **"Adaptive"**
   once this is on? (You can mull this now.)
3. **Phase-B confidence gating** — flip it on in the same sitting (as a *separate*
   commit) or leave for later?

## Bonus while we're sitting: the canon questions (S5.4)

Quick verbal answers I'll write into PRODUCT.md so we don't overclaim:
- **Mobile** — who's it for, and what's deliberately *not* on phones?
- **Album Master** — what does "album mastering" promise beyond consistent loudness?
- **Adaptive wording** — how should we describe the engine honestly to users?

## After you sign off (I handle, gated on your notes)

Lock the constants (`LOCKED-BY-LISTENING(date)`), flip the default **only if you
say so**, regenerate DSP snapshots **in that one commit only**, re-run the
already-mastered matrix + slow fixture lane + mobile lanes, and update the
PRODUCT/APP_BEHAVIOR canon. That AC-5 commit is the single change in this whole
push allowed to move DSP snapshot bytes.
