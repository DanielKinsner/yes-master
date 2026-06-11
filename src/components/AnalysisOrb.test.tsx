// src/components/AnalysisOrb.test.tsx
import { act } from "react";
import { createRoot } from "react-dom/client";
import { afterEach, describe, expect, it } from "vitest";
import { AnalysisOrb } from "./AnalysisOrb";

(globalThis as unknown as { IS_REACT_ACT_ENVIRONMENT: boolean }).IS_REACT_ACT_ENVIRONMENT = true;
afterEach(() => { document.body.innerHTML = ""; });

describe("AnalysisOrb", () => {
  it("renders a presentation canvas and survives jsdom's null 2d context", async () => {
    const container = document.createElement("div");
    document.body.appendChild(container);
    const root = createRoot(container);
    await act(async () => { root.render(<AnalysisOrb phase="orb" />); });
    const canvas = container.querySelector("canvas.wf-orb");
    expect(canvas).not.toBeNull();
    expect(canvas?.getAttribute("aria-hidden")).toBe("true");
    await act(async () => root.unmount());
  });

  it("renders the morph variant with peaks without crashing", async () => {
    const container = document.createElement("div");
    document.body.appendChild(container);
    const root = createRoot(container);
    await act(async () => {
      root.render(<AnalysisOrb phase="morph" peaks={[0.2, 0.9, 0.4]} />);
    });
    expect(container.querySelector("canvas.wf-orb.is-morph")).not.toBeNull();
    await act(async () => root.unmount());
  });
});
