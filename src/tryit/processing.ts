// Browser try-it: everything between the modal and the wasm engine.
//
// One "track" worker owns the decoded track: whole-track analysis (the
// desktop's stages, reported as real progress), the loudest-30 s pick, and
// excerpt extraction. Extra "pool" workers receive only the current 30 s
// excerpt so several renders can run at once. Every finished render is cached
// by its exact settings, and after each user render the engine quietly
// pre-renders the neighbouring style/loudness choices on idle workers, so the
// next tile or pill the user touches is usually already done — the LANDR
// "we prepared it while you listened" trick, without their minute of waiting.

export const CLIP_SECONDS = 30;

export type Measurement = { lufs: number; tp: number };
/** Whole-track analysis, the same measurements the desktop derives its adaptive profile from. */
export type Analysis = { lufs: number; tp: number; lra: number; profile: string; digest: string };
export type Stage = { label: string; fraction: number };
export type RenderSettings = { start: number; style: string; intensity: number; target: number };

export type Job =
  | { kind: "load"; channels: Float32Array[]; sampleRate: number }
  | { kind: "excerpt"; startFrame: number }
  | { kind: "prime"; startFrame: number; original: Float32Array; channels: number; sampleRate: number; sourceLufs: number; profile: string }
  | { kind: "render"; startFrame: number; style: string; intensity: number; target: number };

export type Loaded = { kind: "loaded"; start: number; peaks: Float32Array; duration: number; frames: number; analysis: Analysis; version: string; seconds: number };
export type Excerpt = { kind: "excerpt"; startFrame: number; start: number; original: Float32Array; channels: number; sampleRate: number; source: Measurement };
export type Rendered = { kind: "rendered"; mastered: Float32Array; channels: number; sampleRate: number; output: Measurement; settings: RenderSettings; seconds: number };
export type Reply = { id: number } & (Loaded | Excerpt | { kind: "primed" } | Rendered | { kind: "progress"; stage: Stage } | { kind: "error"; message: string });

const TIMEOUT_MS = 90_000;

/** One worker, one request at a time, FIFO; progress replies stream to `onProgress`. */
export class EngineWorker {
  private serial = 0;
  private stopped: Error | null = null;
  private active: { id: number; resolve: (reply: Reply) => void; reject: (error: Error) => void } | null = null;
  private queue: { job: Job; transfer: Transferable[]; resolve: (reply: Reply) => void; reject: (error: Error) => void }[] = [];
  private timer: ReturnType<typeof setTimeout> | undefined;
  /** Sample frame the excerpt this worker holds starts at; null until primed. */
  primed: number | null = null;
  constructor(private worker: Worker, private onProgress?: (stage: Stage) => void) {
    worker.onmessage = ({ data }: MessageEvent<Reply>) => {
      if (data.id !== this.active?.id) return;
      if (data.kind === "progress") { this.onProgress?.(data.stage); return; }
      clearTimeout(this.timer);
      const current = this.active;
      this.active = null;
      if (data.kind === "error") current.reject(new Error(data.message));
      else current.resolve(data);
      this.pump();
    };
    worker.onerror = () => this.fail(new Error("The preview engine stopped. Please load your track again."));
    worker.onmessageerror = () => this.fail(new Error("Couldn't read the preview. Please load your track again."));
  }
  get busy() { return this.active !== null || this.queue.length > 0; }
  request(job: Job, transfer: Transferable[] = []): Promise<Reply> {
    if (this.stopped) return Promise.reject(this.stopped);
    return new Promise((resolve, reject) => {
      this.queue.push({ job, transfer, resolve, reject });
      this.pump();
    });
  }
  private pump() {
    if (this.active || this.queue.length === 0) return;
    const next = this.queue.shift()!;
    this.active = { id: ++this.serial, resolve: next.resolve, reject: next.reject };
    this.timer = setTimeout(() => this.fail(new Error("Processing took too long. Please try a shorter track.")), TIMEOUT_MS);
    try { this.worker.postMessage({ ...next.job, id: this.active.id }, next.transfer); }
    catch (error) { this.fail(error instanceof Error ? error : new Error(String(error))); }
  }
  private fail(error: Error) {
    this.stopped = error;
    this.active?.reject(error);
    this.active = null;
    this.queue.splice(0).forEach(item => item.reject(error));
    clearTimeout(this.timer);
    this.worker.terminate();
  }
  close() { this.fail(new Error("The preview was closed. Please load your track again.")); }
}

export const cacheKey = (s: RenderSettings, startFrame: number) => `${startFrame}|${s.style}|${Math.round(s.intensity * 1000)}|${s.target}`;

/** The renders worth having ready before the user asks: the other loudness
 *  levels for this style first (the cheapest tempting click), then the other
 *  styles at this loudness. Intensity is continuous, so it isn't guessed. */
export function neighbours(s: RenderSettings, styles: readonly string[], targets: readonly number[]): RenderSettings[] {
  const out: RenderSettings[] = [];
  for (const target of targets) if (target !== s.target) out.push({ ...s, target });
  for (const style of styles) if (style !== s.style) out.push({ ...s, style });
  return out;
}

export type EngineOptions = {
  /** Total workers including the track worker. Default: hardwareConcurrency − 1, clamped to 1..4. */
  workers?: number;
  /** Rendered-PCM cache budget in bytes. */
  cacheBytes?: number;
};

type UserJob = { settings: RenderSettings; epoch: number; resolve: (r: Rendered | null) => void; reject: (e: Error) => void };

export class PreviewEngine {
  private track: EngineWorker;
  private pool: EngineWorker[] = [];
  private sampleRate = 0;
  private frames = 0;
  private analysis: Analysis | null = null;
  private excerpt: Excerpt | null = null;
  private excerptPending: Promise<Excerpt> | null = null;
  private cache = new Map<string, Rendered>();
  private cacheBytes = 0;
  private readonly cacheBudget: number;
  private user: UserJob | null = null;
  private userEpoch = 0;
  private prefetchQueue: RenderSettings[] = [];
  private inFlight = new Set<string>();
  private closed = false;
  constructor(spawn: () => Worker, options: EngineOptions = {}, private onProgress?: (stage: Stage) => void) {
    const cores = typeof navigator !== "undefined" && navigator.hardwareConcurrency ? navigator.hardwareConcurrency : 2;
    const count = Math.max(1, Math.min(4, options.workers ?? cores - 1));
    this.cacheBudget = options.cacheBytes ?? 128e6;
    this.track = new EngineWorker(spawn(), stage => this.onProgress?.(stage));
    for (let i = 1; i < count; i++) this.pool.push(new EngineWorker(spawn()));
  }
  get workerCount() { return 1 + this.pool.length; }
  get currentExcerpt() { return this.excerpt; }

  async load(channels: Float32Array[], sampleRate: number): Promise<Loaded> {
    this.sampleRate = sampleRate;
    this.frames = channels[0]?.length ?? 0;
    const reply = await this.track.request({ kind: "load", channels, sampleRate }, channels.map(c => c.buffer as ArrayBuffer));
    if (reply.kind !== "loaded") throw new Error("Unexpected engine reply.");
    this.analysis = reply.analysis;
    return reply;
  }

  /** Snap a requested start (seconds) to the frame the excerpt will actually begin on. */
  snapStart(start: number): number {
    const length = Math.min(this.frames, CLIP_SECONDS * this.sampleRate);
    return Math.max(0, Math.min(this.frames - length, Math.round(start * this.sampleRate)));
  }

  /** Make `start` the current excerpt on every worker. Idempotent for the same snapped start. */
  setExcerpt(start: number): Promise<Excerpt> {
    const startFrame = this.snapStart(start);
    if (this.excerpt?.startFrame === startFrame) return Promise.resolve(this.excerpt);
    if (this.excerptPending) return this.excerptPending.then(() => this.setExcerpt(start));
    this.excerpt = null;
    this.prefetchQueue = [];
    this.pool.forEach(w => { w.primed = null; });
    this.track.primed = null;
    const pending = this.track.request({ kind: "excerpt", startFrame }).then(async reply => {
      if (reply.kind !== "excerpt") throw new Error("Unexpected engine reply.");
      this.track.primed = startFrame;
      const analysis = this.analysis;
      await Promise.all(this.pool.map(w => w.request({
        kind: "prime", startFrame, original: reply.original, channels: reply.channels, sampleRate: reply.sampleRate,
        sourceLufs: analysis?.lufs ?? Number.NaN, profile: analysis?.profile ?? "",
      }).then(() => { w.primed = startFrame; })));
      this.excerpt = reply;
      return reply;
    });
    this.excerptPending = pending;
    pending.finally(() => { if (this.excerptPending === pending) this.excerptPending = null; this.pump(); }).catch(() => {});
    return pending;
  }

  /** Render for the user. Resolves null when a newer user request superseded it. */
  render(settings: RenderSettings): Promise<Rendered | null> {
    if (this.closed) return Promise.reject(new Error("The preview was closed. Please load your track again."));
    this.invalidate();
    const epoch = ++this.userEpoch;
    const hit = this.lookup(settings);
    if (hit) return Promise.resolve(hit);
    return new Promise((resolve, reject) => {
      this.user = { settings, epoch, resolve, reject };
      this.pump();
    });
  }
  /** Drop the pending user request (it resolves null); queued prefetches are cleared too. */
  invalidate() {
    this.user?.resolve(null);
    this.user = null;
    this.prefetchQueue = [];
  }
  /** Queue background renders; they only ever use workers the user isn't waiting on. */
  prefetch(list: RenderSettings[]) {
    if (this.closed || this.workerCount < 2) return;
    for (const settings of list) {
      const key = this.keyFor(settings);
      if (!key || this.cache.has(key) || this.inFlight.has(key) || this.prefetchQueue.some(s => this.keyFor(s) === key)) continue;
      this.prefetchQueue.push(settings);
    }
    this.pump();
  }
  lookup(settings: RenderSettings): Rendered | null {
    const key = this.keyFor(settings);
    if (!key) return null;
    const hit = this.cache.get(key);
    if (!hit) return null;
    this.cache.delete(key); this.cache.set(key, hit); // most recently used
    return hit;
  }
  private keyFor(settings: RenderSettings): string | null {
    const excerpt = this.excerpt;
    if (!excerpt || this.snapStart(settings.start) !== excerpt.startFrame) return null;
    return cacheKey(settings, excerpt.startFrame);
  }
  private remember(key: string, rendered: Rendered) {
    this.cache.set(key, rendered);
    this.cacheBytes += rendered.mastered.byteLength;
    for (const [k, v] of this.cache) {
      if (this.cacheBytes <= this.cacheBudget || this.cache.size <= 1) break;
      this.cache.delete(k);
      this.cacheBytes -= v.mastered.byteLength;
    }
  }
  private pump() {
    if (this.closed) return;
    const excerpt = this.excerpt;
    if (!excerpt) return;
    const ready = () => [this.track, ...this.pool].filter(w => w.primed === excerpt.startFrame && !w.busy);
    const user = this.user;
    if (user) {
      const key = this.keyFor(user.settings);
      if (!key) { user.reject(new Error("The chosen section changed. Please try again.")); this.user = null; }
      else {
        const hit = this.cache.get(key);
        if (hit) { this.user = null; user.resolve(hit); }
        else if (!this.inFlight.has(key)) {
          const worker = ready()[0];
          if (worker) { this.user = null; this.dispatch(worker, user.settings, key, user); }
        }
        // else: the same render is already in flight (a prefetch); deliver on completion.
      }
    }
    // Prefetch always leaves one worker idle for the user's next request.
    while (this.prefetchQueue.length > 0) {
      const free = ready();
      if (free.length <= 1) break;
      const settings = this.prefetchQueue.shift()!;
      const key = this.keyFor(settings);
      if (!key || this.cache.has(key) || this.inFlight.has(key)) continue;
      this.dispatch(free[free.length - 1], settings, key, null);
    }
  }
  private dispatch(worker: EngineWorker, settings: RenderSettings, key: string, user: UserJob | null) {
    const excerpt = this.excerpt!;
    this.inFlight.add(key);
    worker.request({ kind: "render", startFrame: excerpt.startFrame, style: settings.style, intensity: settings.intensity, target: settings.target })
      .then(reply => {
        this.inFlight.delete(key);
        if (reply.kind !== "rendered") throw new Error("Unexpected engine reply.");
        if (this.excerpt?.startFrame === excerpt.startFrame) this.remember(key, reply);
        const waiting = this.user && this.keyFor(this.user.settings) === key ? this.user : null;
        if (waiting) { this.user = null; waiting.resolve(reply); }
        if (user && user.epoch === this.userEpoch) user.resolve(reply);
        else user?.resolve(null);
      })
      .catch((error: Error) => {
        this.inFlight.delete(key);
        if (user) user.reject(error);
        else if (this.user && this.keyFor(this.user.settings) === key) { const u = this.user; this.user = null; u.reject(error); }
      })
      .finally(() => this.pump());
  }
  close() {
    this.closed = true;
    this.invalidate();
    this.track.close();
    this.pool.forEach(w => w.close());
    this.cache.clear();
    this.cacheBytes = 0;
  }
}
