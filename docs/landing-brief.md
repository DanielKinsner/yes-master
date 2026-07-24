# YES Master — Landing Brief

> **Source for landing-page copy + image generation.** Everything here is
> **present-tense** — what YES Master *is today*. No roadmap, no "coming soon,"
> no version numbers, no dev/build detail. **If a generator reads one file for
> the landing page, read THIS one — not `README.md`** (the README contains
> developer and roadmap sections that must not appear on the landing page).

## In one line

YES Master is a local-first desktop app that masters your finished track or
album — and shows you exactly what it did.

## Hero copy (pick or mix)

**Primary:** Master your track in real time — and see exactly what it did.

**Supporting subheads:**
- Not just louder. More legible.
- Three decisions. One finished master.
- The four Standard sounds. When you need more, the studio opens.
- Real-time on your machine — instant, no upload, no account.

## The promise (what the page sells)

Drop in audio. Hear what the mastering chain is doing in real time. Pick a sound,
set a loudness, shape the intensity. Create a finished, technically-checked
master — all running locally, so it responds the instant you touch it.

Push it as hard as you want: YES Master measures what actually happened and shows
you — live meters, quality checks, a post-render receipt — so you always know the
truth about the file you're shipping. **No waiting. No black box.**

## Two views, one engine (the core visual story)

- **Standard — the hero.** One clean column: pick a **Style**
  (Universal · Clarity · Tape · Oomph), a **Loudness** (Low / Medium / High),
  shape with **Intensity**, hit **Create Master**. A/B Original vs Mastered at
  the same playhead. No clutter, no ceremony.
- **Advanced — the proof.** The full console: eight presets, a 7-band visual EQ,
  width/warmth, compressor controls, live metering, delivery formats, and an
  export review with a measured receipt.

## What's on screen (for image generation)

A focused, studio-grade desktop interface:

- A **waveform** with an Original / Mastered A/B toggle that keeps the same
  playhead.
- A **visual EQ curve** the user shapes by hand (Advanced).
- **Live meters** moving in real time — peak, LUFS (loudness), gain reduction,
  spectrum.
- **Style / preset tiles** and an **Intensity** control.
- A big **Create Master** button (Standard), or an **export review** (Advanced)
  showing delivered LUFS, true peak, dynamic range, and pass/fail checks.
- Any **iPhone / Android** screen shown must be labelled as a not-currently-
  available companion surface, never as part of the desktop beta proof
  (amended 2026-07-24, U1 — see "Mobile status").

## Feature pillars (present tense)

- **Real-time.** The whole mastering chain runs live while audio plays; every
  control responds instantly.
- **Local & instant.** Everything runs on your machine — no upload, no round-trip,
  no account, works offline. (Your audio staying private is a welcome side effect,
  not the pitch.)
- **Honest.** It measures the rendered file and surfaces real problems; it never
  hides what it did.
- **Safe by design.** Exports never overwrite your source. Volume-matched
  auditioning never changes your export level.
- **Adaptive, gently.** It reads each track and eases its moves to fit — it
  tames, never overcooks.
- **Cross-platform.** Windows and macOS desktop. **Desktop is the product you
  can get.** (Amended 2026-07-24, U1: this pillar used to append the phone
  bridges to the same sentence, which reads as three shipping products. The
  engine sharing is true; the availability implication is not. See "Mobile
  status" below.)

## Who it's for

Musicians and producers finishing tracks. Album-minded creators who need
consistent loudness and flow. Anyone with an already-processed or AI-generated
track who wants to add taste without making the file worse.

## Tone & visual direction

Confident, calm, pro-audio. **Mastering studio / control room**, not consumer
gadget: precise meters, clean type, focused dark surfaces, real waveforms and EQ
curves. Trustworthy and legible over flashy. Three established directions to
match: **trust-first** (simple, reassuring) · **control-room** (serious console)
· **mobile-forward** (same vocabulary, pocket-sized).

## Mobile status (added 2026-07-24, U1)

**iPhone and Android are parked.** The shared engine really does power
CI-tested native bridges, and their measured output is pinned bit-parity against
desktop — but no mobile build is downloadable, dated, or owner-approved for
release (owner decisions D9, D15; product policy in `docs/PRODUCT.md` "Mobile
Companions").

The page may state, **once and quietly**, that mobile is not currently
available. It may not:

- give a date, a season, a release order, or the word "soon";
- list mobile features in a way that reads as a product a visitor can obtain;
- show a mobile screenshot as evidence of the desktop beta.

This is the single deliberate exception to the "no roadmap" hard rule below, and
it exists because saying nothing at all about mobile would be *less* honest than
saying "not yet" — a visitor who has seen the mobile screens should not be left
guessing. Anything beyond one date-free sentence is out of bounds.

## Hard rules for generation

- **Present tense only** — describe what it *is*, never what it *will be*. The
  one exception is the date-free mobile-status sentence above.
- **No roadmap, no "coming soon," no version numbers, no build/dev detail.**
- Standard is the hero; Advanced is the proof.
- Load-bearing words: **real-time, simple, honest, legible** (and **local** framed
  as *instant*, not *private*).
- **Every claim on the page needs a row in `docs/CAPABILITY_EVIDENCE_MATRIX.md`.**
  A claim with no row is a defect, not an oversight.
