# YES Master — Ship Plan

*Vera, 2026-07-12 (Fable session). Verified against `origin/main` @ 53d0462; ran the frontend suite: **543 tests pass**.*

## The headline
This is **not** "fairly along" — it's **basically done.** v0.9.0, Tauri 2 desktop (React 19 + Rust DSP engine). The mastering chain is real and owner-accepted (subsonic → EQ → multiband comp → width/warmth → lookahead limiter → LUFS landing), both Standard and Advanced modes work, Album Master works, safety is test-enforced, only 3 TODOs in the whole codebase (all deliberate debug toggles). You have a decision-complete beta plan already (`docs/plans/2026-07-07-beta-execution-plan.md`, D1–D15; `docs/plans/beta-go-no-go.md`).

**So this plan is not a re-derivation — it's a reconciliation + unblock list.** The engineering is checked off. What's left is money and your ears/hands.

## What actually stands between here and a paid beta
Two buckets, nothing engineering:

**💵 Money-gated (this is the finances↔shipping link):**
1. Apple Developer account (~$99/yr) + Azure Trusted Signing (Windows) — without these, the app is unsigned and every install throws a SmartScreen/Gatekeeper wall.
2. GitHub Actions billing fixed (macOS runners are off → can't build/sign in CI).
3. Email provider for the beta signup form (Buttondown was the rec; form is built + safe-disabled pending the pick).
→ **Ballpark to unblock the whole thing: a couple hundred dollars.** That's the gap between a finished mastering app and a *sellable* one.

**🎧 Owner-lane (needs YOU, ~a few hours total):**
4. Replace the updater signing keypair with your real key **before the first signed release** — this is a one-way door (do it first, or early users can never get updates).
5. Run the listening runbook (`docs/plans/2026-07-08-beta-listening-runbook.md`, ~60–90 min single sitting) — confirms the 4 Standard presets are audibly distinct (Oomph is the historically weak one).
6. Confirm real-machine install/run: macOS on the M4, Windows on the current box, "snappy by ear" on both.
7. Landing page: flip the download button from `mailto:` to the real GitHub Release artifact; finalize beta copy (flip date + $29 founder price); wire + verify signup.

## Do-NOT-touch for v1 (already decided, keep OFF)
Adaptive Compressor AC-5, Phase-B confidence gating, album-character inference — all built but intentionally gated off pending your post-beta ear-calibration. `owner_gates_default.rs` is a tripwire that fails if these flip on. Leave them.

## Decisions I need from you
- **Payday status?** The plan assumed accounts start ~07-10. It's 07-12 — started any of them yet? The moment you greenlight, I can do all the CI/secrets/release-workflow wiring (items 1–3, 7) — that part IS agent-work.
- **Email provider** for signup (Buttondown?).
- **Founder price** — $29 confirmed?

## What I'd do first
The instant you fund the signing accounts, I activate `release.yml`, wire the secrets, and produce a signed downloadable draft — then you run the 90-min listening pass and we flip the landing page. **This could be a live paid beta within days of the money, not months.**

## Suite context
This is the flagship and the most ship-ready of the four YES apps. **Ship this FIRST** — it proves the brand, the signing pipeline, and the store, which the other three then reuse.
