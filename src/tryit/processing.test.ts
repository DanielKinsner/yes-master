import { describe, expect, it } from "vitest";
import { cacheKey, EngineWorker, neighbours, PreviewEngine, type Job, type Reply, type Stage } from "./processing";

class FakeWorker {
  onmessage: ((e: { data: Reply }) => void) | null = null;
  onerror: (() => void) | null = null;
  onmessageerror: (() => void) | null = null;
  posted: (Job & { id: number })[] = [];
  terminated = false;
  postMessage(message: Job & { id: number }) { this.posted.push(message); }
  terminate() { this.terminated = true; }
  get last() { return this.posted[this.posted.length - 1]; }
  reply(reply: object) { this.onmessage?.({ data: { ...reply, id: this.last.id } as Reply }); }
}
const tick = () => new Promise(resolve => setTimeout(resolve, 0));
const analysis = { lufs: -18, tp: -3, lra: 6, profile: "{}", digest: "test" };
const loadedReply = { kind: "loaded", start: 10, peaks: new Float32Array(1000), duration: 60, frames: 60 * 48000, analysis, version: "test", seconds: 0.5 } as const;
const excerptReply = (startFrame: number) => ({ kind: "excerpt", startFrame, start: startFrame / 48000, original: new Float32Array(8), channels: 2, sampleRate: 48000, source: { lufs: -18, tp: -3 } } as const);
const renderedReply = (job: Job & { id: number }) => {
  if (job.kind !== "render") throw new Error("expected render");
  return { kind: "rendered", mastered: new Float32Array(8), channels: 2, sampleRate: 48000, output: { lufs: job.target, tp: -1 }, settings: { start: job.startFrame / 48000, style: job.style, intensity: job.intensity, target: job.target }, seconds: 0.1 } as const;
};
const settings = (over: Partial<{ start: number; style: string; intensity: number; target: number }> = {}) => ({ start: 10, style: "universal", intensity: 0.5, target: -11, ...over });

async function ready(workers: number, onProgress?: (s: Stage) => void) {
  const spawned: FakeWorker[] = [];
  const engine = new PreviewEngine(() => { const w = new FakeWorker(); spawned.push(w); return w as unknown as Worker; }, { workers, cacheBytes: 1e6 }, onProgress);
  const [track, ...pool] = spawned;
  const load = engine.load([new Float32Array(60 * 48000)], 48000);
  track.reply(loadedReply);
  await load;
  const excerpt = engine.setExcerpt(10);
  await tick();
  track.reply(excerptReply(10 * 48000));
  await tick();
  pool.forEach(w => w.reply({ kind: "primed" }));
  await excerpt;
  return { engine, track, pool, all: spawned };
}

describe("EngineWorker", () => {
  it("runs one request at a time in order and streams progress", async () => {
    const stages: Stage[] = [];
    const fake = new FakeWorker();
    const worker = new EngineWorker(fake as unknown as Worker, s => stages.push(s));
    const a = worker.request({ kind: "excerpt", startFrame: 0 });
    const b = worker.request({ kind: "excerpt", startFrame: 1 });
    expect(fake.posted.length).toBe(1);
    fake.reply({ kind: "progress", stage: { label: "Reading loudness", fraction: 0.3 } });
    fake.reply(excerptReply(0));
    expect((await a).kind).toBe("excerpt");
    expect(fake.posted.length).toBe(2);
    fake.reply(excerptReply(1));
    expect((await b).kind).toBe("excerpt");
    expect(stages).toEqual([{ label: "Reading loudness", fraction: 0.3 }]);
  });
  it("fails every queued request and terminates when the worker crashes", async () => {
    const fake = new FakeWorker();
    const worker = new EngineWorker(fake as unknown as Worker);
    const a = worker.request({ kind: "excerpt", startFrame: 0 });
    const b = worker.request({ kind: "excerpt", startFrame: 1 });
    fake.onerror?.();
    await expect(a).rejects.toThrow(/stopped/);
    await expect(b).rejects.toThrow(/stopped/);
    expect(fake.terminated).toBe(true);
    await expect(worker.request({ kind: "excerpt", startFrame: 2 })).rejects.toThrow();
  });
});

describe("PreviewEngine", () => {
  it("analyzes on the track worker with real stages, then primes the pool with the excerpt", async () => {
    const stages: string[] = [];
    const { track, pool } = await ready(3, s => stages.push(s.label));
    expect(track.posted[0].kind).toBe("load");
    expect(track.posted[1]).toMatchObject({ kind: "excerpt", startFrame: 10 * 48000 });
    expect(pool.map(w => w.last.kind)).toEqual(["prime", "prime"]);
    expect(pool[0].last).toMatchObject({ kind: "prime", startFrame: 10 * 48000, sourceLufs: -18, profile: "{}" });
  });
  it("renders for the user, caches the result, and pre-renders neighbours on spare workers only", async () => {
    const { engine, track, pool } = await ready(3);
    const first = engine.render(settings());
    await tick();
    expect(track.last.kind).toBe("render");
    engine.prefetch(neighbours(settings(), ["universal", "clarity", "tape", "oomph"], [-14, -11, -9]));
    await tick();
    // Two pool workers free: one takes a prefetch, one stays free for the user.
    expect(pool.filter(w => w.last?.kind === "render").length).toBe(1);
    track.reply(renderedReply(track.last));
    expect((await first)?.output.lufs).toBe(-11);
    await tick();
    // The track worker is free again, so a second prefetch starts; one worker still stays free.
    expect([track, ...pool].filter(w => w.last?.kind === "render" && !w.terminated).length).toBeGreaterThanOrEqual(2);
    // Cache hit: the same settings resolve without any worker traffic.
    const posted = [track, ...pool].map(w => w.posted.length);
    expect((await engine.render(settings()))?.output.lufs).toBe(-11);
    expect([track, ...pool].map(w => w.posted.length)).toEqual(posted);
  });
  it("delivers a prefetch that is already in flight to a user who asks for it", async () => {
    const { engine, track, pool } = await ready(2);
    engine.prefetch([settings({ target: -14 })]);
    await tick();
    const busy = [track, ...pool].find(w => w.last?.kind === "render")!;
    const user = engine.render(settings({ target: -14 }));
    await tick();
    expect([track, ...pool].filter(w => w.last?.kind === "render").length).toBe(1);
    busy.reply(renderedReply(busy.last));
    expect((await user)?.output.lufs).toBe(-14);
  });
  it("supersedes an older user render with a newer one but still caches the older result", async () => {
    const { engine, track } = await ready(1);
    const older = engine.render(settings({ style: "tape" }));
    await tick();
    const newer = engine.render(settings({ style: "oomph" }));
    track.reply(renderedReply(track.last));
    expect(await older).toBeNull();
    await tick();
    expect(track.last).toMatchObject({ kind: "render", style: "oomph" });
    track.reply(renderedReply(track.last));
    expect((await newer)?.settings.style).toBe("oomph");
    expect(engine.lookup(settings({ style: "tape" }))?.settings.style).toBe("tape");
  });
  it("evicts the least recently used render when the cache budget is exceeded", async () => {
    const { engine, track } = await ready(1);
    for (const target of [-14, -11, -9]) {
      const p = engine.render(settings({ target }));
      await tick();
      track.reply({ ...renderedReply(track.last), mastered: new Float32Array(150_000) }); // 600 KB each, 1 MB budget
      await p;
    }
    expect(engine.lookup(settings({ target: -14 }))).toBeNull();
    expect(engine.lookup(settings({ target: -9 }))).not.toBeNull();
  });
  it("changing the section re-primes every worker and invalidates old-section cache keys", async () => {
    const { engine, track, pool } = await ready(2);
    const p = engine.render(settings());
    await tick();
    track.reply(renderedReply(track.last));
    await p;
    const moved = engine.setExcerpt(20);
    await tick();
    track.reply(excerptReply(20 * 48000));
    await tick();
    pool[0].reply({ kind: "primed" });
    await moved;
    expect(engine.lookup(settings())).toBeNull();
    expect(engine.lookup(settings({ start: 20 }))).toBeNull();
    expect(pool[0].last).toMatchObject({ kind: "prime", startFrame: 20 * 48000 });
  });
  it("rejects the pending user render when the engine fails, and everything after close", async () => {
    const { engine, track, all } = await ready(1);
    const p = engine.render(settings());
    await tick();
    track.onerror?.();
    await expect(p).rejects.toThrow(/stopped/);
    engine.close();
    expect(all.every(w => w.terminated)).toBe(true);
    await expect(engine.render(settings())).rejects.toThrow(/closed/);
  });
  it("orders neighbours: other loudness first, then other styles, never intensity", () => {
    const list = neighbours(settings(), ["universal", "clarity", "tape", "oomph"], [-14, -11, -9]);
    expect(list.map(s => `${s.style}@${s.target}`)).toEqual(["universal@-14", "universal@-9", "clarity@-11", "tape@-11", "oomph@-11"]);
    expect(list.every(s => s.intensity === 0.5)).toBe(true);
    expect(cacheKey(settings({ intensity: 0.5004 }), 7)).toBe(cacheKey(settings({ intensity: 0.5 }), 7));
    expect(cacheKey(settings({ intensity: 0.51 }), 7)).not.toBe(cacheKey(settings({ intensity: 0.5 }), 7));
  });
});
