// Static copy for the Settings and Help chrome dialogs. Lives outside the
// React tree so copy edits don't touch App.tsx; the exact strings are pinned
// by App.chrome.test.tsx.

import { STANDARD_EXPORT_DELIVERY } from "./standard-export";

function formatSampleRate(sampleRate: number): string {
  const khz = sampleRate / 1_000;
  return Number.isInteger(khz) ? `${khz} kHz` : `${khz.toFixed(1)} kHz`;
}

function formatDbtp(value: number): string {
  return value < 0 ? `−${Math.abs(value)} dBTP` : `${value} dBTP`;
}

function standardExportFormatCopy(): string {
  return `${formatSampleRate(STANDARD_EXPORT_DELIVERY.sampleRate)}, ${STANDARD_EXPORT_DELIVERY.bitDepth}-bit WAV, ${formatDbtp(STANDARD_EXPORT_DELIVERY.ceilingDbtp)}`;
}

export const SETTINGS_GROUPS: Array<{
  title: string;
  rows: Array<[string, string]>;
}> = [
  {
    title: "Audio Preview",
    rows: [
      ["Preview LUFS", "Off by default"],
      ["Volume Match", "Session-only audition"],
      ["Mastered cache", "Prewarm selected tracks when possible"],
    ],
  },
  {
    title: "Export Defaults",
    rows: [
      ["Standard · Create Master", standardExportFormatCopy()],
      ["Advanced · delivery profile", "Streaming Universal — 48 kHz, 24-bit WAV"],
      ["Warnings", "Advisory unless a technical check is critical"],
    ],
  },
  {
    title: "Project Session",
    rows: [
      ["Recent session", "Autosaved locally"],
      ["Project files", ".ams.json Save As / Open"],
      ["Audio files", "Referenced from disk, not embedded"],
    ],
  },
  {
    title: "App Info",
    rows: [
      ["Build", "Local desktop build"],
      ["Privacy", "Private audio stays on this machine"],
    ],
  },
];

export const HELP_SECTIONS: Array<[string, string]> = [
  [
    "Standard view",
    "Styles choose the character, Low / Medium / High sets loudness, and Create Master writes a finished WAV next to the source without overwriting it.",
  ],
  [
    "Import / Analyze",
    "Import audio files, then let analysis populate loudness, true peak, dynamics, waveform, and source checks before export.",
  ],
  [
    "Original vs Mastered",
    "Switch between Original and Mastered from the track header; playback keeps the same playhead where the backend can seek.",
  ],
  [
    "Volume Match / Preview LUFS",
    "Volume Match is for auditioning only. Preview LUFS estimates export loudness during Mastered playback and does not change the source file.",
  ],
  [
    "Delivery Profile / Format",
    "Delivery Profile owns target LUFS, ceiling, bit depth, and sample rate. Custom lets you choose Source, 44.1 kHz, 48 kHz, or 96 kHz for Track Master export.",
  ],
  [
    "Export Review",
    "Quality notes stay advisory so you can make creative choices, while technical mismatches such as delivery sample-rate disagreement are marked critical.",
  ],
  [
    "Keyboard shortcuts",
    "Space toggles playback. Ctrl/Cmd+Z/Y undo and redo. Shift+drag loop region is Advanced only.",
  ],
  [
    "Glossary",
    "LUFS is overall loudness, dBTP is true-peak headroom, and dynamic range is how much the track breathes between quiet and loud moments.",
  ],
  [
    "Save / Open Project",
    "Save Project writes a .ams.json snapshot. Open Project restores tracks and settings, then refreshes analysis and waveforms when the source files are still available.",
  ],
];
