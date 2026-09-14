import { useCallback, useEffect, useRef, useState } from "react";
import { createPortal } from "react-dom";
import { Knob } from "../components/Knob";
import { PresetIcon, PRESET_ACCENT } from "../components/PresetIcon";
import { STANDARD_LOUDNESS, STANDARD_STYLES, type StandardStyleId } from "../lib/standard-mapping";
import { CLIP_SECONDS, PreviewWorker, type Loaded, type Rendered } from "./processing";
import { auditionBuffer, ComparisonPlayer, type Side } from "./player";
import "./tryit.css";

const fmt = (seconds: number) => `${Math.floor(seconds / 60)}:${String(Math.floor(seconds % 60)).padStart(2, "0")}`;
const level = (result: Rendered | null, match: boolean) => match && result && result.source.lufs > -70 && result.output.lufs > -70
  ? Math.pow(10, (result.source.lufs - result.output.lufs) / 20) : 1;
const describe = (result: Rendered) => {
  if (result.source.lufs <= -70 || result.output.lufs <= -70) return "Too little audible material for a meaningful loudness comparison.";
  if (result.output.tp > -0.95) return "Measured peak is above the −1 dBTP ceiling. Review this result.";
  if (result.output.lufs < result.settings.target - 0.15) return "Below target to preserve peak headroom.";
  if (result.output.lufs > result.settings.target + 0.15) return "Measured loudness is above the selected target.";
  return "Target reached. Compare the sound, then make it yours.";
};

export default function TryItModal({ onClose }: { onClose: () => void }) {
  const [track, setTrack] = useState<(Loaded & { name: string; sampleRate: number; channels: number }) | null>(null);
  const [result, setResult] = useState<Rendered | null>(null);
  const [style, setStyle] = useState<StandardStyleId>("balanced");
  const [intensity, setIntensity] = useState(0.5);
  const [target, setTarget] = useState(-11);
  const [start, setStart] = useState(0);
  const [side, setSide] = useState<Side>("mastered");
  const [match, setMatch] = useState(false);
  const [playing, setPlaying] = useState(false);
  const [position, setPosition] = useState(0);
  const [loading, setLoading] = useState(false);
  const [updating, setUpdating] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [over, setOver] = useState(false);
  const [retry, setRetry] = useState(0);
  const worker = useRef<PreviewWorker | null>(null);
  const player = useRef<ComparisonPlayer | null>(null);
  const epoch = useRef(0);
  const accepted = useRef<Rendered | null>(null);
  const selection = useRef({ side, match });
  selection.current = { side, match };
  const fileInput = useRef<HTMLInputElement | null>(null);
  const panel = useRef<HTMLDivElement | null>(null);
  const closeButton = useRef<HTMLButtonElement | null>(null);
  const wave = useRef<HTMLCanvasElement | null>(null);
  const activeStyle = STANDARD_STYLES.find(s => s.id === style)!;
  const ready = result !== null && !loading;
  const duration = player.current?.duration ?? 0;

  const loadFile = useCallback(async (file: File) => {
    const current = ++epoch.current;
    worker.current?.close();
    worker.current = null;
    player.current?.close();
    player.current = null;
    accepted.current = null;
    setTrack(null); setResult(null); setPlaying(false); setPosition(0);
    setError(null); setLoading(true); setUpdating(false);
    try {
      const ctx = new AudioContext();
      const audio = new ComparisonPlayer(ctx);
      player.current = audio;
      const decoded = await ctx.decodeAudioData(await file.arrayBuffer());
      if (current !== epoch.current) return;
      if (decoded.numberOfChannels > 2) throw new Error("Please choose a mono or stereo mix.");
      if (decoded.duration < 0.5) throw new Error("Please choose at least half a second of audio.");
      const engine = new PreviewWorker(new Worker(new URL("./engine.worker.ts", import.meta.url), { type: "module" }));
      worker.current = engine;
      const reply = await engine.request({ kind: "load", sampleRate: decoded.sampleRate,
        channels: Array.from({ length: decoded.numberOfChannels }, (_, c) => decoded.getChannelData(c).slice()) });
      if (current !== epoch.current || reply?.kind !== "loaded") return;
      setTrack({ ...reply, name: file.name, channels: decoded.numberOfChannels, sampleRate: decoded.sampleRate });
      setStart(reply.start);
    } catch (reason) {
      if (current === epoch.current) {
        worker.current?.close(); worker.current = null;
        player.current?.close(); player.current = null;
        setError(reason instanceof Error ? reason.message : "Couldn't read that file. Try a WAV, MP3, FLAC or M4A.");
      }
    } finally { if (current === epoch.current) setLoading(false); }
  }, []);

  useEffect(() => {
    if (!track || !worker.current) return;
    const engine = worker.current;
    const current = epoch.current;
    let obsolete = false;
    engine.invalidate();
    setUpdating(true);
    setError(null);
    const timer = setTimeout(() => {
      void engine.request({ kind: "render", start, style: activeStyle.preset.kind, intensity, target }).then(reply => {
        if (obsolete || current !== epoch.current || reply?.kind !== "rendered" || !player.current) return;
        const audio = player.current;
        const reset = !accepted.current || accepted.current.settings.start !== reply.settings.start;
        audio.setBuffers(auditionBuffer(audio.context, reply.original, reply.channels, reply.sampleRate),
          auditionBuffer(audio.context, reply.mastered, reply.channels, reply.sampleRate), reset,
          selection.current.side, level(reply, selection.current.match));
        // The player owns the buffers; don't retain duplicate interleaved PCM in React state.
        reply.original = new Float32Array(0); reply.mastered = new Float32Array(0);
        accepted.current = reply;
        setResult(reply); setUpdating(false); setPosition(audio.position);
      }).catch(reason => {
        if (!obsolete && current === epoch.current) { setError(String(reason.message ?? reason)); setUpdating(false); }
      });
    }, 160);
    return () => { obsolete = true; clearTimeout(timer); engine.invalidate(); };
  }, [track, start, activeStyle, intensity, target, retry]);

  const togglePlay = useCallback(async () => {
    const audio = player.current;
    if (!audio || !accepted.current) return;
    if (audio.playing) { audio.pause(); setPosition(audio.position); setPlaying(false); }
    else {
      try { await audio.play(); if (audio === player.current) setPlaying(audio.playing); }
      catch { setError("Playback couldn't start. Press Play to try again."); }
    }
  }, []);
  useEffect(() => { player.current?.select(side, level(result, match)); }, [side, match, result]);
  useEffect(() => {
    if (!playing) return;
    const timer = setInterval(() => setPosition(player.current?.position ?? 0), 50);
    return () => clearInterval(timer);
  }, [playing]);
  const seek = (value: number) => { player.current?.seek(value); setPosition(player.current?.position ?? 0); };

  useEffect(() => {
    const canvas = wave.current;
    if (!canvas || !track) return;
    const ctx = canvas.getContext("2d");
    if (!ctx) return;
    const { width, height } = canvas;
    const x0 = start / track.duration * width;
    const x1 = Math.min(track.duration, start + CLIP_SECONDS) / track.duration * width;
    ctx.clearRect(0, 0, width, height);
    ctx.fillStyle = "#0b0e16"; ctx.fillRect(0, 0, width, height);
    ctx.fillStyle = "#162c4f"; ctx.fillRect(x0, 0, x1 - x0, height);
    for (let x = 0; x < width; x++) {
      const h = Math.max(2, Math.min(1, track.peaks[Math.floor(x * track.peaks.length / width)]) * (height - 14));
      ctx.fillStyle = x >= x0 && x <= x1 ? "#7aa6ff" : "#46516c";
      ctx.fillRect(x, (height - h) / 2, 1, h);
    }
    ctx.fillStyle = "#7aa6ff"; ctx.fillRect(x0, 0, 2, height); ctx.fillRect(x1 - 2, 0, 2, height);
    if (result) {
      const x = (result.settings.start + position) / track.duration * width;
      ctx.fillStyle = "#ffffff"; ctx.fillRect(x, 0, 2, height);
    }
  }, [track, start, result, position]);

  useEffect(() => {
    const previous = document.activeElement as HTMLElement | null;
    const site = document.querySelector<HTMLElement>(".studio-site");
    const wasInert = site?.inert ?? false;
    if (site) site.inert = true;
    closeButton.current?.focus();
    document.body.classList.add("tryit-lock");
    return () => {
      epoch.current++;
      worker.current?.close(); player.current?.close();
      document.body.classList.remove("tryit-lock");
      if (site) site.inert = wasInert;
      previous?.focus();
    };
  }, []);
  useEffect(() => {
    const onKey = (event: KeyboardEvent) => {
      if (event.key === "Escape") { onClose(); return; }
      if (event.key === "Tab") {
        const elements = Array.from(panel.current?.querySelectorAll<HTMLElement>('button:not(:disabled), a[href], input:not(:disabled), summary, [tabindex="0"]') ?? [])
          .filter(el => el.getClientRects().length > 0);
        const first = elements[0], last = elements[elements.length - 1];
        if (event.shiftKey && document.activeElement === first) { event.preventDefault(); last?.focus(); }
        else if (!event.shiftKey && document.activeElement === last) { event.preventDefault(); first?.focus(); }
        return;
      }
      const element = event.target instanceof HTMLElement ? event.target : null;
      if (event.repeat || event.altKey || event.ctrlKey || event.metaKey || element?.closest("input, textarea, select, [contenteditable=true]")) return;
      if (event.code === "Space" && accepted.current) { event.preventDefault(); void togglePlay(); }
      if (event.key.toLowerCase() === "a" && accepted.current) { event.preventDefault(); setSide(s => s === "original" ? "mastered" : "original"); }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [onClose, togglePlay]);

  return createPortal(<>
    <div className="tryit-scrim" aria-hidden="true" />
    <div className="tryit" role="dialog" aria-modal="true" aria-label="Try YES Master on your mix" onClick={event => { if (event.target === event.currentTarget) onClose(); }}>
      <div className="tryit-panel" ref={panel}>
        <header className="tryit-head">
          <div className="tryit-brand">YES Master <span>· Try it on your mix</span></div>
          <span className="tryit-promise"><i />Local processing. No upload.</span>
          <button ref={closeButton} type="button" className="tryit-close" onClick={onClose} aria-label="Close">×</button>
        </header>
        <div className="tryit-body">
          <p className="tryit-scope">Try Standard’s core mastering chain on a 30-second section. The desktop app adds full-track source analysis and export.</p>
          <main className="tryit-console">
            <input ref={fileInput} type="file" hidden accept="audio/*,.wav,.mp3,.flac,.m4a,.aac,.ogg" onChange={event => {
              const file = event.target.files?.[0]; event.target.value = ""; if (file) void loadFile(file);
            }} />
            {!track ? <button type="button" className={"tryit-drop" + (over ? " is-over" : "")} disabled={loading}
              onClick={() => fileInput.current?.click()}
              onDragOver={event => { event.preventDefault(); setOver(true); }} onDragLeave={() => setOver(false)}
              onDrop={event => { event.preventDefault(); setOver(false); const file = event.dataTransfer.files[0]; if (file && !loading) void loadFile(file); }}>
              <span className="tryit-glyph" aria-hidden="true">↓</span>
              <strong>{loading ? "Preparing your track…" : "Choose a mix or drop it here"}</strong>
              <span>WAV, MP3, FLAC or M4A · Your audio stays on this device.</span>
              <small>We’ll prepare the comparison. You press Play.</small>
            </button> : <section className="tryit-wave" aria-label="Track and excerpt">
              <div className="tryit-file"><b title={track.name}>{track.name}</b><button type="button" className="tryit-link" onClick={() => fileInput.current?.click()}>Change track</button></div>
              <div className="tryit-wave-head"><span>Selected section</span><b>{fmt(start)} – {fmt(Math.min(track.duration, start + CLIP_SECONDS))}</b></div>
              <canvas ref={wave} width={1000} height={88} role="img" aria-label="Track waveform, selected excerpt and playback position" />
              {track.duration > CLIP_SECONDS && <label className="tryit-section-picker">Move the 30-second section
                <input type="range" min={0} max={track.duration - CLIP_SECONDS} step={0.1} value={start} aria-label="Section start" aria-valuetext={fmt(start)} onChange={event => setStart(Number(event.target.value))} />
              </label>}
              <div className="tryit-wave-hint">{fmt(track.duration)} total · Starts on the loudest section. Seek below to move within the preview.</div>
            </section>}
            <section className="tryit-transport" aria-label="Audition controls">
              <div className="tryit-listen-row">
                <button type="button" className="tryit-play" disabled={!ready} onClick={() => void togglePlay()} aria-keyshortcuts="Space">
                  <span aria-hidden="true">{playing ? "Ⅱ" : "▶"}</span>{playing ? "Pause" : "Play"}
                </button>
                <div className="tryit-selector" role="group" aria-label="Original or mastered" aria-keyshortcuts="A">
                  <button type="button" disabled={!ready} aria-pressed={side === "original"} onClick={() => setSide("original")}>Original</button>
                  <button type="button" disabled={!ready} aria-pressed={side === "mastered"} onClick={() => setSide("mastered")}>Mastered</button>
                </div>
                <label className="tryit-switch"><input type="checkbox" checked={match} disabled={!ready} onChange={event => setMatch(event.target.checked)} /><span>Volume Match<small>Compare at similar listening levels.</small></span></label>
              </div>
              <div className="tryit-seek"><input type="range" aria-label="Preview position" aria-valuetext={`${fmt(position)} of ${fmt(duration)}`} min={0} max={duration || 1} step={0.05} value={position} disabled={!ready} onChange={event => seek(Number(event.target.value))} /><output aria-label="Playback time">{fmt(position)} / {fmt(duration)}</output></div>
              <div className="tryit-playback-status" role="status">{loading ? "Reading your track locally…" : updating ? (ready ? "Updating preview… Previous comparison remains available." : "Preparing your comparison…") : ready ? (playing ? `Listening to ${side === "original" ? "Original" : "Mastered"}${match ? " · Volume Match on" : ""}` : position > 0 ? "Paused. Press Play to resume." : "Your comparison is ready. Press Play.") : "Choose a track to prepare your comparison."}</div>
              <p className="tryit-hint"><kbd>Space</kbd> Play / pause · <kbd>A</kbd> Original / Mastered · Same playhead on both sides.</p>
            </section>
            {error && <div role="alert" className="tryit-error">{error} {track && <button type="button" className="tryit-link" onClick={() => setRetry(n => n + 1)}>Retry preview</button>}</div>}
            <div className="std-steps">
              <section className="std-step tryit-styles"><span className="std-step-label">1 · Style</span><span className="std-step-hint">Four characters. Make one yours.</span>
                <div className="std-tiles" role="group" aria-label="Style">{STANDARD_STYLES.map(s => <button key={s.id} type="button" disabled={!track} className={"std-tile" + (style === s.id ? " is-active" : "")} style={{ ["--tile-accent" as string]: PRESET_ACCENT[s.preset.kind] }} aria-pressed={style === s.id} onClick={() => setStyle(s.id)}><span className="std-tile-icon"><PresetIcon kind={s.preset.kind} /></span><span className="std-tile-label">{s.label}</span></button>)}</div>
              </section>
              <section className="std-step std-step-intensity"><span className="std-step-label">2 · Intensity</span><span className="std-step-hint">A light touch or stronger character.</span>
                <Knob label="" ariaLabel="Intensity" disabled={!track} size="lg" tone={activeStyle.tone} value={intensity} min={0} max={1} step={0.01} defaultValue={0.5} format={v => `${Math.round(v * 100)}%`} onChange={setIntensity} centerValue />
                <input className="tryit-touch-range" type="range" min={0} max={100} disabled={!track} value={Math.round(intensity * 100)} aria-label="Intensity (slider)" onChange={event => setIntensity(Number(event.target.value) / 100)} />
              </section>
              <section className="std-step"><span className="std-step-label">3 · Loudness</span><span className="std-step-hint">Choose your target.</span>
                <div className="std-seg" role="group" aria-label="Loudness">{STANDARD_LOUDNESS.map(l => <button key={l.id} type="button" disabled={!track} className={"std-seg-option" + (l.lufs === target ? " is-active" : "")} aria-pressed={l.lufs === target} onClick={() => setTarget(l.lufs)}><span className="std-seg-label">{l.label}</span><span className="std-seg-lufs">{l.lufs} LUFS</span></button>)}</div>
                <p className="tryit-hint">The result may sit below your target to preserve peak headroom.</p>
              </section>
            </div>
          </main>
          <aside className="tryit-rail" aria-label="Preview measurements">
            <div className="tryit-out"><span className="std-step-label">Your comparison</span><p className="tryit-hint">Measured over the preview section.</p>
              <div className="tryit-ab">{(["original", "mastered"] as const).map(s => <div key={s} className={"tryit-side" + (s === side ? " is-live" : "")}>
                <div className="tag">{s === "original" ? "Original" : "Mastered"}</div><div className="num">{result ? (s === "original" ? result.source : result.output).lufs.toFixed(1) : "—"}</div><div className="unit">LUFS integrated</div>
                <div className="tp">True peak <b>{result ? (s === "original" ? result.source : result.output).tp.toFixed(1) : "—"}</b> dBTP</div>
              </div>)}</div>
              <p className={"tryit-verify" + (!result || updating ? " is-pending" : "")}>{result ? describe(result) : "Your measured result will appear here."}</p>
              {result && <p className="tryit-hint">{result.settings.style} · {Math.round(result.settings.intensity * 100)}% · Target {result.settings.target} LUFS{updating ? " · Previous preview" : ""}</p>}
              <details className="tryit-details"><summary>Details</summary><p>The browser previews Standard’s core chain without desktop source-aware adjustments. These excerpt measurements don’t predict the full-track export.</p><p>Volume Match changes listening level only. Brief fades soften excerpt loop boundaries; measurements use the unmodified result.</p>{result && <p>{result.version} · {result.sampleRate} Hz · {result.channels} ch · Processed in {result.seconds.toFixed(2)} s. Integrated loudness and true peak measured with BS.1770.</p>}</details>
            </div>
          </aside>
          <footer className="tryit-foot"><span>Go further in the desktop app: Advanced controls, source analysis, album mastering and export.</span><a href="#get-started" onClick={onClose}>Explore the beta →</a></footer>
        </div>
      </div>
    </div>
  </>, document.body);
}
