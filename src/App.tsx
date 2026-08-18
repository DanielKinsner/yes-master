import {
  useCallback,
  useEffect,
  useRef,
  useState,
  type DragEvent as ReactDragEvent,
  type ReactNode,
} from "react";
import { useTrackMaster } from "./hooks/useTrackMaster";
import { useNavigationMachine } from "./hooks/useNavigationMachine";
import { useFirstRunGuide } from "./hooks/useFirstRunGuide";
import { StandardView } from "./components/StandardView";
import { FirstRunOverlay } from "./components/FirstRunOverlay";
import { hasNonManagedEdits } from "./lib/standard-managed";
import { PresetIcon, PRESET_ACCENT, PRESET_TONE } from "./components/PresetIcon";
import { RightRail, MasterOutPanel } from "./components/RightRail";
import { VisualEqPanel } from "./components/VisualEqPanel";
import { AlbumPanel } from "./components/AlbumPanel";
import { AlbumExportReceipt } from "./components/AlbumExportReceipt";
import { Knob, intensityLabel } from "./components/Knob";
import { SignalChain } from "./components/SignalChain";
import { EmptyState } from "./components/EmptyState";
import { Toast } from "./components/Toast";
import { ChromeDialog } from "./components/ChromeDialog";
import { SettingsGroup } from "./components/SettingsGroup";
import { AdvancedPanel } from "./components/AdvancedPanel";
import { DisabledReason, PanelResetButton } from "./components/fields";
import { ExportReceiptCard } from "./components/ExportReceiptCard";
import { WaveformView } from "./components/Waveform";
import { PlayPauseGlyph } from "./components/TransportGlyph";
import { SourceInsight } from "./components/SourceInsight";
import { useInsightReview } from "./hooks/useInsightReview";
import type {
  AnalysisResult,
  AudioOutputDevice,
  ImportedTrack,
  MasteringSettings,
  Preset,
  QualityCheck,
  UserPreset,
} from "./bindings";
import type { PlaybackKindUI, RenderProgressState } from "./hooks/useTrackMaster";
import {
  LOUDNESS_PROFILES,
  effectiveLoudnessTarget,
  loudnessTargetDisplay,
} from "./lib/effective-settings";
import { buildSequenceRows, type SequenceRow } from "./lib/album-sequence";
import { trackCountLabel } from "./lib/album-copy";
import { HELP_SECTIONS, SETTINGS_GROUPS } from "./lib/chrome-content";
import { api, onUpdaterAvailable } from "./lib/api";
import { save } from "./lib/tauri-runtime";
import { requestGuideReset } from "./lib/first-run-guide";
import { isToneFlat } from "./lib/tone-reset";
import { SUPPORTED_FORMATS_COPY } from "./lib/supported-formats";
import { formatDuration } from "./lib/time-format";
import { PRESET_OPTIONS } from "./lib/preset-copy";
import "./App.css";

// The first four mirror the Standard tiles (Universal/Clarity/Tape/Oomph) in
// the same order, per the preset-name unification decision — so the four a
// Standard user knows sit at the front when they flip to Advanced. Keep Oomph
// ahead of Spatial; don't "tidy" this back. PRESET_OPTIONS moved to
// lib/preset-copy.ts so the export receipt can reuse the same label + blurb.

const AUDIO_OUTPUT_STORAGE_KEY = "yes-master:audio-output-device";
const SYSTEM_DEFAULT_AUDIO_OUTPUT = "system-default";

export interface AudioOutputSettingsState {
  devices: AudioOutputDevice[];
  selectedDeviceId: string;
  isLoading: boolean;
  message: string | null;
  error: string | null;
  onSelect: (deviceId: string) => Promise<void>;
  onRefresh: () => Promise<void>;
}

function storedAudioOutputDevice(storage: Storage | undefined | null): string {
  const stored = storage?.getItem(AUDIO_OUTPUT_STORAGE_KEY)?.trim();
  return stored || SYSTEM_DEFAULT_AUDIO_OUTPUT;
}

function persistAudioOutputDevice(
  storage: Storage | undefined | null,
  deviceId: string,
) {
  if (!storage) return;
  if (deviceId === SYSTEM_DEFAULT_AUDIO_OUTPUT) {
    storage.removeItem(AUDIO_OUTPUT_STORAGE_KEY);
  } else {
    storage.setItem(AUDIO_OUTPUT_STORAGE_KEY, deviceId);
  }
}

function audioOutputErrorMessage(err: unknown): string {
  return err instanceof Error ? err.message : String(err);
}

function useAudioOutputSettings(onDeviceRecovered?: () => void): AudioOutputSettingsState {
  const [devices, setDevices] = useState<AudioOutputDevice[]>([]);
  const [selectedDeviceId, setSelectedDeviceId] = useState(() =>
    storedAudioOutputDevice(globalThis.localStorage),
  );
  const [isLoading, setIsLoading] = useState(false);
  const [message, setMessage] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  const refresh = useCallback(async () => {
    setIsLoading(true);
    try {
      const nextDevices = await api.listAudioOutputDevices();
      setDevices(nextDevices);
      setError(null);
    } catch (err) {
      setError(`Could not read audio outputs. ${audioOutputErrorMessage(err)}`);
    } finally {
      setIsLoading(false);
    }
  }, []);

  useEffect(() => {
    let cancelled = false;
    const applyStoredOutput = async () => {
      setIsLoading(true);
      const stored = storedAudioOutputDevice(globalThis.localStorage);
      try {
        await api.setAudioOutputDevice(
          stored === SYSTEM_DEFAULT_AUDIO_OUTPUT ? null : stored,
        );
        const nextDevices = await api.listAudioOutputDevices();
        if (cancelled) return;
        setSelectedDeviceId(stored);
        setDevices(nextDevices);
        setError(null);
      } catch (err) {
        if (cancelled) return;
        persistAudioOutputDevice(globalThis.localStorage, SYSTEM_DEFAULT_AUDIO_OUTPUT);
        setSelectedDeviceId(SYSTEM_DEFAULT_AUDIO_OUTPUT);
        const fallbackError = `Could not use the saved audio output. Using system default. ${audioOutputErrorMessage(err)}`;
        setError(
          fallbackError,
        );
        try {
          await api.setAudioOutputDevice(null);
        } catch {
          // Keep the original saved-device error visible; the device list can
          // still let the user pick a working non-default output.
        }
        try {
          const nextDevices = await api.listAudioOutputDevices();
          if (!cancelled) setDevices(nextDevices);
        } catch (listErr) {
          if (!cancelled) {
            setError(
              `${fallbackError} Could not read audio outputs. ${audioOutputErrorMessage(listErr)}`,
            );
          }
        }
      } finally {
        if (!cancelled) setIsLoading(false);
      }
    };

    void applyStoredOutput();
    return () => {
      cancelled = true;
    };
  }, []);

  const onSelect = useCallback(
    async (deviceId: string) => {
      const nextDeviceId = deviceId || SYSTEM_DEFAULT_AUDIO_OUTPUT;
      const previousDeviceId = selectedDeviceId;
      setSelectedDeviceId(nextDeviceId);
      setIsLoading(true);
      setMessage(null);
      setError(null);
      try {
        await api.setAudioOutputDevice(
          nextDeviceId === SYSTEM_DEFAULT_AUDIO_OUTPUT ? null : nextDeviceId,
        );
        persistAudioOutputDevice(globalThis.localStorage, nextDeviceId);
        const nextDevices = await api.listAudioOutputDevices();
        setDevices(nextDevices);
        setMessage(
          nextDeviceId === SYSTEM_DEFAULT_AUDIO_OUTPUT
            ? "Using system default."
            : "Audio output saved. Press Play to audition through that device.",
        );
        onDeviceRecovered?.();
      } catch (err) {
        setSelectedDeviceId(previousDeviceId);
        persistAudioOutputDevice(globalThis.localStorage, previousDeviceId);
        setError(`Could not switch audio output. ${audioOutputErrorMessage(err)}`);
      } finally {
        setIsLoading(false);
      }
    },
    [selectedDeviceId, onDeviceRecovered],
  );

  return {
    devices,
    selectedDeviceId,
    isLoading,
    message,
    error,
    onSelect,
    onRefresh: refresh,
  };
}

const EMPTY_AUDIO_OUTPUT_SETTINGS: AudioOutputSettingsState = {
  devices: [],
  selectedDeviceId: SYSTEM_DEFAULT_AUDIO_OUTPUT,
  isLoading: false,
  message: null,
  error: null,
  onSelect: async () => {},
  onRefresh: async () => {},
};

function App() {
  const tm = useTrackMaster();
  const audioOutput = useAudioOutputSettings(tm.clearPlaybackDeviceLost);
  const [chromePanel, setChromePanel] = useState<"settings" | "help" | null>(null);
  const [modeNotice, setModeNotice] = useState<string | null>(null);
  // Slice 7b: the version of an available update (null = none / dismissed).
  const [updateAvailable, setUpdateAvailable] = useState<string | null>(null);
  useWebviewZoomShortcuts();

  // B5.1: Standard/Advanced/Album navigation is ONE legal-state machine
  // (lib/navigation-machine.ts). The old split-brain shape — view in
  // useViewMode, returnConfirm here, legality patched by two reactive
  // effects after render — produced the 2a78f4a silent-trap class; now
  // every transition is a reducer table row, the always-clean /
  // Album-only-in-Advanced invariants are entry conditions instead of
  // corrections, and leaving Album on a return to Standard happens at the
  // dispatch site so nothing can re-bounce it.
  const nav = useNavigationMachine({
    hadPriorSession: tm.hadPriorSession,
    isAlbum: tm.mode === "album",
    hasTrack: !!tm.selectedTrack,
    hasNonManagedEdits:
      !!tm.selectedTrack && hasNonManagedEdits(tm.selectedSettings),
    // setMode is plain state (albumIntent/overrides live separately), so
    // leaving and re-entering Album loses no album configuration.
    leaveAlbumMode: () => tm.setMode("track"),
  });
  const { view, setView } = nav;

  // F6 per-track view memory: explicit view choices are recorded for the
  // selected track, so switching tracks restores that track's choice. The
  // force-bounce for a dirty track lives in the navigation machine and records
  // nothing, so it can never clobber a remembered choice.
  const enterAdvanced = () => {
    tm.rememberTrackView(tm.selectedTrackId, "advanced");
    setView("advanced");
  };
  const backToStandard = () => {
    // A silent return (clean track, or nothing selected) reaches Standard now,
    // so record it now; a dirty return opens the confirm dialog and is recorded
    // on completeReturn instead (cancel keeps Advanced, deliberately unrecorded).
    if (!tm.selectedTrack || !hasNonManagedEdits(tm.selectedSettings)) {
      tm.rememberTrackView(tm.selectedTrackId, "standard");
    }
    nav.requestBackToStandard();
  };
  const completeReturnToStandard = () => {
    tm.rememberTrackView(tm.selectedTrackId, "standard");
    nav.completeReturn();
  };
  // First-run guide lives at the root now (L9): the hint renders as a floating
  // FirstRunOverlay below, and StandardView only consumes the step for the
  // Mastered-button pulse. Lifting the hook out of StandardView also lets the
  // header's Advanced affordance finish the guide through its own state rather
  // than poking localStorage behind the hook's back.
  const guide = useFirstRunGuide({
    hasAnalyzedTrack: tm.selectedAnalysis != null,
    playbackKind: tm.transport.playbackKind,
    isPlaying: tm.transport.isPlaying,
  });
  const selectedExportReceipt =
    tm.lastExportReceipt?.trackId === tm.selectedTrackId ? tm.lastExportReceipt : null;
  const selectedExportChecks = selectedExportReceipt?.checks;

  // WYSIWYG: the live Mastered audition equals the export only when the
  // loudness landing + limiter are applied in real time. Standard forces
  // that via the INTERNAL flag — it never touches the user-facing Advanced
  // `Preview LUFS` toggle (see Task 5 step 3g).
  useEffect(() => {
    if (view === null) return;
    tm.setForceWysiwyg(view === "standard");
  }, [view, tm.setForceWysiwyg]);

  // F3: looping is Advanced-only. Standard has no loop toggle or chip, so an
  // armed loop must not survive into it (it kept wrapping playback with no
  // UI explaining why). Region memory survives; re-arming is explicit.
  useEffect(() => {
    if (view === "standard") {
      void tm.disarmLoop();
    }
  }, [view, tm.disarmLoop]);

  // F6: restore the remembered view when the selected track changes (sidebar
  // select, import auto-select, session/project restore). A dirty track still
  // force-bounces to Advanced in the machine; this only re-applies an explicit
  // prior choice, and does nothing for a track with no remembered view.
  const prevSelectedTrackRef = useRef(tm.selectedTrackId);
  useEffect(() => {
    if (tm.selectedTrackId === prevSelectedTrackRef.current) return;
    prevSelectedTrackRef.current = tm.selectedTrackId;
    const remembered = tm.rememberedTrackView(tm.selectedTrackId);
    if (remembered) setView(remembered);
  }, [tm.selectedTrackId, tm.rememberedTrackView, setView]);

  // Slice 7b: surface a newer release as a non-blocking toast. The backend's
  // startup check (Slice 7) emits `updater:available`; the toast's action
  // installs on click and is disabled while an export/render runs, so work is
  // never interrupted.
  useEffect(() => {
    let unlisten: (() => void) | undefined;
    onUpdaterAvailable((version) => setUpdateAvailable(version))
      .then((un) => {
        unlisten = un;
      })
      .catch(() => {
        /* no updater here (e.g. browser preview) — ignore */
      });
    return () => unlisten?.();
  }, []);

  const installUpdate = () => {
    // Success relaunches the app (never returns); a failed/offline download
    // just dismisses the toast — never a modal.
    api.installUpdate().catch((err) => {
      console.warn("Update install failed", err);
      setUpdateAvailable(null);
    });
  };

  const handleModeChange = (nextMode: "track" | "album") => {
    if (nextMode === "album" && view === "standard") {
      setModeNotice("Opening Album Master in Advanced.");
    } else {
      setModeNotice(null);
    }
    tm.setMode(nextMode);
  };

  // U10 — sequence overview rows. Built from data the app already holds; the
  // roles and arc offsets come from the backend's own planner (see
  // `albumPlanPreview`) so what the rail shows is what will actually render.
  const sequenceRows: SequenceRow[] =
    tm.mode === "album"
      ? buildSequenceRows({
          tracks: tm.tracks,
          analysisByTrackId: tm.analysisByTrackId,
          overrideAlbum: tm.overrideAlbum,
          plan: tm.albumPlanPreview,
          // Guarded: album intent is absent until an album exists, and the
          // overview must degrade to "no target shown" rather than throw.
          albumTargetLufs: tm.albumIntent
            ? effectiveLoudnessTarget(tm.albumIntent)
            : null,
        })
      : [];

  return (
    <div className="app-root">
      <TopHeader
        mode={tm.mode}
        onModeChange={handleModeChange}
        onSaveProject={tm.saveProjectAs}
        onOpenProject={tm.openProjectFromDisk}
        onOpenSettings={() => setChromePanel("settings")}
        onOpenHelp={() => setChromePanel("help")}
        canUndo={tm.canUndo}
        canRedo={tm.canRedo}
        onUndo={tm.undo}
        onRedo={tm.redo}
        viewMode={view === "advanced" ? "advanced" : "standard"}
        onEnterAdvanced={() => {
          // Entering Advanced from the chrome ends the first-run guide so it
          // never re-appears. noteEnteredAdvanced persists "done" AND clears
          // the live step, so the floating overlay drops immediately.
          guide.noteEnteredAdvanced();
          enterAdvanced();
        }}
        onBackToStandard={backToStandard}
      />
    <div className={"app" + (view === "standard" ? " app-standard" : "")}>
      {view === "advanced" && (
        <>
      <Sidebar
        tracks={tm.tracks}
        selectedId={tm.selectedTrackId}
        onSelect={tm.selectTrack}
        onRemove={tm.removeTrack}
        onAdd={tm.openImportDialog}
        isAnalyzing={tm.isAnalyzing}
        analysisProgress={tm.analysisProgress}
        mode={tm.mode}
        onReorder={tm.reorderTracks}
        sequenceRows={sequenceRows}
        albumHeader={
          tm.mode === "album" && tm.tracks.length > 0 ? (
            <AlbumPanel
              tracks={tm.tracks}
              albumArcKind={tm.albumArcKind}
              albumIntensity={tm.albumIntensity}
              albumTitle={tm.albumTitle}
              onAlbumArc={tm.setAlbumArc}
              onAlbumIntensity={tm.setAlbumIntensity}
              onAlbumTitle={tm.setAlbumTitle}
              sequenceRows={sequenceRows}
            />
          ) : null
        }
        albumReceipt={
          tm.mode === "album" && tm.albumExportReport ? (
            <AlbumExportReceipt report={tm.albumExportReport} />
          ) : null
        }
      />
      <main className="workspace">
        {tm.selectedTrack ? (
          <TrackMaster tm={tm} />
        ) : (
          <EmptyState
            onAdd={() => {
              // The welcome hero is the onboarding funnel: importing from
              // it lands in Standard (the default face) even when the last
              // session ended in Advanced. With no track loaded the
              // machine's back-to-standard door is always silent (no
              // confirm). The pro paths — sidebar "+ Add Tracks", drag-
              // drop — deliberately keep the current view.
              nav.requestBackToStandard();
              void tm.openImportDialog();
            }}
          />
        )}
      </main>
      <RightRail
        analysis={tm.selectedAnalysis}
        lastChecks={selectedExportChecks}
        advancedSlot={
          tm.selectedTrack ? (
            <AdvancedPanel
              analysis={tm.selectedAnalysis}
              settings={tm.selectedSettings}
              onAdvanced={tm.setAdvanced}
              onInputGain={tm.setInputGain}
              onOutputGain={tm.setOutputGain}
              onLoudnessTarget={tm.setLoudnessTarget}
              onDeliveryProfile={tm.setDeliveryProfile}
              onDeliveryBitDepth={tm.setDeliveryBitDepth}
              onDeliverySampleRate={tm.setDeliverySampleRate}
              showDeliveryFormat
              albumDeliveryFormat={
                tm.mode === "album"
                  ? {
                      bitDepth: tm.albumBitDepth,
                      sampleRate: tm.albumSampleRate,
                      onBitDepth: tm.setAlbumBitDepth,
                      onSampleRate: tm.setAlbumSampleRate,
                    }
                  : undefined
              }
              adaptiveReadout={tm.guardrailReadout}
              compressionPlan={tm.compressionPlan}
              albumMode={tm.mode === "album"}
            />
          ) : undefined
        }
        exportMode={tm.mode === "album" ? "album" : "track"}
        canExport={tm.mode === "album" ? tm.tracks.length > 0 : !!tm.selectedAnalysis}
        isExporting={tm.mode === "album" ? tm.albumRendering : tm.isExporting}
        isRendering={tm.isRendering}
        renderProgress={tm.renderProgress}
        renderFeedback={tm.renderFeedback}
        cancelRenderPending={
          !!tm.renderProgress && tm.cancelRequestedJobId === tm.renderProgress.job_id
        }
        previewStale={tm.previewStale}
        canRenderPreview={!!tm.selectedAnalysis}
        onUpdatePreview={tm.updatePreview}
        onExport={tm.mode === "album" ? tm.exportAlbumPlan : tm.exportMaster}
        onCancelRender={tm.cancelActiveRender}
      />
        </>
      )}
      {view === "standard" && tm.selectedTrack && (
        // StandardView's own enterAdvanced wrapper finishes the guide before
        // switching, so this handler only owns the view change.
        <StandardView tm={tm} guide={guide} onEnterAdvanced={enterAdvanced} />
      )}
      {view === "standard" && !tm.selectedTrack && <EmptyState onAdd={tm.openImportDialog} />}
      {/* First-run coachmark (L9): a floating sibling of the drop/toast
          overlays, not an inline rail chip. Only Standard's flow drives it;
          gate on the same condition that mounts StandardView so a stale step
          can't surface in Advanced (e.g. an Album bounce). */}
      {view === "standard" && tm.selectedTrack && (
        <FirstRunOverlay step={guide.step} onDismiss={guide.dismiss} />
      )}
      {tm.isDragOver && (
        <div className="drop-overlay" aria-hidden>
          <div className="drop-overlay-card">
            <div className="drop-overlay-title">Drop to import</div>
            <div className="drop-overlay-hint">
              {SUPPORTED_FORMATS_COPY}
            </div>
          </div>
        </div>
      )}
      {(tm.playbackDeviceLost ||
        tm.error ||
        tm.projectFeedback ||
        modeNotice ||
        updateAvailable) && (
        <div className="toast-stack" aria-live="polite">
          {tm.playbackDeviceLost && (
            <div className="device-loss-banner" role="alert">
              <div className="device-loss-banner-copy">
                <strong>Playback device disconnected.</strong>
                <span>Choose an output in Settings, then press Play.</span>
              </div>
              <button
                type="button"
                className="ghost-btn device-loss-primary"
                onClick={() => setChromePanel("settings")}
              >
                Choose device
              </button>
              <button
                type="button"
                className="ghost-btn"
                onClick={tm.clearPlaybackDeviceLost}
              >
                Dismiss
              </button>
            </div>
          )}
          {tm.error && (
            <Toast message={tm.error} tone="danger" onClose={tm.clearError} />
          )}
          {modeNotice && (
            <Toast
              message={modeNotice}
              tone="info"
              onClose={() => setModeNotice(null)}
            />
          )}
          {tm.projectFeedback && (
            <Toast
              message={tm.projectFeedback.message}
              tone={tm.projectFeedback.tone}
              onClose={tm.clearProjectFeedback}
            />
          )}
          {updateAvailable && (
            <Toast
              message={`Update available — v${updateAvailable}`}
              tone="info"
              onClose={() => setUpdateAvailable(null)}
              action={{
                label: "Restart to update",
                onClick: installUpdate,
                disabled: tm.isExporting || tm.isRendering,
                disabledTitle:
                  "Finishing your export first — this re-enables the moment it's done.",
              }}
            />
          )}
        </div>
      )}
      {selectedExportReceipt && view === "advanced" && (
        <ExportReceiptCard
          receipt={selectedExportReceipt}
          track={tm.selectedTrack ?? null}
          settings={tm.selectedSettings}
          analysis={tm.selectedAnalysis ?? null}
          onClose={tm.clearExportReceipt}
        />
      )}
      {chromePanel === "settings" && (
        <SettingsPanel
          audioOutput={audioOutput}
          onClose={() => setChromePanel(null)}
        />
      )}
      {chromePanel === "help" && (
        <HelpPanel onClose={() => setChromePanel(null)} />
      )}
      {nav.returnConfirmOpen && (
        <BackToStandardConfirm
          saving={tm.savingPreset}
          onCancel={nav.cancelReturn}
          onReset={() => {
            tm.resetToStandardManaged();
            completeReturnToStandard();
          }}
          onSaveAsPreset={async (name) => {
            // Only reset + switch if the save actually succeeded — the save
            // is async; never discard the user's edits when the write failed.
            const ok = await tm.saveUserPreset(name);
            if (ok) {
              tm.resetToStandardManaged();
              completeReturnToStandard();
            }
            return ok;
          }}
        />
      )}
    </div>
    {view === "advanced" && <BottomStatusBar tm={tm} />}
    </div>
  );
}

function BottomStatusBar({ tm }: { tm: ReturnType<typeof useTrackMaster> }) {
  const peak = tm.transport.peakDbfs;
  const liveLufs = tm.transport.lufsIntegrated;
  const isPlaying = tm.transport.isPlaying;

  const peakDisplay = isPlaying && peak > -80 ? `${peak.toFixed(1)} dBFS` : "—";
  const lufsDisplay = isPlaying && liveLufs > -80 ? `${liveLufs.toFixed(1)} LUFS` : "—";

  // Slice 13c metadata diet: the footer speaks only while something is
  // happening. Coarse session state lives in the header SessionStatus pill;
  // quality verdicts live in Source Insight under the track title. At rest this bar
  // holds only the live meters — no permanent "Ready"/summary chatter.
  let processing: string | null = null;
  if (tm.isExporting) {
    processing = "Exporting…";
  } else if (tm.isRendering) {
    processing = "Rendering audit…";
  } else if (tm.isAnalyzing) {
    processing = tm.analysisProgress?.label ?? "Analyzing…";
  } else if (tm.isLoadingWaveform) {
    processing = "Decoding…";
  }

  return (
    <footer className="bottom-status">
      <div className="bottom-status-left" />
      <div className="bottom-status-center">
        <span className="status-readout">
          <span className="status-readout-label">Live peak</span>
          <span className="status-readout-value">{peakDisplay}</span>
        </span>
        <span className="status-readout">
          <span className="status-readout-label">Live LUFS</span>
          <span className="status-readout-value">{lufsDisplay}</span>
        </span>
      </div>
      <div className="bottom-status-right">
        {processing !== null && (
          <>
            <span className="status-processing-label">Processing</span>
            <span className="status-pill status-warn">{processing}</span>
          </>
        )}
      </div>
    </footer>
  );
}

export function TopHeader({
  mode,
  onModeChange,
  onSaveProject,
  onOpenProject,
  onOpenSettings,
  onOpenHelp,
  canUndo,
  canRedo,
  onUndo,
  onRedo,
  viewMode,
  onEnterAdvanced,
  onBackToStandard,
}: {
  mode: "track" | "album";
  onModeChange: (mode: "track" | "album") => void;
  onSaveProject: () => void;
  onOpenProject: () => void;
  onOpenSettings: () => void;
  onOpenHelp: () => void;
  canUndo: boolean;
  canRedo: boolean;
  onUndo: () => void;
  onRedo: () => void;
  viewMode: "standard" | "advanced";
  onEnterAdvanced: () => void;
  onBackToStandard: () => void;
}) {
  return (
    <header className="top-header">
      <div className="top-header-left">
        <span className="brand-mark" aria-hidden>
          <svg width="22" height="22" viewBox="0 0 24 24" fill="none">
            <path
              d="M4 6h2v12H4zM8 10h2v8H8zM12 4h2v16h-2zM16 8h2v10h-2zM20 12h2v6h-2z"
              fill="currentColor"
            />
          </svg>
        </span>
        <span className="brand-name">YES Master</span>
      </div>
      {/* Spec §chrome: Track/Album tabs render in BOTH views (clicking Album
          from Standard triggers the entry guard's bounce into Advanced);
          Standard gets a single 'Advanced' affordance, never a
          Standard|Advanced segmented control. */}
      <nav className="top-header-tabs" aria-label="Mode">
        <button
          type="button"
          className={"top-tab " + (mode === "track" ? "is-active" : "")}
          onClick={() => onModeChange("track")}
        >
          Track Master
        </button>
        <button
          type="button"
          className={"top-tab " + (mode === "album" ? "is-active" : "")}
          onClick={() => onModeChange("album")}
        >
          Album Master
        </button>
      </nav>
      <div className="top-header-right">
        <button
          type="button"
          className="icon-tile"
          aria-label="Undo — Ctrl+Z"
          title="Undo — Ctrl+Z"
          disabled={!canUndo}
          onClick={onUndo}
        >
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={2} strokeLinecap="round" strokeLinejoin="round">
            <path d="M9 14 4 9l5-5" />
            <path d="M4 9h10a6 6 0 0 1 0 12h-1" />
          </svg>
        </button>
        <button
          type="button"
          className="icon-tile"
          aria-label="Redo — Ctrl+Y"
          title="Redo — Ctrl+Y"
          disabled={!canRedo}
          onClick={onRedo}
        >
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={2} strokeLinecap="round" strokeLinejoin="round">
            <path d="m15 14 5-5-5-5" />
            <path d="M20 9H10a6 6 0 0 0 0 12h1" />
          </svg>
        </button>
        {viewMode === "standard" ? (
          <button type="button" className="ghost-btn top-advanced" onClick={onEnterAdvanced}>
            Advanced
          </button>
        ) : (
          <button type="button" className="ghost-btn top-advanced" onClick={onBackToStandard}>
            ‹ Back to Standard
          </button>
        )}
        <button
          type="button"
          className="icon-tile"
          aria-label="Open project (.ams.json)"
          title="Open project (.ams.json)"
          onClick={onOpenProject}
        >
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={2} strokeLinecap="round" strokeLinejoin="round">
            <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z" />
          </svg>
        </button>
        <button
          type="button"
          className="icon-tile"
          aria-label="Save project (.ams.json)"
          title="Save project as (.ams.json)"
          onClick={onSaveProject}
        >
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={2} strokeLinecap="round" strokeLinejoin="round">
            <path d="M19 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11l5 5v11a2 2 0 0 1-2 2z" />
            <polyline points="17 21 17 13 7 13 7 21" />
            <polyline points="7 3 7 8 15 8" />
          </svg>
        </button>
        <button
          type="button"
          className="icon-tile"
          aria-label="Settings"
          title="Settings"
          onClick={onOpenSettings}
        >
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={2} strokeLinecap="round" strokeLinejoin="round">
            <path d="M12.22 2h-.44a2 2 0 0 0-2 2v.18a2 2 0 0 1-1 1.73l-.43.25a2 2 0 0 1-2 0l-.15-.08a2 2 0 0 0-2.73.73l-.22.38a2 2 0 0 0 .73 2.73l.15.1a2 2 0 0 1 1 1.72v.51a2 2 0 0 1-1 1.74l-.15.09a2 2 0 0 0-.73 2.73l.22.38a2 2 0 0 0 2.73.73l.15-.08a2 2 0 0 1 2 0l.43.25a2 2 0 0 1 1 1.73V20a2 2 0 0 0 2 2h.44a2 2 0 0 0 2-2v-.18a2 2 0 0 1 1-1.73l.43-.25a2 2 0 0 1 2 0l.15.08a2 2 0 0 0 2.73-.73l.22-.39a2 2 0 0 0-.73-2.73l-.15-.08a2 2 0 0 1-1-1.74v-.5a2 2 0 0 1 1-1.74l.15-.09a2 2 0 0 0 .73-2.73l-.22-.38a2 2 0 0 0-2.73-.73l-.15.08a2 2 0 0 1-2 0l-.43-.25a2 2 0 0 1-1-1.73V4a2 2 0 0 0-2-2z" />
            <circle cx="12" cy="12" r="3" />
          </svg>
        </button>
        <button
          type="button"
          className="icon-tile"
          aria-label="Help"
          title="Help"
          onClick={onOpenHelp}
        >
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={2} strokeLinecap="round" strokeLinejoin="round">
            <circle cx="12" cy="12" r="10" />
            <path d="M9.09 9a3 3 0 0 1 5.83 1c0 2-3 3-3 3" />
            <path d="M12 17h.01" />
          </svg>
        </button>
      </div>
    </header>
  );
}

export function SettingsPanel({
  audioOutput = EMPTY_AUDIO_OUTPUT_SETTINGS,
  onClose,
}: {
  audioOutput?: AudioOutputSettingsState;
  onClose: () => void;
}) {
  return (
    <ChromeDialog title="Settings" eyebrow="Current defaults" onClose={onClose}>
      <div className="settings-grid">
        <AudioOutputSelector audioOutput={audioOutput} />
        {SETTINGS_GROUPS.map((group) => (
          <SettingsGroup key={group.title} title={group.title} rows={group.rows} />
        ))}
        <div className="settings-actions">
          <button
            type="button"
            className="ghost-btn"
            onClick={() => {
              requestGuideReset(globalThis.localStorage);
              // Close so the revived chip is visible, not hidden behind
              // this dialog.
              onClose();
            }}
          >
            Show first-run tips again
          </button>
        </div>
      </div>
    </ChromeDialog>
  );
}

function AudioOutputSelector({
  audioOutput,
}: {
  audioOutput: AudioOutputSettingsState;
}) {
  const defaultDevice = audioOutput.devices.find((device) => device.is_default);
  const selectedDeviceId = audioOutput.selectedDeviceId || SYSTEM_DEFAULT_AUDIO_OUTPUT;

  return (
    <section className="settings-group settings-audio-output">
      <div className="settings-group-head">
        <h3>Audio Output</h3>
        <button
          type="button"
          className="ghost-btn settings-refresh-btn"
          disabled={audioOutput.isLoading}
          onClick={() => {
            void audioOutput.onRefresh();
          }}
        >
          Refresh
        </button>
      </div>
      <label className="settings-field-label" htmlFor="audio-output-device">
        Playback device
      </label>
      <select
        id="audio-output-device"
        className="settings-select"
        value={selectedDeviceId}
        disabled={audioOutput.isLoading}
        onChange={(event) => {
          void audioOutput.onSelect(event.currentTarget.value);
        }}
      >
        <option value={SYSTEM_DEFAULT_AUDIO_OUTPUT}>
          {defaultDevice ? `System default (${defaultDevice.name})` : "System default"}
        </option>
        {audioOutput.devices.map((device) => (
          <option key={device.id} value={device.id}>
            {device.name}
          </option>
        ))}
      </select>
      <div className="settings-audio-output-status" aria-live="polite">
        {audioOutput.error ??
          audioOutput.message ??
          (audioOutput.isLoading ? "Checking outputs..." : "Ready")}
      </div>
    </section>
  );
}

export function HelpPanel({ onClose }: { onClose: () => void }) {
  const [diagnosticsNote, setDiagnosticsNote] = useState<string | null>(null);
  // "What am I running?" — version + git hash + build time from the binary
  // itself, so a hand-test can always tell WHICH build is installed.
  const [buildInfo, setBuildInfo] = useState<string | null>(null);
  useEffect(() => {
    let cancelled = false;
    Promise.resolve(api.buildInfo?.())
      .then((info) => {
        if (!cancelled && info) setBuildInfo(info);
      })
      .catch(() => {
        /* browser preview / older backend — line simply stays hidden */
      });
    return () => {
      cancelled = true;
    };
  }, []);
  const saveDiagnostics = async () => {
    try {
      const chosen = await save({
        defaultPath: `YESMaster-diagnostics-${new Date().toISOString().slice(0, 10)}.txt`,
        filters: [{ name: "Text report", extensions: ["txt"] }],
      });
      if (!chosen) return;
      const written = await api.saveDiagnosticsReport(chosen);
      setDiagnosticsNote(`Saved to ${written}`);
    } catch (err) {
      setDiagnosticsNote(err instanceof Error ? err.message : String(err));
    }
  };
  return (
    <ChromeDialog title="Help" eyebrow="Track Master guide" onClose={onClose}>
      <div className="help-sections">
        {HELP_SECTIONS.map(([title, body]) => (
          <section className="help-section" key={title}>
            <h3>{title}</h3>
            <p>{body}</p>
          </section>
        ))}
        <section className="help-section">
          <h3>Something broke?</h3>
          <p>
            Save a diagnostics report — recent app logs and your session
            summary, assembled on this machine. Nothing is sent anywhere;
            share the file only if you choose to.
          </p>
          <button type="button" className="ghost-btn" onClick={saveDiagnostics}>
            Save diagnostics report…
          </button>
          {diagnosticsNote && <p className="help-diagnostics-note">{diagnosticsNote}</p>}
        </section>
        {buildInfo && (
          <p className="help-build-info" title="Version · git hash · build time">
            YES Master {buildInfo}
          </p>
        )}
      </div>
    </ChromeDialog>
  );
}

function BackToStandardConfirm({
  saving,
  onCancel,
  onReset,
  onSaveAsPreset,
}: {
  saving: boolean;
  onCancel: () => void;
  onReset: () => void;
  onSaveAsPreset: (name: string) => Promise<boolean>;
}) {
  const [name, setName] = useState("");
  const [busy, setBusy] = useState(false);
  const disabled = saving || busy;
  const handleSave = async () => {
    const trimmed = name.trim();
    if (!trimmed || disabled) return;
    setBusy(true);
    try {
      // On success the parent unmounts this modal; on failure it stays open
      // with the edits intact so the user can retry.
      await onSaveAsPreset(trimmed);
    } finally {
      setBusy(false);
    }
  };
  return (
    <div className="modal-scrim" role="dialog" aria-modal="true" aria-label="Back to Standard">
      <div className="modal-card">
        <h2 className="modal-title">Back to Standard</h2>
        <p className="modal-body">
          Back to Standard clears your advanced edits but keeps your style,
          intensity, and loudness settings. Save them as a preset first?
        </p>
        <div className="modal-save-row">
          <input
            className="modal-input"
            placeholder="Preset name"
            value={name}
            disabled={disabled}
            onChange={(e) => setName(e.target.value)}
          />
          <button
            type="button"
            className="primary"
            disabled={disabled || name.trim().length === 0}
            onClick={handleSave}
          >
            {busy ? "Saving…" : "Save as preset"}
          </button>
        </div>
        <div className="modal-actions">
          <button type="button" className="ghost-btn" disabled={disabled} onClick={onCancel}>Cancel</button>
          <button type="button" className="danger-btn" disabled={disabled} onClick={onReset}>Reset & continue</button>
        </div>
      </div>
    </div>
  );
}

/**
 * One track's sequence facts, scannable without opening the track (U10).
 *
 * Deliberately terse and single-line-per-fact: the goal is that twelve of
 * these can be read down the rail at the supported minimum window size. Each
 * chip also carries sr-only context, because "−2.1 LU" on its own is
 * meaningless to someone who cannot see the column it sits in.
 */
export function SequenceRowFacts({ row }: { row: SequenceRow }) {
  const facts: Array<{ key: string; text: string; sr: string; cls?: string }> = [];

  if (row.analysisStatus === "pending") {
    facts.push({
      key: "pending",
      text: "analyzing…",
      sr: "Awaiting analysis.",
      cls: "is-pending",
    });
  } else if (row.sourceLufs != null) {
    facts.push({
      key: "source",
      text: `${row.sourceLufs.toFixed(1)}`,
      sr: `Source loudness ${row.sourceLufs.toFixed(1)} LUFS.`,
    });
  }

  if (row.targetLufs != null) {
    facts.push({
      key: "target",
      text: `→ ${row.targetLufs.toFixed(1)}`,
      sr: `Album target ${row.targetLufs.toFixed(1)} LUFS.`,
    });
  }
  if (row.arcOffsetLabel) {
    facts.push({
      key: "arc",
      text: row.arcOffsetLabel,
      sr: `Flow offset ${row.arcOffsetLabel}.`,
    });
  }
  if (row.roleLabel) {
    facts.push({ key: "role", text: row.roleLabel, sr: `Role: ${row.roleLabel}.` });
  }
  if (row.overridesAlbum) {
    facts.push({
      key: "override",
      text: "Overrides",
      sr: "Overrides the album settings — renders with its own sound and its own target.",
      cls: "is-override",
    });
  }
  if (row.hasConcern) {
    facts.push({
      key: "concern",
      text: "review",
      sr: "This track has a quality check to review.",
      cls: "is-warn",
    });
  }

  if (facts.length === 0) return null;

  return (
    <span className="track-sequence-facts">
      {facts.map((fact) => (
        <span key={fact.key} className={`seq-chip ${fact.cls ?? ""}`}>
          <span aria-hidden>{fact.text}</span>
          <span className="sr-only">{fact.sr}</span>
        </span>
      ))}
    </span>
  );
}

function Sidebar({
  tracks,
  selectedId,
  onSelect,
  onRemove,
  onAdd,
  isAnalyzing,
  analysisProgress,
  mode,
  onReorder,
  albumHeader,
  albumReceipt,
  sequenceRows = [],
}: {
  tracks: ImportedTrack[];
  selectedId: string | null;
  onSelect: (id: string) => void;
  onRemove: (id: string) => void;
  onAdd: () => void;
  isAnalyzing: boolean;
  analysisProgress: { label: string; progress: number } | null;
  mode: "track" | "album";
  onReorder: (fromIndex: number, toIndex: number) => void;
  // `overrideAlbum` used to come in here too, purely to drive the duplicate
  // `★` mark removed in U10(a). `sequenceRows` already carries the same fact
  // as `overridesAlbum`, so the second channel went with the second mark.
  albumHeader?: ReactNode;
  albumReceipt?: ReactNode;
  /// U10 — per-track sequence facts for Album mode. Empty in Track mode.
  sequenceRows?: SequenceRow[];
}) {
  const [dragFromIndex, setDragFromIndex] = useState<number | null>(null);
  const [dragOverIndex, setDragOverIndex] = useState<number | null>(null);

  const albumReorderable = mode === "album";

  const handleDragStart = (
    e: ReactDragEvent<HTMLLIElement>,
    index: number,
  ) => {
    if (!albumReorderable) return;
    setDragFromIndex(index);
    e.dataTransfer.effectAllowed = "move";
    e.dataTransfer.setData("text/plain", String(index));
  };

  const handleDragOver = (
    e: ReactDragEvent<HTMLLIElement>,
    index: number,
  ) => {
    if (!albumReorderable || dragFromIndex === null) return;
    e.preventDefault();
    e.dataTransfer.dropEffect = "move";
    if (dragOverIndex !== index) setDragOverIndex(index);
  };

  const handleDrop = (e: ReactDragEvent<HTMLLIElement>, index: number) => {
    if (!albumReorderable || dragFromIndex === null) return;
    e.preventDefault();
    onReorder(dragFromIndex, index);
    setDragFromIndex(null);
    setDragOverIndex(null);
  };

  const handleDragEnd = () => {
    setDragFromIndex(null);
    setDragOverIndex(null);
  };

  // Sum of every track's duration (seconds) — surfaces the album/queue total
  // alongside the count, the way the reference shows "9 tracks · 42:18".
  const totalSeconds = tracks.reduce(
    (acc, t) => acc + (t.duration_seconds ?? 0),
    0,
  );
  // U10: shipped as "1 tracks". Small, but it is the kind of thing that makes
  // a careful user wonder what else nobody checked.
  const totalLabel =
    totalSeconds > 0
      ? `${trackCountLabel(tracks.length)} · ${formatDuration(totalSeconds)}`
      : trackCountLabel(tracks.length);
  return (
    <aside className="sidebar">
      {/* Album Master: the identity block (title + chips + flow) absorbs the
          "Album order / N tracks" header — each fact shown once (Slice 13b).
          Track Master keeps the plain count header. */}
      {mode === "album" ? (
        albumHeader
      ) : (
        <div className="sidebar-section sidebar-head-strip">
          <div className="sidebar-head-titles">
            <span className="section-label">Tracks</span>
            <span className="sidebar-count">{totalLabel}</span>
          </div>
        </div>
      )}

      <ul className="track-list">
        {tracks.length === 0 && (
          <li className="track-empty">
            {mode === "album"
              ? "No album yet. Drop or add tracks, then drag to reorder."
              : "No tracks yet. Drop or add audio."}
          </li>
        )}
        {tracks.map((t, index) => {
          const classes = ["track-row"];
          if (t.id === selectedId) classes.push("active");
          if (dragFromIndex === index) classes.push("dragging");
          if (dragOverIndex === index && dragFromIndex !== index)
            classes.push("drag-over");
          return (
            <li
              key={t.id}
              className={classes.join(" ")}
              draggable={albumReorderable}
              onDragStart={(e) => handleDragStart(e, index)}
              onDragOver={(e) => handleDragOver(e, index)}
              onDrop={(e) => handleDrop(e, index)}
              onDragEnd={handleDragEnd}
              onDragLeave={() => setDragOverIndex(null)}
            >
              <span className="track-index" aria-hidden>
                {(index + 1).toString().padStart(2, "0")}
              </span>
              <button
                type="button"
                className="track-pick"
                onClick={() => onSelect(t.id)}
                title={t.path}
              >
                {/* U10(a) — a `★ override-mark` used to sit here as well. It
                    said the same thing as the "Overrides" sequence chip a few
                    lines below, in the same row, at the same moment: two marks,
                    one fact. The chip is the survivor because it carries a real
                    accessible sentence rather than a title-attribute star, and
                    it sits with the other per-track scan facts. */}
                <span className="track-name">{t.display_name}</span>
                <span className="track-meta">
                  {t.duration_seconds ? formatDuration(t.duration_seconds) : `.${t.source_format}`}
                </span>
                {/* U10 — sequence overview.
                    Album mode held twelve tracks correctly while showing none
                    of the sequence intelligence: no loudness arc, no role, no
                    target, no per-track status. Everything below already
                    existed in the app's own data; it was simply never shown,
                    so a user had to open each track to read the record. */}
                {mode === "album" && sequenceRows[index] && (
                  <SequenceRowFacts row={sequenceRows[index]} />
                )}
              </button>
              {albumReorderable && (
                <div className="track-reorder-controls" aria-label="Album order controls">
                  <button
                    type="button"
                    className="track-reorder-btn"
                    onClick={() => onReorder(index, index - 1)}
                    disabled={index === 0}
                    aria-label={`Move ${t.display_name} up`}
                    title="Move up"
                  >
                    ↑
                  </button>
                  <button
                    type="button"
                    className="track-reorder-btn"
                    onClick={() => onReorder(index, index + 1)}
                    disabled={index === tracks.length - 1}
                    aria-label={`Move ${t.display_name} down`}
                    title="Move down"
                  >
                    ↓
                  </button>
                </div>
              )}
              <button
                type="button"
                className="track-remove"
                onClick={() => onRemove(t.id)}
                aria-label="Remove track"
                title="Remove"
              >
                ×
              </button>
            </li>
          );
        })}
      </ul>

      {/* Post-export receipt lives at the sidebar bottom, near the export
          rail (Slice 13b placement call — flagged in the deviation log). */}
      {albumReceipt}

      <div className="sidebar-footer">
        {isAnalyzing && (
          <div className="sidebar-status">
            {analysisProgress?.label ?? "Analyzing…"}
          </div>
        )}
        <button
          type="button"
          className="primary sidebar-import-btn"
          onClick={onAdd}
        >
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={2.2} strokeLinecap="round" strokeLinejoin="round" aria-hidden>
            <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" />
            <polyline points="17 8 12 3 7 8" />
            <line x1="12" y1="3" x2="12" y2="15" />
          </svg>
          Import Audio
        </button>
      </div>
    </aside>
  );
}

function TrackMaster({ tm }: { tm: ReturnType<typeof useTrackMaster> }) {
  const track = tm.selectedTrack;
  if (!track) return null;
  return (
    <div className="track-master-console">
      {/* Header owns comparison/preview controls; the waveform deck stays
          focused on transport, waveform, and metering. In Album Master the
          per-track Follows/Override control lives inline in the header badge
          row (Slice 13b) so the center column matches Track Master exactly. */}
      <section className="console-hero">
        <TrackHeader
          track={track}
          analysis={tm.selectedAnalysis}
          lastChecks={
            tm.lastExportReceipt?.trackId === tm.selectedTrackId
              ? tm.lastExportReceipt?.checks
              : undefined
          }
          onReanalyze={() => {
            void tm.reanalyzeTrack(track.id);
          }}
          isAlbum={tm.mode === "album"}
          isOverriding={tm.selectedIsOverriding}
          onToggleOverride={() => tm.toggleOverrideAlbum(track.id)}
          playbackKind={tm.transport.playbackKind}
          volumeMatch={tm.transport.volumeMatch}
          exportLufsPreview={tm.transport.exportLufsPreview}
          isAnalyzing={tm.isAnalyzing}
          analysisProgress={tm.analysisProgress}
          isRendering={tm.isRendering}
          isPlaying={tm.transport.isPlaying}
          renderProgress={tm.renderProgress}
          onPlaybackKindChange={tm.setPlaybackKind}
          onVolumeMatchChange={tm.setVolumeMatch}
          onExportLufsPreviewChange={tm.setExportLufsPreview}
        />
        <div className="wf-deck">
          <Transport
            isPlaying={tm.transport.isPlaying}
            loop={tm.transport.loop}
            durationSec={track.duration_seconds ?? 180}
            currentSec={tm.transport.currentTimeSec}
            loopEnabled={!!tm.selectedRegion}
            onPlayPause={tm.togglePlay}
            onLoopToggle={tm.toggleLoop}
          />
          <WaveformView
            peaks={tm.selectedWaveform}
            isLoading={tm.isLoadingWaveform}
            isAnalyzing={tm.isAnalyzing}
            analysisProgress={tm.analysisProgress}
            currentTimeSec={tm.transport.currentTimeSec}
            durationSec={track.duration_seconds ?? 180}
            region={tm.selectedRegion}
            onSeek={tm.seek}
            onSetRegion={tm.setRegion}
            onClearRegion={tm.clearRegion}
          />
          <div className="wf-deck-meters">
            <MasterOutPanel
              isAnalyzing={tm.isAnalyzing}
              peakDbfs={tm.transport.peakDbfs}
              peakLeftDbfs={tm.transport.peakLeftDbfs}
              peakRightDbfs={tm.transport.peakRightDbfs}
              isPlaying={tm.transport.isPlaying}
              lufsMomentary={tm.transport.lufsMomentary}
              lufsIntegrated={tm.transport.lufsIntegrated}
              meterMode="advanced"
              landingPending={
                tm.landingPending &&
                tm.transport.playbackKind === "master" &&
                tm.transport.isPlaying
              }
            />
          </div>
        </div>
      </section>
      <PresetTiles
        selected={tm.selectedSettings.preset}
        onChange={tm.setPreset}
        savingPreset={tm.savingPreset}
        onSave={tm.saveUserPreset}
      />
      <SignalChain settings={tm.selectedSettings} />
      <div
        className={
          "console-controls" +
          (tm.userPresets.length > 0 ? " has-user-presets" : "")
        }
      >
        <UserPresetSection
          presets={tm.userPresets}
          onDelete={tm.deleteUserPreset}
          onApply={tm.applyUserPreset}
        />
        <Macros
          settings={tm.selectedSettings}
          onIntensity={tm.setIntensity}
          onEq={tm.setEqBand}
          onEqPoint={tm.setEqBandPoint}
          onResetTone={tm.resetToneControls}
          onLoudnessTargetProfile={tm.setLoudnessTargetProfile}
          spectrumDb={tm.transport.spectrumDb}
        />
      </div>
    </div>
  );
}

export function OverrideBanner({
  isOverriding,
  onToggle,
}: {
  isOverriding: boolean;
  onToggle: () => void;
}) {
  const explanation = isOverriding
    ? "This track overrides album intent — its own settings will be applied at export."
    : "This track follows album intent — edits below change the album for every following track.";
  return (
    <section className={"override-banner " + (isOverriding ? "is-overriding" : "follows")}>
      <span
        className={"status-pill override-status" + (isOverriding ? " is-overriding" : "")}
        title={explanation}
      >
        {isOverriding ? "Overrides album" : "Follows album"}
        {/* U9/U10: the explanation was hover-only, so keyboard and
            screen-reader users got the two-word chip and nothing else. */}
        <span className="sr-only">. {explanation}</span>
      </span>
      {/* U9/U10 — one segmented choice.
          This shipped with `disabled` on whichever option was ACTIVE. That is
          backwards twice over: a disabled control leaves the tab order, so a
          keyboard user could not focus their current state to find out what it
          was, and a screen reader announced the selected option as
          "unavailable". Selection is state, not absence of availability, so
          both buttons stay enabled and `aria-pressed` carries the choice.
          Re-selecting the active option is a no-op rather than a toggle. */}
      <div className="override-toggle" role="group" aria-label="Album settings for this track">
        <button
          type="button"
          className={!isOverriding ? "on" : ""}
          aria-pressed={!isOverriding}
          onClick={() => {
            if (isOverriding) onToggle();
          }}
        >
          Follow album
        </button>
        <button
          type="button"
          className={isOverriding ? "on" : ""}
          aria-pressed={isOverriding}
          onClick={() => {
            if (!isOverriding) onToggle();
          }}
        >
          Override
        </button>
      </div>
    </section>
  );
}

export function TrackHeader({
  track,
  analysis,
  lastChecks,
  onReanalyze,
  isAlbum,
  isOverriding,
  onToggleOverride,
  playbackKind,
  volumeMatch,
  exportLufsPreview,
  isAnalyzing,
  analysisProgress,
  isRendering,
  isPlaying,
  renderProgress,
  onPlaybackKindChange,
  onVolumeMatchChange,
  onExportLufsPreviewChange,
}: {
  track: ImportedTrack;
  analysis: AnalysisResult | undefined;
  /// Checks from the most recent export of this track (shown inside Insight).
  lastChecks?: QualityCheck[];
  /// Re-analyze lives in Insight now (2026-08-18), not the right rail.
  onReanalyze?: () => void;
  isAlbum: boolean;
  isOverriding: boolean;
  onToggleOverride: () => void;
  playbackKind: PlaybackKindUI;
  volumeMatch: boolean;
  exportLufsPreview: boolean;
  isAnalyzing: boolean;
  analysisProgress: { label: string; progress: number } | null;
  isRendering: boolean;
  isPlaying: boolean;
  renderProgress: RenderProgressState | null;
  onPlaybackKindChange: (kind: PlaybackKindUI) => void;
  onVolumeMatchChange: (on: boolean) => void;
  onExportLufsPreviewChange: (on: boolean) => void;
}) {
  const insightReview = useInsightReview();
  const chips: { key: string; label: string }[] = [];
  if (track.source_format) {
    chips.push({ key: "fmt", label: track.source_format.toUpperCase() });
  }
  if (track.sample_rate) {
    const sr = track.sample_rate;
    const label = sr >= 1000 ? `${(sr / 1000).toFixed(sr % 1000 === 0 ? 0 : 1)} kHz` : `${sr} Hz`;
    chips.push({ key: "sr", label });
  }
  if (track.channels) {
    chips.push({
      key: "ch",
      label: track.channels === 1 ? "Mono" : track.channels === 2 ? "Stereo" : `${track.channels}ch`,
    });
  }
  if (track.duration_seconds) {
    chips.push({ key: "dur", label: formatDuration(track.duration_seconds) });
  }
  return (
    <section className="track-header">
      <div className="track-header-main">
        <div className="track-header-primary">
          <div className="track-header-title-block">
            <h1 className="track-title">{track.display_name}</h1>
            <div className="track-header-meta-row">
              {/* Slice 13c: identity facts as one quiet line, not boxed chips.
                  Analysis state is not repeated here — an unanalyzed track
                  simply has no Insight row yet, and the busy pills show
                  while analysis runs. */}
              <span className="track-meta-line">
                {chips.map((c) => c.label).join(" · ")}
              </span>
              {isAlbum && (
                <OverrideBanner
                  isOverriding={isOverriding}
                  onToggle={onToggleOverride}
                />
              )}
              <SessionStatus
                isRendering={isRendering}
                isAnalyzing={isAnalyzing}
                isPlaying={isPlaying}
                renderProgress={renderProgress}
                analysisProgress={analysisProgress}
              />
            </div>
          </div>
          <div className="track-header-actions">
            <DeckPreviewOptions
              playbackKind={playbackKind}
              canUseMaster={!!analysis}
              volumeMatch={volumeMatch}
              exportLufsPreview={exportLufsPreview}
              onPlaybackKindChange={onPlaybackKindChange}
              onVolumeMatchChange={onVolumeMatchChange}
              onExportLufsPreviewChange={onExportLufsPreviewChange}
            />
          </div>
        </div>
        {analysis && (
          <SourceInsight
            analysis={analysis}
            lastChecks={lastChecks}
            unreviewed={insightReview.isUnreviewed(analysis)}
            isAnalyzing={isAnalyzing}
            onAcknowledge={() => insightReview.acknowledge(analysis)}
            onReanalyze={onReanalyze}
          />
        )}
      </div>
    </section>
  );
}

function SessionStatus({
  isRendering,
  isAnalyzing,
  renderProgress,
  analysisProgress,
  isPlaying,
}: {
  isRendering: boolean;
  isAnalyzing: boolean;
  renderProgress: RenderProgressState | null;
  analysisProgress: { label: string; progress: number } | null;
  isPlaying: boolean;
}) {
  const progressFraction =
    renderProgress !== null
      ? Math.max(0, Math.min(1, renderProgress.fraction))
      : analysisProgress
        ? Math.max(0, Math.min(1, analysisProgress.progress))
        : null;
  const progressPct =
    progressFraction !== null ? Math.round(progressFraction * 100) : null;
  const statusLabel =
    renderProgress !== null && progressPct !== null
      ? `Rendering ${renderProgress?.kind ?? ""} ${progressPct}%`
      : isAnalyzing && analysisProgress
      ? analysisProgress.label
      : isRendering
      ? "Rendering"
      : isPlaying
      ? "Realtime"
      : "Ready";
  const statusTone =
    progressPct !== null || isRendering || isAnalyzing
      ? "session-status-busy"
      : isPlaying
      ? "session-status-live"
      : "session-status-idle";
  return (
    <span className={`session-status ${statusTone}`}>
      <span className="session-dot" aria-hidden />
      <span className="session-status-text">{statusLabel}</span>
      {progressPct !== null && (
        <span className="session-progress" aria-hidden>
          <span
            className="session-progress-fill"
            style={{ width: `${progressPct}%` }}
          />
        </span>
      )}
    </span>
  );
}

function DeckPreviewOptions({
  playbackKind,
  canUseMaster,
  volumeMatch,
  exportLufsPreview,
  onPlaybackKindChange,
  onVolumeMatchChange,
  onExportLufsPreviewChange,
}: {
  playbackKind: PlaybackKindUI;
  canUseMaster: boolean;
  volumeMatch: boolean;
  exportLufsPreview: boolean;
  onPlaybackKindChange: (kind: PlaybackKindUI) => void;
  onVolumeMatchChange: (on: boolean) => void;
  onExportLufsPreviewChange: (on: boolean) => void;
}) {
  return (
    <div className="track-preview-toolbar">
      <div className="track-toolbar-group track-toolbar-group-compare" aria-label="Playback source">
        {/* U10 — Advanced's A/B toggle carried its selected state as a CSS
            class ALONE. Standard's equivalent got `aria-pressed` and an
            accessible disabled reason; the Advanced one, on the app's most-used
            control, was missed. Colour was the only signal that you were
            listening to the master, and the disabled reason was hover-only —
            the same defect U10 already fixed everywhere else. Found by the S-E1
            rapid-A/B scenario, which could not read which side was selected. */}
        <div className="ab-toggle">
          <button
            type="button"
            aria-pressed={playbackKind === "source"}
            className={playbackKind === "source" ? "on" : ""}
            onClick={() => onPlaybackKindChange("source")}
          >
            Original
          </button>
          <button
            type="button"
            aria-pressed={playbackKind === "master"}
            className={playbackKind === "master" ? "on" : ""}
            disabled={!canUseMaster}
            title={!canUseMaster ? "Analyze this track before using Mastered playback." : undefined}
            aria-describedby={
              !canUseMaster ? "advanced-ab-master-disabled-reason" : undefined
            }
            onClick={() => onPlaybackKindChange("master")}
          >
            Mastered
          </button>
          <DisabledReason
            id="advanced-ab-master-disabled-reason"
            reason={
              !canUseMaster
                ? "Analyze this track before using Mastered playback."
                : undefined
            }
          />
        </div>
      </div>
      <div className="track-toolbar-group track-toolbar-group-options" aria-label="Preview options">
        <button
          type="button"
          className={`toolbar-toggle ${volumeMatch ? "is-on" : ""}`}
          aria-pressed={volumeMatch}
          title="Aligns playback loudness for fair tone comparison. Export level is unchanged."
          onClick={() => onVolumeMatchChange(!volumeMatch)}
        >
          <span className="toolbar-toggle-box" aria-hidden />
          <span>Volume Match</span>
        </button>
        <button
          type="button"
          className={`toolbar-toggle ${exportLufsPreview ? "is-on" : ""}`}
          aria-pressed={exportLufsPreview}
          title="Previews export LUFS landing during Mastered playback. The readout settles over a few seconds on heavier chains."
          onClick={() => onExportLufsPreviewChange(!exportLufsPreview)}
        >
          <span className="toolbar-toggle-box" aria-hidden />
          <span>Preview LUFS</span>
        </button>
      </div>
    </div>
  );
}

/// Plain-English commentary on the analysis numbers — one line per dimension.
/// UI_LAYOUT_REVISION_1600x940 L1 — slim Transport.
/// Original/Mastered + Volume Match moved to TrackHeader (their natural
/// home alongside the track title and analysis chips), so this component
/// is now only play/pause + time + loop. Rendered as the left column of
/// the new `.wf-deck` waveform module rather than its own row.
function Transport({
  isPlaying,
  loop,
  durationSec,
  currentSec,
  loopEnabled,
  onPlayPause,
  onLoopToggle,
}: {
  isPlaying: boolean;
  loop: boolean;
  durationSec: number;
  currentSec: number;
  loopEnabled: boolean;
  onPlayPause: () => void;
  onLoopToggle: () => void;
}) {
  return (
    <div className="wf-deck-transport">
      <button
        type="button"
        className="play-btn"
        onClick={onPlayPause}
        aria-label={isPlaying ? "Pause" : "Play"}
      >
        <PlayPauseGlyph playing={isPlaying} size={26} />
      </button>
      <span className="time">
        {formatTime(currentSec)}
        <span className="dim"> / {formatTime(durationSec)}</span>
      </span>
      <button
        type="button"
        className={"icon-btn " + (loop ? "on" : "")}
        onClick={onLoopToggle}
        disabled={!loopEnabled}
        title={
          loopEnabled
            ? "Loop region"
            : "Shift+drag the waveform to define a region first"
        }
      >
        ⟲
      </button>
    </div>
  );
}

// Per-preset accent color. Drives the tile's character glow so the imagery
// feels integrated with the tile rather than pasted on. Matches the color
// language of the generated 3D imagery. Lives in PresetIcon.tsx so the
// Standard style tiles share the exact same hues (visual-parity pass).

/// Keep the WebView at a deterministic 100% app zoom on every fresh launch.
/// The app is now composed for a native 1920x1080 window, so carrying an
/// in-app CSS zoom on top of Windows display scaling makes the console feel
/// blurry and causes breakpoint math to lie. Ctrl+0 still re-applies 100% if
/// the WebView ever inherits a zoom state from the shell.
function useWebviewZoomShortcuts() {
  useEffect(() => {
    const apply = () => {
      // CSS `zoom:` is non-standard but supported by Chromium / WebView2.
      (document.documentElement.style as CSSStyleDeclaration & {
        zoom?: string;
      }).zoom = "1";
    };
    apply();
    const onKey = (event: KeyboardEvent) => {
      if (!(event.ctrlKey || event.metaKey)) return;
      const target = event.target as HTMLElement | null;
      const inEditableField =
        target && (target.tagName === "INPUT" || target.tagName === "TEXTAREA" || target.isContentEditable);
      if (inEditableField) return;
      if (event.key === "0") {
        event.preventDefault();
        apply();
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, []);
}

export function PresetTiles({
  selected,
  onChange,
  savingPreset,
  onSave,
}: {
  selected: Preset;
  onChange: (preset: Preset) => void;
  savingPreset: boolean;
  onSave: (name: string) => Promise<boolean>;
}) {
  const [name, setName] = useState("");
  const [isSaving, setIsSaving] = useState(false);

  const handleSave = async () => {
    if (!name.trim()) return;
    // Only clear the form once the save actually succeeds — otherwise a failed
    // save (e.g. empty/duplicate name rejected by the backend) silently wipes
    // the in-progress name and collapses the input.
    const saved = await onSave(name);
    if (saved) {
      setName("");
      setIsSaving(false);
    }
  };

  return (
    <section className="presets">
      <div className="section-head">
        <span className="section-label">Styles</span>
        {!isSaving ? (
          <button
            type="button"
            className="preset-save-plus"
            onClick={() => setIsSaving(true)}
            aria-label="Save current settings as preset"
            title="Save current settings as preset"
          >
            +
          </button>
        ) : (
          <form
            className="preset-save-inline"
            onSubmit={(e) => {
              e.preventDefault();
              handleSave();
            }}
          >
            <input
              type="text"
              className="preset-save-name"
              placeholder="Preset name"
              value={name}
              onChange={(e) => setName(e.target.value)}
              maxLength={64}
              disabled={savingPreset}
              autoFocus
            />
            <button
              type="submit"
              className="preset-save-confirm"
              disabled={savingPreset || !name.trim()}
              aria-label="Save preset"
              title="Save preset"
            >
              {savingPreset ? "…" : "Save"}
            </button>
            <button
              type="button"
              className="preset-save-cancel"
              onClick={() => {
                setName("");
                setIsSaving(false);
              }}
              aria-label="Cancel preset save"
              title="Cancel"
            >
              ×
            </button>
          </form>
        )}
      </div>
      <div className="tile-row">
        {PRESET_OPTIONS.map((p) => {
          const active = isPresetActive(selected, p.value);
          const accent = PRESET_ACCENT[p.value.kind];
          // U9: selected state was visual only (`.active`), and the character
          // blurb was reachable only by hovering. Both now reach assistive tech:
          // `aria-pressed` carries the state, and the blurb is described text
          // rather than a tooltip. `title` stays for pointer users.
          const blurbId = `preset-blurb-${p.value.kind}`;
          return (
            <button
              key={p.label}
              type="button"
              className={"tile " + (active ? "active" : "")}
              style={{ ["--tile-accent" as never]: accent }}
              onClick={() => onChange(p.value)}
              aria-pressed={active}
              aria-describedby={blurbId}
              title={`${p.label} — ${p.blurb}`}
            >
              <PresetIcon kind={p.value.kind} className="tile-icon" />
              <span className="tile-label">{p.label}</span>
              <span id={blurbId} className="sr-only">
                {p.blurb}
              </span>
            </button>
          );
        })}
      </div>
    </section>
  );
}

export function UserPresetSection({
  presets,
  onDelete,
  onApply,
}: {
  presets: UserPreset[];
  onDelete: (id: string) => void;
  onApply: (preset: UserPreset) => void;
}) {
  if (presets.length === 0) return null;

  return (
    <section className="user-presets">
      <div className="user-preset-row">
        <span className="section-label user-preset-row-label">MY PRESETS</span>
        {presets.map((p) => (
          <div key={p.id} className="user-preset-chip">
            <button
              type="button"
              className="user-preset-apply"
              onClick={() => onApply(p)}
              title={`Apply "${p.name}"`}
            >
              {p.name}
              <span className="user-preset-kind"> · {p.kind}</span>
            </button>
            <button
              type="button"
              className="user-preset-delete"
              onClick={() => onDelete(p.id)}
              aria-label={`Delete preset ${p.name}`}
              title="Delete preset"
            >
              ×
            </button>
          </div>
        ))}
      </div>
    </section>
  );
}

function isPresetActive(a: Preset, b: Preset): boolean {
  if (a.kind === "custom" && b.kind === "custom") return a.id === b.id;
  return a.kind === b.kind;
}

export function Macros({
  settings,
  onIntensity,
  onEq,
  onEqPoint,
  onResetTone,
  onLoudnessTargetProfile,
  spectrumDb,
}: {
  settings: MasteringSettings;
  onIntensity: (v: number) => void;
  // Visual EQ drag-only nodes share the same setter as the knob-bound bands.
  onEq: (
    band: "sub" | "low" | "low-mid" | "mid" | "high-mid" | "high" | "sparkle",
    db: number,
  ) => void;
  // 2026-08-18 — a Visual EQ node drag sets gain AND frequency in one
  // mutation (see useTrackMaster.setEqBandPoint for why it must be one).
  onEqPoint?: (
    band: "sub" | "low" | "low-mid" | "mid" | "high-mid" | "high" | "sparkle",
    db: number,
    hz: number,
  ) => void;
  // Fast reset for this whole area — flatten intensity + every EQ band.
  onResetTone: () => void;
  onLoudnessTargetProfile: (profileId: string) => void;
  // L4b — live FFT spectrum forwarded from PlaybackTick. Empty array
  // means no spectrum yet (idle / Original playback); VisualEqPanel
  // simply omits the spectrum layer in that case.
  spectrumDb: number[];
}) {
  return (
    <section className="macros knobs-row">
      <div className="intensity-block">
        <span className="section-label">INTENSITY</span>
        <Knob
          label=""
          // U9: named by the INTENSITY section label visually, but the control
          // itself shipped with no accessible name at all.
          ariaLabel="Intensity"
          valueText={(v) => `${Math.round(v * 100)} percent, ${intensityLabel(v)}`}
          size="lg"
          tone={PRESET_TONE[settings.preset.kind]}
          value={settings.intensity}
          min={0}
          max={1}
          step={0.01}
          defaultValue={0.5}
          format={(v) => `${Math.round(v * 100)}%`}
          caption={intensityLabel(settings.intensity)}
          onChange={onIntensity}
          centerValue
        />
      </div>
      <div className="tone-shape-block">
        <span className="section-label">TONE SHAPE</span>
        <div className="tone-shape-knobs">
          <Knob
            label="Low"
            size="md"
            tone="cyan"
            value={settings.eq_low_db}
            min={-12}
            max={12}
            step={0.1}
            defaultValue={0}
            format={(v) => `${v > 0 ? "+" : ""}${v.toFixed(1)} dB`}
            onChange={(v) => onEq("low", v)}
          />
          <Knob
            label="Mid"
            size="md"
            tone="purple"
            value={settings.eq_mid_db}
            min={-12}
            max={12}
            step={0.1}
            defaultValue={0}
            format={(v) => `${v > 0 ? "+" : ""}${v.toFixed(1)} dB`}
            onChange={(v) => onEq("mid", v)}
          />
          <Knob
            label="High"
            size="md"
            tone="blue"
            value={settings.eq_high_db}
            min={-12}
            max={12}
            step={0.1}
            defaultValue={0}
            format={(v) => `${v > 0 ? "+" : ""}${v.toFixed(1)} dB`}
            onChange={(v) => onEq("high", v)}
          />
        </div>
      </div>
      {/* UI_LAYOUT_REVISION_1600x940 L4a — EQ promotes out of Tone Shape
          into its own deck cell so it reads as the workspace's primary
          tone-shaping surface. Takes the 1fr column in the deck row;
          the three precision knobs sit to its left, Loudness to its
          right. Compact mode stays on (no header, no node labels),
          but the panel itself now has the horizontal real estate to
          show the curve, nodes, and grid cleanly. */}
      <div className="equalizer-block">
        <div className="equalizer-block-head">
          <span className="section-label">EQUALIZER (Dynamic)</span>
          <PanelResetButton
            label="Reset intensity & EQ to flat"
            onClick={onResetTone}
            disabled={isToneFlat(settings)}
          />
        </div>
        <VisualEqPanel
          settings={settings}
          onEq={onEq}
          onEqPoint={onEqPoint}
          compact
          spectrumDb={spectrumDb}
        />
      </div>
      <LoudnessTarget
        settings={settings}
        onProfileSelect={onLoudnessTargetProfile}
      />
    </section>
  );
}

// LOUDNESS_PROFILES + the effective-target / profileId / displayText
// computation are sourced from src/lib/effective-settings (Vitest-tested
// pure helpers). Single source of truth for both the rendered dropdown
// options AND the readout the LoudnessTarget block shows.

export function LoudnessTarget({
  settings,
  onProfileSelect,
}: {
  settings: MasteringSettings;
  onProfileSelect: (profileId: string) => void;
}) {
  // Display state from the Vitest-tested pure helper — single
  // source of truth for the readout's effective target, dropdown
  // selected value, and formatted display string.
  const { profileId, displayText } = loudnessTargetDisplay(settings);

  const handleProfileChange = (id: string) => {
    onProfileSelect(id);
  };

  return (
    <div className="loudness-target-block">
      <span className="section-label">LOUDNESS TARGET</span>
      <div className="loudness-readout">
        <span className="loudness-number">{displayText}</span>
        <span className="loudness-unit">LUFS</span>
      </div>
      <select
        className="loudness-profile-select"
        value={profileId}
        onChange={(e) => handleProfileChange(e.target.value)}
      >
        {LOUDNESS_PROFILES.map((p) => (
          <option key={p.id} value={p.id}>{p.label}</option>
        ))}
        {profileId === "custom" && (
          <option value="custom">Custom ({displayText} LUFS)</option>
        )}
      </select>
    </div>
  );
}

function formatTime(sec: number): string {
  const m = Math.floor(sec / 60);
  const s = Math.floor(sec % 60);
  return `${m}:${s.toString().padStart(2, "0")}`;
}

// Re-export so the three AdvancedPanel test files keep importing from
// "./App" (their pinned interface) after the B4.2 extraction.
export { AdvancedPanel } from "./components/AdvancedPanel";

export default App;
