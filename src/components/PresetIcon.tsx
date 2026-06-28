// ui-tighten — Preset tile imagery.
//
// Each preset gets its own 3D-rendered PNG (1024×1024 with alpha) under
// `src/assets/presets/`. Vite resolves these static imports into hashed
// URLs at build time, so the bundle ships the actual files instead of
// inlining them as base64. Falls back to a sliders SVG for the `custom`
// kind because user-saved custom presets are open-ended — there's no
// canonical 3D character image for them.

import type { Preset } from "../bindings";
import type { KnobTone } from "./Knob";
import universalSrc from "../assets/presets/universal.png";
import claritySrc from "../assets/presets/clarity.png";
import tapeSrc from "../assets/presets/tape.png";
import spatialSrc from "../assets/presets/spatial.png";
import oomphSrc from "../assets/presets/oomph.png";
import warmthSrc from "../assets/presets/warmth.png";
import punchSrc from "../assets/presets/punch.png";
import loudSrc from "../assets/presets/loud.png";

type IconKind = Preset["kind"];

/// Accent color per preset — matches the language of the generated 3D
/// imagery. The ONE shared source for preset accent hues: Advanced's
/// preset strip and Standard's style tiles both tint from this map, so the
/// same preset glows the same color in both views (visual-parity pass).
export const PRESET_ACCENT: Record<IconKind, string> = {
  universal: "#4d8bff",
  clarity: "#22d3ee",
  tape: "#fbbf24",
  spatial: "#a78bfa",
  oomph: "#f87171",
  warmth: "#fb923c",
  punch: "#ef4444",
  loud: "#60a5fa",
  custom: "#9ca3af",
};

/// Knob accent tone per preset — the named Knob `tone` whose color tracks
/// each preset's identity. The four standard styles match Standard's
/// STANDARD_STYLES tones exactly (universal=blue, clarity=cyan, tape=gold,
/// oomph=red); extended to every preset kind so the Intensity knob can
/// inherit the selected preset's color in both Standard and Advanced.
export const PRESET_TONE: Record<IconKind, KnobTone> = {
  universal: "blue",
  clarity: "cyan",
  tape: "gold",
  spatial: "purple",
  oomph: "red",
  warmth: "orange",
  punch: "red",
  loud: "blue",
  custom: "blue",
};

type IconProps = {
  kind: IconKind;
  className?: string;
  "aria-hidden"?: boolean;
};

const PRESET_IMG: Partial<Record<IconKind, string>> = {
  universal: universalSrc,
  clarity: claritySrc,
  tape: tapeSrc,
  spatial: spatialSrc,
  oomph: oomphSrc,
  warmth: warmthSrc,
  punch: punchSrc,
  loud: loudSrc,
};

const PRESET_ALT: Record<IconKind, string> = {
  universal: "Universal preset",
  clarity: "Clarity preset",
  tape: "Tape preset",
  spatial: "Spatial preset",
  oomph: "Oomph preset",
  warmth: "Warmth preset",
  punch: "Punch preset",
  loud: "Loud preset",
  custom: "Custom preset",
};

export function PresetIcon({ kind, className, ...rest }: IconProps) {
  const ariaHidden = rest["aria-hidden"] ?? true;
  const src = PRESET_IMG[kind];
  if (src) {
    return (
      <img
        src={src}
        alt={ariaHidden ? "" : PRESET_ALT[kind]}
        aria-hidden={ariaHidden}
        className={className}
        draggable={false}
      />
    );
  }
  // Fallback for `custom` — open-ended, no canonical art. Render the
  // sliders icon so user-saved presets still get a recognizable glyph.
  return (
    <svg
      width={20}
      height={20}
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth={2}
      strokeLinecap="round"
      strokeLinejoin="round"
      className={className}
      aria-hidden={ariaHidden}
    >
      <path d="M10 8h4" />
      <path d="M12 21v-9" />
      <path d="M12 8V3" />
      <path d="M17 16h4" />
      <path d="M19 12V3" />
      <path d="M19 21v-5" />
      <path d="M3 14h4" />
      <path d="M5 10V3" />
      <path d="M5 21v-7" />
    </svg>
  );
}
