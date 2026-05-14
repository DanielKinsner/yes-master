# UI Layout Revision - 1600x940 Desktop Target

## Goal

At `1600x940`, Track Master should feel like a fixed mastering console, not a vertically stacked settings page. The main workflow should fit without main-canvas scrolling.

## Key Revisions

### 1. Export Master

Move `Export Master` to the bottom-right.

- It is the final workflow action.
- It should remain prominent and reachable.
- Prefer a sticky bottom-right position inside the right rail or footer.
- Do not place it mid-rail between metering and settings.

### 2. Master Out Metering

Move `Master Out` / fader / level metering into the waveform module.

- Put compact L/R meters and core level readouts at the right end of the waveform column.
- Metering belongs with playback, not buried in the technical settings rail.
- This frees the right rail for advanced controls.

### 3. Waveform + Transport

Collapse waveform and transport into one module.

- Put the play button on the left side of the waveform module.
- Keep time, loop, and seek controls compact near the play button/waveform.
- Put `Original / Mastered` and `Volume Match` in the waveform header/dead space.
- Avoid a separate full-height transport row.

### 4. Presets

Use a shorter preset strip.

- Presets are creative direction buttons, not the visual center of the app.
- Keep icon tiles compact enough that the control deck remains visible.

### 5. Signal Chain

Make the signal chain a slim status strip.

- It should show chain order and active states.
- It should not consume a large full-width row.

### 6. Control Deck

Keep the core control deck visible without scrolling.

- Intensity knob
- Tone knobs
- Embedded visual EQ
- Loudness target
- Undo/redo

These should read as one mastering deck, not separate modules.

### 7. Right Rail

The right rail should be a technical drawer.

Suggested order:

1. Quality Check
2. Delivery Profile
3. Advanced controls
4. Per-Band Compressor
5. Bit Depth / Sample Rate
6. Sticky `Export Master`

The rail may scroll independently. The main canvas should not.

### 8. Per-Band Compressor

Do not show all Low / Mid / High compressor controls at once.

Use:

- collapsed accordion by default
- `Low / Mid / High` tabs when open
- show only the selected band's threshold, ratio, attack, release
- keep `Link stereo` as a compact toggle/checkbox above the tabs

This prevents the right rail from becoming a very tall form.

## Layout Principle

Main canvas = fixed mastering console.

Right rail = scrollable technical drawer.

Left rail = track navigation.

Core workflow at 1600x940 should fit on one screen:

- waveform/playback
- original/mastered comparison
- volume match
- preset selection
- signal chain status
- intensity/tone/EQ controls
- master metering
- export

