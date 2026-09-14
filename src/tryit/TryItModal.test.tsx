import { act } from "react";
import { createRoot, type Root } from "react-dom/client";
import { afterEach, beforeEach, expect, it, vi } from "vitest";
import type { RenderSettings, Rendered, Stage } from "./processing";

type Pending = { settings: RenderSettings; resolve: (reply: Rendered | null) => void };
const mocks = vi.hoisted(() => ({
  requests: [] as Pending[],
  loads: [] as (() => void)[],
  play: vi.fn(), select: vi.fn(), buffers: vi.fn(), close: vi.fn(), seek: vi.fn(), prefetch: vi.fn(), track: vi.fn(),
}));
vi.mock("./processing", () => ({
  CLIP_SECONDS: 30,
  neighbours: () => [],
  PreviewEngine: class {
    workerCount = 2;
    constructor(_spawn: unknown, _options: unknown, private onProgress?: (s: Stage) => void) {}
    load() {
      this.onProgress?.({ label: "Checking dynamics", fraction: 0.5 });
      return new Promise(resolve => mocks.loads.push(() => resolve({
        kind: "loaded", start: 0, peaks: new Float32Array(1000), duration: 40, frames: 40 * 44100, seconds: 0.4, version: "test",
        analysis: { lufs: -18, tp: -3, lra: 6, profile: "{}", digest: "bright 0.30 / low 0.28" },
      })));
    }
    setExcerpt(start: number) {
      return Promise.resolve({ kind: "excerpt", startFrame: Math.round(start * 44100), start, original: new Float32Array(8), channels: 2, sampleRate: 44100, source: { lufs: -18, tp: -6 } });
    }
    render(settings: RenderSettings) { return new Promise<Rendered | null>(resolve => mocks.requests.push({ settings, resolve })); }
    invalidate() { mocks.requests.forEach(r => r.resolve(null)); }
    prefetch = mocks.prefetch;
    close = mocks.close;
  },
}));
vi.mock("./player", () => ({
  auditionBuffer: vi.fn(),
  ComparisonPlayer: class {
    context = {}; duration = 30; position = 0; playing = false;
    setBuffers = mocks.buffers;
    select = mocks.select;
    close = mocks.close;
    async play() { mocks.play(); this.playing = true; }
    pause() { this.playing = false; }
    seek(value: number) { this.position = value; mocks.seek(value); }
  },
}));
vi.mock("./decode", () => ({
  decodeAtSourceRate: async () => ({ audio: { sampleRate: 44100, duration: 40, numberOfChannels: 2, getChannelData: () => new Float32Array(100) }, fileRate: 44100, decodedRate: 44100 }),
  playbackContext: () => ({}),
}));
vi.mock("./analytics", () => ({ trackTryIt: (...args: unknown[]) => mocks.track(...args), minutesBucket: () => "<1m" }));
import TryItModal from "./TryItModal";

let root: Root;
let host: HTMLDivElement;
beforeEach(() => {
  vi.useFakeTimers(); vi.clearAllMocks(); mocks.requests = []; mocks.loads = [];
  vi.stubGlobal("Worker", class {});
  host = document.createElement("div"); document.body.append(host); root = createRoot(host);
  act(() => root.render(<TryItModal onClose={vi.fn()} />));
});
afterEach(() => {
  act(() => root.unmount()); host.remove(); vi.useRealTimers(); vi.unstubAllGlobals();
});
function button(text: string) {
  return [...document.querySelectorAll<HTMLButtonElement>("button")].find(button => button.textContent?.trim() === text)!;
}
async function load() {
  const input = document.querySelector<HTMLInputElement>('input[type="file"]')!;
  Object.defineProperty(input, "files", { configurable: true, value: [{ name: "test.wav", arrayBuffer: async () => new ArrayBuffer(8) }] });
  await act(async () => input.dispatchEvent(new Event("change", { bubbles: true })));
  await act(async () => { mocks.loads.forEach(done => done()); });
  await act(async () => vi.advanceTimersByTimeAsync(200));
}
/** Resolve the most recent unresolved render request with a result for its own settings. */
function finish(index = mocks.requests.length - 1) {
  const { settings, resolve } = mocks.requests[index];
  resolve({ kind: "rendered", mastered: new Float32Array(20), channels: 2, sampleRate: 44100, output: { lufs: settings.target, tp: -1 }, settings, seconds: 0.2 });
}
const key = (init: KeyboardEventInit) => new KeyboardEvent("keydown", { bubbles: true, cancelable: true, ...init });

it("shows the real analysis stages while preparing, then the analyzed file line", async () => {
  const input = document.querySelector<HTMLInputElement>('input[type="file"]')!;
  Object.defineProperty(input, "files", { configurable: true, value: [{ name: "song.wav", arrayBuffer: async () => new ArrayBuffer(8) }] });
  await act(async () => input.dispatchEvent(new Event("change", { bubbles: true })));
  expect(document.body.textContent).toContain("Preparing your track");
  expect(document.querySelector(".tryit-prep-list .is-active")?.textContent).toBe("Checking dynamics");
  expect(document.querySelectorAll(".tryit-prep-list .is-done").length).toBe(3);
  await act(async () => { mocks.loads.forEach(done => done()); });
  expect(document.querySelector(".tryit-chip")?.textContent).toBe("Analyzed");
  expect(document.body.textContent).toContain("bright 0.30 / low 0.28");
  expect(document.body.textContent).toContain("44.1 kHz");
});
it("finishes loading silently and keeps A/B independent of Play, including while paused", async () => {
  await load(); await act(async () => finish());
  expect(mocks.play).not.toHaveBeenCalled();
  expect(button("Play").disabled).toBe(false);
  act(() => button("Original").click());
  expect(button("Original").getAttribute("aria-pressed")).toBe("true");
  expect(mocks.play).not.toHaveBeenCalled();
  await act(async () => button("Play").click());
  expect(button("Pause")).toBeDefined();
  act(() => button("Pause").click());
  act(() => window.dispatchEvent(key({ key: "a" })));
  expect(button("Mastered").getAttribute("aria-pressed")).toBe("true");
  expect(mocks.play).toHaveBeenCalledTimes(1);
  expect(button("Play")).toBeDefined();
  expect(mocks.prefetch).toHaveBeenCalled();
});
it("renders tiles and pills at once, debounces the slider, and drops a superseded render", async () => {
  await load(); await act(async () => finish());
  const before = mocks.requests.length;
  act(() => button("Tape").click());
  await act(async () => vi.advanceTimersByTimeAsync(0));
  expect(mocks.requests.length).toBe(before + 1);
  act(() => button("Oomph").click());
  await act(async () => vi.advanceTimersByTimeAsync(0));
  expect(mocks.requests.length).toBe(before + 2);
  expect(mocks.buffers).toHaveBeenCalledTimes(1);
  await act(async () => finish());
  expect(mocks.buffers).toHaveBeenCalledTimes(2);
  expect(mocks.buffers.mock.calls[1][2]).toBe(false); // same section: keep the playhead
  expect(button("Oomph").getAttribute("aria-pressed")).toBe("true");
  const slider = document.querySelector<HTMLInputElement>('input[type="range"]')!;
  const setter = Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, "value")!.set!;
  act(() => { setter.call(slider, "75"); slider.dispatchEvent(new Event("input", { bubbles: true })); });
  await act(async () => vi.advanceTimersByTimeAsync(60));
  expect(mocks.requests.length).toBe(before + 2);
  await act(async () => vi.advanceTimersByTimeAsync(100));
  expect(mocks.requests.length).toBe(before + 3);
  expect(mocks.requests[before + 2].settings.intensity).toBe(0.75);
});
it("file replacement clears the old comparison and stays silent after processing", async () => {
  await load(); await act(async () => finish());
  await act(async () => button("Play").click());
  await load();
  expect(button("Play").disabled).toBe(true);
  expect(mocks.close).toHaveBeenCalled();
  await act(async () => finish());
  expect(button("Play").disabled).toBe(false);
  expect(mocks.play).toHaveBeenCalledTimes(1);
});
it("Space controls transport but yields to the Volume Match input", async () => {
  await load(); await act(async () => finish());
  await act(async () => button("Mastered").dispatchEvent(key({ code: "Space", key: " " })));
  expect(mocks.play).toHaveBeenCalledTimes(1);
  const input = document.querySelector<HTMLInputElement>('input[type="checkbox"]')!;
  expect(input.checked).toBe(false); // Volume match is off by default
  const space = key({ code: "Space", key: " " });
  act(() => input.dispatchEvent(space));
  expect(space.defaultPrevented).toBe(false);
  expect(button("Pause")).toBeDefined();
});
it("arrow keys seek, shift-arrows move the window, and the focused waveform nudges the window", async () => {
  await load(); await act(async () => finish());
  act(() => window.dispatchEvent(key({ key: "ArrowRight" })));
  expect(mocks.seek).toHaveBeenLastCalledWith(5);
  act(() => window.dispatchEvent(key({ key: "Home" })));
  expect(mocks.seek).toHaveBeenLastCalledWith(0);
  expect(document.querySelector(".tryit-strip-cap span")?.textContent).toBe("0:00 – 0:30");
  act(() => window.dispatchEvent(key({ key: "ArrowRight", shiftKey: true })));
  expect(document.querySelector(".tryit-strip-cap span")?.textContent).toBe("0:05 – 0:35");
  const canvas = document.querySelector<HTMLCanvasElement>("canvas")!;
  expect(canvas.tabIndex).toBe(0);
  act(() => canvas.dispatchEvent(key({ key: "ArrowRight" })));
  expect(document.querySelector(".tryit-strip-cap span")?.textContent).toBe("0:10 – 0:40");
  expect(mocks.seek).toHaveBeenCalledTimes(2);
  await act(async () => vi.advanceTimersByTimeAsync(200));
  await act(async () => finish());
  expect(mocks.buffers.mock.calls.at(-1)?.[2]).toBe(true); // new section: restart at its beginning
});
it("never renders a loudness verdict or warning text", async () => {
  await load(); await act(async () => finish());
  expect(document.querySelector('[role="status"]')).toBeNull();
  expect(document.body.textContent).not.toMatch(/Target reached|above the|Below target|LUFS target/);
});
