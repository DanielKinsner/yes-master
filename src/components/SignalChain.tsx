// Phase 12.2 P3 — explicit signal-chain visualization. Renders a horizontal
// strip of stages above the transport so the user sees, at a glance, what
// the mastering chain is doing to their audio. Each stage lights up when
// backend-resolved processing stage is active. Pending readouts stay unknown.

import { Fragment, type ReactElement } from "react";
import type { MasteringSettings, SignalChainReadout } from "../bindings";

type Stage = {
  key: string;
  label: string;
  detail: string;
  active: boolean | null;
  /// 0..1 intensity used to scale the glow. Below 0.05 reads as off.
  intensity: number;
  icon: () => ReactElement;
};

// Pass 2 (2026-08-19): each stage that has an editable home jumps to it on
// click, so the strip is a map of the console, not decoration. The ids are
// stamped on the target sections (VisualEqPanel, AdvancedPanel cards, the
// Intensity block); SOURCE has no control and stays a plain node.
export const STAGE_JUMP_TARGETS = {
  eq: "jump-visual-eq",
  warmth: "jump-advanced-controls",
  air: "jump-advanced-controls",
  comp: "jump-per-band-compressor",
  width: "jump-advanced-controls",
  sat: "jump-intensity",
  limit: "jump-advanced-controls",
} as const;

export function jumpToStageTarget(key: string): void {
  const id = (STAGE_JUMP_TARGETS as Record<string, string | undefined>)[key];
  if (!id || typeof document === "undefined") return;
  const el = document.getElementById(id);
  if (!el) return;
  el.scrollIntoView({ behavior: "smooth", block: "center" });
  // Brief highlight so the eye lands where the scroll did. Class only; the
  // CSS owns the look and the reduced-motion opt-out.
  el.classList.add("is-jump-target");
  window.setTimeout(() => el.classList.remove("is-jump-target"), 1200);
}

function buildStages(settings: MasteringSettings, resolved?: SignalChainReadout | null): Stage[] {
  const stage = (key: string, label: string, active: boolean | undefined, detail: string, icon: () => ReactElement): Stage => ({
    key, label, active: active ?? null, detail: active === undefined ? "Awaiting analysis" : detail,
    intensity: active ? 0.6 : 0, icon,
  });
  const width = resolved?.width;
  const widthActive = width === undefined ? undefined : Math.abs(width - 1) > 0.001;
  const saturationActive = resolved ? resolved.saturation > 0 : undefined;
  return [
    stage("in", "Source", true, `Intensity ${(settings.intensity * 100).toFixed(0)}%`, SourceIcon),
    stage("eq", "EQ", resolved?.eq_active, resolved?.eq_active ? "Style + adjustments" : "flat", EqIcon),
    stage("warmth", "Warmth", resolved?.warmth_active, resolved?.warmth_active ? "on" : "off", WarmthIcon),
    stage("air", "Air", resolved?.air_active, resolved?.air_active ? "on" : "off", AirIcon),
    stage("comp", "Comp", resolved?.compression_active, resolved?.compression_active ? (settings.advanced.compression_mode ?? "preset") : "off", CompIcon),
    stage("width", "Width", widthActive, widthActive ? `${((width ?? 1) * 100).toFixed(0)}%` : "neutral", WidthIcon),
    stage("sat", "Saturation", saturationActive, saturationActive ? `${((resolved?.saturation ?? 0) * 100).toFixed(0)}% drive` : "off", SatIcon),
    stage("limit", "Limiter", true, "Ceiling protection", LimiterIcon),
  ];
}
export function SignalChain({ settings, resolved }: { settings: MasteringSettings; resolved?: SignalChainReadout | null }) {
  const stages = buildStages(settings, resolved);
  return (
    <section
      className="signal-chain"
      aria-label="Signal chain"
    >
      <div className="signal-chain-track">
        {stages.map((s, i) => (
          <Fragment key={s.key}>
            {i > 0 && (
              <span
                className={
                  "chain-link " + (stages[i - 1].active && s.active ? "is-hot" : "")
                }
                aria-hidden
              />
            )}
            <StageNode stage={s} />
          </Fragment>
        ))}
      </div>
    </section>
  );
}

function StageNode({ stage }: { stage: Stage }) {
  const glowOpacity = stage.active ? Math.max(0.25, stage.intensity) : 0;
  const jumpable = stage.key in STAGE_JUMP_TARGETS;
  const body = (
    <>
      <span
        className="chain-stage-disc"
        style={{
          // Live glow intensity follows the stage's setting magnitude so a
          // hot compressor reads as obviously hotter than a gentle Warmth nudge.
          boxShadow: stage.active
            ? `0 0 22px rgba(77, 139, 255, ${glowOpacity * 0.9}), 0 0 8px rgba(195, 215, 255, ${glowOpacity * 0.35}), inset 0 0 0 1px rgba(176, 205, 255, ${glowOpacity * 0.85})`
            : undefined,
        }}
      >
        <stage.icon />
      </span>
      <span className="chain-stage-label">{stage.label}</span>
      <span className="chain-stage-detail">{stage.detail}</span>
    </>
  );
  const className = `chain-stage ${stage.active === null ? "is-pending" : stage.active ? "is-active" : "is-off"}`;
  if (!jumpable) {
    return (
      <div className={className} title={`${stage.label} — ${stage.detail}`}>
        {body}
      </div>
    );
  }
  return (
    <button
      type="button"
      className={className}
      title={`${stage.label} — ${stage.detail}. Jump to its controls.`}
      onClick={() => jumpToStageTarget(stage.key)}
    >
      {body}
    </button>
  );
}

// Stage icons — small line-style 20×20 svgs, currentColor stroke so they
// inherit the disc's color (which switches between text-2 and accent
// depending on active state).

function SourceIcon() {
  return (
    <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={2} strokeLinecap="round" strokeLinejoin="round">
      <path d="M3 12h3l3-8 4 16 3-8h5" />
    </svg>
  );
}

function EqIcon() {
  return (
    <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={2} strokeLinecap="round" strokeLinejoin="round">
      <line x1="4" y1="21" x2="4" y2="14" />
      <line x1="4" y1="10" x2="4" y2="3" />
      <line x1="12" y1="21" x2="12" y2="12" />
      <line x1="12" y1="8" x2="12" y2="3" />
      <line x1="20" y1="21" x2="20" y2="16" />
      <line x1="20" y1="12" x2="20" y2="3" />
      <line x1="1" y1="14" x2="7" y2="14" />
      <line x1="9" y1="8" x2="15" y2="8" />
      <line x1="17" y1="16" x2="23" y2="16" />
    </svg>
  );
}

function WarmthIcon() {
  return (
    <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={2} strokeLinecap="round" strokeLinejoin="round">
      <path d="M12 3q1 4 4 6.5t3 5.5a1 1 0 0 1-14 0 5 5 0 0 1 1-3 1 1 0 0 0 5 0c0-2-1.5-3-1.5-5q0-2 2.5-4" />
    </svg>
  );
}

function AirIcon() {
  return (
    <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={2} strokeLinecap="round" strokeLinejoin="round">
      <path d="M3 8h12a3 3 0 1 0-3-3" />
      <path d="M3 14h18a3 3 0 1 1-3 3" />
      <path d="M3 19h7" />
    </svg>
  );
}

function CompIcon() {
  return (
    <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={2} strokeLinecap="round" strokeLinejoin="round">
      <path d="M3 17l4-2 3 1 4-6 3 5 4-3" />
      <path d="M3 21h18" />
    </svg>
  );
}

function WidthIcon() {
  return (
    <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={2} strokeLinecap="round" strokeLinejoin="round">
      <polyline points="9 18 3 12 9 6" />
      <polyline points="15 6 21 12 15 18" />
    </svg>
  );
}

function SatIcon() {
  return (
    <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={2} strokeLinecap="round" strokeLinejoin="round">
      <path d="M3 12c3 0 3-6 6-6s3 12 6 12 3-6 6-6" />
    </svg>
  );
}

function LimiterIcon() {
  return (
    <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={2} strokeLinecap="round" strokeLinejoin="round">
      <path d="M4 6h16" />
      <path d="M4 12l4-3 3 2 4-4 3 3 2-2" />
      <path d="M4 18h16" />
    </svg>
  );
}
