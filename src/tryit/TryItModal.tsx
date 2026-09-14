// Try it on your mix — the Standard chain, compiled to WebAssembly, running
// in the visitor's browser. Nothing here touches the network: the file is
// decoded by the browser, mastered in the tab, and forgotten on close.
//
// Reuses the app's own Standard-view pieces (tiles + preset art, the
// Intensity knob, the Low/Medium/High picker) so the modal looks like the
// thing it demonstrates. Styles live in ./tryit.css, scoped under .tryit.
import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { createPortal } from "react-dom";
import { Knob } from "../components/Knob";
import { PresetIcon, PRESET_ACCENT } from "../components/PresetIcon";
import {
  STANDARD_LOUDNESS,
  STANDARD_STYLES,
  type StandardStyleId,
} from "../lib/standard-mapping";
import initEngine, {
  master_standard,
  measure_loudness,
  version as engineVersion,
} from "./engine/yes_master_web.js";
import wasmUrl from "./engine/yes_master_web_bg.wasm?url";
import "./tryit.css";

const CLIP_SECONDS = 30;
const CEILING_DBTP = -1.0;
type Side = "A" | "B";
type Measure = { lufs: number; tp: number; target?: number };

let enginePromise: Promise<unknown> | null = null;
function ensureEngine(): Promise<unknown> {
  if (!enginePromise) enginePromise = initEngine({ module_or_path: wasmUrl });
  return enginePromise;
}

const fmt = (s: number) =>
  `${Math.floor(s / 60)}:${String(Math.floor(s % 60)).padStart(2, "0")}`;

function loudestWindow(buffer: AudioBuffer): number {
  const sr = buffer.sampleRate;
  const n = buffer.length;
  const win = Math.min(n, CLIP_SECONDS * sr);
  if (win >= n) return 0;
  const step = sr;
  const secs = Math.ceil(n / step);
  const energy = new Float64Array(secs);
  for (let c = 0; c < buffer.numberOfChannels; c++) {
    const d = buffer.getChannelData(c);
    for (let i = 0; i < n; i++) energy[(i / step) | 0] += d[i] * d[i];
  }
  const w = Math.ceil(win / step);
  let best = 0;
  let bestE = -1;
  let acc = 0;
  for (let s = 0; s < secs; s++) {
    acc += energy[s];
    if (s >= w) acc -= energy[s - w];
    if (s >= w - 1 && acc > bestE) {
      bestE = acc;
      best = s - w + 1;
    }
  }
  return Math.max(0, Math.min(n - win, best * step));
}

function toInterleaved(buffer: AudioBuffer, start: number, len: number): Float32Array {
  const ch = buffer.numberOfChannels;
  const out = new Float32Array(len * ch);
  for (let c = 0; c < ch; c++) {
    const d = buffer.getChannelData(c);
    for (let i = 0; i < len; i++) out[i * ch + c] = d[start + i];
  }
  return out;
}

function toBuffer(ctx: AudioContext, interleaved: Float32Array, ch: number, sr: number): AudioBuffer {
  const frames = interleaved.length / ch;
  const b = ctx.createBuffer(ch, frames, sr);
  for (let c = 0; c < ch; c++) {
    const d = b.getChannelData(c);
    for (let i = 0; i < frames; i++) d[i] = interleaved[i * ch + c];
  }
  return b;
}

function buildPeaks(buffer: AudioBuffer, cols: number): Float32Array {
  const n = buffer.length;
  const per = n / cols;
  const out = new Float32Array(cols);
  for (let c = 0; c < buffer.numberOfChannels; c++) {
    const d = buffer.getChannelData(c);
    for (let x = 0; x < cols; x++) {
      let m = 0;
      const a = Math.floor(x * per);
      const b = Math.min(n, Math.floor((x + 1) * per));
      for (let i = a; i < b; i += 4) {
        const v = Math.abs(d[i]);
        if (v > m) m = v;
      }
      if (m > out[x]) out[x] = m;
    }
  }
  return out;
}

function verifyText(a: Measure, b: Measure, targetNow: number, match: boolean): string {
  const target = b.target ?? targetNow;
  const hit = Math.abs(b.lufs - target) <= 0.15;
  const under = b.tp <= CEILING_DBTP + 0.05;
  const gain = b.lufs - a.lufs;
  let s = `Landed at ${b.lufs.toFixed(1)} LUFS` +
    (hit ? ` (target ${target})` : ` against a ${target} target, held back by the ${CEILING_DBTP} dBTP ceiling`) +
    `, true peak ${b.tp.toFixed(1)} dBTP` + (under ? `, under the ${CEILING_DBTP} dBTP ceiling` : "") +
    `. ${gain >= 0 ? "+" : ""}${gain.toFixed(1)} dB louder than your original`;
  s += match
    ? ", and you are hearing it at the original's level, so what changed is the tone and the density."
    : ". Turn on Match loudness to hear the tone change without the level change.";
  return s;
}

type Player = {
  srcA: AudioBufferSourceNode; srcB: AudioBufferSourceNode;
  gainA: GainNode; gainB: GainNode;
  anA: AnalyserNode; anB: AnalyserNode;
  startedAt: number;
};

export default function TryItModal({ onClose }: { onClose: () => void }) {
  const [fileName, setFileName] = useState<string | null>(null);
  const [fileInfo, setFileInfo] = useState("");
  const [over, setOver] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [style, setStyle] = useState<StandardStyleId>("balanced");
  const [intensity, setIntensity] = useState(0.5);
  const [target, setTarget] = useState(-11);
  const [match, setMatch] = useState(false);
  const [measA, setMeasA] = useState<Measure | null>(null);
  const [measB, setMeasB] = useState<Measure | null>(null);
  const [pending, setPending] = useState<string | null>(null);
  const [status, setStatus] = useState("");
  const [live, setLive] = useState<Side>("B");
  const [playing, setPlaying] = useState(false);
  const [winLabel, setWinLabel] = useState("");

  const ctxRef = useRef<AudioContext | null>(null);
  const decodedRef = useRef<AudioBuffer | null>(null);
  const peaksRef = useRef<Float32Array | null>(null);
  const winRef = useRef({ start: 0, len: 0 });
  const rawRef = useRef<{ inter: Float32Array; buf: AudioBuffer } | null>(null);
  const masteredRef = useRef<AudioBuffer | null>(null);
  const playerRef = useRef<Player | null>(null);
  const tokenRef = useRef(0);
  const canvasRef = useRef<HTMLCanvasElement | null>(null);
  const meterA = useRef<HTMLElement | null>(null);
  const meterB = useRef<HTMLElement | null>(null);
  const liveRef = useRef<Side>("B");
  const matchRef = useRef(false);
  const measRef = useRef<{ a: Measure | null; b: Measure | null }>({ a: null, b: null });
  const dragRef = useRef<{ on: boolean; off: number }>({ on: false, off: 0 });

  liveRef.current = live;
  matchRef.current = match;
  measRef.current = { a: measA, b: measB };

  const activeStyle = useMemo(() => STANDARD_STYLES.find((s) => s.id === style)!, [style]);
  const matchGain = useCallback(() => {
    const { a, b } = measRef.current;
    return matchRef.current && a && b ? Math.pow(10, (a.lufs - b.lufs) / 20) : 1;
  }, []);

  const applyLive = useCallback((hard: boolean) => {
    const p = playerRef.current;
    const ctx = ctxRef.current;
    if (!p || !ctx) return;
    const t = ctx.currentTime;
    const ramp = hard ? 0 : 0.008;
    p.gainA.gain.cancelScheduledValues(t);
    p.gainB.gain.cancelScheduledValues(t);
    p.gainA.gain.setTargetAtTime(liveRef.current === "A" ? 1 : 0, t, ramp);
    p.gainB.gain.setTargetAtTime(liveRef.current === "B" ? matchGain() : 0, t, ramp);
  }, [matchGain]);

  const stop = useCallback(() => {
    const p = playerRef.current;
    if (p) {
      try { p.srcA.stop(); p.srcB.stop(); } catch { /* already stopped */ }
      playerRef.current = null;
    }
    setPlaying(false);
  }, []);

  const play = useCallback((offset = 0) => {
    const ctx = ctxRef.current;
    const raw = rawRef.current;
    const mastered = masteredRef.current;
    if (!ctx || !raw || !mastered) return;
    void ctx.resume();
    const srcA = ctx.createBufferSource(); srcA.buffer = raw.buf; srcA.loop = true;
    const srcB = ctx.createBufferSource(); srcB.buffer = mastered; srcB.loop = true;
    const gainA = ctx.createGain(); const gainB = ctx.createGain();
    const anA = ctx.createAnalyser(); const anB = ctx.createAnalyser();
    anA.fftSize = anB.fftSize = 2048;
    srcA.connect(anA).connect(gainA).connect(ctx.destination);
    srcB.connect(anB).connect(gainB).connect(ctx.destination);
    const t = ctx.currentTime + 0.05;
    srcA.start(t, offset); srcB.start(t, offset);
    playerRef.current = { srcA, srcB, gainA, gainB, anA, anB, startedAt: t - offset };
    applyLive(true);
    setPlaying(true);
  }, [applyLive]);

  // Live level meters (RMS of what is actually audible on each side).
  useEffect(() => {
    if (!playing) {
      if (meterA.current) meterA.current.style.width = "0%";
      if (meterB.current) meterB.current.style.width = "0%";
      return;
    }
    const tmp = new Float32Array(2048);
    const rms = (an: AnalyserNode, g: number) => {
      an.getFloatTimeDomainData(tmp);
      let s = 0;
      for (let i = 0; i < tmp.length; i++) s += tmp[i] * tmp[i];
      const db = 20 * Math.log10(Math.sqrt(s / tmp.length) * g + 1e-9);
      return Math.max(0, Math.min(1, (db + 40) / 40));
    };
    let raf = 0;
    const loop = () => {
      const p = playerRef.current;
      if (p) {
        if (meterA.current) meterA.current.style.width = `${rms(p.anA, 1) * 100}%`;
        if (meterB.current) meterB.current.style.width = `${rms(p.anB, matchGain()) * 100}%`;
      }
      raf = requestAnimationFrame(loop);
    };
    raf = requestAnimationFrame(loop);
    return () => cancelAnimationFrame(raf);
  }, [playing, matchGain]);

  const drawWave = useCallback(() => {
    const cv = canvasRef.current;
    const decoded = decodedRef.current;
    const peaks = peaksRef.current;
    if (!cv || !decoded || !peaks) return;
    const g = cv.getContext("2d");
    if (!g) return;
    const W = cv.width; const H = cv.height;
    g.clearRect(0, 0, W, H);
    g.fillStyle = "#0b0e16"; g.fillRect(0, 0, W, H);
    const { start, len } = winRef.current;
    const x0 = (start / decoded.length) * W;
    const x1 = ((start + len) / decoded.length) * W;
    g.fillStyle = "rgba(79,134,247,0.16)"; g.fillRect(x0, 0, x1 - x0, H);
    for (let x = 0; x < W; x++) {
      const h = Math.max(2, peaks[x] * (H - 12));
      g.fillStyle = x >= x0 && x <= x1 ? "#7aa6ff" : "#3a4159";
      g.fillRect(x, (H - h) / 2, 1, h);
    }
    g.fillStyle = "#7aa6ff"; g.fillRect(x0 - 1, 0, 2, H); g.fillRect(x1 - 1, 0, 2, H);
    setWinLabel(`${fmt(start / decoded.sampleRate)} – ${fmt((start + len) / decoded.sampleRate)}`);
  }, []);

  const render = useCallback(async () => {
    const ctx = ctxRef.current;
    const raw = rawRef.current;
    if (!ctx || !raw) return;
    const token = ++tokenRef.current;
    setPending(`Mastering ${activeStyle.label} at ${Math.round(intensity * 100)}%, ${target} LUFS…`);
    await new Promise((r) => setTimeout(r, 10));
    const t0 = performance.now();
    const out = master_standard(raw.inter, raw.buf.numberOfChannels, raw.buf.sampleRate, activeStyle.preset.kind, intensity, target);
    if (token !== tokenRef.current) return;
    const m = measure_loudness(out, raw.buf.numberOfChannels, raw.buf.sampleRate);
    const b = { lufs: m[0], tp: m[1], target };
    measRef.current.b = b;
    setMeasB(b);
    masteredRef.current = toBuffer(ctx, out, raw.buf.numberOfChannels, raw.buf.sampleRate);
    setPending(null);
    setStatus(`Mastered ${Math.round(raw.buf.duration)} s in ${((performance.now() - t0) / 1000).toFixed(2)} s on your machine · ${engineVersion()} · BS.1770 measured in the same engine.`);
    const p = playerRef.current;
    if (p) {
      const pos = (ctx.currentTime - p.startedAt) % raw.buf.duration;
      stop();
      play(pos);
    }
  }, [activeStyle, intensity, target, stop, play]);

  const sliceAndRender = useCallback(async () => {
    const ctx = ctxRef.current;
    const decoded = decodedRef.current;
    if (!ctx || !decoded) return;
    const { start, len } = winRef.current;
    const inter = toInterleaved(decoded, start, len);
    rawRef.current = { inter, buf: toBuffer(ctx, inter, decoded.numberOfChannels, decoded.sampleRate) };
    const m = measure_loudness(inter, decoded.numberOfChannels, decoded.sampleRate);
    const a = { lufs: m[0], tp: m[1] };
    measRef.current.a = a;
    setMeasA(a);
    await render();
  }, [render]);

  // Re-master when a control changes (after the first load).
  const firstRender = useRef(true);
  useEffect(() => {
    if (firstRender.current) { firstRender.current = false; return; }
    if (rawRef.current) void render();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [style, intensity, target]);

  useEffect(() => { applyLive(true); }, [live, match, applyLive]);

  const loadFile = useCallback(async (file: File) => {
    stop();
    setError(null);
    setStatus(`Decoding ${file.name}…`);
    const ctx = ctxRef.current ?? new AudioContext();
    ctxRef.current = ctx;
    let decoded: AudioBuffer;
    try {
      decoded = await ctx.decodeAudioData(await file.arrayBuffer());
    } catch {
      setError("Couldn't decode that file. Try a WAV, MP3, FLAC or M4A.");
      setStatus("");
      return;
    }
    decodedRef.current = decoded;
    winRef.current = { len: Math.min(decoded.length, CLIP_SECONDS * decoded.sampleRate), start: loudestWindow(decoded) };
    setFileName(file.name);
    setFileInfo(`${fmt(decoded.duration)} · ${decoded.sampleRate} Hz · ${decoded.numberOfChannels} ch · decoded in this tab`);
    await ensureEngine();
    requestAnimationFrame(() => {
      const cv = canvasRef.current;
      if (cv) { peaksRef.current = buildPeaks(decoded, cv.width); drawWave(); }
    });
    await sliceAndRender();
    play();
  }, [stop, drawWave, sliceAndRender, play]);

  // Window drag on the waveform.
  const canvasX = (ev: React.PointerEvent<HTMLCanvasElement>) => {
    const r = ev.currentTarget.getBoundingClientRect();
    return ((ev.clientX - r.left) / r.width) * (decodedRef.current?.length ?? 0);
  };
  const moveWin = (start: number) => {
    const decoded = decodedRef.current;
    if (!decoded) return;
    const { len } = winRef.current;
    winRef.current.start = Math.round(Math.max(0, Math.min(decoded.length - len, start)));
    drawWave();
  };

  const flip = useCallback(() => setLive((l) => (l === "A" ? "B" : "A")), []);

  // Escape closes; space flips while playing. Lock page scroll behind the modal.
  useEffect(() => {
    document.body.classList.add("tryit-lock");
    const onKey = (ev: KeyboardEvent) => {
      if (ev.key === "Escape") onClose();
      if (ev.code === "Space" && rawRef.current && !(ev.target instanceof HTMLInputElement)) {
        ev.preventDefault();
        if (playerRef.current) flip(); else play();
      }
    };
    window.addEventListener("keydown", onKey);
    return () => {
      window.removeEventListener("keydown", onKey);
      document.body.classList.remove("tryit-lock");
    };
  }, [onClose, flip, play]);

  // Tear down audio on close.
  useEffect(() => () => {
    const p = playerRef.current;
    if (p) { try { p.srcA.stop(); p.srcB.stop(); } catch { /* noop */ } }
    void ctxRef.current?.close();
  }, []);

  const loaded = fileName !== null;

  return createPortal(
    <>
      <div className="tryit-scrim" onClick={onClose} aria-hidden="true" />
      <div className="tryit" role="dialog" aria-modal="true" aria-label="Try YES Master on your mix">
        <div className="tryit-panel">
          <div className="tryit-head">
            <div className="tryit-brand">
              <svg width="22" height="22" viewBox="0 0 24 24" fill="none" aria-hidden="true">
                <path d="M4 6h2v12H4zM8 10h2v8H8zM12 4h2v16h-2zM16 8h2v10h-2zM20 12h2v6h-2z" fill="currentColor" />
              </svg>
              YES Master <span>· Try it on your mix</span>
            </div>
            <div className="tryit-promise"><i />Processed locally in your browser. Your track is never uploaded.</div>
            <button type="button" className="tryit-close" onClick={onClose} aria-label="Close">×</button>
          </div>

          <div className="tryit-body">
            <div>
              {!loaded ? (
                <label
                  className={"tryit-drop" + (over ? " is-over" : "") + (error ? " is-error" : "")}
                  onDragEnter={(e) => { e.preventDefault(); setOver(true); }}
                  onDragOver={(e) => { e.preventDefault(); setOver(true); }}
                  onDragLeave={(e) => { e.preventDefault(); setOver(false); }}
                  onDrop={(e) => { e.preventDefault(); setOver(false); const f = e.dataTransfer.files[0]; if (f) void loadFile(f); }}
                >
                  <input type="file" accept="audio/*,.wav,.mp3,.flac,.m4a,.aac,.ogg" onChange={(e) => { const f = e.target.files?.[0]; if (f) void loadFile(f); }} />
                  <div className="tryit-glyph" aria-hidden="true">
                    <svg width="26" height="26" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><path d="M12 3v12" /><path d="m7 10 5 5 5-5" /><path d="M5 21h14" /></svg>
                  </div>
                  <h3>Drop your mix here</h3>
                  <p>WAV, MP3, FLAC or M4A. Thirty seconds of it gets mastered in this tab.</p>
                  {error && <p style={{ color: "#f87171" }}>{error}</p>}
                  {status && !error && <p>{status}</p>}
                </label>
              ) : (
                <div className="tryit-wave">
                  <div className="tryit-wave-head">
                    <span>{fileName} <b style={{ marginLeft: 8 }}>{winLabel}</b></span>
                    <label style={{ cursor: "pointer", color: "var(--accent-bright)", letterSpacing: 0, textTransform: "none" }}>
                      swap file<input type="file" accept="audio/*,.wav,.mp3,.flac,.m4a,.aac,.ogg" style={{ display: "none" }} onChange={(e) => { const f = e.target.files?.[0]; if (f) void loadFile(f); }} />
                    </label>
                  </div>
                  <canvas
                    ref={canvasRef}
                    width={1800}
                    height={152}
                    aria-label="Waveform. Drag to choose your thirty seconds."
                    onPointerDown={(ev) => {
                      const x = canvasX(ev);
                      const { start, len } = winRef.current;
                      dragRef.current = { on: true, off: x >= start && x <= start + len ? x - start : len / 2 };
                      ev.currentTarget.setPointerCapture(ev.pointerId);
                      moveWin(x - dragRef.current.off);
                    }}
                    onPointerMove={(ev) => { if (dragRef.current.on) moveWin(canvasX(ev) - dragRef.current.off); }}
                    onPointerUp={() => { if (!dragRef.current.on) return; dragRef.current.on = false; void sliceAndRender(); }}
                  />
                  <div className="tryit-wave-hint">{fileInfo}. Drag the window to the part you care about; it starts on the loudest section.</div>
                </div>
              )}

              <div className="std-steps" aria-disabled={!loaded}>
                <div className="std-step">
                  <span className="std-step-label">1 · Style</span>
                  <span className="std-step-hint">Four characters. One click each.</span>
                  <div className="std-tiles" role="group" aria-label="Style">
                    {STANDARD_STYLES.map((s) => (
                      <button
                        key={s.id}
                        type="button"
                        className={"std-tile" + (s.id === style ? " is-active" : "")}
                        style={{ ["--tile-accent" as never]: PRESET_ACCENT[s.preset.kind] }}
                        aria-pressed={s.id === style}
                        onClick={() => setStyle(s.id)}
                      >
                        <span className="std-tile-icon"><PresetIcon kind={s.preset.kind} /></span>
                        <span className="std-tile-label">{s.label}</span>
                      </button>
                    ))}
                  </div>
                </div>
                <div className="std-step std-step-intensity">
                  <span className="std-step-label">2 · Intensity</span>
                  <span className="std-step-hint">A light touch, or a stronger character. Hear it while it plays.</span>
                  <Knob
                    label=""
                    ariaLabel="Intensity"
                    size="lg"
                    tone={activeStyle.tone}
                    value={intensity}
                    min={0}
                    max={1}
                    step={0.01}
                    defaultValue={0.5}
                    format={(v) => `${Math.round(v * 100)}%`}
                    onChange={(v) => setIntensity(Math.max(0, Math.min(1, v)))}
                    centerValue
                  />
                </div>
                <div className="std-step">
                  <span className="std-step-label">3 · Loudness</span>
                  <span className="std-step-hint">Choose your target loudness.</span>
                  <div className="std-seg" role="group" aria-label="Loudness">
                    {STANDARD_LOUDNESS.map((l) => (
                      <button
                        key={l.id}
                        type="button"
                        className={"std-seg-option" + (l.lufs === target ? " is-active" : "")}
                        aria-pressed={l.lufs === target}
                        onClick={() => setTarget(l.lufs)}
                      >
                        <span className="std-seg-label">{l.label}</span>
                        <span className="std-seg-lufs">{l.lufs} LUFS</span>
                      </button>
                    ))}
                  </div>
                </div>
              </div>
            </div>

            <aside className="tryit-rail">
              <div className="tryit-out">
                <span className="std-step-label">Master out</span>
                <div className="tryit-ab">
                  <div className={"tryit-side" + (live === "A" ? " is-live" : "")}>
                    <div className="tag">Original</div>
                    <div className="num">{measA ? measA.lufs.toFixed(1) : "—"}</div>
                    <div className="unit">LUFS INTEGRATED</div>
                    <div className="tp">True peak <b>{measA ? measA.tp.toFixed(1) : "—"}</b> dBTP</div>
                    <div className="tryit-meter"><i ref={(el) => { meterA.current = el; }} /></div>
                  </div>
                  <div className={"tryit-side" + (live === "B" ? " is-live" : "")}>
                    <div className="tag">YES Master</div>
                    <div className="num">{measB ? measB.lufs.toFixed(1) : "—"}</div>
                    <div className="unit">LUFS INTEGRATED</div>
                    <div className="tp">True peak <b>{measB ? measB.tp.toFixed(1) : "—"}</b> dBTP</div>
                    <div className="tryit-meter"><i ref={(el) => { meterB.current = el; }} /></div>
                  </div>
                </div>
                <div className={"tryit-verify" + (pending || !measA || !measB ? " is-pending" : "")}>
                  {pending ?? (measA && measB ? verifyText(measA, measB, target, match) : "Drop a mix to see the measured result.")}
                </div>
              </div>

              <div className="tryit-out tryit-transport">
                <button
                  type="button"
                  className="tryit-btn is-cta"
                  disabled={!loaded || !!pending}
                  onClick={() => (playerRef.current ? flip() : play())}
                >
                  {!playing ? "Play the comparison" : live === "B" ? "Hear the original" : "Hear the master"}
                </button>
                <button type="button" className="tryit-btn is-ghost" disabled={!loaded} onClick={() => (playing ? stop() : play())}>
                  {playing ? "Stop" : "Play"}
                </button>
                <label className="tryit-switch">
                  <input type="checkbox" checked={match} onChange={(e) => setMatch(e.target.checked)} disabled={!loaded} />
                  <span>Match loudness<small>Audition the master at the original's level, so louder can't cheat.</small></span>
                </label>
                <div className="tryit-hint">Press <kbd>space</kbd> to flip. Same playhead, no gap. Nothing to export here; that's the app.</div>
                <div className="tryit-status">{loaded ? status : ""}</div>
              </div>
            </aside>

            <div className="tryit-foot">
              <span>This is the Standard chain from the desktop app, compiled to run in your browser. The app adds Advanced mode, source analysis, album mastering and export.</span>
              <a href="#get-started" onClick={onClose}>Get the free beta →</a>
            </div>
          </div>
        </div>
      </div>
    </>,
    document.body,
  );
}
