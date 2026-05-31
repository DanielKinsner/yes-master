# YES Master iPhone App

This is the active direction for the new iPhone app.

## Boundary

The iPhone app is a new app, not a conversion of the desktop app. The desktop
Tauri app remains the full Track Master / Album Master workbench. iPhone v1 is a
separate Simple-only experience that reuses YES Master's existing mastering
settings shape and DSP/render logic where practical.

The iPhone workspace starts in:

```text
apps/iphone/
```

## v1 Feature List

Use the Simple Mode list from `docs/SIMPLE_ADVANCED_MODE_NOTE_2026-05-29.md`,
excluding adaptive analysis and smart analysis:

- Import one track.
- Pick one of four tone presets: Balanced, Warm, Open, Punch.
- Choose an export profile: Streaming, CD, Custom.
- Toggle Original / Mastered at the same playhead.
- Toggle Volume Match for audition only.
- Toggle LUFS Preview for audition.
- Choose loudness: Low, Medium, High.
- Export the master.

## Preset Mapping

The phone labels map to existing desktop DSP presets:

| iPhone label | Existing preset |
| --- | --- |
| Balanced | Universal |
| Warm | Warmth |
| Open | Clarity |
| Punch | Punch |

No Loud tile is needed in v1 because loudness is its own Simple control.

## Loudness And Export

The iPhone app separates loudness from destination:

- Low = -16 LUFS.
- Medium = -14 LUFS.
- High = -10.5 LUFS.
- Streaming = 48 kHz, 24-bit, -1 dBTP ceiling.
- CD = 44.1 kHz, 16-bit, -1 dBTP ceiling.
- Custom can set sample rate, bit depth, and ceiling.

To avoid changing desktop delivery-profile behavior, the iPhone contract emits
`delivery_profile: "custom"` and fills the explicit advanced fields.

## Non-Negotiables

- Do not reshape the desktop app to make the phone app.
- Do not add adaptive/smart analysis to iPhone v1.
- Volume Match must stay audition-only and must not change export level.
- Keep warning checks advisory unless export is technically invalid.
- Private audio and private renders stay out of git.

## Current Foundation

`apps/iphone/src/simple-mode.ts` defines the tested contract for iPhone v1. It
returns separate audition and export settings so Volume Match can be on for
listening while export remains clean.

Run:

```sh
npm run iphone:test
npm run iphone:typecheck
```
