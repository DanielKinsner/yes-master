// src/App.standard-view.test.tsx
import { act } from "react";
import { createRoot } from "react-dom/client";
import { afterEach, describe, expect, it } from "vitest";

import { TopHeader } from "./App";

(globalThis as unknown as { IS_REACT_ACT_ENVIRONMENT: boolean }).IS_REACT_ACT_ENVIRONMENT = true;

afterEach(() => { document.body.innerHTML = ""; });

async function render(node: React.ReactElement) {
  const container = document.createElement("div");
  document.body.appendChild(container);
  const root = createRoot(container);
  await act(async () => { root.render(node); });
  return { container, root };
}

describe("TopHeader Standard/Advanced affordance", () => {
  it("shows an Advanced door when in Standard", async () => {
    const { container, root } = await render(
      <TopHeader
        mode="track" onModeChange={() => {}}
        onSaveProject={() => {}} onOpenProject={() => {}}
        onOpenSettings={() => {}} onOpenHelp={() => {}}
        canUndo={false} canRedo={false}
        onUndo={() => {}} onRedo={() => {}}
        viewMode="standard" onEnterAdvanced={() => {}} onBackToStandard={() => {}}
      />,
    );
    const btn = Array.from(container.querySelectorAll("button")).find((b) => b.textContent === "Advanced");
    expect(btn).toBeTruthy();
    await act(async () => root.unmount());
  });

  it("shows Back to Standard when in Advanced", async () => {
    const { container, root } = await render(
      <TopHeader
        mode="track" onModeChange={() => {}}
        onSaveProject={() => {}} onOpenProject={() => {}}
        onOpenSettings={() => {}} onOpenHelp={() => {}}
        canUndo={false} canRedo={false}
        onUndo={() => {}} onRedo={() => {}}
        viewMode="advanced" onEnterAdvanced={() => {}} onBackToStandard={() => {}}
      />,
    );
    const btn = Array.from(container.querySelectorAll("button")).find((b) => b.textContent?.includes("Back to Standard"));
    expect(btn).toBeTruthy();
    await act(async () => root.unmount());
  });
});
