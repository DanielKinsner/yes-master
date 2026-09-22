import init, {
  analyze_dynamics, analyze_loudness, analyze_stereo, analyze_tonal, build_profile,
  master_standard, measure_loudness, profile_digest, version,
} from "./engine/yes_master_web.js";
import wasmUrl from "./engine/yes_master_web_bg.wasm?url";
import { CLIP_SECONDS, type Analysis, type Job, type Reply } from "./processing";

// A worker is either the TRACK worker (holds the whole decoded track after
// `load`) or a POOL worker (holds only the current excerpt after `prime`).
// Both render the same way.
let track: { interleaved: Float32Array; channels: number; sampleRate: number; frames: number; analysis: Analysis } | null = null;
let excerpt: { startFrame: number; original: Float32Array; channels: number; sampleRate: number; sourceLufs: number; profile: string } | null = null;

const ready = init({ module_or_path: wasmUrl });
const post = (reply: Reply, transfer: Transferable[] = []) => self.postMessage(reply, { transfer });

/** The desktop's analysis stages, in its order and with its labels
 *  (`analysis.rs::analyze_one_with_progress`), plus the excerpt pick. */
function load(id: number, channels: Float32Array[], sampleRate: number) {
  const progress = (label: string, fraction: number) => post({ id, kind: "progress", stage: { label, fraction } });
  const t = performance.now();
  const ch = channels.length;
  const frames = channels[0].length;
  progress("Analyzing audio", 0);
  const interleaved = new Float32Array(frames * ch);
  const energy = new Float64Array(Math.ceil(frames / sampleRate));
  const peaks = new Float32Array(1000);
  for (let c = 0; c < ch; c++) {
    const channel = channels[c];
    for (let i = 0; i < frames; i++) {
      const v = channel[i];
      if (!Number.isFinite(v)) throw new Error("This file contains invalid audio samples.");
      interleaved[i * ch + c] = v;
      energy[Math.floor(i / sampleRate)] += v * v;
      const x = Math.min(peaks.length - 1, Math.floor(i * peaks.length / frames));
      if (Math.abs(v) > peaks[x]) peaks[x] = Math.abs(v);
    }
  }
  channels.length = 0; // release the planar copies; the interleaved buffer is the track now
  progress("Choosing your 30 seconds", 0.2);
  let sum = 0, best = -1, start = 0;
  for (let s = 0; s < energy.length; s++) {
    sum += energy[s];
    if (s >= CLIP_SECONDS) sum -= energy[s - CLIP_SECONDS];
    if (s >= CLIP_SECONDS - 1 && sum > best) { best = sum; start = s - CLIP_SECONDS + 1; }
  }
  progress("Reading loudness", 0.3);
  const [lufs, tp, lra] = analyze_loudness(interleaved, ch, sampleRate);
  progress("Checking dynamics", 0.5);
  const dynamics = analyze_dynamics(interleaved, ch, sampleRate);
  progress("Evaluating stereo field", 0.6);
  const [correlation, width] = analyze_stereo(interleaved, ch);
  progress("Reading tonal balance", 0.7);
  const tonal = analyze_tonal(interleaved, ch, sampleRate);
  progress("Building mastering context", 0.92);
  const profile = build_profile(tonal, dynamics, lra, correlation, width);
  const analysis: Analysis = { lufs, tp, lra, profile, digest: profile_digest(profile) };
  track = { interleaved, channels: ch, sampleRate, frames, analysis };
  progress("Ready", 1);
  post({
    id, kind: "loaded", peaks, duration: frames / sampleRate, frames, analysis, version: version(),
    start: Math.max(0, Math.min(frames / sampleRate - CLIP_SECONDS, start)),
    seconds: (performance.now() - t) / 1000,
  });
}

function extract(id: number, startFrame: number) {
  if (!track) throw new Error("No track is loaded in this worker.");
  const { interleaved, channels, sampleRate, frames, analysis } = track;
  const length = Math.min(frames, CLIP_SECONDS * sampleRate);
  const from = Math.max(0, Math.min(frames - length, startFrame)) * channels;
  const original = interleaved.slice(from, from + length * channels);
  const [lufs, tp] = measure_loudness(original, channels, sampleRate);
  excerpt = { startFrame, original, channels, sampleRate, sourceLufs: analysis.lufs, profile: analysis.profile };
  // The main thread gets its own copy for playback; this worker keeps `original` for rendering.
  post({ id, kind: "excerpt", startFrame, start: startFrame / sampleRate, original: original.slice(), channels, sampleRate, source: { lufs, tp } });
}

function render(id: number, job: Extract<Job, { kind: "render" }>) {
  if (!excerpt || excerpt.startFrame !== job.startFrame) throw new Error("The chosen section changed. Please try again.");
  const t = performance.now();
  const { original, channels, sampleRate, sourceLufs, profile } = excerpt;
  const result = master_standard(original, channels, sampleRate, job.style, job.intensity, job.target, sourceLufs, profile);
  const output = { lufs: result.lufs, tp: result.tp };
  const mastered = result.samples();
  post({
    id, kind: "rendered", mastered, channels, sampleRate, output,
    settings: { start: job.startFrame / sampleRate, style: job.style, intensity: job.intensity, target: job.target },
    seconds: (performance.now() - t) / 1000,
  }, [mastered.buffer]);
}

self.onmessage = async ({ data }: MessageEvent<Job & { id: number }>) => {
  try {
    await ready;
    if (data.kind === "load") load(data.id, data.channels, data.sampleRate);
    else if (data.kind === "excerpt") extract(data.id, data.startFrame);
    else if (data.kind === "prime") {
      excerpt = { startFrame: data.startFrame, original: data.original, channels: data.channels, sampleRate: data.sampleRate, sourceLufs: data.sourceLufs, profile: data.profile };
      post({ id: data.id, kind: "primed" });
    } else render(data.id, data);
  } catch (error) {
    post({ id: data.id, kind: "error", message: error instanceof Error ? error.message : String(error) });
  }
};
