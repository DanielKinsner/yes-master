import type { AnalysisResult, QualityCheck, QualityLevel } from "../bindings";

// Quality Check rows for the export receipt — the receipt's honesty surface.
//
// Row values use delivered measurements, with an explicitly labelled source
// fallback for older receipts. Row state is driven by the REAL export checks
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
  // Delivered/source measurement for axis rows; null for appended format rows
  // (whose meaning lives entirely in the note) or when analysis is unavailable.
  value: string | null;
  state: QualityRowState;
  note?: string;
}

type Measurements = Pick<AnalysisResult, "true_peak_dbtp" | "lufs_integrated" | "dynamic_range_lu">;

interface Dimension {
  key: string;
  label: string;
  codes: string[];
  value: (a: Measurements) => string;
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
    label: "Source loudness range (LRA)",
    codes: ["dynamic_range_low", "comp_density_on_compressed_source"],
    value: (a) => `${a.dynamic_range_lu.toFixed(1)} LU`,
  },
];

const DIMENSION_CODES = new Set(DIMENSIONS.flatMap((d) => d.codes));

// Human labels for checks that don't belong to a source axis.
const UNMAPPED_LABELS: Record<string, string> = {
  bit_depth_low: "Bit depth",
  sample_rate_mismatch: "Sample rate",
};

function humanizeCheckCode(code: string): string {
  const words = code.trim().replace(/[_-]+/g, " ").replace(/\s+/g, " ");
  return words ? `${words[0].toUpperCase()}${words.slice(1)}` : "Check";
}

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
  delivered?: Measurements | null,
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
      label: delivered ? dim.label.replace("Source", "Delivered") : dim.label,
      value: delivered ? dim.value(delivered) : analysis ? dim.value(analysis) : null,
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
      label: UNMAPPED_LABELS[check.code] ?? humanizeCheckCode(check.code),
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
