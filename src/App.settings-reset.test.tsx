// src/App.settings-reset.test.tsx
import { act } from "react";
import { createRoot } from "react-dom/client";
import { afterEach, describe, expect, it } from "vitest";
import { SettingsPanel } from "./App";
import { FIRST_RUN_GUIDE_KEY } from "./lib/first-run-guide";

(globalThis as unknown as { IS_REACT_ACT_ENVIRONMENT: boolean }).IS_REACT_ACT_ENVIRONMENT = true;
afterEach(() => {
  document.body.innerHTML = "";
  globalThis.localStorage?.removeItem(FIRST_RUN_GUIDE_KEY);
});

describe("SettingsPanel first-run tips reset", () => {
  it("clears the guide flag", async () => {
    globalThis.localStorage?.setItem(FIRST_RUN_GUIDE_KEY, "done");
    const container = document.createElement("div");
    document.body.appendChild(container);
    const root = createRoot(container);
    await act(async () => { root.render(<SettingsPanel onClose={() => {}} />); });
    const btn = Array.from(container.querySelectorAll<HTMLButtonElement>("button"))
      .find((b) => b.textContent?.includes("Show first-run tips again"))!;
    await act(async () => { btn.click(); });
    expect(globalThis.localStorage?.getItem(FIRST_RUN_GUIDE_KEY)).toBeNull();
    await act(async () => root.unmount());
  });
});
