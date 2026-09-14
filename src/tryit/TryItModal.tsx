import React, { useCallback, useEffect, useRef, useState } from "react";
import { createPortal } from "react-dom";
import { PresetIcon, PRESET_ACCENT } from "../components/PresetIcon";
import { STANDARD_LOUDNESS, STANDARD_STYLES, type StandardStyleId } from "../lib/standard-mapping";
import { CLIP_SECONDS, PreviewWorker, type Loaded, type Rendered } from "./processing";
import { auditionBuffer, ComparisonPlayer, type Side } from "./player";
import "./tryit.css";

const fmt = (seconds: number) => `${Math.floor(seconds / 60)}:${String(Math.floor(seconds % 60)).padStart(2, "0")}`;
/** Volume match: bring the ORIGINAL up to the master's loudness wherever its true-peak headroom
 *  allows (ceiling −1 dBTP), and only trim the master by whatever is left. The master keeps its impact;
 *  the comparison is still level-matched. Returns [originalGain, masteredGain]. */
const level = (result: Rendered | null, match: boolean): [number, number] => {
  if (!match || !result || result.source.lufs <= -70 || result.output.lufs <= -70) return [1, 1];
  const delta = result.output.lufs - result.source.lufs;
  if (delta <= 0) return [Math.pow(10, delta / 20), 1];
  const headroom = Math.max(0, -1 - result.source.tp);
  const lift = Math.min(delta, headroom);
  return [Math.pow(10, lift / 20), Math.pow(10, -(delta - lift) / 20)];
};
const describe = (result: Rendered) => {
  if (result.source.lufs <= -70 || result.output.lufs <= -70) return "Too little audible material for a meaningful loudness comparison.";
  if (result.output.tp > -0.95) return "Measured peak is above the −1 dBTP ceiling. Review this result.";
  if (result.output.lufs < result.settings.target - 0.15) return "Below target to preserve peak headroom.";
  if (result.output.lufs > result.settings.target + 0.15) return "Measured loudness is above the selected target.";
  return "Target reached. Compare the sound, then make it yours.";
};


/** A small (i) that reveals text on hover or focus; the text is in the DOM for screen readers. */
function Info({ label, children }: { label: string; children: React.ReactNode }) {
  return (
    <span className="tryit-info">
      <button type="button" className="tryit-info-btn" aria-label={label}>i</button>
      <span className="tryit-info-pop" role="tooltip">{children}</span>
    </span>
  );
}

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
  const drag = useRef<{ on: boolean; grab: number }>({ on: false, grab: 0 });
  const stripSeconds = (event: React.PointerEvent<HTMLCanvasElement>) => {
    const r = event.currentTarget.getBoundingClientRect();
    return ((event.clientX - r.left) / r.width) * (track?.duration ?? 0);
  };
  const moveStart = (seconds: number) => {
    if (!track) return;
    setStart(Math.max(0, Math.min(Math.max(0, track.duration - CLIP_SECONDS), seconds)));
  };

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

  const ringR = 44, ringC = 2 * Math.PI * ringR;
  const progress = duration > 0 ? Math.min(1, position / duration) : 0;
  return createPortal(<>
    <div className="tryit-scrim" aria-hidden="true" />
    <div className="tryit" role="dialog" aria-modal="true" aria-label="Try YES Master on your mix" onClick={event => { if (event.target === event.currentTarget) onClose(); }}>
      <div className="tryit-panel tryit-card" ref={panel}>
        <header className="tryit-head">
          <div className="tryit-brand">YES Master <span>· Try it on your mix</span></div>
          <button ref={closeButton} type="button" className="tryit-close" onClick={onClose} aria-label="Close">×</button>
        </header>
        <input ref={fileInput} type="file" hidden accept="audio/*,.wav,.mp3,.flac,.m4a,.aac,.ogg" onChange={event => {
          const file = event.target.files?.[0]; event.target.value = ""; if (file) void loadFile(file);
        }} />
        {!track ? (
          <button type="button" className={"tryit-drop" + (over ? " is-over" : "")} disabled={loading}
            onClick={() => fileInput.current?.click()}
            onDragOver={event => { event.preventDefault(); setOver(true); }} onDragLeave={() => setOver(false)}
            onDrop={event => { event.preventDefault(); setOver(false); const file = event.dataTransfer.files[0]; if (file && !loading) void loadFile(file); }}>
            <span className="tryit-glyph" aria-hidden="true">↓</span>
            <strong>{loading ? "Preparing your track…" : "Drop your mix here"}</strong>
            <span>WAV, MP3, FLAC or M4A</span>
          </button>
        ) : (
          <div className="tryit-file"><b title={track.name}>{track.name}</b><button type="button" className="tryit-link" onClick={() => fileInput.current?.click()}>change</button></div>
        )}

        <div className="tryit-stage">
          <button type="button" className={"tryit-ring" + (playing ? " is-playing" : "")} disabled={!ready} onClick={() => void togglePlay()} aria-keyshortcuts="Space">
            <svg viewBox="0 0 100 100" aria-hidden="true">
              <circle className="track" cx="50" cy="50" r={ringR} />
              <circle className="fill" cx="50" cy="50" r={ringR} strokeDasharray={ringC} strokeDashoffset={ringC * (1 - progress)} />
              {playing
                ? <g className="glyph"><rect x="38" y="35" width="8" height="30" rx="2" /><rect x="54" y="35" width="8" height="30" rx="2" /></g>
                : <path className="glyph" d="M41 33 L69 50 L41 67 Z" />}
            </svg>
            <span className="sr-only">{playing ? "Pause" : "Play"}</span>
          </button>
          <div className="tryit-stage-side">
            <div className="tryit-selector" role="group" aria-label="Original or mastered" aria-keyshortcuts="A">
              <button type="button" disabled={!ready} aria-pressed={side === "original"} onClick={() => setSide("original")}>Original</button>
              <button type="button" disabled={!ready} aria-pressed={side === "mastered"} onClick={() => setSide("mastered")}>Mastered</button>
            </div>
            <label className="tryit-check"><input type="checkbox" checked={match} disabled={!ready} onChange={event => setMatch(event.target.checked)} /><span>Volume match</span>
              <Info label="About volume match">Use volume match to preview your original and master at the same volume and compare with more accuracy. The original is raised where it has headroom; the master is only trimmed by what’s left.</Info>
            </label>
          </div>
        </div>

        {track && (
          <div className="tryit-strip">
            <canvas ref={wave} width={1000} height={56} role="img" aria-label="Track waveform. Drag the lit window to choose your thirty seconds."
              onPointerDown={event => { const x = stripSeconds(event); const inside = x >= start && x <= start + CLIP_SECONDS; drag.current = { on: true, grab: inside ? x - start : CLIP_SECONDS / 2 }; event.currentTarget.setPointerCapture(event.pointerId); moveStart(x - drag.current.grab); }}
              onPointerMove={event => { if (drag.current.on) moveStart(stripSeconds(event) - drag.current.grab); }}
              onPointerUp={() => { drag.current.on = false; }} onPointerCancel={() => { drag.current.on = false; }} />
            <div className="tryit-strip-cap"><span>{fmt(start)} – {fmt(Math.min(track.duration, start + CLIP_SECONDS))}</span><span>drag the window to pick your 30 s</span></div>
          </div>
        )}

        <div className="tryit-rows">
          <div className="tryit-row">
            <div className="tryit-row-label">Style <Info label="About styles">Choose between 4 mastering styles, each with its own character and feel. <b>Universal:</b> balanced, keeps the focus on your mix. <b>Clarity:</b> open and defined on top, focused below. <b>Tape:</b> warmer weight with a softer edge. <b>Oomph:</b> low-end weight and forward energy.</Info></div>
            <div className="std-tiles tryit-tiles-compact" role="group" aria-label="Style">{STANDARD_STYLES.map(s => <button key={s.id} type="button" disabled={!track} className={"std-tile" + (style === s.id ? " is-active" : "")} style={{ ["--tile-accent" as string]: PRESET_ACCENT[s.preset.kind] }} aria-pressed={style === s.id} onClick={() => setStyle(s.id)}><span className="std-tile-icon"><PresetIcon kind={s.preset.kind} /></span><span className="std-tile-label">{s.label}</span></button>)}</div>
          </div>
          <div className="tryit-row">
            <div className="tryit-row-label">Intensity <Info label="About intensity">Set how strongly the style comes through, from a light touch to a stronger character. You hear it change while the track plays.</Info></div>
            <div className="tryit-slider"><input type="range" min={0} max={100} disabled={!track} value={Math.round(intensity * 100)} aria-label="Intensity" onChange={event => setIntensity(Number(event.target.value) / 100)} style={{ ["--pct" as string]: `${Math.round(intensity * 100)}%`, ["--tone" as string]: PRESET_ACCENT[activeStyle.preset.kind] }} /><output>{Math.round(intensity * 100)}%</output></div>
          </div>
          <div className="tryit-row">
            <div className="tryit-row-label">Loudness <Info label="About loudness">Set your master’s loudness level. <b>Low</b> (−14 LUFS) matches streaming platforms. <b>Medium</b> (−11) is a modern, competitive level. <b>High</b> (−9) is hot and dense. A −1 dBTP ceiling keeps every level clean.</Info></div>
            <div className="tryit-pills" role="group" aria-label="Loudness">{STANDARD_LOUDNESS.map(l => <button key={l.id} type="button" disabled={!track} className={l.lufs === target ? "is-active" : ""} aria-pressed={l.lufs === target} onClick={() => setTarget(l.lufs)}>{l.label}</button>)}</div>
          </div>
        </div>

        {result && <span className="sr-only" role="status">{result.settings.style} · {Math.round(result.settings.intensity * 100)}% · target {result.settings.target} LUFS · {describe(result)}</span>}
        {error && <div role="alert" className="tryit-error">{error} {track && <button type="button" className="tryit-link" onClick={() => setRetry(n => n + 1)}>Retry</button>}</div>}

        <a href="#get-started" className="tryit-cta" onClick={onClose}>Get the free beta</a>
        <p className="tryit-cta-sub">Export and Advanced live in the desktop app.</p>
        <p className="tryit-privacy"><i />Processed locally in your browser. Your track is never uploaded.</p>
      </div>
    </div>
  </>, document.body);
}
