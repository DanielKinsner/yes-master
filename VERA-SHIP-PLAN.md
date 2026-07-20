# YES Master — Ship Plan

*Vera, 2026-07-12 (Fable session), corrected against live repo evidence on
2026-07-13. Current frontend suite: **546 tests pass**.*

## The headline
YES Master is a **strong beta candidate, not ready to publish today.** It is
v0.9.0, Tauri 2 desktop (React 19 + Rust DSP engine). The mastering chain is
real and heavily tested (subsonic → EQ → multiband comp → width/warmth →
lookahead limiter → LUFS landing), both Standard and Advanced modes work,
Album Master works, and safety is test-enforced. The decision-complete beta
plan remains `docs/plans/2026-07-07-beta-execution-plan.md` (D1–D16), with the
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

**💵 Paid trust upgrades (post-beta, not blockers under D16):**

3. Apple Developer notarization and Azure Artifact Signing reduce
   Gatekeeper/SmartScreen friction when budget permits.
4. An email provider enables optional founder-list capture; downloading the
   beta does not require it.

**🎧 Owner-lane (needs YOU, ~a few hours total):**

5. Back up the configured permanent updater private key + passphrase in a
   cross-machine password manager.
6. Re-run the three A/B acceptance checks on the stamped build, then run the listening runbook (`docs/plans/2026-07-08-beta-listening-runbook.md`, ~60–90 min single sitting).
7. Confirm real-machine install/run: macOS on the M4, Windows on the current box, "snappy by ear" on both.
8. Choose the concrete beta end date, review the audited draft release, deploy
   the already-wired ungated landing download, then publish and announce.

## Do-NOT-touch for v1 (already decided, keep OFF)
Adaptive Compressor AC-5, Phase-B confidence gating, album-character inference — all built but intentionally gated off pending your post-beta ear-calibration. `owner_gates_default.rs` is a tripwire that fails if these flip on. Leave them.

## Decisions I need from you
- **Concrete beta end date?** D2 still requires one before the public switch.
- **Email provider?** Optional; may remain disabled for launch.
- **Founder price** — $29 confirmed?

## What I'd do first
First, cut an audited draft from the $0 release workflow. Re-test the A/B
interaction and run the listening/hardware gates on those exact artifacts.
When they pass and the beta end date is set, the owner can publish the draft,
deploy the landing update, and announce the **free public beta**.

## Suite context
This is the flagship and the most ship-ready of the four YES apps. **Ship this FIRST** — it proves the brand, the signing pipeline, and the store, which the other three then reuse.
