import type { AnalysisResult, QualityCheck, QualityLevel } from "../bindings";

// Quality Check rows for the export receipt — the receipt's honesty surface.
//
// The row VALUES are the source analysis measurements (true peak / loudness /
// dynamic range). The row STATE is driven by the REAL export checks
// (src-tauri/src/exports.rs), which describe the delivered master along the
// same three axes. A dimension with no relevant check is genuinely clean
// (green); a warning/critical check flips its row to the honest state + colour
// and surfaces the check message.
//
// Format checks (bit_depth_low, sample_rate_mismatch) map to no source axis, so
// they are appended as their own rows — otherwise a warned export could render
// as an all-green card, which the receipt must never do.

export type QualityRowState = "ok" | "warning" | "critical";

export interface QualityRow {
  key: string;
  label: string;
  // Source measurement for the three axis rows; null for appended format rows
  // (whose meaning lives entirely in the note) or when analysis is unavailable.
  value: string | null;
  state: QualityRowState;
  note?: string;
}

interface Dimension {
  key: string;
  label: string;
  codes: string[];
  value: (a: AnalysisResult) => string;
}

const DIMENSIONS: Dimension[] = [
  {
    key: "true-peak",
    label: "Source true peak",
    codes: ["true_peak_high", "streaming_headroom_low"],
    value: (a) => `${a.true_peak_dbtp.toFixed(1)} dBTP`,
  },
  {
    key: "loudness",
    label: "Source loudness",
    codes: ["lufs_very_loud", "target_not_reached", "non_finite_metering"],
    value: (a) => `${a.lufs_integrated.toFixed(1)} LUFS`,
  },
  {
    key: "dynamic-range",
    label: "Source dynamic range",
    codes: ["dynamic_range_low", "comp_density_on_compressed_source"],
    value: (a) => `${a.dynamic_range_lu.toFixed(1)} LU`,
  },
];

const DIMENSION_CODES = new Set(DIMENSIONS.flatMap((d) => d.codes));

// Human labels for checks that don't belong to a source axis. Unknown future
// codes fall back to "Check" rather than being silently dropped.
const UNMAPPED_LABELS: Record<string, string> = {
  bit_depth_low: "Bit depth",
  sample_rate_mismatch: "Sample rate",
};

const STATE_RANK: Record<QualityRowState, number> = {
  ok: 0,
  warning: 1,
  critical: 2,
};

function levelToState(level: QualityLevel): QualityRowState {
  return level === "critical" ? "critical" : level === "warning" ? "warning" : "ok";
}

export function buildQualityRows(
  checks: QualityCheck[],
  analysis: AnalysisResult | null,
): QualityRow[] {
  const rows: QualityRow[] = DIMENSIONS.map((dim) => {
    // The check with the highest severity in this dimension drives the row.
    let chosen: QualityCheck | null = null;
    for (const check of checks) {
      if (!dim.codes.includes(check.code)) continue;
      if (
        !chosen ||
        STATE_RANK[levelToState(check.level)] > STATE_RANK[levelToState(chosen.level)]
      ) {
        chosen = check;
      }
    }
    return {
      key: dim.key,
      label: dim.label,
      value: analysis ? dim.value(analysis) : null,
      state: chosen ? levelToState(chosen.level) : "ok",
      note: chosen && levelToState(chosen.level) !== "ok" ? chosen.message : undefined,
    };
  });

  // Warning/critical checks that map to no source axis must still be seen.
  for (const check of checks) {
    if (check.code === "export_ok") continue;
    if (DIMENSION_CODES.has(check.code)) continue;
    const state = levelToState(check.level);
    if (state === "ok") continue;
    rows.push({
      key: `extra-${check.code}`,
      label: UNMAPPED_LABELS[check.code] ?? "Check",
      value: null,
      state,
      note: check.message,
    });
  }

  return rows;
}

// True when any row is warned or critical — the card must not present itself as
// fully clean when this is true.
export function hasQualityConcern(rows: QualityRow[]): boolean {
  return rows.some((r) => r.state !== "ok");
}
