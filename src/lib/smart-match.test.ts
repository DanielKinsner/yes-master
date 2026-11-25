import { describe, expect, it } from "vitest";

import type {
  AdvancedSettings,
  MasteringSettings,
  SmartMatchReadout,
  SmartMatchSettings,
} from "../bindings";
import {
  correctionSentence,
  displayReferenceName,
  hasSmartMatchAmountEdit,
  namedMoves,
  presetSettingsKeepingSmartMatch,
  refusalLine,
  resetSmartMatchAmount,
  smartMatchView,
  withSmartMatchAmount,
  withSmartMatchEnabled,
  withSmartMatchReference,
  withoutSmartMatchReference,
  type SmartMatchViewInput,
} from "./smart-match";

const ADVANCED: AdvancedSettings = {
  lufs_offset_db: null,
  ceiling_dbtp: null,
  width: null,
  warmth: 0.25,
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
  bit_depth: null,
  target_sample_rate: null,
};

const PRESET: MasteringSettings = {
  preset: { kind: "clarity" },
  intensity: 0.7,
  eq_sub_db: 0,
  eq_low_db: 1,
  eq_low_mid_db: 0,
  eq_mid_db: 0,
  eq_high_mid_db: 0,
  eq_high_db: 0,
  eq_sparkle_db: 0,
  volume_match: false,
  source_lufs_integrated: null,
  input_gain_db: 0,
  output_gain_db: 0,
  delivery_profile: "streaming-universal",
  album: null,
  advanced: ADVANCED,
};

const SONG_MATCH: SmartMatchSettings = {
  reference: {
    name: "LANDR-It's a coat-original-test-Warm-High",
    spectral_6: { sub: 0.2, low: 0.25, low_mid: 0.2, mid: 0.15, presence: 0.12, air: 0.08 },
  },
  enabled: true,
  amount: 0.5,
};

describe("applying a saved preset keeps the song's Smart Match", () => {
  it("keeps the current reference and takes everything else from the preset", () => {
    const applied = presetSettingsKeepingSmartMatch(PRESET, SONG_MATCH);
    expect(applied.advanced.smart_match).toEqual(SONG_MATCH);
    expect(applied.preset).toEqual({ kind: "clarity" });
    expect(applied.intensity).toBe(0.7);
    expect(applied.advanced.warmth).toBe(0.25);
  });

  it("never imposes a reference a preset somehow carries", () => {
    const smuggled: MasteringSettings = {
      ...PRESET,
      advanced: { ...ADVANCED, smart_match: { ...SONG_MATCH, amount: 1 } },
    };
    expect(presetSettingsKeepingSmartMatch(smuggled, null).advanced).not.toHaveProperty(
      "smart_match",
    );
    expect(presetSettingsKeepingSmartMatch(smuggled, SONG_MATCH).advanced.smart_match).toEqual(
      SONG_MATCH,
    );
  });

  it("does not mutate the preset it was given", () => {
    const before = JSON.stringify(PRESET);
    presetSettingsKeepingSmartMatch(PRESET, SONG_MATCH);
    expect(JSON.stringify(PRESET)).toBe(before);
  });
});

// ---------------------------------------------------------------------------
// Slice 7: the row's wording and the settings edits.
// ---------------------------------------------------------------------------

function withMatch(match: SmartMatchSettings | undefined): MasteringSettings {
  const advanced = { ...ADVANCED };
  if (match) advanced.smart_match = match;
  return { ...PRESET, advanced };
}

function readout(
  state: SmartMatchReadout["state"],
  corrections: number[] = [0, 0, 0, 0, 0, 0],
  partial = false,
): SmartMatchReadout {
  return {
    state,
    amount: 0.5,
    partial,
    bands: corrections.map((correction_db) => ({ gap_db: 0, correction_db, capped: false })),
  };
}

function view(overrides: Partial<SmartMatchViewInput>): ReturnType<typeof smartMatchView> {
  return smartMatchView({
    settings: withMatch(SONG_MATCH),
    readout: readout("applied", [0, 0, -1.2, 0, 0, 0.8]),
    pending: false,
    album: false,
    measuring: false,
    refusal: null,
    analysisReady: true,
    analysisHasProfile: true,
    ...overrides,
  });
}

const SHOWN = "LANDR-It's a coat-original-test-Warm-High";

describe("Smart Match wording (locked copy)", () => {
  it("shortens a name past 60 characters with an ellipsis", () => {
    const long = "x".repeat(61);
    expect(displayReferenceName(long)).toBe(`${"x".repeat(59)}…`);
    expect(Array.from(displayReferenceName(long))).toHaveLength(60);
    expect(displayReferenceName("x".repeat(60))).toBe("x".repeat(60));
  });

  it("names moves of 0.3 dB or more, largest first, at most three, ties in band order", () => {
    expect(namedMoves([0.29, -0.3, 2, -2, 0.5, 1])).toEqual([
      { band: 2, value: 2 },
      { band: 3, value: -2 },
      { band: 5, value: 1 },
    ]);
    expect(namedMoves([0.1, -0.2, 0, 0, 0, 0.29])).toEqual([]);
  });

  it("builds the sentence from the locked phrases for cuts and boosts", () => {
    expect(correctionSentence("Song", [-0.5, 0, 0, 0, 0, 0])).toBe(
      'Matching "Song": tamed 0.5 dB of sub rumble.',
    );
    expect(correctionSentence("Song", [0, 1.04, -1.2, 0, 0, 0])).toBe(
      'Matching "Song": cleared 1.2 dB of mud, added 1.0 dB of bass.',
    );
    expect(correctionSentence("Song", [0, 0, 0, -0.4, 0.6, -0.9])).toBe(
      'Matching "Song": softened 0.9 dB of brightness, added 0.6 dB of presence, eased 0.4 dB of midrange.',
    );
    expect(correctionSentence("Song", [0.5, -0.5, 0.5, 0.5, -0.5, 0.5]).split(", ")).toHaveLength(3);
  });

  it("words every refusal reason", () => {
    expect(refusalLine("too_short")).toBe(
      "That file can't be used as a reference: it is shorter than 30 seconds.",
    );
    expect(refusalLine("silent")).toBe("That file can't be used as a reference: it is silent.");
    expect(refusalLine("unreadable")).toBe(
      "That file can't be used as a reference: it could not be read.",
    );
    expect(refusalLine("low_sample_rate")).toBe(
      "That file can't be used as a reference: its sample rate is too low.",
    );
  });
});

describe("smartMatchView: one line per copy-table state", () => {
  it("no reference", () => {
    const v = view({ settings: withMatch(undefined), readout: readout("no_reference") });
    expect(v.state).toBe("no_reference");
    expect(v.line).toBe("Pick a song you want this track to sit next to.");
    expect(v.name).toBeNull();
    expect(v.bands).toBeNull();
  });

  it("measuring the reference", () => {
    const v = view({ measuring: true });
    expect(v.state).toBe("measuring");
    expect(v.line).toBe("Listening to your reference…");
    expect(v.measuring).toBe(true);
  });

  it("track not analysed yet, and analysed without a six-band profile", () => {
    expect(view({ readout: readout("track_not_ready"), analysisReady: false }).line).toBe(
      "Listening to your track…",
    );
    expect(view({ readout: null, analysisReady: false }).state).toBe("track_not_ready");
    const noProfile = view({ readout: readout("track_not_ready"), analysisHasProfile: false });
    expect(noProfile.state).toBe("no_profile");
    expect(noProfile.line).toBe("Not enough audio to analyze this track.");
  });

  it("applied, with and without the partial notice", () => {
    const v = view({});
    expect(v.state).toBe("applied");
    expect(v.line).toBe(`Matching "${SHOWN}": cleared 1.2 dB of mud, added 0.8 dB of air.`);
    expect(v.partial).toBe(false);
    const partial = view({ readout: readout("applied", [0, -4, 0, 0, 3, 0], true) });
    expect(partial.partial).toBe(true);
    expect(partial.line).toBe(
      `Matching "${SHOWN}": tamed 4.0 dB of boom, added 3.0 dB of presence.` +
        " Your reference is very different from this track, so the match is partial.",
    );
  });

  it("nothing to change, including an applied readout with no nameable move", () => {
    const line = `Already close to "${SHOWN}". Nothing to change.`;
    expect(view({ readout: readout("nothing_to_change") }).line).toBe(line);
    expect(view({ readout: readout("applied", [0.1, 0, 0, 0, 0, 0]) }).line).toBe(line);
  });

  it("switched off", () => {
    const v = view({ settings: withMatch({ ...SONG_MATCH, enabled: false }), readout: readout("switched_off") });
    expect(v.state).toBe("switched_off");
    expect(v.line).toBe(`Match to "${SHOWN}" is off.`);
    expect(v.enabled).toBe(false);
  });

  it("match amount at 0", () => {
    const v = view({ settings: withMatch({ ...SONG_MATCH, amount: 0 }), readout: readout("zero_amount") });
    expect(v.state).toBe("zero_amount");
    expect(v.line).toBe("Match amount is at 0, so Smart Match is idle.");
  });

  it("album mode wins over everything else", () => {
    const v = view({ album: true, measuring: true, refusal: "silent" });
    expect(v.state).toBe("album_disabled");
    expect(v.line).toBe("Smart Match applies to Track Master, not Album renders.");
    expect(v.bands).toBeNull();
  });

  it("a refusal replaces the line but keeps the state and the reference", () => {
    const v = view({ refusal: "silent" });
    expect(v.state).toBe("applied");
    expect(v.name).toBe(SONG_MATCH.reference?.name);
    expect(v.line).toBe("That file can't be used as a reference: it is silent.");
    expect(v.partial).toBe(false);
    expect(view({ settings: withMatch(undefined), refusal: "too_short" }).line).toBe(
      "That file can't be used as a reference: it is shorter than 30 seconds.",
    );
  });

  it("marks a stale readout as pending only where it shows numbers", () => {
    expect(view({ pending: true }).pending).toBe(true);
    expect(view({ pending: true, readout: readout("nothing_to_change") }).pending).toBe(true);
    expect(view({ pending: true, settings: withMatch(undefined) }).pending).toBe(false);
  });

  it("shortens a long name inside every sentence", () => {
    const long = "y".repeat(80);
    const settings = withMatch({ ...SONG_MATCH, reference: { ...SONG_MATCH.reference!, name: long } });
    const v = view({ settings });
    expect(v.name).toBe(long);
    expect(v.line).toContain(`"${"y".repeat(59)}…"`);
  });
});

describe("Smart Match settings edits", () => {
  const REF = SONG_MATCH.reference!;

  it("a new reference starts switched on and keeps an amount already set", () => {
    expect(withSmartMatchReference(withMatch(undefined), REF).advanced.smart_match).toEqual({
      reference: REF,
      enabled: true,
      amount: 0.5,
    });
    const kept = withSmartMatchReference(withMatch({ reference: null, enabled: false, amount: 0.8 }), REF);
    expect(kept.advanced.smart_match).toEqual({ reference: REF, enabled: true, amount: 0.8 });
  });

  it("the switch only changes the switch, and needs a state to change", () => {
    expect(withSmartMatchEnabled(withMatch(SONG_MATCH), false).advanced.smart_match).toEqual({
      ...SONG_MATCH,
      enabled: false,
    });
    const none = withMatch(undefined);
    expect(withSmartMatchEnabled(none, false)).toBe(none);
  });

  it("removing the reference leaves nothing behind unless the amount was changed", () => {
    expect(withoutSmartMatchReference(withMatch(SONG_MATCH)).advanced).not.toHaveProperty("smart_match");
    expect(
      withoutSmartMatchReference(withMatch({ ...SONG_MATCH, amount: 0.7 })).advanced.smart_match,
    ).toEqual({ reference: null, enabled: true, amount: 0.7 });
  });

  it("the amount is clamped to 0..1 and a non-number falls back to the default", () => {
    expect(withSmartMatchAmount(withMatch(SONG_MATCH), 1.4).advanced.smart_match?.amount).toBe(1);
    expect(withSmartMatchAmount(withMatch(SONG_MATCH), -0.2).advanced.smart_match?.amount).toBe(0);
    expect(withSmartMatchAmount(withMatch(SONG_MATCH), Number.NaN).advanced.smart_match?.amount).toBe(0.5);
    expect(withSmartMatchAmount(withMatch(undefined), 0.3).advanced.smart_match).toEqual({
      reference: null,
      enabled: true,
      amount: 0.3,
    });
  });

  it("the Standard reset keeps the reference and its switch, and resets the amount", () => {
    const edited = withMatch({ ...SONG_MATCH, enabled: false, amount: 0.9 });
    expect(hasSmartMatchAmountEdit(edited)).toBe(true);
    const reset = resetSmartMatchAmount(edited);
    expect(reset.advanced.smart_match).toEqual({ ...SONG_MATCH, enabled: false, amount: 0.5 });
    expect(hasSmartMatchAmountEdit(reset)).toBe(false);
    expect(resetSmartMatchAmount(withMatch({ reference: null, enabled: true, amount: 0.9 })).advanced).not.toHaveProperty(
      "smart_match",
    );
    const none = withMatch(undefined);
    expect(resetSmartMatchAmount(none)).toBe(none);
    expect(hasSmartMatchAmountEdit(none)).toBe(false);
  });

  it("no edit mutates the settings it was given", () => {
    const s = withMatch({ ...SONG_MATCH, amount: 0.7 });
    const before = JSON.stringify(s);
    withSmartMatchReference(s, REF);
    withSmartMatchEnabled(s, false);
    withoutSmartMatchReference(s);
    withSmartMatchAmount(s, 0.2);
    resetSmartMatchAmount(s);
    expect(JSON.stringify(s)).toBe(before);
  });
});
