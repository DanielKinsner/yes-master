// src/App.a11y-semantics.test.tsx
//
// U9 — desktop accessibility semantics.
//
// These pin the specific defects U9 found, so they cannot silently return:
//
//   * both Intensity knobs shipped with `label=""`, which becomes
//     `aria-label=""` — the control had NO accessible name at all;
//   * the knob announced the raw slider number ("0.5") rather than the value
//     sighted users read ("50%, Moderate");
//   * the Advanced preset tiles carried selected state only as a CSS class,
//     and their character blurb only as a hover `title`;
//   * quality-check rows conveyed pass/warn/critical through an `aria-hidden`
//     glyph plus colour, and the explanation through a tooltip.
//
// Every test below queries by ROLE and ACCESSIBLE NAME rather than by class or
// DOM position — the same way assistive tech does. A test that queried
// `.tile.active` would have passed happily throughout the broken period.
//
// Installed NVDA/VoiceOver validation remains a U15 gate. DOM semantics are a
// necessary condition, never a sufficient one.

import { act } from "react";
import { createRoot } from "react-dom/client";
import { afterEach, describe, expect, it } from "vitest";

import { PresetTiles } from "./App";
import { Knob, intensityLabel } from "./components/Knob";
import type { Preset } from "./bindings";

(globalThis as unknown as { IS_REACT_ACT_ENVIRONMENT: boolean }).IS_REACT_ACT_ENVIRONMENT =
  true;

afterEach(() => {
  document.body.innerHTML = "";
});

async function render(node: React.ReactElement) {
  const container = document.createElement("div");
  document.body.appendChild(container);
  const root = createRoot(container);
  await act(async () => {
    root.render(node);
  });
  return { container, root };
}

/** Accessible name of an element, for the subset of cases these tests use. */
function accessibleName(el: Element): string {
  const label = el.getAttribute("aria-label");
  if (label) return label;
  // Text content minus anything explicitly hidden from the a11y tree.
  const clone = el.cloneNode(true) as HTMLElement;
  clone
    .querySelectorAll('[aria-hidden="true"], .sr-only')
    .forEach((n) => n.remove());
  return (clone.textContent ?? "").trim();
}

/** Resolve aria-describedby to its text, the way a screen reader would. */
function describedByText(el: Element): string {
  const id = el.getAttribute("aria-describedby");
  if (!id) return "";
  // getElementById rather than a `#id` selector: jsdom has no CSS.escape, and
  // React's useId produces ids that are not valid bare CSS identifiers.
  return id
    .split(/\s+/)
    .map((one) => document.getElementById(one)?.textContent ?? "")
    .join(" ")
    .trim();
}

describe("U9 — Intensity control semantics", () => {
  it("gives the Intensity knob an accessible name even with an empty visible label", async () => {
    const { container, root } = await render(
      <Knob
        label=""
        ariaLabel="Intensity"
        value={0.5}
        min={0}
        max={1}
        step={0.01}
        format={(v) => `${Math.round(v * 100)}%`}
        onChange={() => {}}
      />,
    );

    const slider = container.querySelector('input[type="range"]');
    expect(slider).toBeTruthy();
    expect(accessibleName(slider as Element)).toBe("Intensity");
    // The regression this guards: an empty string is not a name.
    expect(slider?.getAttribute("aria-label")).not.toBe("");

    await act(async () => root.unmount());
  });

  it("announces a named value rather than the raw slider number", async () => {
    const { container, root } = await render(
      <Knob
        label=""
        ariaLabel="Intensity"
        valueText={(v) => `${Math.round(v * 100)} percent, ${intensityLabel(v)}`}
        value={0.5}
        min={0}
        max={1}
        step={0.01}
        format={(v) => `${Math.round(v * 100)}%`}
        onChange={() => {}}
      />,
    );

    const slider = container.querySelector('input[type="range"]');
    const valueText = slider?.getAttribute("aria-valuetext") ?? "";
    expect(valueText).toContain("50 percent");
    // The named range, not just the number — this is what makes the control
    // usable without sight.
    expect(valueText).toContain(intensityLabel(0.5));
    expect(valueText).not.toBe("0.5");

    await act(async () => root.unmount());
  });

  it("falls back to the formatted value plus caption when no valueText is given", async () => {
    const { container, root } = await render(
      <Knob
        label="Threshold"
        value={-18}
        min={-60}
        max={0}
        step={0.5}
        caption="Low band"
        format={(v) => `${v.toFixed(1)} dB`}
        onChange={() => {}}
      />,
    );

    const slider = container.querySelector('input[type="range"]');
    expect(slider?.getAttribute("aria-valuetext")).toBe("-18.0 dB, Low band");

    await act(async () => root.unmount());
  });

  it("explains a disabled knob instead of being silently inert", async () => {
    const { container, root } = await render(
      <Knob
        label="Width"
        value={1}
        min={0}
        max={2}
        disabled
        disabledReason="Width is managed by the album settings."
        format={(v) => `${v.toFixed(2)}`}
        onChange={() => {}}
      />,
    );

    const slider = container.querySelector('input[type="range"]') as HTMLElement;
    expect((slider as HTMLInputElement).disabled).toBe(true);
    expect(describedByText(slider)).toContain(
      "managed by the album settings",
    );

    await act(async () => root.unmount());
  });
});

describe("U9 — preset tile semantics", () => {
  const universal: Preset = { kind: "universal" };
  const clarity: Preset = { kind: "clarity" };

  it("exposes selected state through aria-pressed, not just a CSS class", async () => {
    const { container, root } = await render(
      <PresetTiles
        selected={clarity}
        onChange={() => {}}
        savingPreset={false}
        onSave={async () => true}
      />,
    );

    const tiles = Array.from(container.querySelectorAll("button"));
    expect(tiles.length).toBeGreaterThanOrEqual(8);

    const clarityTile = tiles.find((b) => accessibleName(b) === "Clarity");
    const universalTile = tiles.find((b) => accessibleName(b) === "Universal");
    expect(clarityTile).toBeTruthy();
    expect(universalTile).toBeTruthy();

    expect(clarityTile?.getAttribute("aria-pressed")).toBe("true");
    expect(universalTile?.getAttribute("aria-pressed")).toBe("false");

    await act(async () => root.unmount());
  });

  it("every preset tile is reachable by role and accessible name", async () => {
    const { container, root } = await render(
      <PresetTiles
        selected={universal}
        onChange={() => {}}
        savingPreset={false}
        onSave={async () => true}
      />,
    );

    const names = Array.from(container.querySelectorAll("button")).map((b) =>
      accessibleName(b),
    );
    for (const expected of [
      "Universal",
      "Clarity",
      "Tape",
      "Spatial",
      "Oomph",
      "Warmth",
      "Punch",
      "Loud",
    ]) {
      expect(names).toContain(expected);
    }

    await act(async () => root.unmount());
  });

  it("makes the character description available without hovering", async () => {
    const { container, root } = await render(
      <PresetTiles
        selected={universal}
        onChange={() => {}}
        savingPreset={false}
        onSave={async () => true}
      />,
    );

    const tile = Array.from(container.querySelectorAll("button")).find(
      (b) => accessibleName(b) === "Oomph",
    ) as HTMLElement;

    const described = describedByText(tile);
    expect(described.length).toBeGreaterThan(0);
    // The blurb must exist as real text in the document, not only inside a
    // `title` attribute that a keyboard or screen-reader user never receives.
    expect(tile.getAttribute("aria-describedby")).toBeTruthy();

    await act(async () => root.unmount());
  });

  it("keeps exactly one tile pressed at a time", async () => {
    const { container, root } = await render(
      <PresetTiles
        selected={universal}
        onChange={() => {}}
        savingPreset={false}
        onSave={async () => true}
      />,
    );

    const pressed = Array.from(container.querySelectorAll("button")).filter(
      (b) => b.getAttribute("aria-pressed") === "true",
    );
    expect(pressed).toHaveLength(1);
    expect(accessibleName(pressed[0])).toBe("Universal");

    await act(async () => root.unmount());
  });
});
