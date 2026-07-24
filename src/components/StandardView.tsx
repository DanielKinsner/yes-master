// src/components/StandardView.tsx
//
// Standard view — the default desktop face. Phase 2 layout: the 3-column
// Advanced shell (Tracks rail · center controls · Preview/Delivery/Export
// rail) so the flip to Advanced is seamless. Style tiles reuse the Advanced
// preset artwork (PresetIcon) and now share its names — Universal/Clarity/
// Tape/Oomph read identically across both views. Intensity is the shared
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
import { WaveformView } from "./Waveform";
import { PresetIcon, PRESET_ACCENT } from "./PresetIcon";
import { effectiveLoudnessTarget } from "../lib/effective-settings";
import { standardDeliverySpecLabel, standardExportNotes } from "../lib/standard-export";
import { api } from "../lib/api";
import { MasterOutPanel } from "./RightRail";
import { DisabledReason } from "./fields";
import { formatDuration } from "../lib/time-format";
// L9: the hint copy now lives in FirstRunOverlay, rendered once at the App
// root. The guide STATE still arrives here as a prop so the Mastered A/B can
// pulse and entering Advanced can finish the guide — but the chip no longer
// renders inline in the rails (it used to push their layout).
import type { FirstRunGuide } from "../hooks/useFirstRunGuide";

type TM = ReturnType<typeof useTrackMaster>;

// Inert guide for callers that don't drive the first-run flow (e.g. unit
// tests rendering StandardView in isolation). App always passes the live one.
const NO_GUIDE: FirstRunGuide = {
  step: null,
  dismiss: () => {},
  noteEnteredAdvanced: () => {},
};

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
          // Wrapper div, not a nested button: the remove × must be its own
          // interactive element beside the select row (parity with the
          // Advanced sidebar's remove control).
          <div className="std-track-item" key={t.id}>
            <button
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
                <span className="std-track-dur">{formatDuration(t.duration_seconds)}</span>
              </span>
              <span className="std-track-meter" aria-hidden />
            </button>
            <button
              type="button"
              className="std-track-remove"
              aria-label={`Remove ${t.display_name}`}
              onClick={() => tm.removeTrack(t.id)}
            >
              ×
            </button>
          </div>
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
      {!tm.isAnalyzing && !tm.selectedAnalysis && tm.selectedTrackId && (
        <button
          type="button"
          className="ghost-btn std-reanalyze-track"
          onClick={() => {
            void tm.reanalyzeTrack(tm.selectedTrackId!);
          }}
        >
          Re-analyze
        </button>
      )}
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
/// leaves the flex-absorb fallback layout. If the center column is
/// technically scrollable, scroll events remeasure the current card seams
/// instead of dropping back to the unaligned rail.
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
    refs.center.current?.addEventListener("scroll", schedule, { passive: true });
    schedule();

    return () => {
      observer.disconnect();
      refs.center.current?.removeEventListener("scroll", schedule);
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
  guide,
  canUseMaster,
  onEnterAdvanced,
  seamRefs,
}: {
  tm: TM;
  guide: FirstRunGuide;
  canUseMaster: boolean;
  onEnterAdvanced: () => void;
  seamRefs: RailSeamRefs;
}) {
  // U10 — Create Master is disabled by three separate conditions; before this
  // it explained only one of them, and only on hover. The most common case
  // (no analysis yet) had no explanation at all.
  const createMasterDisabled =
    !tm.selectedAnalysis || tm.isExporting || tm.isRendering;
  const createMasterDisabledReason = tm.isExporting
    ? "An export is already running — it finishes or fails before the next one starts."
    : tm.isRendering
      ? "Unavailable while a render is in progress — they share render state."
      : !tm.selectedAnalysis
        ? "Analyze this track first — Create Master needs the source measurements."
        : undefined;
  const receipt =
    tm.lastExportReceipt?.trackId === tm.selectedTrackId
      ? tm.lastExportReceipt
      : null;
  const notes = receipt ? standardExportNotes(receipt.checks) : null;
  // Success receipt (owner note 2026-06-11: a clean Standard export gave no
  // feedback at all — the receipt only lived in Advanced). Track receipts
  // only; album exports keep their own flow. Invalid renders keep the
  // prominent re-render alert instead of a success card.
  const showDone = receipt != null && notes != null && !notes.invalid;
  const doneLufs = showDone
    ? (receipt.job.measurements?.lufs_integrated ?? null)
    : null;
  const doneFileName = showDone
    ? receipt.outputPath.split(/[\\/]/).pop() ?? receipt.outputPath
    : null;
  const activeRenderProgress =
    (tm.isRendering || tm.isExporting) && tm.renderProgress
      ? {
          percent: Math.round(
            Math.max(0, Math.min(1, tm.renderProgress.fraction)) * 100,
          ),
          label: `${tm.renderProgress.kind[0].toUpperCase()}${tm.renderProgress.kind.slice(
            1,
          )} render`,
        }
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
            title="Aligns playback loudness for fair tone comparison. Export level is unchanged."
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
            disabled={!canUseMaster}
            title={
              !canUseMaster
                ? "Analyze this track before using Mastered playback."
                : undefined
            }
            // U10: the reason was hover-only.
            aria-describedby={!canUseMaster ? "ab-master-disabled-reason" : undefined}
            className={
              (tm.transport.playbackKind === "master" ? "on" : "") +
              (guide.step === "flip" ? " guide-pulse" : "")
            }
            onClick={() => tm.setPlaybackKind("master")}
          >
            Mastered
          </button>
          <DisabledReason
            id="ab-master-disabled-reason"
            reason={
              !canUseMaster
                ? "Analyze this track before using Mastered playback."
                : undefined
            }
          />
        </div>
        <MasterOutPanel
          isAnalyzing={tm.isAnalyzing}
          peakDbfs={tm.transport.peakDbfs}
          peakLeftDbfs={tm.transport.peakLeftDbfs}
          peakRightDbfs={tm.transport.peakRightDbfs}
          isPlaying={tm.transport.isPlaying}
          lufsMomentary={tm.transport.lufsMomentary}
          lufsIntegrated={tm.transport.lufsIntegrated}
          meterMode="standard"
          landingPending={
            tm.landingPending &&
            tm.transport.playbackKind === "master" &&
            tm.transport.isPlaying
          }
        />
      </section>

      <section className="std-rail-card" ref={seamRefs.delivery}>
        <div className="std-rail-title">DELIVERY FORMAT</div>
        {/* State-free name: the recipe is fixed (standardExportSettings), and
            "Streaming" would read as a live profile next to a −9 LUFS target. */}
        <div className="std-delivery-name">Standard WAV</div>
        <div className="std-delivery-spec">{standardDeliverySpecLabel()}</div>
        <div className="std-delivery-note">Create Master writes a WAV file.</div>
        <button type="button" className="ghost-btn std-delivery-change" onClick={onEnterAdvanced}>
          Change
        </button>
      </section>

      <div className="std-rail-export" ref={seamRefs.exportGroup}>
        {/* U10 — Create Master was disabled by THREE conditions but only
            explained one of them, and only on hover. With no analysis yet, the
            app's primary action was dead and silent about why. */}
        <button
          type="button"
          className="primary std-create-master"
          disabled={createMasterDisabled}
          title={createMasterDisabledReason}
          aria-describedby={
            createMasterDisabledReason ? "create-master-disabled-reason" : undefined
          }
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
        <DisabledReason
          id="create-master-disabled-reason"
          reason={createMasterDisabledReason}
        />
        {activeRenderProgress && (
          <div className="std-render-progress" role="status" aria-live="polite">
            <div className="std-render-progress-label">
              <span>
                {activeRenderProgress.label} {activeRenderProgress.percent}%
              </span>
              <button
                type="button"
                className="ghost-btn std-render-cancel"
                disabled={
                  !tm.renderProgress ||
                  tm.cancelRequestedJobId === tm.renderProgress.job_id
                }
                onClick={() => {
                  void tm.cancelActiveRender();
                }}
              >
                {tm.renderProgress &&
                tm.cancelRequestedJobId === tm.renderProgress.job_id
                  ? "Cancelling..."
                  : "Cancel"}
              </button>
            </div>
            <div
              className="std-render-progress-track"
              role="progressbar"
              aria-label={activeRenderProgress.label}
              aria-valuemin={0}
              aria-valuemax={100}
              aria-valuenow={activeRenderProgress.percent}
            >
              <span
                className="std-render-progress-fill"
                style={{ width: `${activeRenderProgress.percent}%` }}
              />
            </div>
          </div>
        )}
        {tm.renderFeedback && (
          <p className="std-export-note" role="status">
            {tm.renderFeedback.message}
          </p>
        )}
        {notes?.invalid && (
          <p className="std-export-block" role="alert">
            Saved, but this master has a problem — re-render: {notes.invalidMessage}
          </p>
        )}
        {notes?.integrityNote && (
          <p className="std-export-note">{notes.integrityNote}</p>
        )}
        {showDone && (
          <div className="std-export-done" role="status">
            <div className="std-export-done-head">
              <span className="std-export-done-check" aria-hidden>
                ✓
              </span>
              Master created
            </div>
            <div className="std-export-done-meta">
              {doneFileName}
              {doneLufs != null && ` · ${doneLufs.toFixed(1)} LUFS`}
            </div>
            <button
              type="button"
              className="ghost-btn std-export-done-open"
              onClick={() => {
                void api.openOutput(receipt.outputPath);
              }}
            >
              Show file
            </button>
            {/* Standard stays lean: the full ExportReceiptCard lives in
                Advanced. This hands off there — the receipt is held in
                tm.lastExportReceipt, so it survives the view switch. */}
            <button
              type="button"
              className="ghost-btn std-export-done-report"
              onClick={onEnterAdvanced}
            >
              View full report
            </button>
          </div>
        )}
      </div>
    </aside>
  );
}

export function StandardView({
  tm,
  guide = NO_GUIDE,
  onEnterAdvanced,
}: {
  tm: TM;
  // First-run guide state (owned by App so the hint can float at the root).
  // Optional only so isolated unit renders stay terse; App always supplies it.
  guide?: FirstRunGuide;
  // Required: a missed wiring must fail tsc, not ship a dead 'Change' button.
  onEnterAdvanced: () => void;
}) {
  const s = tm.selectedSettings;
  const canUseMaster = tm.selectedAnalysis != null;
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
  // Any route into Advanced ends the guide — someone who found Advanced
  // doesn't need first-run tips.
  const enterAdvanced = () => {
    guide.noteEnteredAdvanced();
    onEnterAdvanced();
  };
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
              {formatDuration(tm.transport.currentTimeSec)} / {formatDuration(tm.selectedTrack?.duration_seconds)}
            </span>
          </div>
          <div className="std-wave">
            {/* WaveformView owns the no-peaks branch internally (loading
                card + orb) AND the analyzing→peaks morph transition — it
                must stay mounted across that boundary, so no ternary. */}
            <WaveformView
              peaks={tm.selectedWaveform}
              isLoading={tm.isLoadingWaveform}
              isAnalyzing={tm.isAnalyzing}
              analysisProgress={tm.analysisProgress}
              currentTimeSec={tm.transport.currentTimeSec}
              durationSec={tm.selectedTrack?.duration_seconds ?? 0}
              region={tm.selectedRegion}
              regionsEnabled={false}
              onSeek={tm.seek}
              onSetRegion={tm.setRegion}
              onClearRegion={tm.clearRegion}
            />
          </div>
        </div>

        <div className="std-steps">
          <div className="std-step std-step-style">
            <span className="std-step-label">1 · Style</span>
            <span className="std-step-hint">Choose the character you want.</span>
            <StyleTiles preset={s.preset} onSelect={tm.setPreset} />
          </div>

          {/* Steps 2 & 3 share a 2-up row — Intensity beside Loudness (two
              equal-height cards), per the Standard intensity+loudness layout. */}
          <div className="std-pair">
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
              // U9: the visible label lives in the "2 · Intensity" step
              // heading, so the knob renders none — but it still needs an
              // accessible name, and shipped with `aria-label=""` (i.e. none).
              ariaLabel="Intensity"
              valueText={(v) => `${Math.round(v * 100)} percent, ${intensityLabel(v)}`}
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

          <div className="std-step std-step-loudness" ref={seamRefs.loudness}>
            <span className="std-step-label">3 · Loudness</span>
            <span className="std-step-hint">Choose your target loudness.</span>
            <LoudnessSegmented
              targetLufs={effectiveLoudnessTarget(s)}
              onSelect={tm.setLoudnessTarget}
            />
          </div>
          </div>
        </div>
      </section>

      <StandardRightRail
        tm={tm}
        guide={guide}
        canUseMaster={canUseMaster}
        onEnterAdvanced={enterAdvanced}
        seamRefs={seamRefs}
      />
    </div>
  );
}
