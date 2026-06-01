# YES Master iPhone — Handoff

_Last updated: 2026-05-31, end of the "works-end-to-end" session (Claude). No human was in the loop for the back half, so anything needing an ear/eye on a real device is called out explicitly below._

## TL;DR

The iPhone app is a **Tauri 2 iOS port** that reuses the **desktop DSP engine verbatim** (shared `yes_master_lib` crate + shared `src/bindings.ts` types). "Simple mode" is a thin preset-picker (Style → desktop preset, Loudness → target LUFS, Profile → export format) that feeds the same `engine`/`dsp` code the Mac app uses. **DSP parity is not a concern — it's literally the same math** (proven by `iphone_render_master_to_path_uses_shared_dsp_engine` and a repo-wide grep showing zero reimplemented DSP).

This session fixed the three blockers that made it "feel very broken" on device — silent playback, dead-end export, and a fake waveform — plus live audition, audio-session resilience, and the dominant brand gap. All changes are **iPhone-only** (`apps/iphone/**`); the desktop app is untouched.

## State of play

| Area | Status |
|------|--------|
| Audio plays on device | **Code-correct, needs an ear.** `AVAudioSession` is configured `.playback` + activated (was the silent-playback cause). cpal `try_default()` succeeds on device; nobody has *listened* yet. |
| Scrub / playhead | **Fixed.** Tick no longer fights the drag; seek commits on release. |
| Waveform | **Decode path proven sound** (new CI test). On device it had thrown and shown a fake flat placeholder; now the error is surfaced + the import is copied to a durable path. **Read `iphone_prepare_waveform` in the Xcode/Console log on the next device run** to confirm the remaining cause (Io vs Decode vs path). |
| Export | **Fixed (was a dead end).** Writes a verified non-zero WAV to `Documents/YES Master/`, Files-visible. Needs a device to confirm the file appears under Files › On My iPhone › YES Master. |
| Live mastering controls | **Fixed.** Changing tone/loudness/etc. now re-applies during Mastered playback. |
| Interruption/route recovery | **Half done (iPhone-safe half).** Session re-activates on foreground. The stream-recreate half lives in the shared desktop `audio.rs` and was deliberately **not** touched — see Remaining work. |
| Untethered (no laptop) | **Proven capable.** Release build bundles a self-contained frontend (no dev-server URL). **Action required:** install a *release* build to replace the dev build currently on the phone. See "Make it untethered". |
| Brand | Primary CTA re-pointed teal → desktop blue (the big one). 5 smaller items itemized below. |

## What changed this session (all on `main`, pushed)

Branches: work landed via `fix/iphone-audio-playback` then `fix/iphone-works-e2e`, both fast-forward merged to `main`. Every commit is small and scoped. In order:

1. **scrub race** — playhead no longer yanked back by the 50 ms tick.
2. **AVAudioSession** — `ios_audio` module (objc2 + `dlopen`/`dlsym` AVFoundation, no link-time dep, survives `tauri ios` regen) sets `.playback` + active. This was the silent-playback fix.
3. **waveform diagnostics** — log raw/normalized path + existence + concrete error.
4. **waveform decode test** — `prepare_waveform` through a percent-encoded `file://` URL (passes → decode path is sound).
5. **waveform error surfacing** — distinct "Waveform unavailable" state instead of a fake placeholder.
6. **durable import** — copy the picked file into `app_data_dir()/imports/<hash>/` so analyze/waveform/play/render survive iOS purging `tmp/`.
7. **export (BLOCKER)** — write-then-export: render to `Documents/YES Master/`, verify non-zero, expose via `UIFileSharingEnabled`. Drops the broken `save()` dialog.
8. **live mastering** — `updateMasteringChain` wired to a control-signature effect during Mastered playback.
9. **session reactivation** — `iphone_reactivate_audio_session` + re-activate on `visibilitychange`/`focus`; moved `configure()` after the log plugin so its result is visible on device.
10. **switch guard** — `switchAuditionMode` no longer flips the segment before the play guard.
11. **lufs default** — `previewLufsLanding` defaults to `false` (Simple contract).
12. **brand CTA** — teal `--cta` → `var(--accent-bright)` (desktop blue) everywhere.

Tests after the work: **iphone TS 80/80, iphone Rust 6/6, iphone iOS-target check clean, typecheck clean.**

## Verified vs. needs a device (no human was available)

**Verified statically/automatically:** all of the above compile (host + `aarch64-apple-ios`), TS typechecks, 80 TS + 6 Rust tests pass, the built `dist/index.html` has no dev-server URL.

**Needs an ear/eye on the real iPhone (cannot be checked without a human):**
1. **Baseline audibility** — does import → analyze → Play actually make sound at the right level? (Code says yes; `.playback` ignores the silent switch.)
2. **The waveform error string** — read `iphone_prepare_waveform ... FAILED ...` (or `ok: channels=…`) in the device log to finish the waveform root-cause. If it's a missing-file/path Io error, the durable-import copy likely already fixes it; if it's a codec Decode error, the shared decoder needs attention.
3. **Export lands** — confirm a non-empty WAV appears under Files › On My iPhone › YES Master.
4. **Interruption/route recovery** — phone call / Siri / unplug headphones mid-play, then return; confirm audio resumes (this is exactly the half that still needs the stream-recreate follow-up).

## Make it untethered (one-time, replaces the dev build on the phone)

The app on the phone now is the **dev** build (loads the UI from this laptop's Vite server → white screen when detached). Install a **release** build instead — it bundles the UI into the `.app`:

```bash
# Homebrew cargo on PATH can't cross-compile to iOS — use the rustup toolchain:
export PATH="$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:$PATH"
cd ~/Projects/yes-master/apps/iphone
../../node_modules/.bin/tauri ios build      # release; bundles ../dist, signs with team C4ZKT5JJ3K
# then install the built .ipa/.app to the connected device:
xcrun devicectl device install app --device <DEVICE_ID> <path-to-built .app>
```

`xcrun devicectl list devices` shows the connected iPhone (was "iPhone 16 Pro Max"). A signed device build may prompt for keychain/signing in the GUI the first time. Once installed, the release build runs with the laptop disconnected.

## Desktop safety

**The iPhone work did not touch or break the desktop app** — every change is under `apps/iphone/`, the desktop crate compiles clean, and `git diff` confirms zero desktop-source edits.

⚠️ **Pre-existing, unrelated:** `cargo test --lib` in `src-tauri` currently shows **4 failing tests** — `dsp::tests::preset_byte_identity::{clarity,oomph,tape,universal}_chain_output_sha_snapshot_matches` ("chain-output SHA changed; investigate DSP drift"). These live in `src/dsp.rs` (last changed by desktop commit `fbf2478`, before this session) and fail independently of the iPhone work. Per `AGENTS.md`, investigate the DSP drift before re-snapshotting. **Not caused by this session.**

## Remaining work (deferred, prioritized)

From the multi-agent audit (`/tmp/iphone-audit-report.md` — re-generate if gone; it's reproducible). None are end-to-end blockers.

- **FIX-5 stream-recreate (the other half of interruption recovery).** Reactivating the session isn't always enough — the cached `OutputStream` (in shared `audio.rs`) can stay dead. The clean fix recreates it on reactivation/each play, but that's a desktop-crate change and was kept out of this iPhone-only push. Do it as a deliberate, separately-reviewed change (e.g. additive `AudioPlayer::reset_output()` the desktop never calls), or confirm on device whether cpal already resumes after session reactivation.
- **FIX-8/9** use `resumePlayback` for same-track Play (cheaper than full restart); remove the dead `prepareMasterPreview` command (+ its tests) or adopt it.
- **FIX-12** `spawn_blocking` the decode in `prepare_waveform`/analyze (currently two concurrent blocking decodes on the runtime).
- **FIX-13** capture the `NSError` out-params in `ios_audio` and surface a one-time warning if `setActive` returns false.
- **FIX-14** delete the dead `assetProtocol` config + `protocol-asset` feature + `toIphoneAudioUrl` (audition is native cpal, no `<audio>` element).
- **FIX-7** add a serde-shape guard test for `bindings.ts` ↔ `types.rs` (hand-maintained, currently in sync, unguarded).
- **Brand items 2–6** (all pure CSS/markup, no DSP): desktop 5-bar spectrum SVG brand mark vs the raster PNG; preset-art `mix-blend-mode: screen`; type weights 900→700 / 850→600; letter-spacing on all-caps eyebrow labels; empty-hero brand headline (the `h1` CSS is currently dead). Exact line targets are in the audit report §5.

## Orientation for whoever picks this up

- **DSP is shared, don't reimplement it.** iPhone commands in `apps/iphone/src-tauri/src/lib.rs` delegate into `yes_master_lib`. Simple-mode mapping is `apps/iphone/src/simple-mode.ts`.
- **Audio is the desktop `rodio`/`cpal` engine running on iOS.** Works, but it's why iOS needs the `AVAudioSession` handling and why interruption recovery is fiddly.
- **Toolchain trap:** `cargo` on PATH is Homebrew's (host-only). For any `--target aarch64-apple-ios` command, prepend `$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin` to PATH.
- **Verify lane:** `npm run iphone:typecheck && npm run iphone:test`; `cd apps/iphone/src-tauri && cargo test`; iOS check per the toolchain note.
- **Can't tap the device headlessly.** Exercising import/play/export needs a human; the launch path (and thus the `AVAudioSession activated` log) does not.
