export type Side = "original" | "mastered";
/** A single number gains the mastered side only (legacy); a pair gains [original, mastered]. */
export type MatchGain = number | [number, number];
const toGains = (g: MatchGain): [number, number] => (typeof g === "number" ? [1, g] : g);
const FADE = 0.02;
type Ramp = { from: number; to: number; start: number; end: number };
type Fader = { node: GainNode; ramp: Ramp };
type Voices = { sources: AudioBufferSourceNode[]; sides: Fader[]; bus: Fader; started: number; offset: number };

export function rampValue(ramp: Ramp, time: number) {
  const progress = ramp.end === ramp.start ? 1 : Math.max(0, Math.min(1, (time - ramp.start) / (ramp.end - ramp.start)));
  return ramp.from + (ramp.to - ramp.from) * progress;
}
function fade(fader: Fader, to: number, time: number) {
  const from = rampValue(fader.ramp, time);
  const gain = fader.node.gain;
  gain.cancelScheduledValues(time);
  gain.setValueAtTime(from, time);
  gain.linearRampToValueAtTime(to, time + FADE);
  fader.ramp = { from, to, start: time, end: time + FADE };
}

/** Edge fades are audition-only; measurements always use the unmodified PCM. */
export function auditionBuffer(ctx: AudioContext, pcm: Float32Array, channels: number, sampleRate: number) {
  const frames = pcm.length / channels;
  const buffer = ctx.createBuffer(channels, frames, sampleRate);
  const edge = Math.max(1, Math.min(Math.round(sampleRate * 0.005), Math.floor(frames / 2)));
  for (let c = 0; c < channels; c++) {
    const out = buffer.getChannelData(c);
    for (let i = 0; i < frames; i++) out[i] = pcm[i * channels + c] * Math.min(1, i / edge, (frames - 1 - i) / edge);
  }
  return buffer;
}

export class ComparisonPlayer {
  private buffers: AudioBuffer[] | null = null;
  private voices: Voices | null = null;
  private offset = 0;
  private side: Side = "mastered";
  private gains: [number, number] = [1, 1];
  private generation = 0;
  constructor(readonly context: AudioContext) {}
  get playing() { return this.voices !== null; }
  get duration() { return this.buffers?.[0].duration ?? 0; }
  get position() {
    return this.voices ? (this.voices.offset + Math.max(0, this.context.currentTime - this.voices.started)) % this.duration : this.offset;
  }
  setBuffers(original: AudioBuffer, mastered: AudioBuffer, reset: boolean, side: Side, matchGain: MatchGain) {
    const position = reset ? 0 : this.position;
    this.buffers = [original, mastered];
    this.side = side;
    this.gains = toGains(matchGain);
    this.offset = position;
    if (this.playing) this.start(position, !reset);
  }
  async play() {
    if (!this.buffers || this.playing) return;
    const generation = ++this.generation;
    await this.context.resume();
    if (generation === this.generation && !this.playing) this.start(this.offset);
  }
  pause() {
    this.generation++;
    this.offset = this.position;
    this.retire(this.voices, this.context.currentTime);
    this.voices = null;
  }
  seek(seconds: number) {
    this.offset = Math.max(0, Math.min(Math.max(0, this.duration - 0.001), seconds));
    if (this.playing) this.start(this.offset);
  }
  select(side: Side, matchGain: MatchGain) {
    this.side = side;
    this.gains = toGains(matchGain);
    this.voices?.sides.forEach((fader, i) => fade(fader, this.level(i), this.context.currentTime));
  }
  private level(index: number) { return index === 0 ? (this.side === "original" ? this.gains[0] : 0) : (this.side === "mastered" ? this.gains[1] : 0); }
  private fader(value: number): Fader {
    const node = this.context.createGain();
    node.gain.value = value;
    return { node, ramp: { from: value, to: value, start: 0, end: 0 } };
  }
  private start(position: number, continuous = false) {
    if (!this.buffers) return;
    // Schedule both sources ahead of the render thread's next audio quantum.
    const now = this.context.currentTime + 0.01;
    const offset = (position + (continuous ? 0.01 : 0)) % this.duration;
    const bus = this.fader(0);
    bus.node.connect(this.context.destination);
    const sides = this.buffers.map((_, i) => this.fader(this.level(i)));
    const sources = this.buffers.map((buffer, i) => {
      const source = this.context.createBufferSource();
      source.buffer = buffer;
      source.loop = true;
      source.connect(sides[i].node).connect(bus.node);
      source.start(now, offset);
      return source;
    });
    fade(bus, 1, now);
    this.retire(this.voices, now);
    this.voices = { sources, sides, bus, started: now, offset };
  }
  private retire(voices: Voices | null, time: number) {
    if (!voices) return;
    fade(voices.bus, 0, time);
    voices.sources.forEach(source => {
      source.onended = () => { source.disconnect(); };
      source.stop(time + FADE);
    });
    voices.sources[0].onended = () => {
      voices.sources[0].disconnect();
      voices.sides.forEach(fader => fader.node.disconnect());
      voices.bus.node.disconnect();
    };
  }
  close() { this.pause(); this.buffers = null; void this.context.close(); }
}
