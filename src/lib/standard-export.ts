// src/lib/standard-export.ts
//
// Standard's export is the iPhone's fixed recipe (44.1k / 24-bit / -1 dBTP
// WAV) plus the Standard-chosen loudness, and its review ceremony is
// stripped: no blocking gate, cosmetic warnings suppressed, one tiny
// non-blocking integrity note, hard-stops still surfaced (design spec §6).

import type { MasteringSettings, QualityCheck } from "../bindings";
import { effectiveLoudnessTarget } from "./effective-settings";

/// Wrap the live Standard settings into the known-safe delivery format,
/// preserving the loudness the user (or the default profile) is targeting.
/// Mirrors apps/iphone-native/rust/src/lib.rs::export_settings_for_options.
export function standardExportSettings(s: MasteringSettings): MasteringSettings {
  const target = effectiveLoudnessTarget(s);
  return {
    ...s,
    delivery_profile: "custom",
    advanced: {
      ...s.advanced,
      lufs_offset_db: target,
      ceiling_dbtp: -1,
      bit_depth: 24,
      target_sample_rate: 44_100,
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
