import init, { master_standard, measure_loudness, version } from "./engine/yes_master_web.js";
import wasmUrl from "./engine/yes_master_web_bg.wasm?url";
import { CLIP_SECONDS, type Job, type Reply } from "./processing";

let channels: Float32Array[] = [];
let sampleRate = 0;
const ready = init({ module_or_path: wasmUrl });
const measure = (samples: Float32Array) => {
  const [lufs, tp] = measure_loudness(samples, channels.length, sampleRate);
  return { lufs, tp };
};
self.onmessage = async ({ data }: MessageEvent<Job & { id: number }>) => {
  try {
    await ready;
    if (data.kind === "load") {
      channels = data.channels;
      sampleRate = data.sampleRate;
      const frames = channels[0].length;
      const energy = new Float64Array(Math.ceil(frames / sampleRate));
      const peaks = new Float32Array(1000);
      for (const channel of channels) {
        for (let i = 0; i < frames; i++) {
          const v = channel[i];
          if (!Number.isFinite(v)) throw new Error("This file contains invalid audio samples.");
          energy[Math.floor(i / sampleRate)] += v * v;
          const x = Math.min(peaks.length - 1, Math.floor(i * peaks.length / frames));
          peaks[x] = Math.max(peaks[x], Math.abs(v));
        }
      }
      let sum = 0, best = -1, start = 0;
      for (let s = 0; s < energy.length; s++) {
        sum += energy[s];
        if (s >= CLIP_SECONDS) sum -= energy[s - CLIP_SECONDS];
        if (s >= CLIP_SECONDS - 1 && sum > best) { best = sum; start = s - CLIP_SECONDS + 1; }
      }
      const reply: Reply = { id: data.id, kind: "loaded", start: Math.max(0, Math.min(frames / sampleRate - CLIP_SECONDS, start)), peaks, duration: frames / sampleRate };
      self.postMessage(reply);
    } else {
      const t = performance.now();
      const length = Math.min(channels[0].length, CLIP_SECONDS * sampleRate);
      const start = Math.max(0, Math.min(channels[0].length - length, Math.round(data.start * sampleRate)));
      const original = new Float32Array(length * channels.length);
      for (let i = 0; i < length; i++) for (let c = 0; c < channels.length; c++) original[i * channels.length + c] = channels[c][start + i];
      const mastered = master_standard(original, channels.length, sampleRate, data.style, data.intensity, data.target);
      const reply: Reply = {
        id: data.id, kind: "rendered", original, mastered, channels: channels.length, sampleRate,
        source: measure(original), output: measure(mastered),
        settings: { start: start / sampleRate, style: data.style, intensity: data.intensity, target: data.target },
        seconds: (performance.now() - t) / 1000, version: version(),
      };
      self.postMessage(reply, { transfer: [original.buffer, mastered.buffer] });
    }
  } catch (error) {
    self.postMessage({ id: data.id, kind: "error", message: error instanceof Error ? error.message : String(error) });
  }
};
