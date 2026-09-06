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
    // No "Build" row here: Help owns the real version + build stamp
    // (build_info command). A placeholder string beside a real one reads as
    // a bug (ship review 2026-09-01, S6.2).
    rows: [["Privacy", "Private audio stays on this machine"]],
  },
];

export const HELP_SECTIONS: Array<[string, string]> = [
  [
    "Standard view",
    "Styles choose the character, Low / Medium / High sets loudness, and Create Master asks where to save your WAV or MP3 master — it never overwrites your source.",
  ],
  [
    "Import / Analyze",
    "Each track becomes ready as its analysis completes. The track list shows how many remain. Drag a track's grip to reorder it, or focus the grip and use Up/Down. Album export follows this order.",
  ],
  [
    "Original vs Mastered",
    "Switch between Original and Mastered from the track header; playback keeps the same playhead where the backend can seek.",
  ],
  [
    "Volume Match / Preview LUFS",
    "Volume Match is for auditioning only. When Preview LUFS is enabled, the selected ready track starts measuring automatically. Measuring clears when its preview level is ready; your source file stays unchanged.",
  ],
  [
    "Delivery Profile / Format",
    "Delivery Profile sets the loudness target, ceiling and WAV format. WAV is the default. MP3 offers smaller files at 320, 256, 192 or 128 kbps and uses a compatible sample rate. Both formats use your chosen mastering settings.",
  ],
  [
    "Export Review",
    "Quality notes stay advisory so you can make creative choices, while technical mismatches such as delivery sample-rate disagreement are marked critical.",
  ],
  [
    "Keyboard shortcuts",
    "Space toggles playback. Left/Right seek 5 s (Shift: 30 s). Home or Return to start jumps to zero, keeps playing or paused, and turns looping off while retaining the region. A flips Original/Mastered; L toggles the loop (Advanced only). Ctrl/Cmd+Z/Y undo and redo. Shift+drag defines a loop in Advanced. Press ? for the full list.",
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
