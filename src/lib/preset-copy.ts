import type { Preset } from "../bindings";

// Single source for preset display copy — the label + one-line blurb shown in
// Advanced's preset tiles AND the export receipt's Mastering Style block. The
// receipt reuses this so the description a user reads on export is identical to
// the one they chose from (13c: each fact has one home). The order is the
// preset-name unification order — the four Standard styles sit up front, Oomph
// ahead of Spatial. Don't "tidy" this back.
export const PRESET_OPTIONS: { value: Preset; label: string; blurb: string }[] = [
  { value: { kind: "universal" }, label: "Universal", blurb: "Safe, well-rounded default" },
  { value: { kind: "clarity" }, label: "Clarity", blurb: "Vocal/upper-mid definition" },
  { value: { kind: "tape" }, label: "Tape", blurb: "Saturation, glue, softer top" },
  { value: { kind: "oomph" }, label: "Oomph", blurb: "Low-end weight, punch" },
  { value: { kind: "spatial" }, label: "Spatial", blurb: "Width and depth" },
  { value: { kind: "warmth" }, label: "Warmth", blurb: "Fuller, smoother body" },
  { value: { kind: "punch" }, label: "Punch", blurb: "Transient impact" },
  { value: { kind: "loud" }, label: "Loud", blurb: "Density + level, with safety" },
];

// Resolve a preset to its label + blurb. `custom` presets are open-ended (no
// canonical copy), so they get a neutral label and no marketing blurb.
export function presetCopy(preset: Preset): { label: string; blurb: string } {
  const found = PRESET_OPTIONS.find((p) => p.value.kind === preset.kind);
  if (found) return { label: found.label, blurb: found.blurb };
  return { label: "Custom", blurb: "Your saved settings" };
}
