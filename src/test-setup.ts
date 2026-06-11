// Vitest global setup (vite.config.ts `test.setupFiles`).
//
// jsdom ships no canvas implementation: HTMLCanvasElement.getContext logs a
// noisy "Not implemented" jsdomError on every call, which buries real stderr
// regressions once AnalysisOrb renders in component trees. Return null
// silently instead — that is exactly the component's documented no-draw path.
HTMLCanvasElement.prototype.getContext = (() =>
  null) as typeof HTMLCanvasElement.prototype.getContext;
