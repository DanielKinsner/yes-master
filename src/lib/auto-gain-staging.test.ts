import { describe, expect, it } from "vitest";
import type { AutoGainStagingReadout, MasteringSettings } from "../bindings";
import { ADAPTIVE_STRENGTH_DEFAULT, EQ_BAND_DEFAULTS } from "../bindings";
import {
  AUTO_GAIN_STAGING_COPY,
  autoGainStagingView,
  dbText,
  withAutoGainStaging,
  withLoudnessPriority,
} from "./auto-gain-staging";

const BASE: MasteringSettings = {
  preset: { kind: "universal" },
  intensity: 0.5,
  eq_sub_db: 0,
  eq_low_db: 0,
  eq_low_mid_db: 0,
  eq_mid_db: 0,
  eq_high_mid_db: 0,
  eq_high_db: 0,
  eq_sparkle_db: 0,
  eq_bands: { ...EQ_BAND_DEFAULTS },
  volume_match: false,
  input_gain_db: 0,
  output_gain_db: 0,
  delivery_profile: "streaming-universal",
  advanced: {
    lufs_offset_db: -9,
    ceiling_dbtp: null,
    width: null,
    warmth: null,
    presence_air: null,
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
    adaptive_strength: ADAPTIVE_STRENGTH_DEFAULT,
    bit_depth: null,
    target_sample_rate: null,
  },
} as unknown as MasteringSettings;

const ON: MasteringSettings = {
  ...BASE,
  advanced: { ...BASE.advanced, auto_gain_staging: { enabled: true, priority: "clean" } },
};

function settled(
  overrides: Partial<Extract<AutoGainStagingReadout, { state: "settled" }>> = {},
): AutoGainStagingReadout {
  return {
    state: "settled",
    level_in_db: -3.3,
    limiter_drive_db: 0,
    limiter_load_db: 0,
    stopped_at_clean_limit: false,
    stopped_by_peak_check: false,
    delivered_lufs: -9,
    target_lufs: -9,
    ...overrides,
  };
}

function line(settings: MasteringSettings, readout: AutoGainStagingReadout | null, opts: { album?: boolean; measuring?: boolean } = {}) {
  return autoGainStagingView({
    settings,
    readout,
    album: opts.album ?? false,
    measuring: opts.measuring ?? false,
  });
}

describe("auto gain staging copy (plan locked copy table)", () => {
  it("formats one decimal place with a real minus sign", () => {
    expect(dbText(-3.34, true)).toBe("−3.3 dB");
    expect(dbText(2.06, true)).toBe("+2.1 dB");
    expect(dbText(3.14, false)).toBe("3.1 dB");
    expect(dbText(0.04, true)).toBe("0.0 dB");
    expect(dbText(-0.04, true)).toBe("0.0 dB");
  });

  it("Off", () => {
    const view = line(BASE, settled());
    expect(view.state).toBe("off");
    expect(view.line).toBe(
      "Sets the level for this song automatically, so it is limited only as much as your loudness needs.",
    );
    expect(view.showPriority).toBe(false);
    expect(view.enabled).toBe(false);
  });

  it("On, still measuring: never a settled line while pending", () => {
    expect(line(ON, null).line).toBe("Measuring this song…");
    expect(line(ON, { state: "measuring" }).line).toBe("Measuring this song…");
    const pending = line(ON, settled(), { measuring: true });
    expect(pending.line).toBe("Measuring this song…");
    expect(pending.pending).toBe(true);
    expect(pending.showPriority).toBe(true);
  });

  it("On, no limiting needed", () => {
    const view = line(ON, settled({ level_in_db: -3.3, limiter_drive_db: 0, limiter_load_db: 0.04 }));
    expect(view.state).toBe("no_limiting");
    expect(view.line).toBe("Level in −3.3 dB · limiter drive 0.0 dB · no limiting needed.");
    expect(view.pending).toBe(false);
  });

  it("On, limiting", () => {
    const view = line(ON, settled({ level_in_db: 2.1, limiter_drive_db: 4.5, limiter_load_db: 3.1 }));
    expect(view.state).toBe("limiting");
    expect(view.line).toBe("Level in +2.1 dB · limiter drive +4.5 dB · limiting up to 3.1 dB.");
  });

  it("On, stopped at the clean limit", () => {
    const view = line(
      ON,
      settled({ limiter_load_db: 6, stopped_at_clean_limit: true, delivered_lufs: -11.23, target_lufs: -9 }),
    );
    expect(view.state).toBe("stopped_at_clean_limit");
    expect(view.line).toBe("Reached −11.2 LUFS of −9.0. Going louder needs more than 6 dB of limiting.");
  });

  it("On, stopped by the peak check (Checkpoint B)", () => {
    const view = line(
      { ...ON, advanced: { ...ON.advanced, auto_gain_staging: { enabled: true, priority: "reach_target" } } },
      settled({ limiter_load_db: 20, stopped_by_peak_check: true, delivered_lufs: -11.3, target_lufs: -9 }),
    );
    expect(view.state).toBe("stopped_by_peak_check");
    expect(view.line).toBe(
      "Reached −11.3 LUFS of −9.0. This song's peaks stop it going louder under your ceiling.",
    );
    expect(view.priority).toBe("reach_target");
  });

  it("On, no loudness target", () => {
    const view = line(ON, { state: "no_target" });
    expect(view.state).toBe("no_target");
    expect(view.line).toBe("Choose a loudness target to use auto gain staging.");
  });

  it("On, no usable measurement", () => {
    const view = line(ON, { state: "unusable" });
    expect(view.state).toBe("unusable");
    expect(view.line).toBe("Not enough audio to measure this track.");
  });

  it("Album mode is disabled, whatever the switch says", () => {
    for (const settings of [BASE, ON]) {
      const view = line(settings, settled(), { album: true });
      expect(view.state).toBe("album_disabled");
      expect(view.line).toBe("Auto gain staging applies to Track Master, not Album renders.");
      expect(view.disabled).toBe(true);
      expect(view.showPriority).toBe(false);
    }
    expect(line(ON, { state: "album" }).state).toBe("album_disabled");
  });

  it("labels the priority choices as locked", () => {
    expect(AUTO_GAIN_STAGING_COPY.priority).toBe("Loudness priority");
    expect([AUTO_GAIN_STAGING_COPY.clean, AUTO_GAIN_STAGING_COPY.reachTarget]).toEqual([
      "Clean",
      "Reach target",
    ]);
    expect(AUTO_GAIN_STAGING_COPY.label).toBe("Auto gain staging");
    expect(AUTO_GAIN_STAGING_COPY.tag).toBe("Beta");
  });
});

describe("auto gain staging settings edits", () => {
  it("the switch keeps the chosen priority", () => {
    const reach = { ...ON, advanced: withLoudnessPriority(ON, "reach_target") };
    expect(reach.advanced.auto_gain_staging).toEqual({ enabled: true, priority: "reach_target" });
    const off = withAutoGainStaging(reach, false);
    expect(off.auto_gain_staging).toEqual({ enabled: false, priority: "reach_target" });
    expect(withAutoGainStaging({ ...reach, advanced: off }, true).auto_gain_staging).toEqual({
      enabled: true,
      priority: "reach_target",
    });
  });

  it("turning the switch on from absent starts at Clean", () => {
    expect(withAutoGainStaging(BASE, true).auto_gain_staging).toEqual({ enabled: true, priority: "clean" });
  });
});
