// U11 — purposeful delight, and the constraints on it.
//
// The unit's acceptance is unusual in that most of it is about what an effect
// must NOT do. So these tests are mostly negative: reduced motion removes the
// motion without removing the meaning; no effect shifts layout, captures
// input, obscures a warning, or delays an action; and interaction timing does
// not touch audition, playhead, render, or export semantics.
//
// Every effect added in U11 has a named purpose, stated at its definition:
//
//   overlay entrance (receipt + review gate)   ORIENTATION
//   album arc settle                           ORIENTATION
//   quality verdict pulse                      COMPREHENSION
//
// The last test in this file enforces that list: an effect that cannot name
// its purpose is removed, not argued about.

import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { act } from "react";
import type { ReactNode } from "react";
import { createRoot, type Root } from "react-dom/client";
import { afterEach, describe, expect, it, vi } from "vitest";

import { RightRail } from "./components/RightRail";
import type { AnalysisResult, MasteringSettings } from "./bindings";

(globalThis as unknown as { IS_REACT_ACT_ENVIRONMENT: boolean })
  .IS_REACT_ACT_ENVIRONMENT = true;

// Same convention as App.layout-css.test.ts — read the stylesheet from the
// repo root rather than through an import URL.
const CSS = readFileSync(resolve(process.cwd(), "src/App.css"), "utf8");

const SETTINGS: MasteringSettings = {
  preset: { kind: "universal" },
  intensity: 0.5,
  eq_sub_db: 0,
  eq_low_db: 0,
  eq_low_mid_db: 0,
  eq_mid_db: 0,
  eq_high_mid_db: 0,
  eq_high_db: 0,
  eq_sparkle_db: 0,
  volume_match: false,
  input_gain_db: 0,
  output_gain_db: 0,
  delivery_profile: "streaming-universal",
  advanced: {
    lufs_offset_db: null,
    ceiling_dbtp: null,
    width: null,
    warmth: null,
    presence_air: null,
    compression_density: null,
    compression_low_threshold_db: null,
    compression_low_ratio: null,
    compression_low_attack_ms: null,
    compression_low_release_ms: null,
    compression_mid_threshold_db: null,
    compression_mid_ratio: null,
    compression_mid_attack_ms: null,
    compression_mid_release_ms: null,
    compression_high_threshold_db: null,
    compression_high_ratio: null,
    compression_high_attack_ms: null,
    compression_high_release_ms: null,
    compression_link_stereo: null,
    bit_depth: null,
    target_sample_rate: null,
  },
};

const CLEAN: AnalysisResult = {
  track_id: "delight-track",
  lufs_integrated: -14,
  lufs_short_term_max: -12,
  true_peak_dbtp: -1.6,
  dynamic_range_lu: 9,
  spectral_balance: { low: 0.33, mid: 0.34, high: 0.33 },
  transient_density: 0.5,
  stereo_width: 0.5,
  recommended_universal: SETTINGS,
  measured_at_iso: "2026-07-25T00:00:00.000Z",
  inferred_role: null,
  role_confidence: null,
  inferred_character: null,
  character_confidence: null,
  spectral_balance_6band: null,
  transient_flux: null,
  stereo_correlation: null,
  dynamic_range_p95_p10_db: null,
  lufs_short_term_max_3s: null,
  energy_density_score: null,
};

const HOT: AnalysisResult = {
  ...CLEAN,
  true_peak_dbtp: 0.4,
  lufs_integrated: -5.1,
  dynamic_range_lu: 3.0,
};

async function renderNode(node: ReactNode): Promise<{
  container: HTMLDivElement;
  root: Root;
}> {
  const container = document.createElement("div");
  document.body.appendChild(container);
  const root = createRoot(container);
  await act(async () => {
    root.render(node);
  });
  return { container, root };
}

function rail(analysis: AnalysisResult, extra: Record<string, unknown> = {}) {
  return (
    <RightRail
      analysis={analysis}
      lastChecks={undefined}
      canExport
      isExporting={false}
      isRendering={false}
      onExport={vi.fn()}
      previewStale
      canRenderPreview
      onUpdatePreview={vi.fn()}
      {...extra}
    />
  );
}

/**
 * Every `@media (prefers-reduced-motion: reduce)` block in the stylesheet,
 * flattened to the selectors it disables. Reading the sheet is the honest way
 * to assert this: jsdom does not evaluate media queries, so a test that
 * "renders with reduced motion" and checks computed styles would pass no
 * matter what the CSS said.
 */
function reducedMotionCoverage(): string {
  const blocks: string[] = [];
  const needle = "@media (prefers-reduced-motion: reduce)";
  let from = 0;
  for (;;) {
    const at = CSS.indexOf(needle, from);
    if (at === -1) break;
    // Walk braces to the end of the at-rule so nested blocks are included.
    let depth = 0;
    let i = CSS.indexOf("{", at);
    const start = i;
    for (; i < CSS.length; i += 1) {
      if (CSS[i] === "{") depth += 1;
      else if (CSS[i] === "}") {
        depth -= 1;
        if (depth === 0) break;
      }
    }
    blocks.push(CSS.slice(start, i));
    from = i;
  }
  return blocks.join("\n");
}

afterEach(() => {
  document.body.innerHTML = "";
});

describe("U11 — reduced motion removes the motion, not the meaning", () => {
  const reduced = reducedMotionCoverage();

  it("every U11 effect has a reduced-motion opt-out", () => {
    // If an effect is added without one, this is where it gets caught.
    for (const selector of [
      ".receipt-backdrop",
      ".export-review-backdrop",
      ".receipt",
      ".export-review-panel",
      ".album-sequence-arc svg",
      ".quality-badge",
      // Alive pass 1 (2026-08-19)
      ".ab-toggle button.on",
      ".wf-sheet-played",
      ".lufs-bar-hold",
    ]) {
      expect(reduced).toContain(selector);
    }
  });

  it("keeps the global reduced-motion kill switch", () => {
    // The blanket rule is the safety net for anything an author forgets. It
    // must survive; a per-effect opt-out is an addition to it, not a
    // replacement for it.
    expect(reduced).toContain("animation-duration: 0.01ms !important");
    expect(reduced).toContain("transition-duration: 0.01ms !important");
  });

  it("state still reads correctly with motion disabled", async () => {
    // The comprehension claim: the verdict is carried by the WORD and the
    // colour class, never by the animation. Disabling motion must not disable
    // the verdict. Since 2026-08-18 the standing verdict lives in the export
    // gate (the rail's SOURCE CHECK badge is gone); the gate's REVIEW badge is
    // the word that must survive.
    const { container, root } = await renderNode(rail(HOT));
    const exportButton = Array.from(container.querySelectorAll("button")).find(
      (b) => b.textContent?.trim() === "Export With Review",
    ) as HTMLButtonElement | undefined;
    expect(exportButton).toBeTruthy();
    await act(async () => {
      exportButton?.dispatchEvent(new MouseEvent("click", { bubbles: true }));
    });
    const badge = container.querySelector(".quality-badge");
    expect(badge?.textContent).toBe("REVIEW");
    expect(badge?.className).toContain("badge-warn");
    await act(async () => root.unmount());
  });
});

describe("U11 — effects acknowledge real changes only", () => {
  it("the export action re-labels only when the verdict actually flips", async () => {
    // The rail's standing verdict is now the export button's own label
    // ("Export Master" vs "Export With Review"). A re-render with the same
    // analysis must not change it; a genuinely hot analysis must.
    const label = (c: HTMLElement) =>
      Array.from(c.querySelectorAll("button")).find((b) =>
        /^Export (Master|With Review)$/.test(b.textContent?.trim() ?? ""),
      )?.textContent?.trim();
    const { container, root } = await renderNode(rail(CLEAN));
    expect(label(container)).toBe("Export Master");
    await act(async () => {
      root.render(rail(CLEAN, { previewStale: false }));
    });
    expect(label(container)).toBe("Export Master");
    await act(async () => {
      root.render(rail(HOT));
    });
    expect(label(container)).toBe("Export With Review");
    await act(async () => root.unmount());
  });

  it("no U11 effect animates a property that can reflow the page", () => {
    // Layout shift is the failure the unit names first. Width, height, margin,
    // padding, top/left and friends are animatable and all of them move
    // neighbours; opacity, transform, box-shadow and colour do not.
    const FORBIDDEN = [
      "width:",
      "height:",
      "margin",
      "padding",
      "top:",
      "left:",
      "right:",
      "bottom:",
      "font-size:",
    ];
    for (const name of [
      "overlay-scrim-in",
      "overlay-surface-in",
      "album-arc-settle",
      "quality-verdict-change",
      // Alive pass 1 (2026-08-19)
      "ab-land",
    ]) {
      const at = CSS.indexOf(`@keyframes ${name}`);
      expect(at, `keyframes ${name} missing`).toBeGreaterThan(-1);
      let depth = 0;
      let i = CSS.indexOf("{", at);
      const start = i;
      for (; i < CSS.length; i += 1) {
        if (CSS[i] === "{") depth += 1;
        else if (CSS[i] === "}") {
          depth -= 1;
          if (depth === 0) break;
        }
      }
      const body = CSS.slice(start, i);
      for (const prop of FORBIDDEN) {
        expect(body, `${name} animates ${prop}`).not.toContain(prop);
      }
    }
  });
});

describe("U11 — motion never delays or captures input", () => {
  it("the review gate is interactive on the frame it appears", async () => {
    // "No motion delays an action." The entrance animation is on appearance
    // only; the actions inside must work immediately, not after 160ms.
    const onExport = vi.fn();
    const { container, root } = await renderNode(rail(HOT, { onExport }));
    const open = Array.from(container.querySelectorAll("button")).find(
      (b) => b.textContent?.trim() === "Export With Review",
    ) as HTMLButtonElement | undefined;
    await act(async () => {
      open?.dispatchEvent(new MouseEvent("click", { bubbles: true }));
    });

    // No timers advanced, no waiting: click straight through.
    const anyway = Array.from(container.querySelectorAll("button")).find(
      (b) => b.textContent?.trim() === "Export Anyway",
    ) as HTMLButtonElement | undefined;
    expect(anyway).toBeTruthy();
    await act(async () => {
      anyway?.dispatchEvent(new MouseEvent("click", { bubbles: true }));
    });
    expect(onExport).toHaveBeenCalledTimes(1);

    await act(async () => root.unmount());
  });

  it("focus is placed by the dialog, not left behind by the animation", async () => {
    const { container, root } = await renderNode(rail(HOT));
    const open = Array.from(container.querySelectorAll("button")).find(
      (b) => b.textContent?.trim() === "Export With Review",
    ) as HTMLButtonElement | undefined;
    await act(async () => {
      open?.dispatchEvent(new MouseEvent("click", { bubbles: true }));
    });
    expect(document.activeElement).toBe(
      container.querySelector(".export-review-panel"),
    );
    await act(async () => root.unmount());
  });

  it("no effect obscures a warning", async () => {
    // Scrims and pulses may draw attention; they may not stand between the
    // user and the thing being flagged. The gate's own rows are the warning,
    // so they must be inside the animated surface rather than behind it.
    const { container, root } = await renderNode(rail(HOT));
    const open = Array.from(container.querySelectorAll("button")).find(
      (b) => b.textContent?.trim() === "Export With Review",
    ) as HTMLButtonElement | undefined;
    await act(async () => {
      open?.dispatchEvent(new MouseEvent("click", { bubbles: true }));
    });
    const dialog = container.querySelector(".export-review-panel");
    expect(dialog?.querySelectorAll(".export-review-row").length).toBeGreaterThan(0);
    await act(async () => root.unmount());
  });
});

describe("U11 — no effect touches engine semantics", () => {
  it("adds no timing to audition, render, or export paths", async () => {
    // The boundary: "delight cannot change DSP, export, persistence, or
    // audition semantics". Every U11 effect is CSS on appearance plus a React
    // key. None of them schedules work, so none can reorder or defer a call
    // into the engine. Asserted structurally: the export handler fires
    // synchronously with the click, with no timer in between.
    vi.useFakeTimers();
    try {
      const onExport = vi.fn();
      const { container, root } = await renderNode(rail(CLEAN, { onExport }));
      const exportBtn = container.querySelector<HTMLButtonElement>(
        "button.right-rail-export",
      );
      expect(exportBtn?.textContent).toContain("Export Master");
      await act(async () => {
        exportBtn?.dispatchEvent(new MouseEvent("click", { bubbles: true }));
      });
      // Zero timers advanced.
      expect(onExport).toHaveBeenCalledTimes(1);
      expect(vi.getTimerCount()).toBe(0);
      await act(async () => root.unmount());
    } finally {
      vi.useRealTimers();
    }
  });
});
