import React, { useCallback, useEffect, useRef, useState } from "react";
import { createPortal } from "react-dom";
import { PresetIcon, PRESET_ACCENT } from "../components/PresetIcon";
import { STANDARD_LOUDNESS, STANDARD_STYLES, type StandardStyleId } from "../lib/standard-mapping";
import { CLIP_SECONDS, neighbours, PreviewEngine, type Analysis, type Excerpt, type Measurement, type RenderSettings, type Stage } from "./processing";
import { auditionBuffer, ComparisonPlayer, type Side } from "./player";
import { decodeAtSourceRate, playbackContext } from "./decode";
import { minutesBucket, trackTryIt } from "./analytics";
import { AnalysisOrb } from "../components/AnalysisOrb";
import { MORPH_MS } from "../lib/analysis-orb";
import { prefersReducedMotion } from "../lib/motion";
import "./tryit.css";

/** The worker's analysis stages, in order, for the preparation checklist. */
const STAGES = ["Analyzing audio", "Choosing your 30 seconds", "Reading loudness", "Checking dynamics", "Evaluating stereo field", "Reading tonal balance", "Building mastering context"];
const STYLE_KINDS = STANDARD_STYLES.map(s => s.preset.kind);
const TARGETS = STANDARD_LOUDNESS.map(l => l.lufs);
const SEEK_STEP = 5;
const fmt = (seconds: number) => `${Math.floor(seconds / 60)}:${String(Math.floor(seconds % 60)).padStart(2, "0")}`;
const kHz = (rate: number) => `${(rate / 1000).toString().replace(/\.0$/, "")} kHz`;

/** Volume match: both sides play at ONE fixed reference level that does not move with the
 *  loudness target: the loudest the ORIGINAL can be lifted to without its true peak crossing
 *  −1 dBTP. The original always gets that lift; the master is trimmed (or lifted, capped by its
 *  own headroom) to meet it. Changing Low/Medium/High then changes the sound, not the volume.
 *  Returns [originalGain, masteredGain]. */
export const level = (source: Measurement | null, output: Measurement | null, match: boolean): [number, number] => {
  if (!match || !source || !output || source.lufs <= -70 || output.lufs <= -70) return [1, 1];
  const liftDb = Math.max(0, -1 - source.tp);
  const referenceLufs = source.lufs + liftDb;
  const masterDb = Math.min(referenceLufs - output.lufs, Math.max(0, -1 - output.tp));
  return [Math.pow(10, liftDb / 20), Math.pow(10, masterDb / 20)];
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

/** The app's analysis wait: the particle orb with the real stage label and bar
 *  underneath (Waveform.tsx shows exactly this while a track analyzes). Sits in
 *  the waveform strip's slot so the orb can fly into the waveform when done. */
function Preparing({ stage, motion }: { stage: Stage | null; motion: boolean }) {
  const label = stage?.label && stage.label !== "Ready" ? stage.label : STAGES[0];
  const done = stage ? (stage.label === "Ready" ? STAGES.length : Math.max(0, STAGES.indexOf(stage.label))) : 0;
  return (
    <div className={"tryit-strip is-analyzing" + (motion ? " has-orb" : "")} role="status" aria-live="polite">
      {motion && <AnalysisOrb phase="orb" />}
      <div className="tryit-prep-bar" aria-hidden="true"><i style={{ ["--pct" as string]: `${Math.round((stage?.fraction ?? 0) * 100)}%` }} /></div>
      <div className="tryit-strip-cap"><span className="tryit-prep-label">{label}</span><span>{done} of {STAGES.length}</span></div>
    </div>
  );
}

type Track = { name: string; duration: number; peaks: Float32Array; sampleRate: number; fileRate: number | null; channels: number; analysis: Analysis; workers: number };
type Accepted = { settings: RenderSettings; output: Measurement; source: Measurement };

export default function TryItModal({ onClose }: { onClose: () => void }) {
  const [track, setTrack] = useState<Track | null>(null);
  const [stage, setStage] = useState<Stage | null>(null);
  const [fileName, setFileName] = useState("");
  const [result, setResult] = useState<Accepted | null>(null);
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
  const [morphing, setMorphing] = useState(false);
  const motion = useRef(!prefersReducedMotion());
  const engine = useRef<PreviewEngine | null>(null);
  const player = useRef<ComparisonPlayer | null>(null);
  const epoch = useRef(0);
  const accepted = useRef<Accepted | null>(null);
  const excerpt = useRef<{ data: Excerpt; buffer: AudioBuffer } | null>(null);
  const selection = useRef({ side, match });
  selection.current = { side, match };
  /** Continuous controls (slider, window drag) debounce; tiles and pills render at once. */
  const continuous = useRef(false);
  const fileInput = useRef<HTMLInputElement | null>(null);
  const panel = useRef<HTMLDivElement | null>(null);
  const closeButton = useRef<HTMLButtonElement | null>(null);
  const wave = useRef<HTMLCanvasElement | null>(null);
  const activeStyle = STANDARD_STYLES.find(s => s.id === style)!;
  const ready = result !== null && !loading;
  const duration = player.current?.duration ?? 0;
  const drag = useRef<{ on: boolean; grab: number; x0: number; moved: boolean; inside: boolean }>({ on: false, grab: 0, x0: 0, moved: false, inside: false });
  const stripSeconds = (event: React.PointerEvent<HTMLCanvasElement>) => {
    const r = event.currentTarget.getBoundingClientRect();
    return ((event.clientX - r.left) / r.width) * (track?.duration ?? 0);
  };
  const moveStart = (seconds: number) => {
    if (!track) return;
    continuous.current = true;
    setStart(Math.max(0, Math.min(Math.max(0, track.duration - CLIP_SECONDS), seconds)));
  };
  const seekTo = (seconds: number) => {
    const audio = player.current;
    if (!audio || !accepted.current) return;
    audio.seek(Math.max(0, Math.min(audio.duration, seconds)));
    setPosition(audio.position);
  };

  const loadFile = useCallback(async (file: File) => {
    const current = ++epoch.current;
    engine.current?.close(); engine.current = null;
    player.current?.close(); player.current = null;
    accepted.current = null; excerpt.current = null;
    setTrack(null); setResult(null); setPlaying(false); setPosition(0); setStage(null); setFileName(file.name);
    setError(null); setLoading(true); setUpdating(false);
    trackTryIt("load_start", { format: file.name.split(".").pop()?.toLowerCase() ?? "" });
    const started = performance.now();
    try {
      const { audio: decoded, fileRate, decodedRate } = await decodeAtSourceRate(await file.arrayBuffer());
      if (current !== epoch.current) return;
      if (decoded.numberOfChannels > 2) throw new Error("Please choose a mono or stereo mix.");
      if (decoded.duration < 0.5) throw new Error("Please choose at least half a second of audio.");
      const audio = new ComparisonPlayer(playbackContext(decodedRate));
      player.current = audio;
      const memory = (navigator as Navigator & { deviceMemory?: number }).deviceMemory;
      const preview = new PreviewEngine(
        () => new Worker(new URL("./engine.worker.ts", import.meta.url), { type: "module" }),
        { cacheBytes: memory !== undefined && memory <= 4 ? 48e6 : 128e6 },
        next => { if (current === epoch.current) setStage(next); },
      );
      engine.current = preview;
      const loaded = await preview.load(Array.from({ length: decoded.numberOfChannels }, (_, c) => decoded.getChannelData(c).slice()), decodedRate);
      if (current !== epoch.current) return;
      setTrack({ name: file.name, duration: loaded.duration, peaks: loaded.peaks, sampleRate: decodedRate, fileRate, channels: decoded.numberOfChannels, analysis: loaded.analysis, workers: preview.workerCount });
      setMorphing(motion.current);
      continuous.current = false;
      setStart(loaded.start);
      trackTryIt("loaded", { format: file.name.split(".").pop()?.toLowerCase() ?? "", length: minutesBucket(loaded.duration), rate: decodedRate, channels: decoded.numberOfChannels, analysis_s: Math.round(loaded.seconds * 10) / 10, total_s: Math.round((performance.now() - started) / 100) / 10, workers: preview.workerCount });
    } catch (reason) {
      if (current === epoch.current) {
        engine.current?.close(); engine.current = null;
        player.current?.close(); player.current = null;
        const message = reason instanceof Error && reason.message ? reason.message : "Couldn't read that file. Try a WAV, MP3, FLAC or M4A.";
        setError(message);
        trackTryIt("load_error", { message: message.slice(0, 80) });
      }
    } finally { if (current === epoch.current) { setLoading(false); setStage(null); } }
  }, []);

  useEffect(() => {
    if (!track || !engine.current || !player.current) return;
    const preview = engine.current;
    const audio = player.current;
    const current = epoch.current;
    let obsolete = false;
    preview.invalidate();
    setUpdating(true);
    setError(null);
    const delay = continuous.current ? 120 : 0;
    continuous.current = false;
    const timer = setTimeout(async () => {
      try {
        const data = await preview.setExcerpt(start);
        if (obsolete || current !== epoch.current) return;
        if (excerpt.current?.data.startFrame !== data.startFrame) {
          excerpt.current = { data, buffer: auditionBuffer(audio.context, data.original, data.channels, data.sampleRate) };
          data.original = new Float32Array(0); // the AudioBuffer owns the playback copy now
        }
        const settings: RenderSettings = { start: data.start, style: activeStyle.preset.kind, intensity, target };
        const reply = await preview.render(settings);
        if (!reply || obsolete || current !== epoch.current) return;
        const reset = !accepted.current || accepted.current.settings.start !== reply.settings.start;
        audio.setBuffers(excerpt.current.buffer, auditionBuffer(audio.context, reply.mastered, reply.channels, reply.sampleRate), reset,
          selection.current.side, level(data.source, reply.output, selection.current.match));
        accepted.current = { settings: reply.settings, output: reply.output, source: data.source };
        setResult(accepted.current); setUpdating(false); setPosition(audio.position);
        preview.prefetch(neighbours(settings, STYLE_KINDS, TARGETS));
      } catch (reason) {
        if (!obsolete && current === epoch.current) { setError(reason instanceof Error ? reason.message : String(reason)); setUpdating(false); }
      }
    }, delay);
    return () => { obsolete = true; clearTimeout(timer); preview.invalidate(); };
  }, [track, start, activeStyle, intensity, target, retry]);

  const togglePlay = useCallback(async () => {
    const audio = player.current;
    if (!audio || !accepted.current) return;
    if (audio.playing) { audio.pause(); setPosition(audio.position); setPlaying(false); trackTryIt("pause"); }
    else {
      try { await audio.play(); if (audio === player.current) setPlaying(audio.playing); trackTryIt("play"); }
      catch { setError("Playback couldn't start. Press Play to try again."); }
    }
  }, []);
  useEffect(() => { player.current?.select(side, level(result?.source ?? null, result?.output ?? null, match)); }, [side, match, result]);
  useEffect(() => {
    // Presentation only, like the app: the orb's particles fly into the real
    // waveform for one short window; any interaction cuts it early.
    if (!morphing) return;
    const cut = () => setMorphing(false);
    const timer = setTimeout(cut, MORPH_MS);
    window.addEventListener("pointerdown", cut);
    window.addEventListener("keydown", cut);
    return () => { clearTimeout(timer); window.removeEventListener("pointerdown", cut); window.removeEventListener("keydown", cut); };
  }, [morphing]);
  const morphPeaks = React.useMemo(() => (track && morphing ? Array.from(track.peaks) : null), [track, morphing]);
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
    trackTryIt("open");
    const previous = document.activeElement as HTMLElement | null;
    const site = document.querySelector<HTMLElement>(".studio-site");
    const wasInert = site?.inert ?? false;
    if (site) site.inert = true;
    closeButton.current?.focus();
    document.body.classList.add("tryit-lock");
    return () => {
      epoch.current++;
      engine.current?.close(); player.current?.close();
      document.body.classList.remove("tryit-lock");
      if (site) site.inert = wasInert;
      previous?.focus();
    };
  }, []);
  useEffect(() => {
    const onKey = (event: KeyboardEvent) => {
      if (event.key === "Escape") { onClose(); return; }
      if (event.key === "Tab") {
        const elements = Array.from(panel.current?.querySelectorAll<HTMLElement>('button:not(:disabled), a[href], input:not(:disabled), canvas[tabindex], [tabindex="0"]') ?? [])
          .filter(el => el.getClientRects().length > 0);
        const first = elements[0], last = elements[elements.length - 1];
        if (event.shiftKey && document.activeElement === first) { event.preventDefault(); last?.focus(); }
        else if (!event.shiftKey && document.activeElement === last) { event.preventDefault(); first?.focus(); }
        return;
      }
      const element = event.target instanceof HTMLElement ? event.target : null;
      if (event.altKey || event.ctrlKey || event.metaKey || element?.closest("input, textarea, select, [contenteditable=true]")) return;
      if (event.repeat && (event.code === "Space" || event.key.toLowerCase() === "a")) return;
      if (!accepted.current) return;
      if (event.code === "Space") { event.preventDefault(); void togglePlay(); }
      else if (event.key.toLowerCase() === "a") { event.preventDefault(); setSide(s => s === "original" ? "mastered" : "original"); trackTryIt("ab", { via: "key" }); }
      else if (event.key === "ArrowLeft" || event.key === "ArrowRight") {
        event.preventDefault();
        const step = (event.key === "ArrowLeft" ? -1 : 1) * SEEK_STEP;
        if (event.shiftKey) moveStart(start + step);
        else seekTo((player.current?.position ?? 0) + step);
      } else if (event.key === "Home") { event.preventDefault(); seekTo(0); }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [onClose, togglePlay, start, track]);

  const ringR = 47, ringC = 2 * Math.PI * ringR;
  const progress = duration > 0 ? Math.min(1, position / duration) : 0;
  return createPortal(<>
    <div className="tryit-scrim" aria-hidden="true" />
    <div className="tryit" role="dialog" aria-modal="true" aria-label="Try YES Master on your mix" onClick={event => { if (event.target === event.currentTarget) onClose(); }}>
      <div className="tryit-card" ref={panel}>
        <header className="tryit-head">
          <div className="tryit-brand">YES Master <span>· Try it on your mix</span></div>
          <button ref={closeButton} type="button" className="tryit-close" onClick={onClose} aria-label="Close">×</button>
        </header>
        <input ref={fileInput} type="file" hidden accept="audio/*,.wav,.mp3,.flac,.m4a,.aac,.ogg,.aif,.aiff" onChange={event => {
          const file = event.target.files?.[0]; event.target.value = ""; if (file) void loadFile(file);
        }} />
        {!track && !loading ? (
          <button type="button" className={"tryit-drop" + (over ? " is-over" : "")}
            onClick={() => fileInput.current?.click()}
            onDragOver={event => { event.preventDefault(); setOver(true); }} onDragLeave={() => setOver(false)}
            onDrop={event => { event.preventDefault(); setOver(false); const file = event.dataTransfer.files[0]; if (file) void loadFile(file); }}>
            <span className="tryit-glyph" aria-hidden="true">↓</span>
            <strong>Drop your mix here</strong>
            <span>WAV, AIFF, FLAC, MP3 or M4A</span>
          </button>
        ) : !track ? (
          <div className="tryit-file"><span>Preparing</span><b title={fileName}>{fileName}</b></div>
        ) : (
          <div className="tryit-file">
            <b title={track.name}>{track.name}</b>
            <span className="tryit-chip">Analyzed</span>
            <Info label="About the analysis">
              Before mastering, YES Master reads your <b>whole track</b> — loudness, dynamics, stereo field and tonal balance — and adapts each style to it, the same way the desktop app does.
              {track.analysis.digest && <> Your mix: <b>{track.analysis.digest}</b>.</>}
              {" "}Processed at <b>{kHz(track.sampleRate)}</b>{track.fileRate === track.sampleRate ? ", your file's own rate" : ""}, on {track.workers} {track.workers === 1 ? "thread" : "threads"}.
            </Info>
            <button type="button" className="tryit-link" onClick={() => fileInput.current?.click()}>change</button>
          </div>
        )}

        <div className="tryit-stage">
          <button type="button" className={"tryit-ring" + (playing ? " is-playing" : "") + (updating ? " is-updating" : "")} disabled={!ready} onClick={() => void togglePlay()} aria-keyshortcuts="Space">
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
              <button type="button" disabled={!ready} aria-pressed={side === "original"} onClick={() => { setSide("original"); trackTryIt("ab", { side: "original" }); }}>Original</button>
              <button type="button" disabled={!ready} aria-pressed={side === "mastered"} onClick={() => { setSide("mastered"); trackTryIt("ab", { side: "mastered" }); }}>Mastered</button>
            </div>
            <label className="tryit-check"><input type="checkbox" checked={match} disabled={!ready} onChange={event => { setMatch(event.target.checked); trackTryIt("match", { on: event.target.checked }); }} /><span>Volume match</span>
              <Info label="About volume match">Use volume match to preview your original and master at the same volume and compare with more accuracy. Both play at one fixed level, so changing the loudness target changes the sound, not the volume.</Info>
            </label>
          </div>
        </div>

        {loading && !track && <Preparing stage={stage} motion={motion.current} />}
        {track && (
          <div className={"tryit-strip" + (morphing ? " is-morphing" : "")}>
            <div className="tryit-strip-wave">
            <canvas ref={wave} width={1000} height={72} role="img" tabIndex={0}
              aria-label="Track waveform. Drag the lit window to choose your thirty seconds; click inside it to move the playhead. Arrow keys nudge the window."
              onKeyDown={event => {
                if (event.key === "ArrowLeft" || event.key === "ArrowRight") { event.preventDefault(); event.stopPropagation(); moveStart(start + (event.key === "ArrowLeft" ? -SEEK_STEP : SEEK_STEP)); }
                else if (event.key === "Home") { event.preventDefault(); event.stopPropagation(); moveStart(0); }
                else if (event.key === "End") { event.preventDefault(); event.stopPropagation(); moveStart(track.duration); }
              }}
              onPointerDown={event => {
                const x = stripSeconds(event);
                const inside = x >= start && x <= start + CLIP_SECONDS;
                drag.current = { on: true, grab: inside ? x - start : CLIP_SECONDS / 2, x0: event.clientX, moved: false, inside };
                event.currentTarget.setPointerCapture(event.pointerId);
                event.currentTarget.focus({ preventScroll: true });
                if (!inside) moveStart(x - drag.current.grab);
              }}
              onPointerMove={event => {
                if (!drag.current.on) return;
                if (Math.abs(event.clientX - drag.current.x0) > 3) drag.current.moved = true;
                if (drag.current.moved) moveStart(stripSeconds(event) - drag.current.grab);
              }}
              onPointerUp={event => {
                const d = drag.current; drag.current = { ...d, on: false };
                if (!d.on) return;
                if (d.moved || !d.inside) trackTryIt("section");
                else seekTo(stripSeconds(event) - start);
              }}
              onPointerCancel={() => { drag.current.on = false; }} />
            {morphPeaks && <AnalysisOrb phase="morph" peaks={morphPeaks} />}
            </div>
            <div className="tryit-strip-cap"><span>{fmt(start)} – {fmt(Math.min(track.duration, start + CLIP_SECONDS))}</span><span>drag to pick your 30 s · click inside to seek <kbd>←</kbd><kbd>→</kbd></span></div>
          </div>
        )}

        <div className="tryit-rows">
          <div className="tryit-row">
            <div className="tryit-row-label">Style <Info label="About styles">Choose between 4 mastering styles, each with its own character and feel. <b>Universal:</b> balanced, keeps the focus on your mix. <b>Clarity:</b> open and defined on top, focused below. <b>Tape:</b> warmer weight with a softer edge. <b>Oomph:</b> low-end weight and forward energy.</Info></div>
            <div className="std-tiles" role="group" aria-label="Style">{STANDARD_STYLES.map(s => <button key={s.id} type="button" disabled={!track} className={"std-tile" + (style === s.id ? " is-active" : "")} style={{ ["--tile-accent" as string]: PRESET_ACCENT[s.preset.kind] }} aria-pressed={style === s.id} onClick={() => { setStyle(s.id); trackTryIt("style", { style: s.preset.kind }); }}><span className="std-tile-icon"><PresetIcon kind={s.preset.kind} /></span><span className="std-tile-label">{s.label}</span></button>)}</div>
          </div>
          <div className="tryit-row">
            <div className="tryit-row-label">Intensity <Info label="About intensity">Set how strongly the style comes through, from a light touch to a stronger character. You hear it change while the track plays.</Info></div>
            <div className="tryit-slider"><input type="range" min={0} max={100} disabled={!track} value={Math.round(intensity * 100)} aria-label="Intensity" onChange={event => { continuous.current = true; setIntensity(Number(event.target.value) / 100); }} onPointerUp={() => trackTryIt("intensity", { value: Math.round(intensity * 100) })} style={{ ["--pct" as string]: `${Math.round(intensity * 100)}%`, ["--tone" as string]: PRESET_ACCENT[activeStyle.preset.kind] }} /><output>{Math.round(intensity * 100)}%</output></div>
          </div>
          <div className="tryit-row">
            <div className="tryit-row-label">Loudness <Info label="About loudness">Set your master’s loudness level. <b>Low</b> (−14 LUFS) matches streaming platforms. <b>Medium</b> (−11) is a modern, competitive level. <b>High</b> (−9) is hot and dense. A −1 dBTP ceiling keeps every level clean.</Info></div>
            <div className="tryit-pills" role="group" aria-label="Loudness">{STANDARD_LOUDNESS.map(l => <button key={l.id} type="button" disabled={!track} className={l.lufs === target ? "is-active" : ""} aria-pressed={l.lufs === target} onClick={() => { setTarget(l.lufs); trackTryIt("loudness", { target: l.lufs }); }}>{l.label}</button>)}</div>
          </div>
        </div>

        {error && <div role="alert" className="tryit-error">{error} {track && <button type="button" className="tryit-link" onClick={() => setRetry(n => n + 1)}>Retry</button>}</div>}

        <a href="#get-started" className="tryit-cta" onClick={() => { trackTryIt("cta"); onClose(); }}>Get the free beta</a>
        <p className="tryit-cta-sub">Export and Advanced live in the desktop app.</p>
        <p className="tryit-privacy"><i />Processed locally in your browser. Your track is never uploaded.</p>
      </div>
    </div>
  </>, document.body);
}
