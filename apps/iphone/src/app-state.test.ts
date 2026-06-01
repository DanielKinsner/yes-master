import { describe, expect, it } from "vitest";
import {
  attachIphoneTrack,
  initialIphoneAppState,
  markIphoneAnalysisReady,
  selectIphoneLoudness,
  selectIphoneTone,
  setIphonePlayhead,
  switchIphonePlayback,
  toggleIphoneLufsPreview,
  toggleIphoneVolumeMatch,
  toIphoneSimplePlan,
} from "./app-state";

describe("iPhone app state", () => {
  it("starts as a Simple-only empty iPhone app", () => {
    expect(initialIphoneAppState.mode).toBe("simple");
    expect(initialIphoneAppState.track).toBeNull();
    expect(initialIphoneAppState.selectedTone).toBe("balanced");
    expect(initialIphoneAppState.selectedLoudness).toBe("medium");
    expect(initialIphoneAppState.playback).toBe("original");
  });

  it("tracks import and analysis readiness for one iPhone track", () => {
    const imported = attachIphoneTrack(initialIphoneAppState, {
      id: "track-1",
      displayName: "rough mix.wav",
      path: "/private/rough mix.wav",
      sourceFormat: "wav",
      durationSeconds: 182.4,
      sampleRate: 44_100,
      channels: 2,
    });

    expect(imported.track?.displayName).toBe("rough mix.wav");
    expect(imported.track?.sampleRate).toBe(44_100);
    expect(imported.track?.channels).toBe(2);
    expect(imported.analysisStatus).toBe("needed");

    const ready = markIphoneAnalysisReady(imported);

    expect(ready.track?.displayName).toBe("rough mix.wav");
    expect(ready.analysisStatus).toBe("ready");
  });

  it("preserves the playhead when switching Original and Mastered", () => {
    const atChorus = setIphonePlayhead(initialIphoneAppState, 64.25);
    const mastered = switchIphonePlayback(atChorus, "mastered");
    const original = switchIphonePlayback(mastered, "original");

    expect(mastered.playheadSeconds).toBe(64.25);
    expect(original.playheadSeconds).toBe(64.25);
  });

  it("feeds the Simple contract from the selected phone controls", () => {
    const state = toggleIphoneLufsPreview(
      toggleIphoneVolumeMatch(
        selectIphoneLoudness(selectIphoneTone(initialIphoneAppState, "warm"), "high"),
      ),
    );

    const plan = toIphoneSimplePlan(state);

    expect(plan.auditionSettings.preset).toEqual({ kind: "warmth" });
    expect(plan.auditionSettings.volume_match).toBe(false);
    expect(plan.exportSettings.volume_match).toBe(false);
    expect(plan.exportSettings.advanced.lufs_offset_db).toBe(-9);
    expect(plan.exportSettings.advanced.bit_depth).toBe(24);
    expect(plan.exportSettings.advanced.target_sample_rate).toBe(44_100);
    expect(plan.previewLufsLanding).toBe(true);
  });

  it("keeps Volume Match and LUFS Preview mutually exclusive", () => {
    const volumeMatched = toggleIphoneVolumeMatch(initialIphoneAppState);
    const lufsPreviewed = toggleIphoneLufsPreview(volumeMatched);
    const volumeMatchedAgain = toggleIphoneVolumeMatch(lufsPreviewed);

    expect(volumeMatched.volumeMatch).toBe(true);
    expect(volumeMatched.lufsPreview).toBe(false);
    expect(lufsPreviewed.volumeMatch).toBe(false);
    expect(lufsPreviewed.lufsPreview).toBe(true);
    expect(volumeMatchedAgain.volumeMatch).toBe(true);
    expect(volumeMatchedAgain.lufsPreview).toBe(false);
  });

  it("always feeds the fixed iPhone WAV export format into the Simple plan", () => {
    const plan = toIphoneSimplePlan(initialIphoneAppState);

    expect(plan.exportSettings.advanced.ceiling_dbtp).toBe(-1);
    expect(plan.exportSettings.advanced.bit_depth).toBe(24);
    expect(plan.exportSettings.advanced.target_sample_rate).toBe(44_100);
    expect(plan.exportSettings.advanced.lufs_offset_db).toBe(-11);
  });
});
