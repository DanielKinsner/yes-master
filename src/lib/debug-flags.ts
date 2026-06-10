// Debug/iteration flags, persisted per machine in localStorage. Deliberately
// NOT MasteringSettings fields — they must never reach the wire, the chain,
// or an export receipt. (Owner decision 2026-06-09, consolidated backlog P3.)

const ADAPTIVE_READOUT_KEY = "yes-master:debug:adaptive-readout";

/// The per-axis AdaptiveReadout under the Adapt Strength slider is an
/// iteration aid for calibrating the guardrails by ear (owner TODO
/// 2026-06-08; docs/ADAPTIVE_DSP_NEXT_STEPS.md — "hide, don't delete").
/// Hidden by default for release; re-surface it during tuning sessions from
/// devtools:
///   localStorage.setItem("yes-master:debug:adaptive-readout", "1")
export function adaptiveReadoutEnabled(): boolean {
  try {
    return globalThis.localStorage?.getItem(ADAPTIVE_READOUT_KEY) === "1";
  } catch {
    return false;
  }
}

export function setAdaptiveReadoutEnabled(enabled: boolean): void {
  try {
    if (enabled) {
      globalThis.localStorage?.setItem(ADAPTIVE_READOUT_KEY, "1");
    } else {
      globalThis.localStorage?.removeItem(ADAPTIVE_READOUT_KEY);
    }
  } catch {
    // localStorage unavailable (non-browser test env) — flag stays off.
  }
}
