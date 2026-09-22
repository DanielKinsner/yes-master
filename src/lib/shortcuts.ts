// Keyboard shortcuts — one catalogue, read by the transport key handler
// (hooks/useTrackMaster.ts), the `?` overlay (App.tsx), and the tests, so
// the overlay can never advertise a key the handler does not honour.
//
// Pass 4 (2026-08-19). Keys are plain (no modifier) unless stated; every
// handler skips text entry (inputs except range/number/checkbox/radio,
// textareas, contenteditable) so typing is never hijacked, and the seek /
// loop / A-B keys ALSO skip range/number inputs and selects, where arrows
// and letters already mean something.

export type ShortcutEntry = {
  keys: string[];
  label: string;
  /// "advanced" = only meaningful in Advanced (Standard has no loop UI).
  scope?: "advanced";
};

export const SEEK_STEP_SEC = 5;
export const SEEK_STEP_LARGE_SEC = 30;

export const SHORTCUTS: readonly ShortcutEntry[] = [
  { keys: ["Space"], label: "Play / pause" },
  { keys: ["←", "→"], label: `Seek ${SEEK_STEP_SEC} s` },
  { keys: ["Shift", "←/→"], label: `Seek ${SEEK_STEP_LARGE_SEC} s` },
  { keys: ["Home"], label: "Jump to start" },
  { keys: ["A"], label: "Flip Original / Mastered" },
  { keys: ["L"], label: "Toggle loop region", scope: "advanced" },
  { keys: ["Ctrl", "Z"], label: "Undo" },
  { keys: ["Ctrl", "Shift", "Z"], label: "Redo (or Ctrl+Y)" },
  { keys: ["?"], label: "Show this list" },
];

/// True when a keydown on `target` is the user typing TEXT — the one case
/// every global shortcut must yield to.
export function isTextEntryTarget(target: EventTarget | null): boolean {
  const el = target as HTMLElement | null;
  if (!el) return false;
  const tag = el.tagName;
  const type = (el as HTMLInputElement).type;
  return (
    tag === "TEXTAREA" ||
    (el.isContentEditable ?? false) ||
    (tag === "INPUT" &&
      type !== "range" &&
      type !== "number" &&
      type !== "checkbox" &&
      type !== "radio")
  );
}

/// True while an aria-modal dialog is open. Everything behind a modal is
/// inert, so no global shortcut may reach the workspace (2026-09-22: Space
/// in Help started playback, A flipped Original/Mastered behind it). Every
/// app dialog is mounted only while open, so presence means open.
export function isModalDialogOpen(): boolean {
  return document.querySelector('[role="dialog"][aria-modal="true"]') !== null;
}

/// True when the focused control already consumes arrows / letters (knob
/// range inputs, number fields, selects) — seek / A / L stay out of its way.
export function isValueControlTarget(target: EventTarget | null): boolean {
  const el = target as HTMLElement | null;
  if (!el) return false;
  const tag = el.tagName;
  return tag === "INPUT" || tag === "SELECT" || tag === "TEXTAREA" || (el.isContentEditable ?? false);
}
