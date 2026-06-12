# YES Master iPhone App

The iPhone app lives at `apps/iphone-native`. It is a SwiftUI-native app that
replaces the earlier Tauri iPhone prototype. See `apps/iphone-native/README.md`
and `apps/iphone-native/HANDOFF.md` for current status and development notes.

## Boundary

The iPhone app is a new app, not a conversion of the desktop app. The desktop
Tauri app remains the full Track Master / Album Master workbench. iPhone v1 is a
separate Simple-only experience that reuses YES Master's existing mastering
settings shape and DSP/render logic where practical.

## v1 Feature List

The shipped Simple-only flow keeps adaptive analysis hidden behind the Rust
bridge:

- Import one track.
- Pick one of four Standard styles: Balanced, Bright, Warm, Heavy.
- Set preset intensity with a slider (the manual safety valve against overcooking).
- Choose loudness: Low, Medium, High.
- Toggle Original / Mastered at the same playhead.
- Toggle Volume Match for audition only (never changes export level).
- Create and share the master.

There is no LUFS Preview control and no export-profile picker in v1; the delivery
format is fixed (see below).

The user-facing contract stays simple: no adaptive readout, no expert-band UI,
and no confidence-gate control. The native Rust bridge resolves the same
source-profile context used by desktop Track Master so preview and export hear
the current shared engine.

## Preset Mapping

The phone labels use the shipped Standard vocabulary. The source of truth is
`src/standard-mapping-parity.json`, which is shared across desktop, iPhone, and
Android parity tests.

| iPhone label | Existing preset |
| --- | --- |
| Balanced | Universal |
| Bright | Clarity |
| Warm | Tape |
| Heavy | Oomph |

No Loud tile is needed in v1 because loudness is its own Simple control.

## Loudness And Export

Loudness is a single control mapped to absolute targets:

- Low = -14 LUFS.
- Medium = -11 LUFS.
- High = -9 LUFS.

The delivery format is fixed at 44.1 kHz, 24-bit, -1 dBTP ceiling. There is no
Streaming/CD/Custom picker in v1.

To avoid changing desktop delivery-profile behavior, the iPhone contract emits
`delivery_profile: "custom"` and fills the explicit advanced fields; the loudness
target is sent via `lufs_offset_db`, which the Custom profile reads as the
absolute target.

## Non-Negotiables

- Do not reshape the desktop app to make the phone app.
- Do not expose desktop adaptive/smart-analysis controls in iPhone v1.
- Volume Match must stay audition-only and must not change export level.
- Keep warning checks advisory unless export is technically invalid.
- Private audio and private renders stay out of git.
