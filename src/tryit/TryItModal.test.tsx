import { act } from "react";
import { createRoot, type Root } from "react-dom/client";
import { afterEach, beforeEach, expect, it, vi } from "vitest";
import type { Job, Reply, Rendered } from "./processing";

const mocks = vi.hoisted(() => ({
  requests: [] as { job: Job; resolve: (reply: Reply) => void }[],
  play: vi.fn(), select: vi.fn(), buffers: vi.fn(), close: vi.fn(),
}));
vi.mock("./processing", () => ({
  CLIP_SECONDS: 30,
  PreviewWorker: class {
    invalidate() {}
    close() {}
    request(job: Job) {
      if (job.kind === "load") return Promise.resolve({ kind: "loaded", start: 0, peaks: new Float32Array(1000), duration: 40 });
      return new Promise<Reply>(resolve => mocks.requests.push({ job, resolve }));
    }
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
    seek(value: number) { this.position = value; }
  },
}));
import TryItModal from "./TryItModal";
let root: Root;
let host: HTMLDivElement;
beforeEach(() => {
  vi.useFakeTimers(); vi.clearAllMocks(); mocks.requests = [];
  vi.stubGlobal("Worker", class {});
  vi.stubGlobal("AudioContext", class {
    decodeAudioData = async () => ({ sampleRate: 44100, duration: 40, numberOfChannels: 2, getChannelData: () => new Float32Array(100) });
  });
  host = document.createElement("div"); document.body.append(host); root = createRoot(host);
  act(() => root.render(<TryItModal onClose={vi.fn()} />));
});
afterEach(() => {
  act(() => root.unmount()); host.remove(); vi.useRealTimers(); vi.unstubAllGlobals();
});
function button(text: string) {
  return [...document.querySelectorAll<HTMLButtonElement>("button")].find(button => button.textContent?.replace(/[▶Ⅱ]/g, "").trim() === text)!;
}
async function load() {
  const input = document.querySelector<HTMLInputElement>('input[type="file"]')!;
  Object.defineProperty(input, "files", { configurable: true, value: [{ name: "test.wav", arrayBuffer: async () => new ArrayBuffer(8) }] });
  await act(async () => input.dispatchEvent(new Event("change", { bubbles: true })));
  await act(async () => vi.advanceTimersByTimeAsync(200));
}
function finish(index: number) {
  const { job, resolve } = mocks.requests[index];
  if (job.kind !== "render") throw new Error("Expected render");
  const reply: Rendered & { id: number } = { id: index, kind: "rendered", original: new Float32Array(20), mastered: new Float32Array(20), channels: 2, sampleRate: 44100,
    source: { lufs: -18, tp: -6 }, output: { lufs: job.target, tp: -1 }, settings: job, seconds: 0.2, version: "test" };
  resolve(reply);
}
it("finishes loading silently and keeps A/B independent of Play, including while paused", async () => {
  await load(); await act(async () => finish(0));
  expect(mocks.play).not.toHaveBeenCalled();
  expect(button("Play").disabled).toBe(false);
  act(() => button("Original").click());
  expect(button("Original").getAttribute("aria-pressed")).toBe("true");
  expect(mocks.play).not.toHaveBeenCalled();
  await act(async () => button("Play").click());
  expect(button("Pause")).toBeDefined();
  act(() => button("Pause").click());
  act(() => window.dispatchEvent(new KeyboardEvent("keydown", { key: "a", bubbles: true })));
  expect(button("Mastered").getAttribute("aria-pressed")).toBe("true");
  expect(mocks.play).toHaveBeenCalledTimes(1);
  expect(button("Play")).toBeDefined();
});
it("ignores an older render and keeps the accepted comparison usable while updating", async () => {
  await load(); await act(async () => finish(0));
  act(() => button("Tape").click());
  await act(async () => vi.advanceTimersByTimeAsync(200));
  act(() => button("Oomph").click());
  await act(async () => finish(1));
  expect(mocks.buffers).toHaveBeenCalledTimes(1);
  expect(button("Play").disabled).toBe(false);
  await act(async () => vi.advanceTimersByTimeAsync(200));
  await act(async () => finish(2));
  expect(mocks.buffers).toHaveBeenCalledTimes(2);
  expect(document.body.textContent).toContain("oomph");
});
it("file replacement clears the old comparison and stays silent after processing", async () => {
  await load(); await act(async () => finish(0));
  await act(async () => button("Play").click());
  await load();
  expect(button("Play").disabled).toBe(true);
  expect(mocks.close).toHaveBeenCalled();
  await act(async () => finish(1));
  expect(button("Play").disabled).toBe(false);
  expect(mocks.play).toHaveBeenCalledTimes(1);
});
it("Space controls transport but yields to the Volume Match input", async () => {
  await load(); await act(async () => finish(0));
  await act(async () => button("Mastered").dispatchEvent(new KeyboardEvent("keydown", { code: "Space", key: " ", bubbles: true, cancelable: true })));
  expect(mocks.play).toHaveBeenCalledTimes(1);
  const input = document.querySelector<HTMLInputElement>('input[type="checkbox"]')!;
  const key = new KeyboardEvent("keydown", { code: "Space", key: " ", bubbles: true, cancelable: true });
  act(() => input.dispatchEvent(key));
  expect(key.defaultPrevented).toBe(false);
  expect(button("Pause")).toBeDefined();
});
