import { act } from "react";
import { createRoot } from "react-dom/client";
import { describe, expect, it } from "vitest";
import App from "./App";

function renderApp() {
  const container = document.createElement("div");
  document.body.appendChild(container);
  const root = createRoot(container);

  act(() => {
    root.render(<App />);
  });

  return { container, root };
}

function click(container: HTMLElement, selector: string) {
  const element = container.querySelector<HTMLElement>(selector);
  if (!element) throw new Error(`Missing element ${selector}`);
  act(() => {
    element.dispatchEvent(new MouseEvent("click", { bubbles: true }));
  });
}

describe("iPhone app shell", () => {
  it("opens as a Simple-only phone app without desktop advanced controls", () => {
    const { container, root } = renderApp();

    expect(container.textContent).toContain("YES Master");
    expect(container.textContent).toContain("Simple");
    expect(container.textContent).toContain("Import Track");
    expect(container.textContent).not.toContain("Advanced");
    expect(container.textContent).not.toContain("Album Master");

    act(() => root.unmount());
  });

  it("lets the user pick tone, loudness, profile, audition mode, and export", () => {
    const { container, root } = renderApp();

    click(container, "[data-testid='iphone-import']");
    click(container, "[data-testid='tone-warm']");
    click(container, "[data-testid='loudness-high']");
    click(container, "[data-testid='profile-cd']");
    click(container, "[data-testid='playback-mastered']");
    click(container, "[data-testid='volume-match']");
    click(container, "[data-testid='lufs-preview']");

    expect(container.textContent).toContain("Warm");
    expect(container.textContent).toContain("-10.5 LUFS");
    expect(container.textContent).toContain("44.1 kHz");
    expect(container.textContent).toContain("16-bit");
    expect(container.textContent).toContain("Mastered");
    expect(container.textContent).toContain("Export Master");

    act(() => root.unmount());
  });
});
