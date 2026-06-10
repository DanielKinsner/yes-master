// Static copy for the Settings and Help chrome dialogs. Lives outside the
// React tree so copy edits don't touch App.tsx; the exact strings are pinned
// by App.chrome.test.tsx.

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
      ["Delivery profile", "Streaming Universal"],
      ["Rendered format", "48 kHz, 24-bit WAV"],
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
    "Save / Open Project",
    "Save Project writes a .ams.json snapshot. Open Project restores tracks and settings, then refreshes analysis and waveforms when the source files are still available.",
  ],
];
