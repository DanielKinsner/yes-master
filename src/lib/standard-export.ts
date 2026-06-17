// src/lib/standard-export.ts
//
// Standard's export is the iPhone's fixed recipe (44.1k / 24-bit / -1 dBTP
// WAV) plus the Standard-chosen loudness, and its review ceremony is
// stripped: no blocking gate, cosmetic warnings suppressed, one tiny
// non-blocking integrity note, hard-stops still surfaced (design spec §6).

import type { MasteringSettings, QualityCheck } from "../bindings";
import parity from "../standard-mapping-parity.json";
import { effectiveLoudnessTarget } from "./effective-settings";
import { STANDARD_LOUDNESS } from "./standard-mapping";

export const STANDARD_EXPORT_DELIVERY: {
  sampleRate: number;
  bitDepth: number;
  ceilingDbtp: number;
  lufsClamp: readonly [number, number];
} = {
  sampleRate: parity.delivery.sample_rate,
  bitDepth: parity.delivery.bit_depth,
  ceilingDbtp: parity.delivery.ceiling_dbtp,
  lufsClamp: parity.delivery.lufs_clamp as [number, number],
};

function clampStandardTarget(target: number | null): number | null {
  if (target === null) {
    return null;
  }
  const [minLufs, maxLufs] = STANDARD_EXPORT_DELIVERY.lufsClamp;
  return Math.min(maxLufs, Math.max(minLufs, target));
}

function snapStandardTarget(target: number | null): number | null {
  const clamped = clampStandardTarget(target);
  if (clamped === null) {
    return null;
  }
  return STANDARD_LOUDNESS.reduce((closest, candidate) =>
    Math.abs(candidate.lufs - clamped) < Math.abs(closest.lufs - clamped)
      ? candidate
      : closest,
  ).lufs;
}

/// Wrap the live Standard settings into the known-safe delivery format,
/// landing the loudness on Standard's Low/Medium/High grid even when the
/// live settings came back from Advanced with an off-grid profile/target.
/// Mirrors apps/iphone-native/rust/src/lib.rs::export_settings_for_options
/// for the fixed delivery recipe.
export function standardExportSettings(s: MasteringSettings): MasteringSettings {
  const target = snapStandardTarget(effectiveLoudnessTarget(s));
  return {
    ...s,
    delivery_profile: "custom",
    advanced: {
      ...s.advanced,
      lufs_offset_db: target,
      ceiling_dbtp: STANDARD_EXPORT_DELIVERY.ceilingDbtp,
      bit_depth: STANDARD_EXPORT_DELIVERY.bitDepth,
      target_sample_rate: STANDARD_EXPORT_DELIVERY.sampleRate,
    },
  };
}

export interface StandardExportNotes {
  /// A technical hard-stop was found in the rendered output. Standard
  /// renders first, then checks — so the file exists but must not be
  /// treated as a usable master. Present prominently as "saved, but this
  /// master has a problem — re-render", never celebrate it. This is NOT a
  /// pre-render block: under Standard's pinned 44.1k/24-bit format the only
  /// critical that can fire is a corrupt/non-finite render, which is
  /// unknowable before rendering (see plan Task 13 canon wording).
  invalid: boolean;
  invalidMessage?: string;
  /// A genuine integrity issue (true-peak slipped over) — a tiny inline
  /// note alongside success, never a modal.
  integrityNote?: string;
}

const INTEGRITY_CODES = new Set(["true_peak_high"]);

export function standardExportNotes(checks: QualityCheck[]): StandardExportNotes {
  const critical = checks.find((c) => c.level === "critical");
  if (critical) {
    return { invalid: true, invalidMessage: critical.message };
  }
  const integrity = checks.find((c) => INTEGRITY_CODES.has(c.code));
  if (integrity) {
    return { invalid: false, integrityNote: integrity.message };
  }
  // Everything else (cosmetic warnings + info) is suppressed in Standard.
  return { invalid: false };
}
