import {
  useEffect,
  useState,
  type DragEvent as ReactDragEvent,
} from "react";
import { useTrackMaster } from "./hooks/useTrackMaster";
import { useNavigationMachine } from "./hooks/useNavigationMachine";
import { StandardView } from "./components/StandardView";
import { hasNonManagedEdits } from "./lib/standard-managed";
import { PresetIcon, PRESET_ACCENT } from "./components/PresetIcon";
import { RightRail, MasterOutPanel } from "./components/RightRail";
import { VisualEqPanel } from "./components/VisualEqPanel";
import { AlbumPanel } from "./components/AlbumPanel";
import { Knob, intensityLabel } from "./components/Knob";
import { SignalChain } from "./components/SignalChain";
import { EmptyState } from "./components/EmptyState";
import { StatusDot } from "./components/StatusDot";
import { Toast } from "./components/Toast";
import { ChromeDialog } from "./components/ChromeDialog";
import { SettingsGroup } from "./components/SettingsGroup";
import { AdvancedPanel } from "./components/AdvancedPanel";
import { PanelResetButton } from "./components/fields";
import { ExportReceiptCard } from "./components/ExportReceiptCard";
import { WaveformView } from "./components/Waveform";
import type {
  AnalysisResult,
  ImportedTrack,
  MasteringSettings,
  Preset,
  UserPreset,
} from "./bindings";
import type { PlaybackKindUI } from "./hooks/useTrackMaster";
import { LOUDNESS_PROFILES, loudnessTargetDisplay } from "./lib/effective-settings";
import { HELP_SECTIONS, SETTINGS_GROUPS } from "./lib/chrome-content";
import { markGuideFinished, resetGuide } from "./lib/first-run-guide";
import { isToneFlat } from "./lib/tone-reset";
import "./App.css";

const PRESET_OPTIONS: { value: Preset; label: string; blurb: string }[] = [
  { value: { kind: "universal" }, label: "Universal", blurb: "Safe, well-rounded default" },
  { value: { kind: "clarity" }, label: "Clarity", blurb: "Vocal/upper-mid definition" },
  { value: { kind: "tape" }, label: "Tape", blurb: "Saturation, glue, softer top" },
  { value: { kind: "spatial" }, label: "Spatial", blurb: "Width and depth" },
  { value: { kind: "oomph" }, label: "Oomph", blurb: "Low-end weight, punch" },
  { value: { kind: "warmth" }, label: "Warmth", blurb: "Fuller, smoother body" },
  { value: { kind: "punch" }, label: "Punch", blurb: "Transient impact" },
  { value: { kind: "loud" }, label: "Loud", blurb: "Density + level, with safety" },
];

function App() {
  const tm = useTrackMaster();
  const [chromePanel, setChromePanel] = useState<"settings" | "help" | null>(null);
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

  // WYSIWYG: the live Mastered audition equals the export only when the
  // loudness landing + limiter are applied in real time. Standard forces
  // that via the INTERNAL flag — it never touches the user-facing Advanced
  // `Preview LUFS` toggle (see Task 5 step 3g).
  useEffect(() => {
    if (view === null) return;
    tm.setForceWysiwyg(view === "standard");
  }, [view, tm.setForceWysiwyg]);

  return (
    <div className="app-root">
      <TopHeader
        mode={tm.mode}
        onModeChange={tm.setMode}
        onSaveProject={tm.saveProjectAs}
        onOpenProject={tm.openProjectFromDisk}
        onOpenSettings={() => setChromePanel("settings")}
        onOpenHelp={() => setChromePanel("help")}
        viewMode={view === "advanced" ? "advanced" : "standard"}
        onEnterAdvanced={() => {
          // Entering Advanced from the chrome unmounts StandardView without
          // notice — end the first-run guide here so it never re-appears.
          markGuideFinished(globalThis.localStorage, "done");
          setView("advanced");
        }}
        onBackToStandard={nav.requestBackToStandard}
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
        overrideAlbum={tm.overrideAlbum}
      />
      <main className={"workspace" + (tm.mode === "album" ? " workspace-album" : "")}>
        {tm.mode === "album" && tm.tracks.length > 0 && (
          <AlbumPanel
            tracks={tm.tracks}
            albumArcKind={tm.albumArcKind}
            albumIntensity={tm.albumIntensity}
            albumTitle={tm.albumTitle}
            albumRendering={tm.albumRendering}
            albumExportReport={tm.albumExportReport}
            albumSampleRate={tm.albumSampleRate}
            albumBitDepth={tm.albumBitDepth}
            onAlbumArc={tm.setAlbumArc}
            onAlbumIntensity={tm.setAlbumIntensity}
            onAlbumTitle={tm.setAlbumTitle}
            onExportAlbum={tm.exportAlbumPlan}
            onAlbumSampleRate={tm.setAlbumSampleRate}
            onAlbumBitDepth={tm.setAlbumBitDepth}
          />
        )}
        {tm.selectedTrack ? (
          <TrackMaster tm={tm} />
        ) : (
          <EmptyState onAdd={tm.openImportDialog} />
        )}
      </main>
      <RightRail
        analysis={tm.selectedAnalysis}
        lastChecks={tm.lastExportReceipt?.checks}
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
              showDeliveryFormat={tm.mode !== "album"}
              adaptiveReadout={tm.guardrailReadout}
              albumMode={tm.mode === "album"}
            />
          ) : undefined
        }
        canExport={!!tm.selectedAnalysis}
        isExporting={tm.isExporting}
        isRendering={tm.isRendering}
        previewStale={tm.previewStale}
        canRenderPreview={!!tm.selectedTrack}
        onUpdatePreview={tm.updatePreview}
        onExport={tm.exportMaster}
      />
        </>
      )}
      {view === "standard" && tm.selectedTrack && (
        <StandardView tm={tm} onEnterAdvanced={() => setView("advanced")} />
      )}
      {view === "standard" && !tm.selectedTrack && <EmptyState onAdd={tm.openImportDialog} />}
      {tm.isDragOver && (
        <div className="drop-overlay" aria-hidden>
          <div className="drop-overlay-card">
            <div className="drop-overlay-title">Drop to import</div>
            <div className="drop-overlay-hint">
              WAV · AIFF · FLAC · MP3 · M4A · AAC · OGG · Opus
            </div>
          </div>
        </div>
      )}
      {tm.error ? (
        <Toast message={tm.error} tone="danger" onClose={tm.clearError} />
      ) : tm.projectFeedback ? (
        <Toast
          message={tm.projectFeedback.message}
          tone={tm.projectFeedback.tone}
          onClose={tm.clearProjectFeedback}
        />
      ) : null}
      {tm.lastExportReceipt && view === "advanced" && (
        <ExportReceiptCard
          receipt={tm.lastExportReceipt}
          onClose={tm.clearExportReceipt}
        />
      )}
      {chromePanel === "settings" && (
        <SettingsPanel onClose={() => setChromePanel(null)} />
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
            nav.completeReturn();
          }}
          onSaveAsPreset={async (name) => {
            // Only reset + switch if the save actually succeeded — the save
            // is async; never discard the user's edits when the write failed.
            const ok = await tm.saveUserPreset(name);
            if (ok) {
              tm.resetToStandardManaged();
              nav.completeReturn();
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
  const analysis = tm.selectedAnalysis;
  const peak = tm.transport.peakDbfs;
  const liveLufs = tm.transport.lufsIntegrated;
  const isPlaying = tm.transport.isPlaying;

  const peakDisplay = isPlaying && peak > -80 ? `${peak.toFixed(1)} dBFS` : "—";
  const lufsDisplay = isPlaying && liveLufs > -80 ? `${liveLufs.toFixed(1)} LUFS` : "—";

  let processing: { tone: "idle" | "busy" | "ok"; text: string };
  if (tm.isExporting) {
    processing = { tone: "busy", text: "Exporting…" };
  } else if (tm.isRendering) {
    processing = { tone: "busy", text: "Rendering audit…" };
  } else if (tm.isAnalyzing) {
    processing = { tone: "busy", text: tm.analysisProgress?.label ?? "Analyzing…" };
  } else if (tm.isLoadingWaveform) {
    processing = { tone: "busy", text: "Decoding…" };
  } else if (tm.selectedTrack) {
    processing = { tone: "ok", text: "Ready" };
  } else {
    processing = { tone: "idle", text: "Idle" };
  }

  return (
    <footer className="bottom-status">
      <div className="bottom-status-left">
        {/* L5 polish — terser labels so the bottom bar reads quiet
            rather than debug-flavored. Full text in tooltip if needed. */}
        <StatusDot
          tone={tm.selectedTrack ? (analysis ? "ok" : "warn") : "idle"}
          label={
            !tm.selectedTrack
              ? "No track"
              : analysis
              ? "Analyzed"
              : "Analyzing"
          }
        />
        <StatusDot
          tone={
            tm.lastExportReceipt
              ? tm.lastExportReceipt.checks.some((c) => c.level === "critical")
                ? "bad"
                : tm.lastExportReceipt.checks.some((c) => c.level === "warning")
                ? "warn"
                : "ok"
              : "idle"
          }
          label={
            tm.lastExportReceipt
              ? tm.lastExportReceipt.checks.some((c) => c.level === "critical")
                ? "Quality failed"
                : tm.lastExportReceipt.checks.some((c) => c.level === "warning")
                ? "Quality review"
                : "Quality OK"
              : "Quality —"
          }
        />
      </div>
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
        <span className="status-processing-label">Processing</span>
        <span className={`status-pill status-${processing.tone === "busy" ? "warn" : processing.tone === "ok" ? "ok" : "idle"}`}>
          {processing.text}
        </span>
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

export function SettingsPanel({ onClose }: { onClose: () => void }) {
  return (
    <ChromeDialog title="Settings" eyebrow="Current defaults" onClose={onClose}>
      <div className="settings-grid">
        {SETTINGS_GROUPS.map((group) => (
          <SettingsGroup key={group.title} title={group.title} rows={group.rows} />
        ))}
        <div className="settings-actions">
          <button
            type="button"
            className="ghost-btn"
            onClick={() => resetGuide(globalThis.localStorage)}
          >
            Show first-run tips again
          </button>
        </div>
      </div>
    </ChromeDialog>
  );
}

export function HelpPanel({ onClose }: { onClose: () => void }) {
  return (
    <ChromeDialog title="Help" eyebrow="Track Master guide" onClose={onClose}>
      <div className="help-sections">
        {HELP_SECTIONS.map(([title, body]) => (
          <section className="help-section" key={title}>
            <h3>{title}</h3>
            <p>{body}</p>
          </section>
        ))}
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
          Back to Standard resets your manual edits to the preset&apos;s clean
          sound. Save them as a preset first?
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
  overrideAlbum,
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
  overrideAlbum: Set<string>;
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
  const totalLabel = totalSeconds > 0 ? `${tracks.length} tracks · ${formatDuration(totalSeconds)}` : `${tracks.length} tracks`;
  return (
    <aside className="sidebar">
      <div className="sidebar-section sidebar-head-strip">
        <div className="sidebar-head-titles">
          <span className="section-label">
            {mode === "album" ? "Album order" : "Tracks"}
          </span>
          <span className="sidebar-count">{totalLabel}</span>
        </div>
      </div>

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
                <span className="track-name">
                  {t.display_name}
                  {mode === "album" && overrideAlbum.has(t.id) && (
                    <span className="override-mark" title="Overrides album intent">
                      ★
                    </span>
                  )}
                </span>
                <span className="track-meta">
                  {t.duration_seconds ? formatDuration(t.duration_seconds) : `.${t.source_format}`}
                </span>
              </button>
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
    <div className={"track-master-console" + (tm.mode === "album" ? " is-album" : "")}>
      {tm.mode === "album" && (
        <OverrideBanner
          isOverriding={tm.selectedIsOverriding}
          onToggle={() => tm.toggleOverrideAlbum(track.id)}
        />
      )}
      {/* Header owns comparison/preview controls; the waveform deck stays
          focused on transport, waveform, and metering. */}
      <section className="console-hero">
        <TrackHeader
          track={track}
          analysis={tm.selectedAnalysis}
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
          onResetTone={tm.resetToneControls}
          onLoudnessTargetProfile={tm.setLoudnessTargetProfile}
          spectrumDb={tm.transport.spectrumDb}
        />
      </div>
    </div>
  );
}

function OverrideBanner({
  isOverriding,
  onToggle,
}: {
  isOverriding: boolean;
  onToggle: () => void;
}) {
  return (
    <section className={"override-banner " + (isOverriding ? "is-overriding" : "follows")}>
      <div className="override-info">
        <span className="section-label">Album adaptation</span>
        <span className="override-state">
          {isOverriding
            ? "This track overrides album intent · its own settings will be applied at export"
            : "This track follows album intent · edits below change the album for every following track"}
        </span>
      </div>
      <div className="override-toggle">
        <button
          type="button"
          className={!isOverriding ? "on" : ""}
          aria-pressed={!isOverriding}
          onClick={onToggle}
          disabled={!isOverriding}
        >
          Follow album
        </button>
        <button
          type="button"
          className={isOverriding ? "on" : ""}
          aria-pressed={isOverriding}
          onClick={onToggle}
          disabled={isOverriding}
        >
          Override
        </button>
      </div>
    </section>
  );
}

function TrackHeader({
  track,
  analysis,
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
  playbackKind: PlaybackKindUI;
  volumeMatch: boolean;
  exportLufsPreview: boolean;
  isAnalyzing: boolean;
  analysisProgress: { label: string; progress: number } | null;
  isRendering: boolean;
  isPlaying: boolean;
  renderProgress: { fraction: number; kind: "preview" | "master" | "album" } | null;
  onPlaybackKindChange: (kind: PlaybackKindUI) => void;
  onVolumeMatchChange: (on: boolean) => void;
  onExportLufsPreviewChange: (on: boolean) => void;
}) {
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
              <div className="track-meta-chips">
                {chips.map((c) => (
                  <span key={c.key} className="meta-chip">{c.label}</span>
                ))}
              </div>
              {analysis && <span className="status-pill status-ok">Analyzed</span>}
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
              volumeMatch={volumeMatch}
              exportLufsPreview={exportLufsPreview}
              onPlaybackKindChange={onPlaybackKindChange}
              onVolumeMatchChange={onVolumeMatchChange}
              onExportLufsPreviewChange={onExportLufsPreviewChange}
            />
          </div>
        </div>
        {analysis && <AnalysisSummary analysis={analysis} />}
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
  renderProgress: { fraction: number; kind: "preview" | "master" | "album" } | null;
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
  volumeMatch,
  exportLufsPreview,
  onPlaybackKindChange,
  onVolumeMatchChange,
  onExportLufsPreviewChange,
}: {
  playbackKind: PlaybackKindUI;
  volumeMatch: boolean;
  exportLufsPreview: boolean;
  onPlaybackKindChange: (kind: PlaybackKindUI) => void;
  onVolumeMatchChange: (on: boolean) => void;
  onExportLufsPreviewChange: (on: boolean) => void;
}) {
  return (
    <div className="track-preview-toolbar">
      <div className="track-toolbar-group track-toolbar-group-compare" aria-label="Playback source">
        <div className="ab-toggle">
          <button
            type="button"
            className={playbackKind === "source" ? "on" : ""}
            onClick={() => onPlaybackKindChange("source")}
          >
            Original
          </button>
          <button
            type="button"
            className={playbackKind === "master" ? "on" : ""}
            onClick={() => onPlaybackKindChange("master")}
          >
            Mastered
          </button>
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

function formatDuration(seconds: number): string {
  const total = Math.max(0, Math.round(seconds));
  const m = Math.floor(total / 60);
  const s = total % 60;
  return `${m}:${s.toString().padStart(2, "0")}`;
}

/// Plain-English commentary on the analysis numbers — one line per dimension.
/// Phase 12.1 Dan feedback: "A more prominent assessment of what was done
/// after analyzation, even perhaps in plain English in a dropdown underneath
/// the stats." Each line maps a numeric range to a short, non-alarmist phrase.
function AnalysisSummary({ analysis }: { analysis: AnalysisResult }) {
  const lines: string[] = [];

  // Loudness commentary.
  const lufs = analysis.lufs_integrated;
  if (lufs > -8) {
    lines.push(
      `Source very loud at ${lufs.toFixed(1)} LUFS — may sound flat vs streaming references.`,
    );
  } else if (lufs > -12) {
    lines.push(`Source loud at ${lufs.toFixed(1)} LUFS — streaming-loud territory.`);
  } else if (lufs > -16) {
    lines.push(`Source ${lufs.toFixed(1)} LUFS — close to typical streaming targets.`);
  } else {
    lines.push(`Source ${lufs.toFixed(1)} LUFS — conservative loudness, room to push.`);
  }

  // Dynamics commentary.
  const dr = analysis.dynamic_range_lu;
  if (dr < 5) {
    lines.push(`Highly compressed (DR ${dr.toFixed(1)} LU) — limited dynamic contrast.`);
  } else if (dr < 8) {
    lines.push(`Moderately compressed (DR ${dr.toFixed(1)} LU).`);
  } else {
    lines.push(`Healthy dynamic range (DR ${dr.toFixed(1)} LU).`);
  }

  // Spectral commentary.
  const high = analysis.spectral_balance.high;
  const low = analysis.spectral_balance.low;
  if (high > 0.35) {
    lines.push("Bright, presence-forward spectrum.");
  } else if (high < 0.18) {
    lines.push("Dark, low-mid-focused spectrum.");
  } else if (low > 0.45) {
    lines.push("Low-heavy spectrum.");
  } else {
    lines.push("Balanced spectrum.");
  }

  // Stereo width commentary.
  const w = analysis.stereo_width;
  if (w > 0.7) {
    lines.push("Wide stereo image.");
  } else if (w < 0.3) {
    lines.push("Narrow / mono-leaning stereo image.");
  } else {
    lines.push("Standard stereo image.");
  }

  // True peak commentary.
  const tp = analysis.true_peak_dbtp;
  if (tp > -0.1) {
    lines.push(`True peak ${tp.toFixed(2)} dBTP — at or above the digital ceiling, risky.`);
  } else if (tp > -1.0) {
    lines.push(`True peak ${tp.toFixed(2)} dBTP — fine digitally, lossy codecs may overshoot.`);
  } else {
    lines.push(`True peak ${tp.toFixed(2)} dBTP — comfortable headroom.`);
  }

  // Most actionable headline = source loudness. Commentary stays collapsed so
  // the insight can sit on the metadata row without becoming a second header.
  const [headline, ...rest] = lines;
  const expandedLines = [headline, ...rest];
  return (
    <details className="analysis-summary">
      <summary>
        <span className="analysis-summary-text">
          <span className="analysis-summary-eyebrow">Insight</span>
          <span className="analysis-summary-headline">{headline}</span>
        </span>
      </summary>
      {expandedLines.length > 0 && (
        <ul>
          {expandedLines.map((line, i) => (
            <li key={i}>{line}</li>
          ))}
        </ul>
      )}
    </details>
  );
}

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
        {isPlaying ? "⏸" : "▶"}
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
  onSave: (name: string) => void;
}) {
  const [name, setName] = useState("");
  const [isSaving, setIsSaving] = useState(false);

  const handleSave = () => {
    if (!name.trim()) return;
    onSave(name);
    setName("");
    setIsSaving(false);
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
          return (
            <button
              key={p.label}
              type="button"
              className={"tile " + (active ? "active" : "")}
              style={{ ["--tile-accent" as never]: accent }}
              onClick={() => onChange(p.value)}
              title={`${p.label} — ${p.blurb}`}
            >
              <PresetIcon kind={p.value.kind} className="tile-icon" />
              <span className="tile-label">{p.label}</span>
              <span className="tile-blurb">{p.blurb}</span>
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
          size="lg"
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
