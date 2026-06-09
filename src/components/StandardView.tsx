// src/components/StandardView.tsx
//
// Standard view — the default desktop face. Phase 2 layout: the 3-column
// Advanced shell (Tracks rail · center controls · Preview/Delivery/Export
// rail) so the flip to Advanced is seamless. Style tiles reuse the Advanced
// preset artwork (PresetIcon) so Balanced/Bright/Warm/Heavy read as the same
// Universal/Clarity/Tape/Oomph presets in Advanced. Intensity is the shared
// Knob. One transport (no duplicates). Binds to per-track selectedSettings.

import type { Preset } from "../bindings";
import {
  STANDARD_LOUDNESS,
  STANDARD_STYLES,
  loudnessToTarget,
  presetToStyle,
  styleToPreset,
  targetToLoudness,
} from "../lib/standard-mapping";
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

function StandardRightRail({
  tm,
  onEnterAdvanced,
}: {
  tm: TM;
  onEnterAdvanced: () => void;
}) {
  const notes = tm.lastExportReceipt
    ? standardExportNotes(tm.lastExportReceipt.checks)
    : null;
  return (
    <aside className="std-rail">
      <section className="std-rail-card">
        <div className="std-rail-title">PREVIEW</div>
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
        <button
          type="button"
          className={`toolbar-toggle std-volume-match ${tm.transport.volumeMatch ? "is-on" : ""}`}
          aria-pressed={tm.transport.volumeMatch}
          onClick={() => tm.setVolumeMatch(!tm.transport.volumeMatch)}
        >
          <span className="toolbar-toggle-box" aria-hidden />
          <span>Volume Match</span>
        </button>
      </section>

      <section className="std-rail-card">
        <div className="std-rail-title">DELIVERY FORMAT</div>
        {/* State-free name: the recipe is fixed (standardExportSettings), and
            "Streaming" would read as a live profile next to a −9 LUFS target. */}
        <div className="std-delivery-name">Standard WAV</div>
        <div className="std-delivery-spec">44.1 kHz · 24-bit · −1 dBTP</div>
        <button type="button" className="ghost-btn std-delivery-change" onClick={onEnterAdvanced}>
          Change
        </button>
      </section>

      <div className="std-rail-export">
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
  return (
    <div className="standard-view">
      <TracksRail tm={tm} />

      <section className="std-center">
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

          <div className="std-step std-step-intensity">
            <span className="std-step-label">2 · Intensity</span>
            <span className="std-step-hint">Set how strong the effect is.</span>
            <Knob
              label=""
              size="lg"
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

          <div className="std-step">
            <span className="std-step-label">3 · Loudness</span>
            <span className="std-step-hint">Choose your target loudness.</span>
            <LoudnessSegmented
              targetLufs={effectiveLoudnessTarget(s)}
              onSelect={tm.setLoudnessTarget}
            />
          </div>
        </div>
      </section>

      <StandardRightRail tm={tm} onEnterAdvanced={onEnterAdvanced} />
    </div>
  );
}
