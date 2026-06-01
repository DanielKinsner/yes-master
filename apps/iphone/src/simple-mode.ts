import type {
  AdvancedSettings,
  MasteringSettings,
  Preset,
} from "../../../src/bindings";

export type IphoneSimpleFeature =
  | "single-track-import"
  | "tone-presets"
  | "original-mastered-toggle"
  | "volume-match-toggle"
  | "lufs-preview-toggle"
  | "loudness-choice"
  | "export-action";

export const IPHONE_SIMPLE_FEATURES: readonly IphoneSimpleFeature[] = [
  "single-track-import",
  "tone-presets",
  "original-mastered-toggle",
  "volume-match-toggle",
  "lufs-preview-toggle",
  "loudness-choice",
  "export-action",
];

export type IphoneSimpleTone = "balanced" | "warm" | "open" | "punch";
export type IphoneSimpleLoudness = "low" | "medium" | "high";

export interface IphoneSimpleToneOption {
  id: IphoneSimpleTone;
  label: string;
  preset: Preset;
}

export interface IphoneSimpleLoudnessOption {
  id: IphoneSimpleLoudness;
  label: string;
  targetLufs: number;
}

export interface IphoneSimplePlanInput {
  tone?: IphoneSimpleTone;
  loudness?: IphoneSimpleLoudness;
  volumeMatch?: boolean;
  lufsPreview?: boolean;
}

export interface IphoneSimplePlan {
  auditionSettings: MasteringSettings;
  exportSettings: MasteringSettings;
  previewLufsLanding: boolean;
  usesAdaptiveAnalysis: false;
}

export const iphoneSimpleToneOptions: readonly IphoneSimpleToneOption[] = [
  { id: "balanced", label: "Balanced", preset: { kind: "universal" } },
  { id: "warm", label: "Warm", preset: { kind: "warmth" } },
  { id: "open", label: "Open", preset: { kind: "clarity" } },
  { id: "punch", label: "Punch", preset: { kind: "punch" } },
];

export const iphoneSimpleLoudnessOptions: readonly IphoneSimpleLoudnessOption[] = [
  { id: "low", label: "Low", targetLufs: -14 },
  { id: "medium", label: "Medium", targetLufs: -11 },
  { id: "high", label: "High", targetLufs: -9 },
];

const IPHONE_FIXED_CEILING_DBTP = -1;
const IPHONE_FIXED_BIT_DEPTH = 24;
const IPHONE_FIXED_SAMPLE_RATE = 44_100;

export function buildIphoneSimplePlan(
  input: IphoneSimplePlanInput = {},
): IphoneSimplePlan {
  const tone = findOption(iphoneSimpleToneOptions, input.tone ?? "balanced");
  const loudness = findOption(
    iphoneSimpleLoudnessOptions,
    input.loudness ?? "medium",
  );
  const advanced = buildAdvancedSettings(loudness.targetLufs);
  const baseSettings = buildBaseSettings(tone.preset, advanced);
  const exportSettings = {
    ...baseSettings,
    volume_match: false,
  };
  const auditionSettings = {
    ...baseSettings,
    volume_match: input.volumeMatch ?? false,
  };

  return {
    auditionSettings,
    exportSettings,
    previewLufsLanding: input.lufsPreview ?? false,
    usesAdaptiveAnalysis: false,
  };
}

function buildBaseSettings(
  preset: Preset,
  advanced: AdvancedSettings,
): MasteringSettings {
  return {
    preset,
    intensity: 0.5,
    eq_sub_db: 0,
    eq_low_db: 0,
    eq_low_mid_db: 0,
    eq_mid_db: 0,
    eq_high_mid_db: 0,
    eq_high_db: 0,
    eq_sparkle_db: 0,
    volume_match: false,
    input_gain_db: 0,
    output_gain_db: 0,
    delivery_profile: "custom",
    advanced,
  };
}

function buildAdvancedSettings(
  targetLufs: number,
): AdvancedSettings {
  return {
    lufs_offset_db: targetLufs,
    ceiling_dbtp: IPHONE_FIXED_CEILING_DBTP,
    width: null,
    warmth: null,
    presence_air: null,
    compression_mode: "preset",
    compression_density: null,
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
    compression_link_stereo: null,
    bit_depth: IPHONE_FIXED_BIT_DEPTH,
    target_sample_rate: IPHONE_FIXED_SAMPLE_RATE,
  };
}

function findOption<T extends { id: string }>(
  options: readonly T[],
  id: T["id"],
): T {
  const option = options.find((candidate) => candidate.id === id);
  if (!option) {
    throw new Error(`Unknown iPhone Simple option: ${id}`);
  }
  return option;
}
