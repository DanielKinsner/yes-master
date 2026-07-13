# YES Master — Ship Plan

*Vera, 2026-07-12 (Fable session), corrected against live repo evidence on
2026-07-13. Current frontend suite: **546 tests pass**.*

## The headline
YES Master is a **strong beta candidate, not ready to publish today.** It is
v0.9.0, Tauri 2 desktop (React 19 + Rust DSP engine). The mastering chain is
real and heavily tested (subsonic → EQ → multiband comp → width/warmth →
lookahead limiter → LUFS landing), both Standard and Advanced modes work,
Album Master works, and safety is test-enforced. The decision-complete beta
plan remains `docs/plans/2026-07-07-beta-execution-plan.md` (D1–D15), with the
single authoritative gate list in `docs/plans/beta-go-no-go.md`.

**This is a reconciliation + unblock list.** Core feature engineering is
substantially complete. What remains is one audition trust issue, release
infrastructure, landing-page activation, and owner listening/hardware proof.

## What actually stands between here and the free public beta
Three buckets:

**🛠 Engineering / verification:**

1. GitHub Actions now runs on free public-repository runners. The macOS path-test
   failure was corrected in `66f7006`; keep the latest full run green before
   treating CI as cleared.
2. Original↔Mastered audition still needs closure. The Volume Match stall is
   fixed (`7fc58f6`) and superseded frantic-toggle errors are mechanically
   guarded (`0f6ab86`), but the residual near-zero dip still needs an owner
   re-test. If it survives Volume Match A/B, the documented next option is a
   planned single-stream/sample-synchronous switch — not an unreviewed rush fix.

**💵 Money-gated (this is the finances↔shipping link):**

3. Apple Developer account (~$99/yr) + Azure Trusted Signing (Windows) — without these, the app is unsigned and installs encounter Gatekeeper/SmartScreen friction.
4. Email provider for the beta signup form (Buttondown was the rec; form is built + safe-disabled pending the pick).

→ **Ballpark to unblock the whole thing: a couple hundred dollars.** That's the gap between a finished mastering app and a *sellable* one.

**🎧 Owner-lane (needs YOU, ~a few hours total):**

5. Replace the updater signing keypair with your real key **before the first signed release** — this is a one-way door (do it first, or early users can never get updates).
6. Re-run the three A/B acceptance checks on the stamped build, then run the listening runbook (`docs/plans/2026-07-08-beta-listening-runbook.md`, ~60–90 min single sitting).
7. Confirm real-machine install/run: macOS on the M4, Windows on the current box, "snappy by ear" on both.
8. Landing page: flip the download button from `mailto:` to the real GitHub Release artifact; finalize beta copy (flip date + $29 founder price); wire + verify signup.

## Do-NOT-touch for v1 (already decided, keep OFF)
Adaptive Compressor AC-5, Phase-B confidence gating, album-character inference — all built but intentionally gated off pending your post-beta ear-calibration. `owner_gates_default.rs` is a tripwire that fails if these flip on. Leave them.

## Decisions I need from you
- **Payday status?** The moment the signing accounts exist, the remaining
  secrets/release-workflow work can proceed. Normal CI no longer needs billing
  while the repository is public.
- **Email provider** for signup (Buttondown?).
- **Founder price** — $29 confirmed?

## What I'd do first
First, re-test the A/B interaction on the build containing `0f6ab86`. In
parallel, establish the signing accounts and permanent updater key. Once those
exist, activate `release.yml`, wire the secrets, and produce a signed
downloadable draft. After the listening/hardware gates and landing wiring pass,
the owner can launch the **free public beta**.

## Suite context
This is the flagship and the most ship-ready of the four YES apps. **Ship this FIRST** — it proves the brand, the signing pipeline, and the store, which the other three then reuse.
