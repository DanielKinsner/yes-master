import { describe, expect, it } from "vitest";

import { buildQualityRows, hasQualityConcern } from "./receipt-quality";
import type { AnalysisResult, QualityCheck } from "../bindings";

// buildQualityRows only reads three source measurements — cast a minimal stub.
const analysis = {
  true_peak_dbtp: 2.5,
  lufs_integrated: -7.5,
  dynamic_range_lu: 6.6,
} as unknown as AnalysisResult;

function check(level: QualityCheck["level"], code: string, message = code): QualityCheck {
  return { level, code, message };
}

describe("buildQualityRows", () => {
  it("uses delivered values beside export checks rather than unrelated source measurements", () => {
    const rows = buildQualityRows([check("warning", "true_peak_high")], analysis, {
      true_peak_dbtp: 0.2, lufs_integrated: -14, dynamic_range_lu: 3,
    });
    expect(rows[0]).toMatchObject({ label: "Delivered true peak", value: "0.2 dBTP", state: "warning" });
    expect(rows[1].value).toBe("-14.0 LUFS");
    expect(rows[2].label).toBe("Delivered loudness range (LRA)");
  });
  it("renders the three source rows all-clean when the export is ok", () => {
    const rows = buildQualityRows([check("info", "export_ok")], analysis);
    expect(rows.map((r) => r.key)).toEqual(["true-peak", "loudness", "dynamic-range"]);
    expect(rows.every((r) => r.state === "ok")).toBe(true);
    expect(rows.map((r) => r.value)).toEqual(["2.5 dBTP", "-7.5 LUFS", "6.6 LU"]);
    expect(rows.every((r) => r.note === undefined)).toBe(true);
    expect(hasQualityConcern(rows)).toBe(false);
  });

  it("flips only the loudness row to warning and carries its message", () => {
    const rows = buildQualityRows(
      [check("warning", "lufs_very_loud", "Integrated loudness is -6.0 LUFS.")],
      analysis,
    );
    const byKey = Object.fromEntries(rows.map((r) => [r.key, r]));
    expect(byKey["loudness"].state).toBe("warning");
    expect(byKey["loudness"].note).toBe("Integrated loudness is -6.0 LUFS.");
    expect(byKey["true-peak"].state).toBe("ok");
    expect(byKey["dynamic-range"].state).toBe("ok");
    expect(hasQualityConcern(rows)).toBe(true);
  });

  it("maps the streaming-headroom advisory to the true-peak row", () => {
    const rows = buildQualityRows([check("warning", "streaming_headroom_low")], analysis);
    expect(rows.find((r) => r.key === "true-peak")?.state).toBe("warning");
  });

  it("appends a row for a critical check that maps to no source axis", () => {
    // sample_rate_mismatch is a format defect — it must surface so the card is
    // never all-green while a critical issue exists.
    const rows = buildQualityRows(
      [check("critical", "sample_rate_mismatch", "Rendered 44100 Hz != 48000 Hz.")],
      analysis,
    );
    expect(rows.slice(0, 3).every((r) => r.state === "ok")).toBe(true);
    const extra = rows.find((r) => r.key === "extra-sample_rate_mismatch");
    expect(extra).toMatchObject({
      label: "Sample rate",
      state: "critical",
      value: null,
      note: "Rendered 44100 Hz != 48000 Hz.",
    });
    expect(hasQualityConcern(rows)).toBe(true);
  });

  it("turns an unknown future check code into a useful human label", () => {
    const rows = buildQualityRows(
      [check("critical", "true_peak_over_ceiling", "True peak exceeds delivery ceiling.")],
      analysis,
    );
    expect(rows.find((r) => r.key === "extra-true_peak_over_ceiling")).toMatchObject({
      label: "True peak over ceiling",
      state: "critical",
    });
  });

  it("takes the worst level when a dimension has multiple checks", () => {
    const rows = buildQualityRows(
      [
        check("warning", "dynamic_range_low", "DR low"),
        check("warning", "comp_density_on_compressed_source", "already compressed"),
      ],
      analysis,
    );
    expect(rows.find((r) => r.key === "dynamic-range")?.state).toBe("warning");
  });

  it("still computes states with null analysis, leaving values blank", () => {
    const rows = buildQualityRows([check("warning", "lufs_very_loud")], null);
    expect(rows.find((r) => r.key === "loudness")).toMatchObject({
      value: null,
      state: "warning",
    });
  });
});
