// src/components/StandardView.tsx
//
// Standard view — the default desktop face. Phase 2 layout: the 3-column
// Advanced shell (Tracks rail · center controls · Preview/Delivery/Export
// rail) so the flip to Advanced is seamless. Style tiles reuse the Advanced
// preset artwork (PresetIcon) so Balanced/Bright/Warm/Heavy read as the same
// Universal/Clarity/Tape/Oomph presets in Advanced. Intensity is the shared
// Knob. One transport (no duplicates). Binds to per-track selectedSettings.

import { useLayoutEffect, useRef, type RefObject } from "react";
import type { Preset } from "../bindings";
import {
  STANDARD_LOUDNESS,
  STANDARD_STYLES,
  loudnessToTarget,
  presetToStyle,
  styleToPreset,
  targetToLoudness,
} from "../lib/standard-mapping";
import { computeRailAlignment } from "../lib/rail-alignment";
import type { useTrackMaster } from "../hooks/useTrackMaster";
import { Knob, intensityLabel } from "./Knob";
import { WaveformLoading, WaveformView } from "./Waveform";
import { PresetIcon, PRESET_ACCENT } from "./PresetIcon";
import { effectiveLoudnessTarget } from "../lib/effective-settings";
import { standardExportNotes } from "../lib/standard-export";
import { MasterOutPanel } from "./RightRail";

type TM = ReturnType<typeof useTrackMaster>;

function fmtDuration(sec: number | null | undefined): string {
  if (sec == null || !Number.isFinite(sec)) return "";
  const m = Math.floor(sec / 60);
  const r = Math.floor(sec % 60);
  return `${m}:${r.toString().padStart(2, "0")}`;
}

/// Truthful hero subtitle: the qualifier is derived from the measurement
/// against the user's selected loudness target (±1.5 LU reads as "close"
/// at mastering tolerances) — never asserted unconditionally.
export function sourceLufsCopy(
  sourceLufs: number,
  targetLufs: number | null,
): string {
  const measured = `Source ${sourceLufs.toFixed(1)} LUFS`;
  if (targetLufs == null || !Number.isFinite(targetLufs)) return measured;
  const delta = sourceLufs - targetLufs;
  const target = `your ${targetLufs.toFixed(0)} LUFS target`;
  if (Math.abs(delta) <= 1.5) return `${measured} · close to ${target}`;
  return delta < 0
    ? `${measured} · quieter than ${target}`
    : `${measured} · louder than ${target}`;
}

/// Quick-set intensity zones. Labels are exactly Knob's `intensityLabel`
/// vocabulary, and each value lands inside its own zone band, so the active
/// chip doubles as a zone indicator for any knob position.
export const INTENSITY_ZONES: ReadonlyArray<{ label: string; value: number }> = [
  { label: "Subtle", value: 0.1 },
  { label: "Restrained", value: 0.3 },
  { label: "Moderate", value: 0.5 },
  { label: "Driving", value: 0.8 },
  { label: "Aggressive", value: 0.95 },
];

export function StyleTiles({
  preset,
  onSelect,
}: {
  preset: Preset;
  onSelect: (preset: Preset) => void;
}) {
  const activeStyle = presetToStyle(preset);
  return (
    <div className="std-tiles" role="group" aria-label="Style">
      {STANDARD_STYLES.map((s) => {
        const tilePreset = styleToPreset(s.id);
        return (
          <button
            key={s.id}
            type="button"
            className={"std-tile" + (s.id === activeStyle ? " is-active" : "")}
            // Same accent variable + hue as Advanced's preset strip, so one
            // preset glows one color across both views.
            style={{ ["--tile-accent" as never]: PRESET_ACCENT[tilePreset.kind] }}
            aria-pressed={s.id === activeStyle}
            onClick={() => onSelect(tilePreset)}
          >
            <span className="std-tile-icon">
              <PresetIcon kind={tilePreset.kind} />
            </span>
            <span className="std-tile-label">{s.label}</span>
            <span className="std-tile-subtitle">{s.subtitle}</span>
          </button>
        );
      })}
    </div>
  );
}

export function LoudnessSegmented({
  targetLufs,
  onSelect,
}: {
  targetLufs: number | null;
  onSelect: (targetLufs: number) => void;
}) {
  const active = targetToLoudness(targetLufs);
  return (
    <div className="std-seg" role="group" aria-label="Loudness">
      {STANDARD_LOUDNESS.map((l) => (
        <button
          key={l.id}
          type="button"
          className={"std-seg-option" + (l.id === active ? " is-active" : "")}
          aria-pressed={l.id === active}
          onClick={() => onSelect(loudnessToTarget(l.id))}
        >
          <span className="std-seg-label">{l.label}</span>
          <span className="std-seg-lufs">{l.lufs} LUFS</span>
        </button>
      ))}
    </div>
  );
}

function TracksRail({ tm }: { tm: TM }) {
  return (
    <aside className="std-tracks">
      <div className="std-tracks-head">TRACKS</div>
      <div className="std-tracks-list">
        {tm.tracks.map((t, index) => (
          <button
            key={t.id}
            type="button"
            className={
              "std-track-row" + (t.id === tm.selectedTrackId ? " is-active" : "")
            }
            onClick={() => tm.selectTrack(t.id)}
          >
            <span className="std-track-index" aria-hidden>
              {(index + 1).toString().padStart(2, "0")}
            </span>
            <span className="std-track-copy">
              <span className="std-track-name">{t.display_name}</span>
              <span className="std-track-dur">{fmtDuration(t.duration_seconds)}</span>
            </span>
            <span className="std-track-meter" aria-hidden />
          </button>
        ))}
      </div>
      <button
        type="button"
        className="ghost-btn std-add-track"
        onClick={tm.openImportDialog}
      >
        + Add Tracks
      </button>
      <div
        className={
          "std-tracks-status" +
          (!tm.isAnalyzing && !tm.selectedAnalysis ? " is-idle" : "")
        }
      >
        <span className="std-status-dot" aria-hidden />
        {tm.isAnalyzing
          ? "Analyzing…"
          : tm.selectedAnalysis
            ? "Analyzed"
            : "Not analyzed"}
      </div>
    </aside>
  );
}

/// Refs the seam-alignment hook needs from both columns. The rail and
/// center are separate components, so StandardView owns the refs and
/// threads the rail's subset down as a prop.
type RailSeamRefs = {
  center: RefObject<HTMLElement | null>;
  intensity: RefObject<HTMLDivElement | null>;
  loudness: RefObject<HTMLDivElement | null>;
  rail: RefObject<HTMLElement | null>;
  preview: RefObject<HTMLElement | null>;
  delivery: RefObject<HTMLElement | null>;
  exportGroup: RefObject<HTMLDivElement | null>;
};

/// Seam alignment (2026-06-09): mirror the center column's card seams in
/// the rail — Preview bottom flush with the Intensity card, export group
/// flush with Loudness — by measuring both columns and setting two pixel
/// vars consumed by `.std-rail.is-aligned` in App.css. The decision logic
/// lives in lib/rail-alignment.ts (pure, unit-tested); this hook is only
/// the measurement glue. No ResizeObserver (node/jsdom) → no-op, which
/// leaves the flex-absorb fallback layout.
function useRailSeamAlignment(refs: RailSeamRefs) {
  useLayoutEffect(() => {
    if (typeof ResizeObserver === "undefined") return;
    let frame = 0;

    const apply = () => {
      frame = 0;
      const rail = refs.rail.current;
      const center = refs.center.current;
      const intensity = refs.intensity.current;
      const loudness = refs.loudness.current;
      const preview = refs.preview.current;
      const delivery = refs.delivery.current;
      const exportGroup = refs.exportGroup.current;
      if (
        !rail ||
        !center ||
        !intensity ||
        !loudness ||
        !preview ||
        !delivery ||
        !exportGroup
      ) {
        return;
      }
      const railRect = rail.getBoundingClientRect();
      const railStyle = getComputedStyle(rail);
      const result = computeRailAlignment({
        previewTop: preview.getBoundingClientRect().top,
        intensityBottom: intensity.getBoundingClientRect().bottom,
        loudnessBottom: loudness.getBoundingClientRect().bottom,
        railContentBottom:
          railRect.bottom - (parseFloat(railStyle.paddingBottom) || 0),
        deliveryHeight: delivery.getBoundingClientRect().height,
        exportHeight: exportGroup.getBoundingClientRect().height,
        railGap: parseFloat(railStyle.rowGap || railStyle.gap) || 16,
        centerScrolls: center.scrollHeight > center.clientHeight + 1,
      });
      if (result) {
        rail.style.setProperty("--std-preview-h", `${result.previewHeightPx}px`);
        rail.style.setProperty(
          "--std-export-mb",
          `${result.exportMarginBottomPx}px`,
        );
        rail.classList.add("is-aligned");
      } else {
        rail.classList.remove("is-aligned");
        rail.style.removeProperty("--std-preview-h");
        rail.style.removeProperty("--std-export-mb");
      }
    };

    // Coalesce observer bursts into one measurement per frame. The Preview
    // card is deliberately NOT observed — apply() sets its height, so
    // observing it would loop the observer.
    const schedule = () => {
      if (!frame) frame = requestAnimationFrame(apply);
    };
    const observer = new ResizeObserver(schedule);
    for (const ref of [
      refs.rail,
      refs.center,
      refs.intensity,
      refs.loudness,
      refs.delivery,
      refs.exportGroup,
    ]) {
      if (ref.current) observer.observe(ref.current);
    }
    schedule();

    return () => {
      observer.disconnect();
      if (frame) cancelAnimationFrame(frame);
      const rail = refs.rail.current;
      if (rail) {
        rail.classList.remove("is-aligned");
        rail.style.removeProperty("--std-preview-h");
        rail.style.removeProperty("--std-export-mb");
      }
    };
    // Refs are stable for the lifetime of the view; mount-only is intended.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);
}

function StandardRightRail({
  tm,
  onEnterAdvanced,
  seamRefs,
}: {
  tm: TM;
  onEnterAdvanced: () => void;
  seamRefs: RailSeamRefs;
}) {
  const notes = tm.lastExportReceipt
    ? standardExportNotes(tm.lastExportReceipt.checks)
    : null;
  return (
    <aside className="std-rail" ref={seamRefs.rail}>
      <section className="std-rail-card std-rail-preview" ref={seamRefs.preview}>
        <div className="std-rail-head">
          <div className="std-rail-title">PREVIEW</div>
          <button
            type="button"
            className={`toolbar-toggle std-volume-match ${tm.transport.volumeMatch ? "is-on" : ""}`}
            aria-pressed={tm.transport.volumeMatch}
            onClick={() => tm.setVolumeMatch(!tm.transport.volumeMatch)}
          >
            <span className="toolbar-toggle-box" aria-hidden />
            <span>Volume Match</span>
          </button>
        </div>
        <div className="std-rail-ab" role="group" aria-label="Playback source">
          <button
            type="button"
            aria-pressed={tm.transport.playbackKind === "source"}
            className={tm.transport.playbackKind === "source" ? "on" : ""}
            onClick={() => tm.setPlaybackKind("source")}
          >
            Original
          </button>
          <button
            type="button"
            aria-pressed={tm.transport.playbackKind === "master"}
            className={tm.transport.playbackKind === "master" ? "on" : ""}
            onClick={() => tm.setPlaybackKind("master")}
          >
            Mastered
          </button>
        </div>
        <MasterOutPanel
          isAnalyzing={tm.isAnalyzing}
          peakDbfs={tm.transport.peakDbfs}
          peakLeftDbfs={tm.transport.peakLeftDbfs}
          peakRightDbfs={tm.transport.peakRightDbfs}
          isPlaying={tm.transport.isPlaying}
          lufsMomentary={tm.transport.lufsMomentary}
          lufsIntegrated={tm.transport.lufsIntegrated}
        />
      </section>

      <section className="std-rail-card" ref={seamRefs.delivery}>
        <div className="std-rail-title">DELIVERY FORMAT</div>
        {/* State-free name: the recipe is fixed (standardExportSettings), and
            "Streaming" would read as a live profile next to a −9 LUFS target. */}
        <div className="std-delivery-name">Standard WAV</div>
        <div className="std-delivery-spec">44.1 kHz · 24-bit · −1 dBTP</div>
        <button type="button" className="ghost-btn std-delivery-change" onClick={onEnterAdvanced}>
          Change
        </button>
      </section>

      <div className="std-rail-export" ref={seamRefs.exportGroup}>
        <button
          type="button"
          className="primary std-create-master"
          disabled={!tm.selectedAnalysis || tm.isExporting || tm.isRendering}
          onClick={() => {
            void tm.exportStandardMaster();
          }}
        >
          {tm.isExporting
            ? "Creating…"
            : tm.isRendering
              ? "Preparing…"
              : "Create Master"}
        </button>
        {notes?.invalid && (
          <p className="std-export-block" role="alert">
            Saved, but this master has a problem — re-render: {notes.invalidMessage}
          </p>
        )}
        {notes?.integrityNote && (
          <p className="std-export-note">{notes.integrityNote}</p>
        )}
      </div>
    </aside>
  );
}

export function StandardView({
  tm,
  onEnterAdvanced,
}: {
  tm: TM;
  // Required: a missed wiring must fail tsc, not ship a dead 'Change' button.
  onEnterAdvanced: () => void;
}) {
  const s = tm.selectedSettings;
  const sourceLufs = tm.selectedAnalysis?.lufs_integrated ?? null;
  // Knob arc follows the active style's tone (Knob's red/gold/cyan are the
  // same hexes as PRESET_ACCENT's oomph/tape/clarity); "blue" when the
  // current preset isn't one of the reference-4.
  const activeTone =
    STANDARD_STYLES.find((st) => st.id === presetToStyle(s.preset))?.tone ??
    "blue";
  const seamRefs: RailSeamRefs = {
    center: useRef<HTMLElement | null>(null),
    intensity: useRef<HTMLDivElement | null>(null),
    loudness: useRef<HTMLDivElement | null>(null),
    rail: useRef<HTMLElement | null>(null),
    preview: useRef<HTMLElement | null>(null),
    delivery: useRef<HTMLElement | null>(null),
    exportGroup: useRef<HTMLDivElement | null>(null),
  };
  useRailSeamAlignment(seamRefs);
  return (
    <div className="standard-view">
      <TracksRail tm={tm} />

      <section className="std-center" ref={seamRefs.center}>
        <div className="std-hero-head">
          <h1 className="std-title">{tm.selectedTrack?.display_name ?? "No track"}</h1>
          <p className="std-source">
            {sourceLufs != null
              ? sourceLufsCopy(sourceLufs, effectiveLoudnessTarget(s))
              : tm.isAnalyzing
                ? "Analyzing source…"
                : "Source not analyzed yet"}
          </p>
        </div>

        <div className="std-wave-deck">
          <div className="std-play-stack">
            <button
              type="button"
              className="std-play"
              aria-label={tm.transport.isPlaying ? "Pause" : "Play"}
              onClick={tm.togglePlay}
            >
              <span
                className={
                  "std-play-glyph" + (tm.transport.isPlaying ? " is-pause" : "")
                }
                aria-hidden
              >
                {tm.transport.isPlaying ? "❚❚" : "►"}
              </span>
            </button>
            <span className="std-time">
              {fmtDuration(tm.transport.currentTimeSec)} / {fmtDuration(tm.selectedTrack?.duration_seconds)}
            </span>
          </div>
          <div className="std-wave">
            {tm.selectedWaveform ? (
              <WaveformView
                peaks={tm.selectedWaveform}
                isLoading={tm.isLoadingWaveform}
                isAnalyzing={tm.isAnalyzing}
                analysisProgress={tm.analysisProgress}
                currentTimeSec={tm.transport.currentTimeSec}
                durationSec={tm.selectedTrack?.duration_seconds ?? 0}
                region={tm.selectedRegion}
                onSeek={tm.seek}
                onSetRegion={tm.setRegion}
                onClearRegion={tm.clearRegion}
              />
            ) : (
              <WaveformLoading
                isAnalyzing={tm.isAnalyzing}
                isLoadingWaveform={tm.isLoadingWaveform}
                analysisProgress={tm.analysisProgress}
              />
            )}
          </div>
        </div>

        <div className="std-steps">
          <div className="std-step">
            <span className="std-step-label">1 · Style</span>
            <span className="std-step-hint">Choose the character you want.</span>
            <StyleTiles preset={s.preset} onSelect={tm.setPreset} />
          </div>

          <div
            className="std-step std-step-intensity"
            ref={seamRefs.intensity}
            // The active style's accent tints the zone chips and matches the
            // knob arc tone — intensity visibly belongs to the chosen style.
            style={{ ["--tile-accent" as never]: PRESET_ACCENT[s.preset.kind] }}
          >
            <span className="std-step-label">2 · Intensity</span>
            <span className="std-step-hint">Set how strong the effect is.</span>
            <div className="std-zone-chips" role="group" aria-label="Intensity presets">
              {INTENSITY_ZONES.map((z) => (
                <button
                  key={z.label}
                  type="button"
                  className={
                    "std-zone-chip" +
                    (intensityLabel(s.intensity) === z.label ? " is-active" : "")
                  }
                  aria-pressed={intensityLabel(s.intensity) === z.label}
                  onClick={() => tm.setIntensity(z.value)}
                >
                  {z.label}
                </button>
              ))}
            </div>
            <Knob
              label=""
              size="lg"
              tone={activeTone}
              value={s.intensity}
              min={0}
              max={1}
              step={0.01}
              defaultValue={0.5}
              format={(v) => `${Math.round(v * 100)}%`}
              caption={intensityLabel(s.intensity)}
              onChange={tm.setIntensity}
              centerValue
            />
          </div>

          <div className="std-step" ref={seamRefs.loudness}>
            <span className="std-step-label">3 · Loudness</span>
            <span className="std-step-hint">Choose your target loudness.</span>
            <LoudnessSegmented
              targetLufs={effectiveLoudnessTarget(s)}
              onSelect={tm.setLoudnessTarget}
            />
          </div>
        </div>
      </section>

      <StandardRightRail
        tm={tm}
        onEnterAdvanced={onEnterAdvanced}
        seamRefs={seamRefs}
      />
    </div>
  );
}
