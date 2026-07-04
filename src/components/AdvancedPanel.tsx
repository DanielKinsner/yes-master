import { useState } from "react";
import { Knob } from "./Knob";
import { GainField, NumberField, PanelResetButton, SelectField } from "./fields";
import type {
  AnalysisResult,
  CompressionMode,
  CompressionPlan,
  DeliveryProfile,
  GuardrailReadout,
  MasteringSettings,
  Preset,
} from "../bindings";
import {
  ADAPTIVE_STRENGTH_DEFAULT,
  DELIVERY_PROFILE_DISPLAY,
  DELIVERY_PROFILE_TARGET_LUFS,
} from "../bindings";
import {
  effectiveBitDepth,
  effectiveCeilingDbtp,
  effectiveSampleRate,
  loudnessTargetDisplay,
} from "../lib/effective-settings";
import { compressorAutoReadouts } from "../lib/compressor-auto";
import { adaptiveReadoutEnabled } from "../lib/debug-flags";

/// UI_LAYOUT_REVISION_1600x940 L3 — AdvancedPanel renders four
/// separate rail cards (Delivery Profile, Advanced Controls,
/// Per-Band Compressor, Bit Depth + Sample Rate) instead of one
/// monolithic section, so the rail reads as discrete technical
/// drawers in the order the spec lays out.
export function AdvancedPanel({
  analysis,
  settings,
  onAdvanced,
  onInputGain,
  onOutputGain,
  onLoudnessTarget,
  onDeliveryProfile,
  onDeliveryBitDepth,
  onDeliverySampleRate,
  showDeliveryFormat = true,
  albumDeliveryFormat,
  adaptiveReadout,
  compressionPlan,
  albumMode = false,
}: {
  analysis?: AnalysisResult;
  settings: MasteringSettings;
  onAdvanced: (adv: MasteringSettings["advanced"]) => void;
  onInputGain: (db: number) => void;
  onOutputGain: (db: number) => void;
  onLoudnessTarget: (targetLufs: number | null) => void;
  onDeliveryProfile: (profile: DeliveryProfile) => void;
  onDeliveryBitDepth: (bitDepth: number | null) => void;
  onDeliverySampleRate: (sampleRate: number | null) => void;
  /// Track Master uses settings-derived delivery format; Album Master passes
  /// album-wide format values into the same rail card so the control stays in
  /// the same place across Advanced surfaces.
  showDeliveryFormat?: boolean;
  albumDeliveryFormat?: {
    bitDepth: number | null;
    sampleRate: number | null;
    onBitDepth: (bitDepth: number | null) => void;
    onSampleRate: (sampleRate: number | null) => void;
  };
  adaptiveReadout?: GuardrailReadout | null;
  compressionPlan?: CompressionPlan | null;
  albumMode?: boolean;
}) {
  const a = settings.advanced;
  const update = (
    field: keyof MasteringSettings["advanced"],
    value: number | boolean | null,
  ) => {
    onAdvanced({ ...a, [field]: value });
  };
  return (
    <>
      <DeliveryProfileCard
        settings={settings}
        onDeliveryProfile={onDeliveryProfile}
      />
      <AdvancedControlsCard
        settings={settings}
        update={update}
        onAdvanced={onAdvanced}
        onInputGain={onInputGain}
        onOutputGain={onOutputGain}
        onLoudnessTarget={onLoudnessTarget}
        adaptiveReadout={adaptiveReadout}
        albumMode={albumMode}
      />
      <PerBandCompressorCard
        analysis={analysis}
        settings={settings}
        a={a}
        onAdvanced={onAdvanced}
        onUpdate={update}
        compressionPlan={compressionPlan}
      />
      {showDeliveryFormat && (
        <DeliveryFormatCard
          settings={settings}
          bitDepth={albumDeliveryFormat?.bitDepth}
          sampleRate={albumDeliveryFormat?.sampleRate}
          onBitDepth={albumDeliveryFormat?.onBitDepth ?? onDeliveryBitDepth}
          onSampleRate={
            albumDeliveryFormat?.onSampleRate ?? onDeliverySampleRate
          }
          autoSampleRateLabel={albumDeliveryFormat ? "Auto" : "Source"}
          note={
            albumDeliveryFormat
              ? "Album Master exports WAV files."
              : "Track Master exports WAV files."
          }
        />
      )}
    </>
  );
}

function DeliveryProfileCard({
  settings,
  onDeliveryProfile,
}: {
  settings: MasteringSettings;
  onDeliveryProfile: (profile: DeliveryProfile) => void;
}) {
  const profile = settings.delivery_profile;
  return (
    <section className="panel rail-card rail-card-delivery">
      <header className="panel-head">
        <span className="panel-title">DELIVERY PROFILE</span>
      </header>
      <select
        id="delivery-profile-select"
        className="loudness-profile-select rail-card-select"
        aria-label="Delivery profile"
        value={profile}
        onChange={(e) => onDeliveryProfile(e.target.value as DeliveryProfile)}
      >
        {(Object.keys(DELIVERY_PROFILE_DISPLAY) as DeliveryProfile[]).map(
          (p) => (
            <option key={p} value={p}>
              {DELIVERY_PROFILE_DISPLAY[p]}
              {DELIVERY_PROFILE_TARGET_LUFS[p] !== null
                ? ` · ${DELIVERY_PROFILE_TARGET_LUFS[p]} LUFS`
                : ""}
            </option>
          ),
        )}
      </select>
    </section>
  );
}

/// Per-axis live readout of what the adaptive guardrails are trimming, shown
/// directly under the Adapt Strength slider.
///
/// Iteration aid for calibrating the guardrails by ear. Gated behind the
/// localStorage debug flag (lib/debug-flags.ts) since 2026-06-09 per the
/// owner TODO — hidden by default for release, never deleted, re-surfaced
/// for tuning sessions. Tracked in docs/ADAPTIVE_DSP_NEXT_STEPS.md.
///
/// Each row pairs the REALIZED trim
/// (post character-floor for EQ; raw for comp/width) with the SOURCE context that
/// drove it — the share/correlation vs that axis's deadband — so a "-0%" is
/// legibly "source in range," not a dead control. Brightness/low trim when their
/// share EXCEEDS the deadband; width trims when correlation is BELOW it (lower
/// correlation = wider). Deadband fields come from the backend (GuardrailReadout)
/// so the thresholds never drift from the DSP constants. All numbers are chain
/// trims, BEFORE the post-chain LUFS landing.
function AdaptiveReadout({ readout }: { readout: GuardrailReadout }) {
  const pct = (v: number) => Math.round(v * 100);
  const share = (v: number) => v.toFixed(2);
  const corr = readout.stereo_correlation;
  const drLabel =
    readout.dynamic_range_db >= 100
      ? "DR n/a"
      : `DR ${readout.dynamic_range_db.toFixed(1)} dB`;
  const axes = [
    {
      key: "highs",
      label: "Highs",
      trim: readout.bright_trim,
      context:
        readout.bright_deadband != null
          ? `presence+air ${share(readout.brightness_share)} / ${share(readout.bright_deadband)}`
          : null,
      inRange:
        readout.bright_deadband != null &&
        readout.brightness_share <= readout.bright_deadband,
    },
    {
      key: "lows",
      label: "Lows",
      trim: readout.low_trim,
      context:
        readout.low_deadband != null
          ? `sub+low ${share(readout.low_share)} / ${share(readout.low_deadband)}`
          : null,
      inRange:
        readout.low_deadband != null && readout.low_share <= readout.low_deadband,
    },
    {
      key: "comp",
      label: "Comp",
      trim: readout.density_trim,
      context: drLabel,
      inRange: false,
    },
    {
      key: "width",
      label: "Width",
      trim: readout.width_trim,
      context:
        corr == null
          ? "mono"
          : readout.width_corr_deadband != null
            ? `corr ${share(corr)} / ${share(readout.width_corr_deadband)}`
            : `corr ${share(corr)}`,
      inRange:
        corr == null ||
        (readout.width_corr_deadband != null &&
          corr >= readout.width_corr_deadband),
    },
  ];
  return (
    <div
      className="adaptive-readout"
      style={{
        display: "flex",
        flexDirection: "column",
        gap: "0.15rem",
        fontSize: "0.72rem",
        opacity: 0.85,
        padding: "0.25rem 0.1rem 0.1rem",
      }}
    >
      <div style={{ fontWeight: 600, opacity: 0.9 }}>
        Adaptive trims (chain, pre-landing)
      </div>
      {axes.map((ax) => (
        <div
          key={ax.key}
          style={{
            display: "flex",
            justifyContent: "space-between",
            gap: "0.5rem",
          }}
        >
          <span>
            {ax.label} -{pct(ax.trim)}%
          </span>
          {ax.context && (
            <span style={{ opacity: 0.6, textAlign: "right" }}>
              {ax.context}
              {ax.trim <= 0.0001 && ax.inRange ? " · in range" : ""}
            </span>
          )}
        </div>
      ))}
    </div>
  );
}

function AdvancedControlsCard({
  settings,
  update,
  onAdvanced,
  onInputGain,
  onOutputGain,
  onLoudnessTarget,
  adaptiveReadout,
  albumMode = false,
}: {
  settings: MasteringSettings;
  update: (
    field: keyof MasteringSettings["advanced"],
    value: number | boolean | null,
  ) => void;
  onAdvanced: (adv: MasteringSettings["advanced"]) => void;
  onInputGain: (db: number) => void;
  onOutputGain: (db: number) => void;
  onLoudnessTarget: (targetLufs: number | null) => void;
  adaptiveReadout?: GuardrailReadout | null;
  albumMode?: boolean;
}) {
  const a = settings.advanced;
  const effectiveLufsTarget = loudnessTargetDisplay(settings).current;
  const effectiveCeiling = effectiveCeilingDbtp(settings);
  const resetAdvancedControls = () => {
    onInputGain(0);
    onOutputGain(0);
    onAdvanced({
      ...a,
      lufs_offset_db: null,
      ceiling_dbtp: null,
      width: null,
      warmth: null,
      presence_air: null,
      compression_density: null,
      // Reset Adapt Strength to its explicit default (0.5), like every other
      // Advanced control. We write the EXPLICIT default rather than null: null
      // also resolves to the default downstream, but it displays ambiguously and
      // the durable "off" is 0.0 — so an explicit value keeps the readout honest
      // (B4 only warned against writing null, not against resetting the field).
      adaptive_strength: ADAPTIVE_STRENGTH_DEFAULT,
    });
  };
  return (
    <details className="panel rail-card rail-card-advanced">
      <summary className="panel-head panel-head-summary">
        <span className="panel-title">ADVANCED CONTROLS</span>
        <PanelResetButton
          label="Reset advanced controls"
          onClick={resetAdvancedControls}
        />
        <span className="panel-chevron" aria-hidden>⌄</span>
      </summary>
      <div className="advanced-grid rail-card-body">
        <GainField
          label="Input gain"
          value={settings.input_gain_db}
          onChange={onInputGain}
        />
        <GainField
          label="Output gain"
          value={settings.output_gain_db}
          onChange={onOutputGain}
        />
        <NumberField
          label="LUFS target"
          value={effectiveLufsTarget}
          step={0.5}
          min={-24}
          max={-6}
          format={(v) => `${v.toFixed(1)} LUFS`}
          onChange={onLoudnessTarget}
        />
        <NumberField
          label="Ceiling"
          value={effectiveCeiling}
          step={0.1}
          min={-3}
          max={0}
          format={(v) => `${v.toFixed(1)} dBTP`}
          onChange={(v) => update("ceiling_dbtp", v)}
        />
        <NumberField
          label="Width"
          value={a.width}
          step={0.05}
          min={0}
          max={2}
          format={(v) => v.toFixed(2)}
          onChange={(v) => update("width", v)}
        />
        <NumberField
          label="Warmth"
          value={a.warmth}
          step={0.05}
          min={0}
          max={1}
          format={(v) => v.toFixed(2)}
          onChange={(v) => update("warmth", v)}
        />
        <NumberField
          label="Presence/Air"
          value={a.presence_air}
          step={0.05}
          min={0}
          max={1}
          format={(v) => v.toFixed(2)}
          onChange={(v) => update("presence_air", v)}
        />
      </div>
      {/* Adapt Strength + its live trim readout, grouped as ONE block so the
          slider's effect is legible right beside it (the slider used to sit in
          the grid with the readout floated separately below). Each axis shows
          its realized trim AND the source share vs that axis's deadband, so a
          "-0%" reads as "source already in range" rather than looking broken. */}
      <div className="adaptive-block rail-card-body" style={{ paddingTop: "0.2rem" }}>
        <NumberField
          label="Adapt strength"
          value={a.adaptive_strength ?? ADAPTIVE_STRENGTH_DEFAULT}
          step={0.05}
          min={0}
          max={1}
          format={(v) => (v <= 0.0001 ? "Off" : `${Math.round(v * 100)}%`)}
          disabled={albumMode}
          onChange={(v) => update("adaptive_strength", v)}
        />
        {albumMode ? (
          <div
            style={{ fontSize: "0.72rem", opacity: 0.7, padding: "0.3rem 0.1rem" }}
          >
            Adaptive applies to Track Master export, not Album renders.
          </div>
        ) : adaptiveReadout?.active && adaptiveReadoutEnabled() ? (
          // Debug-gated (owner TODO 2026-06-08 / backlog P3): hidden by
          // default for release; the localStorage flag re-surfaces it for
          // guardrail calibration sessions.
          <AdaptiveReadout readout={adaptiveReadout} />
        ) : null}
      </div>
    </details>
  );
}

function PerBandCompressorCard({
  analysis,
  settings,
  a,
  onAdvanced,
  onUpdate,
  compressionPlan,
}: {
  analysis?: AnalysisResult;
  settings: MasteringSettings;
  a: MasteringSettings["advanced"];
  onAdvanced: (adv: MasteringSettings["advanced"]) => void;
  onUpdate: (
    field: keyof MasteringSettings["advanced"],
    value: number | boolean | null,
  ) => void;
  compressionPlan?: CompressionPlan | null;
}) {
  type Band = "low" | "mid" | "high";
  const [active, setActive] = useState<Band>("low");
  const autoReadouts = compressorAutoReadouts(settings);
  const presetSummary = presetCompressionSummary(settings, autoReadouts.low);
  const compressorMode: CompressionMode = a.compression_mode ?? "preset";
  const manualEnabled = compressorMode === "manual";
  const adaptivePlan =
    compressorMode === "preset" && compressionPlan?.active ? compressionPlan : null;
  const sourceLooksCompressed =
    (analysis?.dynamic_range_lu ?? Number.POSITIVE_INFINITY) < 6.0;
  const setCompressorMode = (mode: CompressionMode) => {
    if (mode === "manual") {
      onAdvanced(materializeManualCompressor(settings, a));
      return;
    }
    onAdvanced({ ...a, compression_mode: mode });
  };
  const resetPerBandCompressor = () => {
    onAdvanced(resetCompressorSettingsToCurrentMode(settings, a));
  };
  // (The per-band auto-label strings that used to live here were computed
  // but never rendered — the JSX below reads the numeric
  // `autoReadouts[active]` values directly. Audit Batch H tail.)
  const bandFields: Record<Band, {
    threshold: number | null;
    ratio: number | null;
    attack: number | null;
    release: number | null;
    onThreshold: (v: number | null) => void;
    onRatio: (v: number | null) => void;
    onAttack: (v: number | null) => void;
    onRelease: (v: number | null) => void;
  }> = {
    low: {
      threshold: a.compression_low_threshold_db,
      ratio: a.compression_low_ratio,
      attack: a.compression_low_attack_ms,
      release: a.compression_low_release_ms,
      onThreshold: (v) => onUpdate("compression_low_threshold_db", v),
      onRatio: (v) => onUpdate("compression_low_ratio", v),
      onAttack: (v) => onUpdate("compression_low_attack_ms", v),
      onRelease: (v) => onUpdate("compression_low_release_ms", v),
    },
    mid: {
      threshold: a.compression_mid_threshold_db,
      ratio: a.compression_mid_ratio,
      attack: a.compression_mid_attack_ms,
      release: a.compression_mid_release_ms,
      onThreshold: (v) => onUpdate("compression_mid_threshold_db", v),
      onRatio: (v) => onUpdate("compression_mid_ratio", v),
      onAttack: (v) => onUpdate("compression_mid_attack_ms", v),
      onRelease: (v) => onUpdate("compression_mid_release_ms", v),
    },
    high: {
      threshold: a.compression_high_threshold_db,
      ratio: a.compression_high_ratio,
      attack: a.compression_high_attack_ms,
      release: a.compression_high_release_ms,
      onThreshold: (v) => onUpdate("compression_high_threshold_db", v),
      onRatio: (v) => onUpdate("compression_high_ratio", v),
      onAttack: (v) => onUpdate("compression_high_attack_ms", v),
      onRelease: (v) => onUpdate("compression_high_release_ms", v),
    },
  };
  const activeBandFields = bandFields[active];
  return (
    <section className="panel rail-card rail-card-per-band">
      <header className="panel-head">
        <span className="panel-title">PER-BAND COMPRESSOR</span>
        <PanelResetButton
          label="Reset per-band compressor"
          onClick={resetPerBandCompressor}
        />
      </header>
      <div className="compressor-mode-tabs" role="tablist" aria-label="Compressor mode">
        {(["preset", "manual", "off"] as CompressionMode[]).map((mode) => (
          <button
            key={mode}
            type="button"
            role="tab"
            aria-selected={compressorMode === mode}
            className={
              "compressor-mode-tab" +
              (compressorMode === mode ? " is-active" : "")
            }
            onClick={() => setCompressorMode(mode)}
          >
            {mode === "preset" ? "Preset" : mode === "manual" ? "Manual" : "Off"}
          </button>
        ))}
      </div>
      <div className="compressor-mode-note">
        {compressorMode === "preset" &&
          `Preset values from ${presetDisplayName(settings.preset)}.`}
        {compressorMode === "manual" &&
          "Manual values replace preset compression."}
        {compressorMode === "off" &&
          "Creative compressor bypassed; limiter and delivery checks remain active."}
      </div>
      {compressorMode === "preset" && (
        <div className="compressor-density-field">
          <NumberField
            label="Preset density"
            value={a.compression_density}
            step={0.05}
            min={0}
            max={1}
            format={(v) => v.toFixed(2)}
            onChange={(v) => onUpdate("compression_density", v)}
          />
        </div>
      )}
      {sourceLooksCompressed && compressorMode === "preset" && (
        <div className="compressor-source-note" role="note">
          Source dynamic range is low; lower density or switch Off if preset compression collapses movement.
        </div>
      )}
      {!manualEnabled && compressorMode === "preset" && (
        <div className="compressor-preset-summary">{presetSummary}</div>
      )}
      {adaptivePlan?.guidance && (
        <div className="compressor-adaptive-guidance" role="note">
          {adaptivePlan.guidance}
        </div>
      )}
      {adaptivePlan && (
        <div className="compressor-adaptive-bands" aria-label="Adaptive compressor plan">
          {(["low", "mid", "high"] as Band[]).map((band) => (
            <div key={band} className="compressor-adaptive-band">
              <span>
                {bandLabel(band)} {adaptivePlan[band].threshold_db.toFixed(1)} dB ·{" "}
                {adaptivePlan[band].ratio.toFixed(1)}:1
              </span>
              {adaptivePlan[band].adaptive && (
                <span className="compressor-adaptive-tag">Adaptive</span>
              )}
            </div>
          ))}
        </div>
      )}
      {!manualEnabled && compressorMode === "off" && (
        <div className="compressor-preset-summary">
          Per-band controls inactive.
        </div>
      )}
      {manualEnabled && (
        <>
          <label className="per-band-link-stereo">
            <input
              type="checkbox"
              checked={a.compression_link_stereo !== false}
              onChange={(e) =>
                onUpdate("compression_link_stereo", e.target.checked ? null : false)
              }
            />
            <span>Link stereo</span>
          </label>
          <div className="per-band-tabs" role="tablist">
            {(["low", "mid", "high"] as Band[]).map((band) => (
              <button
                key={band}
                type="button"
                role="tab"
                aria-selected={active === band}
                className={"per-band-tab" + (active === band ? " is-active" : "")}
                onClick={() => setActive(band)}
              >
                {band === "low" ? "Low" : band === "mid" ? "Mid" : "High"}
              </button>
            ))}
          </div>
          <div className="per-band-active-body">
            <CompressionKnobGrid
              threshold={activeBandFields.threshold ?? autoReadouts[active].thresholdDb}
              ratio={activeBandFields.ratio ?? autoReadouts[active].ratio}
              attack={activeBandFields.attack ?? autoReadouts[active].attackMs}
              release={activeBandFields.release ?? autoReadouts[active].releaseMs}
              defaultThreshold={autoReadouts[active].thresholdDb}
              defaultRatio={autoReadouts[active].ratio}
              defaultAttack={autoReadouts[active].attackMs}
              defaultRelease={autoReadouts[active].releaseMs}
              onThreshold={activeBandFields.onThreshold}
              onRatio={activeBandFields.onRatio}
              onAttack={activeBandFields.onAttack}
              onRelease={activeBandFields.onRelease}
            />
          </div>
        </>
      )}
    </section>
  );
}

function bandLabel(band: "low" | "mid" | "high"): string {
  return band === "low" ? "Low" : band === "mid" ? "Mid" : "High";
}

function CompressionKnobGrid({
  threshold,
  ratio,
  attack,
  release,
  defaultThreshold,
  defaultRatio,
  defaultAttack,
  defaultRelease,
  onThreshold,
  onRatio,
  onAttack,
  onRelease,
}: {
  threshold: number;
  ratio: number;
  attack: number;
  release: number;
  defaultThreshold: number;
  defaultRatio: number;
  defaultAttack: number;
  defaultRelease: number;
  onThreshold: (v: number | null) => void;
  onRatio: (v: number | null) => void;
  onAttack: (v: number | null) => void;
  onRelease: (v: number | null) => void;
}) {
  return (
    <div className="compressor-knob-grid">
      <Knob
        label="Threshold"
        value={threshold}
        min={-60}
        max={0}
        step={0.5}
        defaultValue={defaultThreshold}
        size="sm"
        tone="blue"
        format={(v) => `${v.toFixed(1)} dB`}
        onChange={onThreshold}
      />
      <Knob
        label="Ratio"
        value={ratio}
        min={1}
        max={20}
        step={0.1}
        defaultValue={defaultRatio}
        size="sm"
        tone="cyan"
        format={(v) => `${v.toFixed(1)}:1`}
        onChange={onRatio}
      />
      <Knob
        label="Attack"
        value={attack}
        min={0.5}
        max={200}
        step={1}
        defaultValue={defaultAttack}
        size="sm"
        tone="purple"
        format={(v) => `${v.toFixed(0)} ms`}
        onChange={onAttack}
      />
      <Knob
        label="Release"
        value={release}
        min={5}
        max={2000}
        step={5}
        defaultValue={defaultRelease}
        size="sm"
        tone="green"
        format={(v) => `${v.toFixed(0)} ms`}
        onChange={onRelease}
      />
    </div>
  );
}

function presetDisplayName(preset: Preset): string {
  switch (preset.kind) {
    case "universal":
      return "Universal";
    case "clarity":
      return "Clarity";
    case "tape":
      return "Tape";
    case "spatial":
      return "Spatial";
    case "oomph":
      return "Oomph";
    case "warmth":
      return "Warmth";
    case "punch":
      return "Punch";
    case "loud":
      return "Loud";
    case "custom":
      return "Custom";
  }
}

function presetCompressionSummary(
  settings: MasteringSettings,
  readout: ReturnType<typeof compressorAutoReadouts>["low"],
): string {
  const defaultDensity = settings.preset.kind === "custom" ? 0 : 0.5;
  const density = Math.max(
    0,
    Math.min(1, settings.advanced.compression_density ?? defaultDensity),
  );
  if (density <= 0.001) {
    return `Preset compression inactive · Density ${density.toFixed(2)}`;
  }
  return `Effective compression · ${readout.thresholdLabel} · ${readout.ratioLabel} · ${readout.attackLabel} · ${readout.releaseLabel}`;
}

function materializeManualCompressor(
  settings: MasteringSettings,
  advanced: MasteringSettings["advanced"],
): MasteringSettings["advanced"] {
  const readouts = compressorAutoReadouts(settings);
  return {
    ...advanced,
    compression_mode: "manual",
    compression_low_threshold_db:
      advanced.compression_low_threshold_db ?? readouts.low.thresholdDb,
    compression_low_ratio: advanced.compression_low_ratio ?? readouts.low.ratio,
    compression_low_attack_ms:
      advanced.compression_low_attack_ms ?? readouts.low.attackMs,
    compression_low_release_ms:
      advanced.compression_low_release_ms ?? readouts.low.releaseMs,
    compression_mid_threshold_db:
      advanced.compression_mid_threshold_db ?? readouts.mid.thresholdDb,
    compression_mid_ratio: advanced.compression_mid_ratio ?? readouts.mid.ratio,
    compression_mid_attack_ms:
      advanced.compression_mid_attack_ms ?? readouts.mid.attackMs,
    compression_mid_release_ms:
      advanced.compression_mid_release_ms ?? readouts.mid.releaseMs,
    compression_high_threshold_db:
      advanced.compression_high_threshold_db ?? readouts.high.thresholdDb,
    compression_high_ratio: advanced.compression_high_ratio ?? readouts.high.ratio,
    compression_high_attack_ms:
      advanced.compression_high_attack_ms ?? readouts.high.attackMs,
    compression_high_release_ms:
      advanced.compression_high_release_ms ?? readouts.high.releaseMs,
  };
}

function resetCompressorSettingsToCurrentMode(
  settings: MasteringSettings,
  advanced: MasteringSettings["advanced"],
): MasteringSettings["advanced"] {
  const mode = advanced.compression_mode ?? "preset";
  const base = {
    ...advanced,
    compression_mode: mode,
    compression_density: null,
    compression_link_stereo: null,
  };
  if (mode === "manual") {
    const resetAdvanced = {
      ...base,
      compression_low_threshold_db: null,
      compression_low_ratio: null,
      compression_low_attack_ms: null,
      compression_low_release_ms: null,
      compression_mid_threshold_db: null,
      compression_mid_ratio: null,
      compression_mid_attack_ms: null,
      compression_mid_release_ms: null,
      compression_high_threshold_db: null,
      compression_high_ratio: null,
      compression_high_attack_ms: null,
      compression_high_release_ms: null,
    };
    return materializeManualCompressor(
      { ...settings, advanced: resetAdvanced },
      resetAdvanced,
    );
  }
  return {
    ...base,
    compression_low_threshold_db: null,
    compression_low_ratio: null,
    compression_low_attack_ms: null,
    compression_low_release_ms: null,
    compression_mid_threshold_db: null,
    compression_mid_ratio: null,
    compression_mid_attack_ms: null,
    compression_mid_release_ms: null,
    compression_high_threshold_db: null,
    compression_high_ratio: null,
    compression_high_attack_ms: null,
    compression_high_release_ms: null,
  };
}

function DeliveryFormatCard({
  settings,
  bitDepth,
  sampleRate,
  onBitDepth,
  onSampleRate,
  autoSampleRateLabel = "Source",
  note = "Track Master exports WAV files.",
}: {
  settings: MasteringSettings;
  bitDepth?: number | null;
  sampleRate?: number | null;
  onBitDepth: (bitDepth: number | null) => void;
  onSampleRate: (sampleRate: number | null) => void;
  autoSampleRateLabel?: string;
  note?: string;
}) {
  const effectiveBitDepthValue =
    bitDepth === undefined ? effectiveBitDepth(settings) : bitDepth;
  const effectiveSampleRateValue =
    sampleRate === undefined ? effectiveSampleRate(settings) : sampleRate;
  return (
    <section className="panel rail-card rail-card-format">
      <header className="panel-head">
        <span className="panel-title">DELIVERY FORMAT</span>
      </header>
      <div className="rail-card-body">
        <SelectField
          label="Bit depth"
          value={effectiveBitDepthValue}
          options={[
            { value: null, label: "Auto" },
            { value: 16, label: "16-bit" },
            { value: 24, label: "24-bit" },
            { value: 32, label: "32-bit float" },
          ]}
          onChange={onBitDepth}
        />
        <SelectField
          label="Sample rate"
          value={effectiveSampleRateValue}
          options={[
            { value: null, label: autoSampleRateLabel },
            { value: 44_100, label: "44.1 kHz" },
            { value: 48_000, label: "48 kHz" },
            { value: 96_000, label: "96 kHz" },
          ]}
          onChange={onSampleRate}
        />
        <p className="format-note">{note}</p>
      </div>
    </section>
  );
}
