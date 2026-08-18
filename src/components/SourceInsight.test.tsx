import { act } from "react";
import { createRoot, type Root } from "react-dom/client";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import type { AnalysisResult, QualityCheck } from "../bindings";
import { useInsightReview } from "../hooks/useInsightReview";
import { SourceInsight } from "./SourceInsight";

// Source Insight (2026-08-18) — the analysis disclosure under the track
// title, which took over the right rail's SOURCE CHECK. Pins: the
// measurements are presented (with the old severities), Re-analyze lives
// here, and REVIEW is an unacknowledged-revision flag, not a warning.

const HOT: AnalysisResult = {
  track_id: "track-1",
  lufs_integrated: -10.5,
  lufs_short_term_max: -8,
  true_peak_dbtp: 0.2,
  dynamic_range_lu: 3.3,
  spectral_balance: { low: 0.4, mid: 0.4, high: 0.2 },
  transient_density: 0.5,
  stereo_width: 0.5,
  recommended_universal: undefined as unknown as AnalysisResult["recommended_universal"],
  measured_at_iso: "2026-08-18T10:00:00Z",
} as AnalysisResult;

let root: Root | null = null;
let host: HTMLDivElement | null = null;
function mount(node: React.ReactNode) {
  host = document.createElement("div");
  document.body.appendChild(host);
  root = createRoot(host);
  act(() => root!.render(node));
  return host;
}
function rerender(node: React.ReactNode) {
  act(() => root!.render(node));
}
afterEach(() => {
  act(() => root?.unmount());
  host?.remove();
  root = null;
  host = null;
});
beforeEach(() => {
  window.localStorage.clear();
});

const click = (el: Element | null | undefined) =>
  act(() => {
    el?.dispatchEvent(new MouseEvent("click", { bubbles: true }));
  });

describe("SourceInsight — presentation", () => {
  it("shows the headline collapsed and the structured measurements when opened", () => {
    const el = mount(<SourceInsight analysis={HOT} unreviewed={false} onAcknowledge={vi.fn()} />);
    expect(el.textContent).toContain("Source loud at -10.5 LUFS");
    expect(el.querySelector(".source-insight-panel")).toBeNull();
    click(el.querySelector(".source-insight-toggle"));
    const panel = el.querySelector(".source-insight-panel");
    expect(panel).not.toBeNull();
    // The old Source Check facts, now structured — value + reading + status.
    expect(panel!.textContent).toContain("Loudness");
    expect(panel!.textContent).toContain("-10.5 LUFS");
    expect(panel!.textContent).toContain("True peak");
    expect(panel!.textContent).toContain("0.20 dBTP");
    expect(panel!.textContent).toContain("Dynamic range");
    expect(panel!.textContent).toContain("3.3 LU");
    // Severity is text, not just a glyph/colour.
    expect(panel!.textContent).toContain("Problem");
    expect(el.querySelector(".source-insight-item.is-problem")).not.toBeNull();
  });

  it("hosts Re-analyze (and disables it while analyzing)", () => {
    const onReanalyze = vi.fn();
    const el = mount(
      <SourceInsight analysis={HOT} unreviewed={false} onAcknowledge={vi.fn()} onReanalyze={onReanalyze} />,
    );
    click(el.querySelector(".source-insight-toggle"));
    const btn = el.querySelector<HTMLButtonElement>(".source-insight-reanalyze");
    expect(btn?.textContent).toContain("Re-analyze");
    click(btn);
    expect(onReanalyze).toHaveBeenCalledTimes(1);
    rerender(
      <SourceInsight analysis={HOT} unreviewed={false} onAcknowledge={vi.fn()} onReanalyze={onReanalyze} isAnalyzing />,
    );
    expect(el.querySelector<HTMLButtonElement>(".source-insight-reanalyze")?.disabled).toBe(true);
  });

  it("lists last-export checks as a second group with severity as text", () => {
    const checks: QualityCheck[] = [
      { code: "true_peak_high", level: "warning", message: "True peak reached -0.2 dBTP." },
      { code: "export_ok", level: "info", message: "ok" },
    ];
    const el = mount(<SourceInsight analysis={HOT} lastChecks={checks} unreviewed={false} onAcknowledge={vi.fn()} />);
    click(el.querySelector(".source-insight-toggle"));
    expect(el.textContent).toContain("Last export");
    expect(el.textContent).toContain("True peak above safe ceiling");
    expect(el.textContent).toContain("Caution. True peak reached -0.2 dBTP.");
  });

  it("closes on Escape", () => {
    const el = mount(<SourceInsight analysis={HOT} unreviewed={false} onAcknowledge={vi.fn()} />);
    click(el.querySelector(".source-insight-toggle"));
    expect(el.querySelector(".source-insight-panel")).not.toBeNull();
    act(() => {
      window.dispatchEvent(new KeyboardEvent("keydown", { key: "Escape" }));
    });
    expect(el.querySelector(".source-insight-panel")).toBeNull();
  });
});

// A tiny host that wires the hook exactly the way TrackHeader does.
function Host({ analysis, checks }: { analysis: AnalysisResult; checks?: QualityCheck[] }) {
  const review = useInsightReview();
  return (
    <SourceInsight
      analysis={analysis}
      lastChecks={checks}
      unreviewed={review.isUnreviewed(analysis)}
      onAcknowledge={() => review.acknowledge(analysis)}
      onReanalyze={vi.fn()}
    />
  );
}

describe("SourceInsight — REVIEW is an unacknowledged revision, not a warning", () => {
  it("shows REVIEW for a fresh analysis, clears it on acknowledge, and keeps findings", () => {
    const el = mount(<Host analysis={HOT} />);
    const badge = () => el.querySelector<HTMLButtonElement>(".source-insight-review");
    expect(badge()).not.toBeNull();
    // Clicking the badge acknowledges without opening the panel.
    click(badge());
    expect(badge()).toBeNull();
    expect(el.querySelector(".source-insight-panel")).toBeNull();
    // The findings — including the hot true peak — are still there.
    click(el.querySelector(".source-insight-toggle"));
    expect(el.querySelector(".source-insight-item.is-problem")).not.toBeNull();
    expect(el.textContent).toContain("Reviewed ✓");
  });

  it("stays reviewed across re-renders and open/close, returns for a new revision or source", () => {
    const el = mount(<Host analysis={HOT} />);
    click(el.querySelector(".source-insight-toggle"));
    click(el.querySelector(".source-insight-ack"));
    expect(el.querySelector(".source-insight-review")).toBeNull();
    // Same revision, new object identity (ordinary settings churn) → still reviewed.
    rerender(<Host analysis={{ ...HOT }} />);
    click(el.querySelector(".source-insight-toggle")); // close
    click(el.querySelector(".source-insight-toggle")); // open
    expect(el.querySelector(".source-insight-review")).toBeNull();
    // Re-analyze → new measured_at → REVIEW is back.
    rerender(<Host analysis={{ ...HOT, measured_at_iso: "2026-08-18T11:00:00Z" }} />);
    expect(el.querySelector(".source-insight-review")).not.toBeNull();
    // A different source (track id) is a new revision too.
    click(el.querySelector(".source-insight-review"));
    rerender(<Host analysis={{ ...HOT, track_id: "track-2" }} />);
    expect(el.querySelector(".source-insight-review")).not.toBeNull();
  });

  it("acknowledgement persists across remounts (localStorage)", () => {
    const el = mount(<Host analysis={HOT} />);
    click(el.querySelector(".source-insight-review"));
    act(() => root?.unmount());
    host?.remove();
    const el2 = mount(<Host analysis={HOT} />);
    expect(el2.querySelector(".source-insight-review")).toBeNull();
  });
});
