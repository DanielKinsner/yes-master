# YES Master iPhone — Handoff

_Last updated: 2026-06-01, end of the no-device cleanup session (Codex). No iPhone was connected; anything needing an ear/eye on a real device is called out explicitly below._

## TL;DR

The iPhone app is a **Tauri 2 iOS port** that reuses the **desktop DSP engine verbatim** (shared `yes_master_lib` crate + shared `src/bindings.ts` types). "Simple mode" is a thin preset-picker (Style → desktop preset, Loudness → target LUFS, Profile → export format) that feeds the same `engine`/`dsp` code the Mac app uses. **DSP parity is not a concern — it's literally the same math** (proven by `iphone_render_master_to_path_uses_shared_dsp_engine` and a repo-wide grep showing zero reimplemented DSP).

The prior device-focused session fixed the three blockers that made it "feel very broken" on device — silent playback, dead-end export, and a fake waveform — plus live audition, audio-session resilience, and the dominant brand gap. The 2026-06-01 no-device pass completed the remaining code-only audit items that were safe without a phone. All changes are **iPhone-only** (`apps/iphone/**`); the desktop app is untouched.

## State of play

| Area | Status |
|------|--------|
| Audio plays on device | **Code-correct, needs an ear.** `AVAudioSession` is configured `.playback` + activated (was the silent-playback cause). cpal `try_default()` succeeds on device; nobody has *listened* yet. |
| Scrub / playhead | **Fixed.** Tick no longer fights the drag; seek commits on release. |
| Waveform | **Decode path proven sound** (new CI test). On device it had thrown and shown a fake flat placeholder; now the error is surfaced + the import is copied to a durable path. **Read `iphone_prepare_waveform` in the Xcode/Console log on the next device run** to confirm the remaining cause (Io vs Decode vs path). |
| Export | **Fixed (was a dead end).** Writes a verified non-zero WAV to `Documents/YES Master/`, Files-visible. Needs a device to confirm the file appears under Files › On My iPhone › YES Master. |
| Live mastering controls | **Fixed.** Changing tone/loudness/etc. now re-applies during Mastered playback. |
| Interruption/route recovery | **Half done (iPhone-safe half).** Session re-activates on foreground, captures `NSError` details, and emits a one-time warning if activation fails. The stream-recreate half lives in shared desktop `audio.rs` and was deliberately **not** touched — see Remaining work. |
| Untethered (no laptop) | **Proven capable.** Release build bundles a self-contained frontend (no dev-server URL). **Action required:** install a *release* build to replace the dev build currently on the phone. See "Make it untethered". |
| Brand | **No-device brand pass done.** Desktop blue CTA, 5-bar spectrum mark, screen-blended preset art, calmer weights, tracked all-caps labels, and empty hero headline are all in place. |

## Recent changes

### 2026-06-01 premium moodboard branch

Branch: `codex/iphone-premium-moodboard` (not merged to `main` yet).

Purpose: use the provided reference image as a mood board, while keeping the current iPhone workflow intact. This is visual polish only; no DSP, export, import, or playback behavior changed.

Commits on the branch:

1. `ca4e8dc style(iphone): add premium blue moodboard shell`
2. `9c8d8ba style(iphone): add premium section hierarchy`
3. `860e3e5 style(iphone): polish premium export surfaces`

What changed:

- Deeper navy/black shell with electric-blue focus lighting.
- Glowing import orb, subtle waveform/ring treatment, and brighter blue active controls.
- Numbered `Style / Loudness / Profile` section labels inspired by the reference.
- More premium tone cards, metering/waveform panels, export button, processing overlay, and Master Ready sheet.

Preview command:

```bash
npm run iphone:dev -- --host 127.0.0.1
```

Then open `http://127.0.0.1:5174/`. A preview server was left running during the branch handoff for immediate review.

### 2026-05-31 works-end-to-end session

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

### 2026-06-01 no-device cleanup session

Commit: `8087c5d fix(iphone): finish no-device cleanup items`.

1. **Review of prior 13 commits:** no blocker found. The AVAudioSession `dlopen`/`objc2` approach is reasonable for surviving `tauri ios` regeneration; write-then-export to `Documents/YES Master/` plus Info.plist sharing keys is the right no-picker path; durable import is the right fix for tmp/InBox lifetime risk. Things I would change were all code-only and are now done: capture NSError details, remove dead asset/preview paths, add a serde shape guard, and make pause→play use native resume when safe.
2. **FIX-8/9:** same-track pause→play now calls `resumePlayback()` when the loaded source is still valid. If the user changed Mastered settings or scrubbed while paused, it rebuilds playback instead of resuming stale audio. The dead `prepareMasterPreview` command/API/tests were removed.
3. **FIX-12:** iPhone analyze and waveform commands now offload the blocking decode work through `spawn_blocking`.
4. **FIX-13:** `ios_audio` now captures `NSError` out-params, logs the localized description when available, and emits one one-time UI warning if activation fails.
5. **FIX-14:** dead `assetProtocol`, `protocol-asset`, and `toIphoneAudioUrl` were removed.
6. **FIX-7:** Rust serde guard added for the Simple-mode JSON shape used by shared `bindings.ts` / `types.rs`.
7. **Brand items 2-6:** desktop SVG spectrum mark, preset-art screen blend, lighter typography, tracked eyebrow labels, and empty hero headline landed.

Latest no-device verification: **iphone TS 84/84, iphone Rust 6/6, iOS-target check clean, typecheck clean.**

### 2026-06-01 late-night obvious wins

Commits:

- `aadf06e test(iphone): strengthen no-device guards`
- `0f70f48 fix(iphone): preserve resume after live control updates`

Small, high-confidence follow-ups from the independent review:

1. **Serde drift guard strengthened:** the Simple-mode JSON test now asserts every sent field and exact serialize/deserialize key round-trip, so typo/rename drift is much harder to miss.
2. **CSS test helper hardened:** `cssBlock` now reads balanced braces instead of stopping at the first `}`, so nested CSS functions do not trick the test.
3. **Resume optimization finished:** after a Mastered control change is live-applied while audio is playing, pause→play now keeps using native `resumePlayback()` instead of rebuilding the stream.

Latest no-device verification after these commits: **iphone TS 85/85, iphone Rust 6/6, iOS-target check clean, typecheck clean.**

## Independent review of the 2026-06-01 cleanup (Claude, 4 adversarial reviewers + re-run verification)

**Verdict: solid.** No blocker/high/medium defects in any of the four change areas (Rust/audio, frontend resume logic, config/dead-code removal, brand/CSS). **No regressions** — the prior session's fixes were byte-diffed and confirmed intact (AVAudioSession activate path, `copy_into_app_imports`, `resolve_export_path` + non-zero verify, scrub tick-vs-drag, `switchAuditionMode` guard, foreground `reactivateAudioSession`, CTA blue). Re-verified independently: typecheck clean, TS 84/84, Rust 6/6. Spot-checks that held up under scrutiny: the objc2 `NSError` out-param handling is ABI-correct with no leak/over-release; `spawn_blocking`+`block_on` is sound on the Tauri multi-thread runtime; `protocol-asset`/`convertFileSrc` were genuinely dead (playback is fully native, no `<audio>` element) so removal is safe; the dead `prepareMasterPreview` removal left no dangling references.

### Things to look into (none blocking; ordered by value)

1. **[low — informational] Redundant `updateMasteringChain` on Mastered resume.** Resume flips `isAuditionPlaying` and re-fires the live-apply effect once with unchanged settings — idempotent, harmless.
2. **[low — defensive] `analysis` (source LUFS) is absent from `masteredControlSignature`.** Not currently reachable (a ready track's `analysis` doesn't mutate), but worth a code comment if that ever changes, so resume wouldn't miss a source-LUFS update.
3. **[low — cosmetic] Minor leftovers.** `browserRenderJob` still types `kind` as `"preview" | "master"` though only `"master"` is passed; the stale `apps/iphone/dist/` bundle may contain older strings until the next release build refreshes it.

### Added to the on-device checklist (Part B) — two brand visual checks

- **Preset-art `mix-blend-mode: screen` at 48px:** `screen` makes black transparent, so any non-pure-black matte / anti-aliased edge / compression ringing in the preset PNGs (clarity/punch/universal/warmth) will show as bright halos — and the art is now larger (38→48px), magnifying it. Confirm the thumbnails look clean on device.
- **Short-viewport hero crowding:** the new empty-hero headline adds an `auto` row above the orb inside a height-capped panel (~30–40px). On an SE-class (short) screen confirm the import circle + label + checkbox row aren't crowded/clipped.

## Verified vs. needs a device (no human was available)

**Verified statically/automatically:** all code compiles on host and `aarch64-apple-ios`, TS typechecks, 85 TS + 6 Rust tests pass. No device build or on-device run was attempted in the 2026-06-01 session.

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

## Part B — on-device session (run tomorrow, phone connected + a human present)

Everything else in this doc can be done with nothing plugged in. **This section needs the phone and
you** — no tool can tap the screen or hear the speaker. The phone is connected **only for the
install**; then unplug it (running untethered is exactly what you're confirming). Codex can do steps
1–2 with the phone connected; the tap/listen steps are yours.

Prereqs: iPhone connected (USB simplest), unlocked, "Trust This Computer" accepted.

1. **Build + install the release build** (replaces the dev build on the phone):
   ```bash
   export PATH="$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:$PATH"
   cd ~/Projects/yes-master/apps/iphone
   ../../node_modules/.bin/tauri ios build
   xcrun devicectl list devices                          # note the iPhone's device id
   xcrun devicectl device install app --device <ID> <built .app path>
   ```
   (First signed build may pop a keychain/signing prompt — approve it.)

2. **Untethered check:** unplug the phone, open YES Master from the home screen → it should open to
   the real UI, not a white screen. ✅ confirms the bundled frontend runs with no laptop.

3. **Audio (the gating unknown):** import a track → wait for "Ready" → Play. You should hear it (the
   ring/silent switch doesn't matter — `.playback` overrides it). In Xcode/Console confirm the
   startup line `AVAudioSession activated (category_ok=true, active_ok=true)`.

4. **Waveform root-cause (one line):** right after importing, find `iphone_prepare_waveform` in the
   console and read which case it is:
   - `... ok: channels=N first_len=M` → waveform is fine now (the durable-import copy fixed it).
   - `... FAILED ... Io ...` → missing/unreadable file (should already be fixed; flag it if not).
   - `... FAILED ... Decode ...` → a codec the shared decoder rejects → hand to Codex for the decoder.
   Paste that one line and the waveform is settled.

5. **Scrub:** while playing, drag the playhead — it should move freely, not snap back. ✅

6. **Export:** Create Master → Files app → On My iPhone → YES Master → confirm a **non-empty** `.wav`
   is there and plays. ✅

7. **Interruption recovery (the known gap):** start playback, trigger an interruption (call yourself /
   Siri / unplug headphones), end it, return to the app, press Play again.
   - Audio comes back → foreground reactivation is sufficient.
   - Stays dead → that's the stream-recreate follow-up (see "Remaining work"); hand to Codex.

8. **Brand visual checks (from the 2026-06-01 review):**
   - **Preset thumbnails:** the Style cards now use `mix-blend-mode: screen` at a larger size — confirm
     the four preset images have no bright halo/fringe around the edges (a non-black matte would show).
   - **Short screens:** on an SE-class iPhone, confirm the new empty-hero headline doesn't crowd or clip
     the import circle, its label, or the checkbox row.

Anything that fails here is a concrete next task — none of it blocks the others.

## Desktop safety

**The iPhone work did not touch or break the desktop app** — every change is under `apps/iphone/`, the desktop crate compiles clean, and `git diff` confirms zero desktop-source edits.

⚠️ **Pre-existing, unrelated:** `cargo test --lib` in `src-tauri` currently shows **4 failing tests** — `dsp::tests::preset_byte_identity::{clarity,oomph,tape,universal}_chain_output_sha_snapshot_matches` ("chain-output SHA changed; investigate DSP drift"). These live in `src/dsp.rs` (last changed by desktop commit `fbf2478`, before this session) and fail independently of the iPhone work. Per `AGENTS.md`, investigate the DSP drift before re-snapshotting. **Not caused by this session.**

## Remaining work (deferred, prioritized)

From the multi-agent audit (`/tmp/iphone-audit-report.md` — re-generate if gone; it's reproducible). None are end-to-end blockers.

- **FIX-5 stream-recreate (the other half of interruption recovery).** Reactivating the session isn't always enough — the cached `OutputStream` (in shared `audio.rs`) can stay dead. The clean fix recreates it on reactivation/each play, but that's a desktop-crate change and was kept out of this iPhone-only push. Do it as a deliberate, separately-reviewed change (e.g. additive `AudioPlayer::reset_output()` the desktop never calls), or confirm on device whether cpal already resumes after session reactivation.
- **Device validation is now the main next step.** Install a release build and do Part B above. The next concrete code task should come from what the phone proves: audibility, waveform log, export visibility, or interruption recovery.
- **Optional polish after device signoff:** consider making the export receipt display the backend-resolved Files path rather than the suggested filename; current render/report logic uses the real backend path, but the receipt copy can be more explicit.

## Orientation for whoever picks this up

- **DSP is shared, don't reimplement it.** iPhone commands in `apps/iphone/src-tauri/src/lib.rs` delegate into `yes_master_lib`. Simple-mode mapping is `apps/iphone/src/simple-mode.ts`.
- **Audio is the desktop `rodio`/`cpal` engine running on iOS.** Works, but it's why iOS needs the `AVAudioSession` handling and why interruption recovery is fiddly.
- **Toolchain trap:** `cargo` on PATH is Homebrew's (host-only). For any `--target aarch64-apple-ios` command, prepend `$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin` to PATH.
- **Verify lane:** `npm run iphone:typecheck && npm run iphone:test`; `cd apps/iphone/src-tauri && cargo test`; iOS check per the toolchain note.
- **Can't tap the device headlessly.** Exercising import/play/export needs a human; the launch path (and thus the `AVAudioSession activated` log) does not.
