// src/App.settings-reset.test.tsx
import { act } from "react";
import { createRoot } from "react-dom/client";
import { afterEach, describe, expect, it, vi } from "vitest";
import { SettingsPanel } from "./App";
import { FIRST_RUN_GUIDE_KEY } from "./lib/first-run-guide";

(globalThis as unknown as { IS_REACT_ACT_ENVIRONMENT: boolean }).IS_REACT_ACT_ENVIRONMENT = true;
afterEach(() => {
  document.body.innerHTML = "";
  globalThis.localStorage?.removeItem(FIRST_RUN_GUIDE_KEY);
});

describe("SettingsPanel first-run tips reset", () => {
  it("writes the reset marker and closes the dialog so the chip is visible", async () => {
    globalThis.localStorage?.setItem(FIRST_RUN_GUIDE_KEY, "done");
    const onClose = vi.fn();
    const container = document.createElement("div");
    document.body.appendChild(container);
    const root = createRoot(container);
    await act(async () => { root.render(<SettingsPanel onClose={onClose} />); });
    const btn = Array.from(container.querySelectorAll<HTMLButtonElement>("button"))
      .find((b) => b.textContent?.includes("Show first-run tips again"))!;
    await act(async () => { btn.click(); });
    // "reset" (not cleared): the marker survives to the next mount so an
    // explicit reset is never swallowed by the fast-user silent finish.
    expect(globalThis.localStorage?.getItem(FIRST_RUN_GUIDE_KEY)).toBe("reset");
    expect(onClose).toHaveBeenCalled();
    await act(async () => root.unmount());
  });
});
