# YES Master - Codex Adversarial Review Notes (2026-05-29)

## Right Rail Correction

Earlier review wording overstated the right-rail issue. The app opens at the
native Tauri launch size of `1920x1080`; `1440x860` is only the declared resize
floor in `src-tauri/tauri.conf.json`.

At `1920x1080`, the supplied screenshot shows no right-rail overlap. When
`ADVANCED CONTROLS` is expanded, the right rail becomes a normal vertical
scrolling surface. That up/down scrolling behavior is expected because Source
Check, Delivery Profile, Advanced Controls, Per-Band Compressor, Delivery
Format, Tools, and the sticky Export group exceed the visible rail height.

Reframed finding:

> Right rail scroll/reachability watch item, not a launch overlap bug. Verify
> that the sticky export group never hides an unreachable control at the
> declared resize minimum, but do not treat the current `1920x1080` launch view
> as broken.
