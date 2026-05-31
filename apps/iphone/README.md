# YES Master iPhone

This folder is the new iPhone app workspace. It is separate from the desktop
Tauri app.

## v1 Shape

- Simple-only iPhone app.
- One track at a time.
- Four tone choices: Balanced, Warm, Open, Punch.
- Loudness choices: Low, Medium, High.
- Export profiles: Streaming, CD, Custom.
- Original/Mastered audition at the same playhead.
- Volume Match is for audition only and stays out of export.
- LUFS Preview is an audition option.
- No adaptive analysis or smart analysis in v1.

The current shared contract lives in `src/simple-mode.ts`. It maps the phone UI
choices to the existing desktop mastering settings shape so the next slice can
wire the iPhone shell into the same DSP/render logic without changing the
desktop app.

## Commands

```sh
npm run iphone:dev
npm run iphone:build
npm run iphone:test
npm run iphone:typecheck
```
