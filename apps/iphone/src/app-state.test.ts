import { describe, expect, it } from "vitest";
import {
  attachIphoneTrack,
  initialIphoneAppState,
  markIphoneAnalysisReady,
  selectIphoneExportProfile,
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
    expect(initialIphoneAppState.selectedExportProfile).toBe("streaming");
    expect(initialIphoneAppState.playback).toBe("original");
  });

  it("tracks import and analysis readiness for one iPhone track", () => {
    const imported = attachIphoneTrack(initialIphoneAppState, {
      id: "track-1",
      displayName: "rough mix.wav",
      durationSeconds: 182.4,
    });

    expect(imported.track?.displayName).toBe("rough mix.wav");
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
        selectIphoneExportProfile(
          selectIphoneLoudness(selectIphoneTone(initialIphoneAppState, "warm"), "high"),
          "cd",
        ),
      ),
    );

    const plan = toIphoneSimplePlan(state);

    expect(plan.auditionSettings.preset).toEqual({ kind: "warmth" });
    expect(plan.auditionSettings.volume_match).toBe(true);
    expect(plan.exportSettings.volume_match).toBe(false);
    expect(plan.exportSettings.advanced.lufs_offset_db).toBe(-10.5);
    expect(plan.exportSettings.advanced.bit_depth).toBe(16);
    expect(plan.previewLufsLanding).toBe(true);
  });
});
