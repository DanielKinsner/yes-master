// src/lib/auto-gain-staging.ts
//
// Auto gain staging (Beta), plan Slice 6. Advanced only. The locked copy
// table and the view the row renders. Every number comes from the backend's
// readout, which is the computation the audio uses (decision A9): this file
// formats numbers, it never computes a gain.

import type {
  AutoGainStagingReadout,
  AutoGainStagingSettings,
  LoudnessPriority,
  MasteringSettings,
} from "../bindings";

export const AUTO_GAIN_STAGING_COPY = {
  label: "Auto gain staging",
  tag: "Beta",
  off: "Sets the level for this song automatically, so it is limited only as much as your loudness needs.",
  measuring: "Measuring this song…",
  noTarget: "Choose a loudness target to use auto gain staging.",
  unusable: "Not enough audio to measure this track.",
  album: "Auto gain staging applies to Track Master, not Album renders.",
  priority: "Loudness priority",
  clean: "Clean",
  reachTarget: "Reach target",
} as const;

export const LOUDNESS_PRIORITIES: ReadonlyArray<{ value: LoudnessPriority; label: string }> = [
  { value: "clean", label: AUTO_GAIN_STAGING_COPY.clean },
  { value: "reach_target", label: AUTO_GAIN_STAGING_COPY.reachTarget },
];

const MINUS = "−";

/// One decimal place and a real minus sign; a value that rounds to zero is
/// `0.0`. `signed` adds a `+` to positive values. Matches the export
/// receipt's `gain_staging::db_text`.
export function dbText(value: number, signed: boolean): string {
  const rounded = Math.round(value * 10) / 10;
  if (rounded === 0) return "0.0 dB";
  if (rounded < 0) return `${MINUS}${(-rounded).toFixed(1)} dB`;
  return `${signed ? "+" : ""}${rounded.toFixed(1)} dB`;
}

function lufsText(value: number): string {
  const rounded = Math.round(value * 10) / 10;
  if (rounded === 0) return "0.0";
  return rounded < 0 ? `${MINUS}${(-rounded).toFixed(1)}` : rounded.toFixed(1);
}

export type AutoGainStagingState =
  | "off"
  | "album_disabled"
  | "measuring"
  | "no_target"
  | "unusable"
  | "no_limiting"
  | "limiting"
  | "stopped_at_clean_limit"
  | "stopped_by_peak_check";

export interface AutoGainStagingView {
  state: AutoGainStagingState;
  enabled: boolean;
  priority: LoudnessPriority;
  line: string;
  /// The line is not settled (rule 8): the switch's result is still coming.
  pending: boolean;
  /// Album renders: the switch cannot be used.
  disabled: boolean;
  /// The priority control shows only while the switch is on.
  showPriority: boolean;
}

export function autoGainStagingSettings(s: MasteringSettings): AutoGainStagingSettings {
  return s.advanced.auto_gain_staging ?? { enabled: false, priority: "clean" };
}

/// The line for a settled readout (the copy table's last four rows).
export function settledLine(
  readout: Extract<AutoGainStagingReadout, { state: "settled" }>,
): { state: AutoGainStagingState; line: string } {
  if (readout.stopped_at_clean_limit) {
    return {
      state: "stopped_at_clean_limit",
      line: `Reached ${lufsText(readout.delivered_lufs)} LUFS of ${lufsText(readout.target_lufs)}. Going louder needs more than 6 dB of limiting.`,
    };
  }
  if (readout.stopped_by_peak_check) {
    return {
      state: "stopped_by_peak_check",
      line: `Reached ${lufsText(readout.delivered_lufs)} LUFS of ${lufsText(readout.target_lufs)}. This song's peaks stop it going louder under your ceiling.`,
    };
  }
  const gains = `Level in ${dbText(readout.level_in_db, true)} · limiter drive ${dbText(readout.limiter_drive_db, true)}`;
  if (Math.round(Math.max(0, readout.limiter_load_db) * 10) === 0) {
    return { state: "no_limiting", line: `${gains} · no limiting needed.` };
  }
  return {
    state: "limiting",
    line: `${gains} · limiting up to ${dbText(readout.limiter_load_db, false)}.`,
  };
}

export interface AutoGainStagingViewInput {
  settings: MasteringSettings;
  /// The backend's line for these settings, when one has arrived.
  readout: AutoGainStagingReadout | null;
  album: boolean;
  /// A resolution for the current settings is in flight.
  measuring: boolean;
}

export function autoGainStagingView(input: AutoGainStagingViewInput): AutoGainStagingView {
  const { enabled, priority } = autoGainStagingSettings(input.settings);
  const base = { enabled, priority, pending: false, disabled: false, showPriority: enabled };
  if (input.album) {
    return {
      ...base,
      state: "album_disabled",
      line: AUTO_GAIN_STAGING_COPY.album,
      disabled: true,
      showPriority: false,
    };
  }
  if (!enabled) {
    return { ...base, state: "off", line: AUTO_GAIN_STAGING_COPY.off };
  }
  const readout = input.readout;
  if (readout?.state === "no_target") {
    return { ...base, state: "no_target", line: AUTO_GAIN_STAGING_COPY.noTarget };
  }
  if (readout?.state === "album") {
    return {
      ...base,
      state: "album_disabled",
      line: AUTO_GAIN_STAGING_COPY.album,
      disabled: true,
      showPriority: false,
    };
  }
  if (input.measuring || !readout || readout.state === "measuring") {
    return { ...base, state: "measuring", line: AUTO_GAIN_STAGING_COPY.measuring, pending: true };
  }
  if (readout.state === "unusable") {
    return { ...base, state: "unusable", line: AUTO_GAIN_STAGING_COPY.unusable };
  }
  return { ...base, ...settledLine(readout) };
}

/// The settings after the switch is flipped. The priority is kept, so turning
/// the switch back on restores it; switched off, the chain is exactly today's
/// and the receipt says `Auto gain staging: off`.
export function withAutoGainStaging(
  s: MasteringSettings,
  enabled: boolean,
): MasteringSettings["advanced"] {
  const { priority } = autoGainStagingSettings(s);
  return { ...s.advanced, auto_gain_staging: { enabled, priority } };
}

export function withLoudnessPriority(
  s: MasteringSettings,
  priority: LoudnessPriority,
): MasteringSettings["advanced"] {
  const { enabled } = autoGainStagingSettings(s);
  return { ...s.advanced, auto_gain_staging: { enabled, priority } };
}
