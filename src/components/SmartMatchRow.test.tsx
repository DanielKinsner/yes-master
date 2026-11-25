import { act, type ReactElement } from "react";
import { createRoot, type Root } from "react-dom/client";
import { afterEach, describe, expect, it, vi } from "vitest";

import type { SmartMatchView } from "../lib/smart-match";
import { SmartMatchCard, SmartMatchRow } from "./SmartMatchRow";

(globalThis as unknown as { IS_REACT_ACT_ENVIRONMENT: boolean })
  .IS_REACT_ACT_ENVIRONMENT = true;

async function render(ui: ReactElement): Promise<{ container: HTMLDivElement; root: Root }> {
  const container = document.createElement("div");
  document.body.appendChild(container);
  const root = createRoot(container);
  await act(async () => {
    root.render(ui);
  });
  return { container, root };
}

afterEach(() => {
  document.body.innerHTML = "";
});

const BANDS = [
  { gap_db: -1.2, correction_db: -0.6, capped: false },
  { gap_db: -9.5, correction_db: -4, capped: true },
  { gap_db: 0.1, correction_db: 0, capped: false },
  { gap_db: 0.4, correction_db: 0, capped: false },
  { gap_db: 7.8, correction_db: 3, capped: true },
  { gap_db: 1.6, correction_db: 0.8, capped: false },
];

function makeView(overrides: Partial<SmartMatchView> = {}): SmartMatchView {
  return {
    state: "applied",
    name: "Reference Song",
    enabled: true,
    amount: 0.5,
    line: 'Matching "Reference Song": tamed 4.0 dB of boom, added 3.0 dB of presence.',
    partial: false,
    measuring: false,
    pending: false,
    bands: BANDS,
    ...overrides,
  };
}

const actions = () => ({ onChoose: vi.fn(), onToggle: vi.fn(), onRemove: vi.fn() });

function click(el: Element | null | undefined) {
  el?.dispatchEvent(new MouseEvent("click", { bubbles: true }));
}

// Every copy-table state, as the row shows it.
const STATES: Array<[string, Partial<SmartMatchView>]> = [
  ["no_reference", { state: "no_reference", name: null, bands: null, line: "Pick a song you want this track to sit next to." }],
  ["measuring", { state: "measuring", measuring: true, line: "Listening to your reference…" }],
  ["track_not_ready", { state: "track_not_ready", line: "Listening to your track…" }],
  ["no_profile", { state: "no_profile", line: "Not enough audio to analyze this track." }],
  ["applied", {}],
  [
    "applied (partial)",
    {
      partial: true,
      line:
        'Matching "Reference Song": tamed 4.0 dB of boom, added 3.0 dB of presence.' +
        " Your reference is very different from this track, so the match is partial.",
    },
  ],
  ["nothing_to_change", { state: "nothing_to_change", line: 'Already close to "Reference Song". Nothing to change.' }],
  ["switched_off", { state: "switched_off", enabled: false, line: 'Match to "Reference Song" is off.' }],
  ["zero_amount", { state: "zero_amount", amount: 0, line: "Match amount is at 0, so Smart Match is idle." }],
  ["album_disabled", { state: "album_disabled", bands: null, line: "Smart Match applies to Track Master, not Album renders." }],
  ["refused", { state: "no_reference", name: null, bands: null, line: "That file can't be used as a reference: it is silent." }],
];

describe("SmartMatchRow", () => {
  it.each(STATES)("shows the %s line as a live status", async (_name, overrides) => {
    const view = makeView(overrides);
    const { container, root } = await render(<SmartMatchRow view={view} {...actions()} />);
    const status = container.querySelector('[role="status"]');
    expect(status?.textContent).toBe(view.line);
    expect(status?.getAttribute("aria-live")).toBe("polite");
    // The full line is available on hover even when Standard clamps it.
    expect(status?.getAttribute("title")).toBe(view.line);
    expect(container.querySelector(".smart-match-row")?.getAttribute("data-state")).toBe(view.state);
    expect(status?.classList.contains("is-partial")).toBe(view.partial);
    expect(container.textContent).toContain("Smart Match");
    expect(container.textContent).toContain("Beta");
    await act(async () => root.unmount());
  });

  it("offers Choose reference without a reference", async () => {
    const a = actions();
    const { container, root } = await render(
      <SmartMatchRow view={makeView({ state: "no_reference", name: null, bands: null })} {...a} />,
    );
    const choose = container.querySelector<HTMLButtonElement>(".smart-match-choose");
    expect(choose?.textContent).toBe("Choose reference…");
    expect(choose?.disabled).toBe(false);
    expect(container.querySelector('[role="switch"]')).toBeNull();
    await act(async () => click(choose));
    expect(a.onChoose).toHaveBeenCalledTimes(1);
    await act(async () => root.unmount());
  });

  it("with a reference: name, switch, Change and Remove each do their one job", async () => {
    const a = actions();
    const { container, root } = await render(<SmartMatchRow view={makeView()} {...a} />);
    const name = container.querySelector(".smart-match-name");
    expect(name?.textContent).toBe("Reference Song");
    expect(name?.getAttribute("aria-label")).toBe("Reference: Reference Song");
    expect(name?.getAttribute("tabindex")).toBe("0");

    const toggle = container.querySelector<HTMLButtonElement>('[role="switch"]');
    expect(toggle?.getAttribute("aria-checked")).toBe("true");
    expect(toggle?.getAttribute("aria-label")).toBe("Smart Match");
    await act(async () => click(toggle));
    expect(a.onToggle).toHaveBeenCalledTimes(1);

    const change = container.querySelector(".smart-match-change");
    expect(change?.textContent).toBe("Change…");
    await act(async () => click(change));
    expect(a.onChoose).toHaveBeenCalledTimes(1);

    const remove = container.querySelector('[aria-label="Remove reference"]');
    await act(async () => click(remove));
    expect(a.onRemove).toHaveBeenCalledTimes(1);
    await act(async () => root.unmount());
  });

  it("shows the switch off when switched off", async () => {
    const { container, root } = await render(
      <SmartMatchRow view={makeView({ state: "switched_off", enabled: false })} {...actions()} />,
    );
    expect(container.querySelector('[role="switch"]')?.getAttribute("aria-checked")).toBe("false");
    await act(async () => root.unmount());
  });

  it("shortens a long name on screen and keeps the full name on hover and for screen readers", async () => {
    const long = "A".repeat(75);
    const { container, root } = await render(<SmartMatchRow view={makeView({ name: long })} {...actions()} />);
    const name = container.querySelector(".smart-match-name");
    expect(name?.textContent).toBe(`${"A".repeat(59)}…`);
    expect(name?.getAttribute("title")).toBe(long);
    expect(name?.getAttribute("aria-label")).toBe(`Reference: ${long}`);
    await act(async () => root.unmount());
  });

  it("is busy and locks its controls while a pick is measured", async () => {
    const { container, root } = await render(
      <SmartMatchRow view={makeView({ state: "measuring", measuring: true })} {...actions()} />,
    );
    expect(container.querySelector(".smart-match-row")?.getAttribute("aria-busy")).toBe("true");
    for (const b of container.querySelectorAll<HTMLButtonElement>("button")) {
      expect(b.disabled).toBe(true);
    }
    await act(async () => root.unmount());
  });

  it("locks Choose in Album mode even when the song has a reference", async () => {
    const a = actions();
    const { container, root } = await render(
      <SmartMatchRow view={makeView({ state: "album_disabled", bands: null })} {...a} />,
    );
    const choose = container.querySelector<HTMLButtonElement>(".smart-match-choose");
    expect(choose?.disabled).toBe(true);
    expect(container.querySelector('[role="switch"]')).toBeNull();
    await act(async () => click(choose));
    expect(a.onChoose).not.toHaveBeenCalled();
    await act(async () => root.unmount());
  });

  it("marks numbers from the previous settings as pending", async () => {
    const { container, root } = await render(<SmartMatchRow view={makeView({ pending: true })} {...actions()} />);
    expect(container.querySelector('[role="status"]')?.classList.contains("is-pending")).toBe(true);
    await act(async () => root.unmount());
  });
});

describe("SmartMatchCard (Advanced)", () => {
  it("shows the six-band strip: the gap and how much the match closes, caps marked", async () => {
    const { container, root } = await render(
      <SmartMatchCard view={makeView()} {...actions()} onAmount={vi.fn()} />,
    );
    const strip = container.querySelector('table[aria-label="Smart Match bands"]');
    expect(strip).not.toBeNull();
    const headers = Array.from(strip!.querySelectorAll("thead th")).map((th) => th.textContent);
    expect(headers).toEqual(["Band", "Sub", "Bass", "Low mids", "Mids", "Presence", "Air"]);
    const [gapRow, closedRow] = Array.from(strip!.querySelectorAll("tbody tr"));
    expect(Array.from(gapRow.querySelectorAll("td")).map((td) => td.textContent)).toEqual([
      "−1.2", "−9.5", "+0.1", "+0.4", "+7.8", "+1.6",
    ]);
    const closed = Array.from(closedRow.querySelectorAll("td"));
    expect(closed.map((td) => td.textContent)).toEqual(["−0.6", "−4.0", "0.0", "0.0", "+3.0", "+0.8"]);
    expect(closed[1].classList.contains("is-capped")).toBe(true);
    expect(closed[1].getAttribute("aria-label")).toBe("Bass correction −4.0 dB, limited by a cap");
    expect(closed[0].classList.contains("is-capped")).toBe(false);
    expect(container.querySelector("#jump-smart-match")).not.toBeNull();
    await act(async () => root.unmount());
  });

  it("shows dashes before there are numbers", async () => {
    const { container, root } = await render(
      <SmartMatchCard view={makeView({ state: "no_reference", name: null, bands: null })} {...actions()} onAmount={vi.fn()} />,
    );
    const cells = Array.from(container.querySelectorAll("tbody td")).map((td) => td.textContent);
    expect(cells).toHaveLength(12);
    expect(new Set(cells)).toEqual(new Set(["—"]));
    await act(async () => root.unmount());
  });

  it("carries both locked captions", async () => {
    const { container, root } = await render(
      <SmartMatchCard view={makeView()} {...actions()} onAmount={vi.fn()} />,
    );
    const captions = Array.from(container.querySelectorAll(".smart-match-caption")).map((p) => p.textContent);
    expect(captions).toEqual([
      "Tone only. Loudness and width are not matched.",
      "Moves are applied before the compressor and limiter, so the final change can be smaller on loud material.",
    ]);
    await act(async () => root.unmount());
  });

  it("the Match amount slider reports its value and is locked without a reference", async () => {
    const onAmount = vi.fn();
    const { container, root } = await render(
      <SmartMatchCard view={makeView()} {...actions()} onAmount={onAmount} />,
    );
    const slider = container.querySelector<HTMLInputElement>('input[type="range"][aria-label="Match amount"]');
    expect(slider?.value).toBe("0.5");
    expect(slider?.disabled).toBe(false);
    expect(container.textContent).toContain("50%");
    await act(async () => {
      const setter = Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, "value")?.set;
      setter?.call(slider, "0.75");
      slider?.dispatchEvent(new Event("input", { bubbles: true }));
    });
    expect(onAmount).toHaveBeenLastCalledWith(0.75);
    await act(async () => root.unmount());

    const bare = await render(
      <SmartMatchCard view={makeView({ state: "no_reference", name: null, bands: null })} {...actions()} onAmount={vi.fn()} />,
    );
    expect(
      bare.container.querySelector<HTMLInputElement>('input[type="range"][aria-label="Match amount"]')?.disabled,
    ).toBe(true);
    await act(async () => bare.root.unmount());
  });

  it("shows only the row in Album mode", async () => {
    const album = makeView({
      state: "album_disabled",
      bands: null,
      line: "Smart Match applies to Track Master, not Album renders.",
    });
    const { container, root } = await render(
      <SmartMatchCard view={album} {...actions()} onAmount={vi.fn()} />,
    );
    expect(container.querySelector("table")).toBeNull();
    expect(container.querySelector('input[aria-label="Match amount"]')).toBeNull();
    expect(container.querySelector('[role="status"]')?.textContent).toBe(
      "Smart Match applies to Track Master, not Album renders.",
    );
    await act(async () => root.unmount());
  });
});
