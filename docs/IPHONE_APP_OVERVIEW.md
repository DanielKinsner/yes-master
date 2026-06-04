# YES Master iPhone App

YES Master for iPhone is a focused, local mastering app for turning one track
into a polished master without opening the full desktop workbench.

The iPhone app is at `apps/iphone-native`. It is a SwiftUI-native app using
Rust (via `yes_master_lib`) for mastering and native iOS APIs for file import,
playback, and export.

## What It Can Do

- Import one song from the phone.
- Let the user choose a mastering style: Balanced, Warm, Open, or Punch.
- Let the user set preset intensity with a slider.
- Let the user choose loudness: Low, Medium, or High.
- Audition the original and mastered versions without losing the playhead.
- Offer Volume Match for fair listening only (never changes the export).
- Create a finished master when the user is ready.

## The Goal

The iPhone app should feel like a premium, simple mastering tool: fast to
understand, visually confident, and focused on the core workflow.

The main flow is:

1. Import a track.
2. Pick a style, intensity, and loudness.
3. Preview original versus mastered.
4. Export the master.

## What It Is Not

- It is not the desktop app converted to a phone layout.
- It is not trying to expose every advanced desktop control.
- It is not exposing adaptive or smart-analysis controls in v1.

## Experience Rules

- Keep the phone app simple.
- Keep audio private and local.
- Reuse the mastering sound and DSP logic that make the desktop app valuable.
- Keep adaptive/source analysis hidden in the Rust bridge unless a later product
  decision deliberately adds an expert phone surface.
- Make the interface feel like a product, not a settings panel.
- Keep the Mac and Windows desktop apps untouched.

## What Success Looks Like

A musician should be able to open the app, load a track, make a few clear
choices, hear the result, and export a master without needing to understand the
technical details behind mastering.
