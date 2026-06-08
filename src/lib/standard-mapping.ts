// src/lib/standard-mapping.ts
//
// The Standard view's friendly vocabulary <-> the engine's real values.
// This is the ONE frontend source of truth for "what a style means",
// mirroring the iPhone bridge's native_preset() mapping
// (apps/iphone-native/rust/src/lib.rs). Keep the two in lockstep.

import type { Preset } from "../bindings";

export type StandardStyleId = "balanced" | "bright" | "warm" | "heavy";
export type StandardLoudnessId = "low" | "medium" | "high";

export type StandardTone = "blue" | "cyan" | "gold" | "purple";

/// The four tiles, in display order, with the metadata the UI renders.
/// `tone` reuses the Knob/accent tone vocabulary already in the app.
export const STANDARD_STYLES: ReadonlyArray<{
  id: StandardStyleId;
  label: string;
  subtitle: string;
  preset: Preset;
  tone: StandardTone; // accent tone, drives the tile's accent color
}> = [
  { id: "balanced", label: "Balanced", subtitle: "Clean balance", preset: { kind: "universal" }, tone: "blue" },
  { id: "bright", label: "Bright", subtitle: "Air & detail", preset: { kind: "clarity" }, tone: "cyan" },
  { id: "warm", label: "Warm", subtitle: "Glue & body", preset: { kind: "tape" }, tone: "gold" },
  { id: "heavy", label: "Heavy", subtitle: "Sub & weight", preset: { kind: "oomph" }, tone: "purple" },
];

/// The three loudness steps, in display order (matches the iPhone's
/// NativeLoudness: Low/Medium/High -> -14/-11/-9 LUFS).
export const STANDARD_LOUDNESS: ReadonlyArray<{
  id: StandardLoudnessId;
  label: string;
  lufs: number;
}> = [
  { id: "low", label: "Low", lufs: -14 },
  { id: "medium", label: "Medium", lufs: -11 },
  { id: "high", label: "High", lufs: -9 },
];

export function styleToPreset(style: StandardStyleId): Preset {
  const found = STANDARD_STYLES.find((s) => s.id === style);
  if (!found) throw new Error(`unknown standard style: ${style}`);
  return found.preset;
}

/// Reverse: the engine preset -> the Standard tile that owns it, or null
/// when the current preset isn't one of the reference-4 (e.g. a track
/// returning from Advanced still carrying `spatial`). UI shows no active
/// tile in that case until the user picks one.
export function presetToStyle(preset: Preset): StandardStyleId | null {
  // Custom presets have unique ids — a future custom tile must not be matched
  // by kind alone, so bail out before the kind-based lookup below.
  if (preset.kind === "custom") return null;
  const found = STANDARD_STYLES.find((s) => s.preset.kind === preset.kind);
  return found ? found.id : null;
}

export function loudnessToTarget(loudness: StandardLoudnessId): number {
  const found = STANDARD_LOUDNESS.find((l) => l.id === loudness);
  if (!found) throw new Error(`unknown standard loudness: ${loudness}`);
  return found.lufs;
}

/// Reverse: an effective LUFS target -> the matching step, or null when
/// the value is absent or off the three-step grid.
export function targetToLoudness(lufs: number | null): StandardLoudnessId | null {
  if (lufs === null) return null;
  for (const l of STANDARD_LOUDNESS) {
    if (Math.abs(l.lufs - lufs) < 1e-3) return l.id;
  }
  return null;
}
