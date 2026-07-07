# YES Master — Beta Listening Runbook (single sitting)

> **Purpose.** One scripted ~60–90 minute listening pass that covers everything
> that is **beta-blocking by ear**. Working through it and filling in the
> pass/fail lines IS the listening note that closes the Manual Listening Gate
> (OPEN_THREADS #4) and the preset-distinctness question (#7). AC-5, Phase-B
> confidence gating, and the album character system stay **gated OFF for beta**
> (owner decision D7) and are NOT calibrated here — do not enable them.
>
> **This is a checklist, not prose.** Each step says what to play, what to do,
> and what to listen for, then leaves a `Result:` line. If a step fails, write
> what you heard — that is the signal, not "it sounded bad."

## Before you start (setup)

- [ ] Build under test (fill in): version ______  commit ______  machine ______
- [ ] Run the runbook on **both** the current Windows box and the M4 MacBook Pro
      (the real-time headline depends on both feeling snappy).
- [ ] Source material staged locally (private, never committed):
  - **A. Normal source** — an unmastered / lightly-processed mix you know well.
  - **B. Already-mastered source** — a loud, finished commercial-style track (the
    stress-test class: the app must not make it obviously worse).
  - **C. Long source** — ≥ 25 min (or loop A to that length) for the seek/timeout case.
  - **D. Nyquist sources** — one **8 kHz** and one **11.025 kHz** WAV (down-sample
    a normal clip; these prove the low-rate clamp doesn't ring or alias).
- [ ] Good monitoring: your usual speakers/headphones, fixed listening level, a
      quiet room. Do the whole sitting at one volume.
- [ ] Standard is the default view; switch to Advanced only where a step says to.

---

## Part 1 — Core Track Master (normal source)

### 1.1 Import → analyze → audition
Play **A**. Import, let analysis finish, press play, toggle **Original ↔ Mastered**.
- Listen for: the switch is **click-free** and keeps the **same playhead**;
  Mastered is a believable, not-broken version of the source.
- Result (PASS / FAIL + note): __________________________________________

### 1.2 The four Standard styles at matched loudness (closes #7)
Set loudness to **Medium (−11 LUFS)**. Cycle **Universal → Clarity → Tape → Oomph**
at a fixed Intensity, listening to the SAME passage each time.
- Listen for: each preset is **audibly distinct** at matched loudness — Clarity
  brighter/opener, Tape warmer/softer, Oomph bolder low-end, Universal the
  neutral centre. None should sound identical to another, and none muddy/pumping.
- **Oomph is the historical weak point** — confirm it is bolder *without* mud or
  pumping. If any pair is indistinguishable or Oomph misbehaves, capture which
  and how (this is the note that would gate a re-tune).
- Result (PASS / FAIL + which presets + note): _____________________________

### 1.3 Loudness trio
On Universal, step **Low (−14) → Medium (−11) → High (−9)**.
- Listen for: loudness clearly increases each step; **High** stays clean (no
  obvious distortion / pumping introduced just by pushing level).
- Result: ______________________________________________________________

### 1.4 Live-control sweeps during playback (responsiveness)
With audio **playing** in Mastered, sweep each control and listen for the change
to apply smoothly in real time, no dropouts/zipper noise/freezes:
- [ ] Intensity  - [ ] Tone / EQ bands (Advanced)  - [ ] Output gain (Advanced)
- [ ] Compressor mode Preset ↔ Manual ↔ Off (Advanced)
- [ ] Preview LUFS toggle  - [ ] Volume Match on/off
- Result (note any control that lags, glitches, or needs a restart): ________

### 1.5 Volume Match is audition-only
Turn **Volume Match ON**, compare Original↔Mastered (should feel level-matched for
fair tone comparison), then **export** and confirm the exported level is the
Standard target, **not** the matched level.
- Listen/verify for: VM changes only what you hear while auditioning; export
  loudness is unchanged by it.
- Result: ______________________________________________________________

---

## Part 2 — Stress classes

### 2.1 Already-mastered source
Play **B**. Analyze, audition Mastered at Medium.
- Listen for: the app does **not** make an already-loud, finished track
  obviously worse (no added harshness/pumping/pinched dynamics by default). If
  it does, note the preset + loudness where it happens.
- Result (PASS / FAIL + note): _____________________________________________

### 2.2 Nyquist / low-rate sources
Play the **8 kHz** then the **11.025 kHz** source (D). Audition Mastered.
- Listen for: no ringing, aliasing, or harsh top-end artifacts from the
  low-rate clamp; the master sounds like a clean band-limited version, not
  broken.
- Result (8 kHz): _____________________  (11.025 kHz): _____________________

### 2.3 Long source — seek & timeout
Play **C**. Enable **Preview LUFS**. **Seek** to several points across the full
length (early, middle, past 20 min) in **Mastered** mode.
- Listen for: each seek resumes promptly with **live, current** coefficients (no
  stale/last-track meters), no "preview still preparing" dead-end, no runaway
  loudness. Note any seek that stalls or never becomes ready.
- Result: ______________________________________________________________

---

## Part 3 — Export by ear (clean vs warning)

### 3.1 Clean export
From a **normal** source at a sensible loudness (no warnings shown), **Create
Master** / Export. Open the rendered WAV in your player.
- Listen for: the exported file matches what you auditioned (WYSIWYG); no
  surprises, no clipping, correct loudness ballpark.
- Result: ______________________________________________________________

### 3.2 Warning export
Drive a source into a **warning** state (e.g. already-mastered **B** pushed to
**High**, or hot settings) so the review/warning path appears. Export **anyway**
through the review flow.
- Listen for: the warning was *honest* (the output is genuinely hotter / more
  compressed as flagged), and the file is still technically valid. Compare the
  clean export (3.1) and this one back-to-back.
- Result: ______________________________________________________________

---

## Part 4 — macOS real-time snappiness (M4)

Repeat **1.1**, **1.4**, and **2.3** on the M4 MacBook Pro.
- Listen/feel for: audition start, Original↔Mastered switching, live sweeps, and
  seeks feel **immediate** — the "real time" headline holds on the Mac just as on
  Windows. Note any Mac-only lag or stutter.
- Result: ______________________________________________________________

---

## Gate sign-off (this is the listening note)

Fill this in after the sitting. If everything is PASS, the Manual Listening Gate
and #7 (preset distinctness) are closed for beta.

- Date / listener: __________________________
- Build (version + commit): __________________________
- Machines: Windows ____ / M4 ____
- **Overall verdict:** BETA-READY BY EAR  /  BLOCKERS FOUND (list below)
- Preset distinctness (#7): PASS / FAIL — notes: __________________________
- Already-mastered behaviour: PASS / FAIL — notes: _______________________
- Any DSP/preset change this implies is **owner-gated**: capture the note here
  first; do not change calibration constants without it.
- Blockers / follow-ups: __________________________________________________

> After signing off: record the outcome in `docs/OPEN_THREADS_AND_DECISIONS.md`
> (close #4 and #7, or log the specific blockers), and drop this filled-in file
> next to the release evidence.
