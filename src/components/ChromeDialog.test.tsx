import { StrictMode, act, useState } from "react";
import { createRoot, type Root } from "react-dom/client";
import { afterEach, describe, expect, it, vi } from "vitest";
import { ChromeDialog } from "./ChromeDialog";

(globalThis as unknown as { IS_REACT_ACT_ENVIRONMENT: boolean }).IS_REACT_ACT_ENVIRONMENT = true;

let root: Root | null = null;

function mount() {
  function Harness() {
    const [open, setOpen] = useState(false);
    const [count, setCount] = useState(0);
    return <>
      <button onClick={() => setOpen(true)}>Open Help</button>
      <button>Background action</button>
      {open && <ChromeDialog title="Help" eyebrow="Guide" onClose={() => setOpen(false)}>
        <button disabled>Unavailable</button>
        <button hidden>Hidden</button>
        <button onClick={() => setCount(count + 1)}>Diagnostics {count}</button>
      </ChromeDialog>}
    </>;
  }
  const container = document.createElement("div");
  document.body.append(container);
  act(() => {
    root = createRoot(container);
    root.render(<StrictMode><Harness /></StrictMode>);
  });
  const opener = container.querySelector<HTMLButtonElement>("button")!;
  opener.focus();
  act(() => opener.click());
  return { container, opener };
}

function tab(shiftKey = false) {
  const event = new KeyboardEvent("keydown", { key: "Tab", shiftKey, bubbles: true, cancelable: true });
  act(() => document.activeElement?.dispatchEvent(event));
  return event;
}

afterEach(() => {
  act(() => root?.unmount());
  root = null;
  document.body.innerHTML = "";
  vi.useRealTimers();
});

describe("ChromeDialog keyboard behavior", () => {
  it("keeps forward and reverse Tab inside the visible enabled controls", () => {
    const { container } = mount();
    const dialog = container.querySelector<HTMLElement>('[role="dialog"]')!;
    const close = container.querySelector<HTMLButtonElement>('[aria-label="Close Help"]')!;
    const diagnostics = Array.from(dialog.querySelectorAll("button")).at(-1)!;
    expect(document.activeElement).toBe(dialog);
    expect(tab().defaultPrevented).toBe(true);
    expect(document.activeElement).toBe(close);
    tab();
    expect(document.activeElement).toBe(diagnostics);
    tab();
    expect(document.activeElement).toBe(close);
    tab(true);
    expect(document.activeElement).toBe(diagnostics);
    tab(true);
    expect(document.activeElement).toBe(close);
  });

  it("does not steal focus when content updates and the close callback changes", () => {
    const { container } = mount();
    const diagnostics = Array.from(container.querySelectorAll("button")).at(-1)!;
    diagnostics.focus();
    act(() => diagnostics.click());
    expect(diagnostics.textContent).toBe("Diagnostics 1");
    expect(document.activeElement).toBe(diagnostics);
  });

  it.each(["Escape", "close", "backdrop"])("restores the opener after %s closes the dialog", (method) => {
    vi.useFakeTimers();
    const { container, opener } = mount();
    act(() => {
      if (method === "Escape") window.dispatchEvent(new KeyboardEvent("keydown", { key: "Escape" }));
      else container.querySelector<HTMLElement>(method === "close" ? '[aria-label="Close Help"]' : ".chrome-dialog-backdrop")!.click();
    });
    act(() => vi.runAllTimers());
    expect(container.querySelector('[role="dialog"]')).toBeNull();
    expect(document.activeElement).toBe(opener);
  });
});
