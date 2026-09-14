export const CLIP_SECONDS = 30;
export type RenderSettings = { start: number; style: string; intensity: number; target: number };
export type Job = { kind: "load"; channels: Float32Array[]; sampleRate: number } | ({ kind: "render" } & RenderSettings);
export type Measurement = { lufs: number; tp: number };
export type Loaded = { kind: "loaded"; start: number; peaks: Float32Array; duration: number };
export type Rendered = {
  kind: "rendered"; original: Float32Array; mastered: Float32Array;
  channels: number; sampleRate: number; source: Measurement; output: Measurement;
  settings: RenderSettings; seconds: number; version: string;
};
export type Reply = { id: number } & (Loaded | Rendered | { kind: "error"; message: string });

/** One running job and one replaceable pending job; obsolete results never escape. */
export class PreviewWorker {
  private serial = 0;
  private stopped: Error | null = null;
  private active: { id: number; resolve: (reply: Reply | null) => void; reject: (error: Error) => void } | null = null;
  private pending: { job: Job; id: number; resolve: (reply: Reply | null) => void; reject: (error: Error) => void } | null = null;
  private timer: ReturnType<typeof setTimeout> | undefined;
  constructor(private worker: Worker) {
    worker.onmessage = ({ data }: MessageEvent<Reply>) => {
      if (data.id !== this.active?.id) return;
      clearTimeout(this.timer);
      const current = this.active;
      this.active = null;
      if (data.id !== this.serial) current.resolve(null);
      else if (data.kind === "error") current.reject(new Error(data.message));
      else current.resolve(data);
      this.flush();
    };
    worker.onerror = () => this.fail(new Error("The preview engine stopped. Please load your track again."));
    worker.onmessageerror = () => this.fail(new Error("Couldn't read the preview. Please load your track again."));
  }
  request(job: Job): Promise<Reply | null> {
    if (this.stopped) return Promise.reject(this.stopped);
    this.invalidate();
    return new Promise((resolve, reject) => {
      this.pending = { job, id: this.serial, resolve, reject };
      this.flush();
    });
  }
  invalidate() {
    this.serial++;
    this.pending?.resolve(null);
    this.pending = null;
  }
  private flush() {
    if (this.active || !this.pending) return;
    const next = this.pending;
    this.pending = null;
    this.active = next;
    this.timer = setTimeout(() => this.fail(new Error("Processing took too long. Please try a shorter track.")), 60000);
    try {
      const transfer = next.job.kind === "load" ? next.job.channels.map(c => c.buffer as ArrayBuffer) : [];
      this.worker.postMessage({ ...next.job, id: next.id }, transfer);
    } catch (error) { this.fail(error instanceof Error ? error : new Error(String(error))); }
  }
  private fail(error: Error) {
    this.stopped = error;
    this.active?.reject(error);
    this.pending?.reject(error);
    this.close();
  }
  close() {
    this.stopped ??= new Error("The preview was closed. Please load your track again.");
    clearTimeout(this.timer);
    this.invalidate();
    this.active?.resolve(null);
    this.active = null;
    this.worker.terminate();
  }
}
