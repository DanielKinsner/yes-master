import type {
  IphoneSimpleCustomExport,
  IphoneSimpleExportProfile,
  IphoneSimpleLoudness,
  IphoneSimplePlan,
  IphoneSimpleTone,
} from "./simple-mode";
import { buildIphoneSimplePlan } from "./simple-mode";

export type IphoneAppMode = "simple";
export type IphonePlayback = "original" | "mastered";
export type IphoneAnalysisStatus = "idle" | "needed" | "ready";

export interface IphoneCustomExportSettings {
  ceilingDbtp: number;
  bitDepth: number | null;
  sampleRate: number | null;
}

export interface IphoneTrack {
  id: string;
  displayName: string;
  path: string;
  sourceFormat: string;
  durationSeconds: number | null;
}

export interface IphoneAppState {
  mode: IphoneAppMode;
  track: IphoneTrack | null;
  analysisStatus: IphoneAnalysisStatus;
  selectedTone: IphoneSimpleTone;
  selectedLoudness: IphoneSimpleLoudness;
  selectedExportProfile: IphoneSimpleExportProfile;
  customExport: IphoneCustomExportSettings;
  playback: IphonePlayback;
  playheadSeconds: number;
  volumeMatch: boolean;
  lufsPreview: boolean;
}

export const initialIphoneAppState: IphoneAppState = {
  mode: "simple",
  track: null,
  analysisStatus: "idle",
  selectedTone: "balanced",
  selectedLoudness: "medium",
  selectedExportProfile: "streaming",
  customExport: {
    ceilingDbtp: -1,
    bitDepth: null,
    sampleRate: null,
  },
  playback: "original",
  playheadSeconds: 0,
  volumeMatch: false,
  lufsPreview: false,
};

export function attachIphoneTrack(
  state: IphoneAppState,
  track: IphoneTrack,
): IphoneAppState {
  return {
    ...state,
    track,
    analysisStatus: "needed",
    playback: "original",
    playheadSeconds: 0,
  };
}

export function markIphoneAnalysisReady(state: IphoneAppState): IphoneAppState {
  if (!state.track) return state;
  return {
    ...state,
    analysisStatus: "ready",
  };
}

export function selectIphoneTone(
  state: IphoneAppState,
  selectedTone: IphoneSimpleTone,
): IphoneAppState {
  return {
    ...state,
    selectedTone,
  };
}

export function selectIphoneLoudness(
  state: IphoneAppState,
  selectedLoudness: IphoneSimpleLoudness,
): IphoneAppState {
  return {
    ...state,
    selectedLoudness,
  };
}

export function selectIphoneExportProfile(
  state: IphoneAppState,
  selectedExportProfile: IphoneSimpleExportProfile,
): IphoneAppState {
  return {
    ...state,
    selectedExportProfile,
  };
}

export function setIphoneCustomExport(
  state: IphoneAppState,
  customExport: IphoneCustomExportSettings,
): IphoneAppState {
  return {
    ...state,
    customExport,
  };
}

export function switchIphonePlayback(
  state: IphoneAppState,
  playback: IphonePlayback,
): IphoneAppState {
  return {
    ...state,
    playback,
  };
}

export function setIphonePlayhead(
  state: IphoneAppState,
  playheadSeconds: number,
): IphoneAppState {
  return {
    ...state,
    playheadSeconds: Math.max(0, playheadSeconds),
  };
}

export function toggleIphoneVolumeMatch(state: IphoneAppState): IphoneAppState {
  const volumeMatch = !state.volumeMatch;
  return {
    ...state,
    volumeMatch,
    lufsPreview: volumeMatch ? false : state.lufsPreview,
  };
}

export function toggleIphoneLufsPreview(state: IphoneAppState): IphoneAppState {
  const lufsPreview = !state.lufsPreview;
  return {
    ...state,
    volumeMatch: lufsPreview ? false : state.volumeMatch,
    lufsPreview,
  };
}

export function toIphoneSimplePlan(state: IphoneAppState): IphoneSimplePlan {
  return buildIphoneSimplePlan({
    tone: state.selectedTone,
    loudness: state.selectedLoudness,
    exportProfile: state.selectedExportProfile,
    volumeMatch: state.volumeMatch,
    lufsPreview: state.lufsPreview,
    customExport: toSimpleCustomExport(state.customExport),
  });
}

function toSimpleCustomExport(
  customExport: IphoneCustomExportSettings,
): IphoneSimpleCustomExport {
  return {
    ceilingDbtp: customExport.ceilingDbtp,
    bitDepth: customExport.bitDepth ?? undefined,
    sampleRate: customExport.sampleRate ?? undefined,
  };
}
