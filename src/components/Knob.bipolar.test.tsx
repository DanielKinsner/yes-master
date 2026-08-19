import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { act } from "react";
import { createRoot, type Root } from "react-dom/client";
import { afterEach, describe, expect, it, vi } from "vitest";

import {
  BIPOLAR_BOOST_COLOR,
  BIPOLAR_CUT_COLOR,
  Knob,
  bipolarRole,
} from "./Knob";

// 2026-08-19 (owner): EQ gain knobs + input/output gain are BIPOLAR — the
// arc grows from the centre, blue above 0 dB and amber below, matching the
// Visual EQ's boost/cut palette. These pin that contract so the knob and the
// EQ nodes can't drift apart.

let root: Root | null = null;
let host: HTMLDivElement | null = null;
function mount(node: React.ReactNode) {
  host = document.createElement("div");
  document.body.appendChild(host);
  root = createRoot(host);
  act(() => root!.render(node));
  return host;
}
afterEach(() => {
  act(() => root?.unmount());
  host?.remove();
  root = null;
  host = null;
});

function knob(value: number) {
  return (
    <Knob
      label="Low"
      value={value}
      min={-12}
      max={12}
      step={0.1}
      defaultValue={0}
      bipolar
      format={(v) => `${v.toFixed(1)} dB`}
      onChange={vi.fn()}
    />
  );
}

describe("Knob — bipolar boost/cut", () => {
  it("classifies sign with a half-step deadband at the centre", () => {
    expect(bipolarRole(0, 0, 0.1)).toBe("flat");
    expect(bipolarRole(0.04, 0, 0.1)).toBe("flat");
    expect(bipolarRole(0.1, 0, 0.1)).toBe("boost");
    expect(bipolarRole(-0.1, 0, 0.1)).toBe("cut");
  });

  it("goes blue above 0 dB, amber below, neutral at rest", () => {
    const el = mount(knob(3));
    const knobEl = el.querySelector(".knob")!;
    expect(knobEl.className).toContain("knob-bipolar");
    expect(knobEl.className).toContain("knob-role-boost");
    expect((knobEl as HTMLElement).style.getPropertyValue("--knob-tone")).toBe(BIPOLAR_BOOST_COLOR);

    act(() => root!.render(knob(-3)));
    expect(knobEl.className).toContain("knob-role-cut");
    expect((knobEl as HTMLElement).style.getPropertyValue("--knob-tone")).toBe(BIPOLAR_CUT_COLOR);

    act(() => root!.render(knob(0)));
    expect(knobEl.className).toContain("knob-role-flat");
  });

  it("draws no value arc at the centre and an arc on either side of it", () => {
    const el = mount(knob(0));
    // Track arc only (the full 270° background) — no lit value arc.
    const arcs = () => el.querySelectorAll(`path[stroke="${BIPOLAR_BOOST_COLOR}"], path[stroke="${BIPOLAR_CUT_COLOR}"]`);
    expect(arcs().length).toBe(0);
    act(() => root!.render(knob(6)));
    expect(arcs().length).toBe(1);
    expect(arcs()[0].getAttribute("stroke")).toBe(BIPOLAR_BOOST_COLOR);
    act(() => root!.render(knob(-6)));
    expect(arcs().length).toBe(1);
    expect(arcs()[0].getAttribute("stroke")).toBe(BIPOLAR_CUT_COLOR);
  });

  it("uses the same two hexes as the Visual EQ tokens in App.css", () => {
    const css = readFileSync(resolve(process.cwd(), "src/App.css"), "utf8");
    expect(css).toContain(`--eq-boost: ${BIPOLAR_BOOST_COLOR};`);
    expect(css).toContain(`--eq-cut: ${BIPOLAR_CUT_COLOR};`);
  });
});
