import { describe, expect, it, vi } from "vitest";
import { auditionBuffer, ComparisonPlayer } from "./player";

function context() {
  const gains: ReturnType<typeof gain>[] = [];
  const sources: ReturnType<typeof source>[] = [];
  function gain() {
    return { gain: { value: 1, cancelScheduledValues: vi.fn(), setValueAtTime: vi.fn(), linearRampToValueAtTime: vi.fn() }, connect: vi.fn((node: unknown) => node), disconnect: vi.fn() };
  }
  function source() {
    return { buffer: null, loop: false, start: vi.fn(), stop: vi.fn(), disconnect: vi.fn(), connect: vi.fn((node: unknown) => node), onended: null };
  }
  const ctx = {
    currentTime: 0, destination: {}, resume: vi.fn(async () => {}), close: vi.fn(async () => {}),
    createGain: () => { const node = gain(); gains.push(node); return node; },
    createBufferSource: () => { const node = source(); sources.push(node); return node; },
    createBuffer: (channels: number, frames: number, rate: number) => {
      const data = Array.from({ length: channels }, () => new Float32Array(frames));
      return { duration: frames / rate, getChannelData: (channel: number) => data[channel] };
    },
  };
  return { ctx, audio: ctx as unknown as AudioContext, gains, sources };
}
const buffer = { duration: 30 } as AudioBuffer;

describe("browser comparison transport", () => {
  it("loads silently, pauses at the playhead, and selects a side without starting audio", async () => {
    const { ctx, audio, sources } = context();
    const player = new ComparisonPlayer(audio);
    player.setBuffers(buffer, buffer, true, "mastered", [1, 1]);
    expect(sources).toHaveLength(0);
    await player.play();
    ctx.currentTime = 5.26;
    player.pause();
    expect(player.position).toBe(5.25);
    player.select("original", [1, 1]);
    expect(sources).toHaveLength(2);
    expect(player.playing).toBe(false);
    await player.play();
    expect(sources[2].start).toHaveBeenCalledWith(5.27, 5.25);
    expect(sources[3].start).toHaveBeenCalledWith(5.27, 5.25);
  });
  it("ramps rapid A/B changes from their current levels without restarting or summing above unity", async () => {
    const { ctx, audio, gains, sources } = context();
    const player = new ComparisonPlayer(audio);
    player.setBuffers(buffer, buffer, true, "mastered", [1, 1]);
    await player.play();
    ctx.currentTime = 1;
    player.select("original", [1, 1]);
    ctx.currentTime = 1.01;
    player.select("mastered", [1, 1]);
    const a = gains[1].gain.setValueAtTime.mock.lastCall![0];
    const b = gains[2].gain.setValueAtTime.mock.lastCall![0];
    expect(a).toBeCloseTo(0.5);
    expect(b).toBeCloseTo(0.5);
    expect(a + b).toBeCloseTo(1);
    expect(gains[1].gain.linearRampToValueAtTime).toHaveBeenLastCalledWith(0, 1.03);
    expect(sources).toHaveLength(2);
    expect(player.position).toBe(1);
  });
  it("crossfades updated previews at the current position and resets only for a new section", async () => {
    const { ctx, audio, sources } = context();
    const player = new ComparisonPlayer(audio);
    player.setBuffers(buffer, buffer, true, "mastered", [1, 1]);
    await player.play();
    ctx.currentTime = 12;
    player.setBuffers(buffer, buffer, false, "mastered", [1, 0.5]);
    expect(sources[0].stop.mock.lastCall![0]).toBeCloseTo(12.03);
    expect(sources[2].start).toHaveBeenCalledWith(12.01, 12);
    player.setBuffers(buffer, buffer, true, "original", [1, 1]);
    expect(sources[4].start).toHaveBeenCalledWith(12.01, 0);
  });
  it("doesn't start after close while AudioContext.resume is pending", async () => {
    const { ctx, audio, sources } = context();
    let resume!: () => void;
    ctx.resume.mockImplementation(() => new Promise<void>(resolve => { resume = resolve; }));
    const player = new ComparisonPlayer(audio);
    player.setBuffers(buffer, buffer, true, "mastered", [1, 1]);
    const playing = player.play();
    player.close(); resume(); await playing;
    expect(sources).toHaveLength(0);
  });
  it("softens both loop edges without changing the measured PCM or interior samples", () => {
    const { audio } = context();
    const pcm = new Float32Array(1000).fill(0.25);
    const original = pcm.slice();
    const faded = auditionBuffer(audio, pcm, 2, 1000);
    for (let c = 0; c < 2; c++) {
      const data = faded.getChannelData(c);
      expect(data[0]).toBe(0); expect(data[499]).toBe(0);
      expect(data[250]).toBe(0.25);
    }
    expect(pcm).toEqual(original);
  });
});
