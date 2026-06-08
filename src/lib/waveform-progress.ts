// What the central waveform deck shows while a track is being prepared.
//
// Importing a track is a two-phase wait the user stares straight at: first
// `analyzeTracks` (the slow part — tonal balance, dynamics, stereo field),
// then `prepareWaveform` (decode → peaks). Analysis exposes a staged,
// determinate-feeling progress (ANALYSIS_PROGRESS_STAGES in useTrackMaster);
// the waveform decode does not, so it reads as indeterminate. This helper
// collapses the two busy flags + the staged progress into a single view
// model so the deck can render a real progress bar instead of static text.

export type WaveformLoadingMode = "analyzing" | "loading" | "idle";

export interface WaveformLoadingView {
  mode: WaveformLoadingMode;
  label: string;
  /// 0..100 for the determinate analysis bar; null when there is no
  /// meaningful fraction (indeterminate decode, or idle).
  percent: number | null;
}

export function waveformLoadingView(input: {
  isAnalyzing: boolean;
  analysisProgress: { label: string; progress: number } | null;
  isLoadingWaveform: boolean;
}): WaveformLoadingView {
  if (input.isAnalyzing) {
    const fraction = input.analysisProgress
      ? Math.max(0, Math.min(1, input.analysisProgress.progress))
      : 0;
    return {
      mode: "analyzing",
      label: input.analysisProgress?.label ?? "Analyzing…",
      percent: Math.round(fraction * 100),
    };
  }
  if (input.isLoadingWaveform) {
    return { mode: "loading", label: "Loading waveform…", percent: null };
  }
  return { mode: "idle", label: "No waveform yet.", percent: null };
}
