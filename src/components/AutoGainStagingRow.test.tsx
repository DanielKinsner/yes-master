import { act } from "react";
import { createRoot, type Root } from "react-dom/client";
import { afterEach, describe, expect, it, vi } from "vitest";
import type { AutoGainStagingView } from "../lib/auto-gain-staging";
import { AutoGainStagingRow } from "./AutoGainStagingRow";

(globalThis as unknown as { IS_REACT_ACT_ENVIRONMENT: boolean }).IS_REACT_ACT_ENVIRONMENT = true;

let root: Root | null = null;
let host: HTMLDivElement | null = null;

function render(view: AutoGainStagingView) {
  const onToggle = vi.fn();
  const onPriority = vi.fn();
  host = document.createElement("div");
  document.body.appendChild(host);
  root = createRoot(host);
  act(() => {
    root!.render(<AutoGainStagingRow view={view} onToggle={onToggle} onPriority={onPriority} />);
  });
  return { host, onToggle, onPriority };
}

afterEach(() => {
  act(() => root?.unmount());
  host?.remove();
  root = null;
  host = null;
});

const OFF: AutoGainStagingView = {
  state: "off",
  enabled: false,
  priority: "clean",
  line: "Sets the level for this song automatically, so it is limited only as much as your loudness needs.",
  pending: false,
  disabled: false,
  showPriority: false,
};

describe("AutoGainStagingRow", () => {
  it("shows the switch, the Beta tag and the off line, without the priority", () => {
    const { host, onToggle } = render(OFF);
    const toggle = host.querySelector<HTMLButtonElement>('[role="switch"]')!;
    expect(toggle.getAttribute("aria-checked")).toBe("false");
    expect(toggle.disabled).toBe(false);
    expect(host.textContent).toContain("Auto gain staging");
    expect(host.textContent).toContain("Beta");
    expect(host.querySelector('[role="status"]')!.textContent).toBe(OFF.line);
    expect(host.querySelector('[role="radiogroup"]')).toBeNull();
    act(() => toggle.click());
    expect(onToggle).toHaveBeenCalledTimes(1);
  });

  it("shows Loudness priority while on and reports the choice", () => {
    const { host, onPriority } = render({
      ...OFF,
      state: "limiting",
      enabled: true,
      showPriority: true,
      line: "Level in +2.1 dB · limiter drive +4.5 dB · limiting up to 3.1 dB.",
    });
    expect(host.querySelector('[role="switch"]')!.getAttribute("aria-checked")).toBe("true");
    const group = host.querySelector('[role="radiogroup"]')!;
    expect(group.getAttribute("aria-labelledby")).toBe("ags-priority-label");
    expect(host.textContent).toContain("Loudness priority");
    const options = Array.from(group.querySelectorAll<HTMLButtonElement>('[role="radio"]'));
    expect(options.map((o) => o.textContent)).toEqual(["Clean", "Reach target"]);
    expect(options.map((o) => o.getAttribute("aria-checked"))).toEqual(["true", "false"]);
    act(() => options[1].click());
    expect(onPriority).toHaveBeenCalledWith("reach_target");
  });

  it("marks a pending line busy", () => {
    const { host } = render({
      ...OFF,
      state: "measuring",
      enabled: true,
      showPriority: true,
      pending: true,
      line: "Measuring this song…",
    });
    expect(host.querySelector(".ags-row")!.getAttribute("aria-busy")).toBe("true");
    expect(host.querySelector('[role="status"]')!.className).toContain("is-pending");
  });

  it("is disabled in Album mode", () => {
    const { host, onToggle } = render({
      ...OFF,
      state: "album_disabled",
      enabled: true,
      disabled: true,
      line: "Auto gain staging applies to Track Master, not Album renders.",
    });
    const toggle = host.querySelector<HTMLButtonElement>('[role="switch"]')!;
    expect(toggle.disabled).toBe(true);
    expect(toggle.getAttribute("aria-checked")).toBe("false");
    act(() => toggle.click());
    expect(onToggle).not.toHaveBeenCalled();
    expect(host.querySelector('[role="radiogroup"]')).toBeNull();
  });
});
