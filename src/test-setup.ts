// Vitest global setup (vite.config.ts `test.setupFiles`).
//
// jsdom ships no canvas implementation: HTMLCanvasElement.getContext logs a
// noisy "Not implemented" jsdomError on every call, which buries real stderr
// regressions once AnalysisOrb renders in component trees. Return null
// silently instead — that is exactly the component's documented no-draw path.
import { afterEach, beforeEach, vi } from "vitest";

HTMLCanvasElement.prototype.getContext = (() =>
  null) as typeof HTMLCanvasElement.prototype.getContext;

const ACT_WARNING = /not wrapped in act/;
let consoleErrorSpy: ReturnType<typeof vi.spyOn> | null = null;

beforeEach(() => {
  const originalError = console.error;
  consoleErrorSpy = vi.spyOn(console, "error").mockImplementation((...args) => {
    const message = args.map((arg) => String(arg)).join(" ");
    if (ACT_WARNING.test(message)) {
      throw new Error(`React act warning reached console.error: ${message}`);
    }
    originalError(...args);
  });
});

afterEach(() => {
  consoleErrorSpy?.mockRestore();
  consoleErrorSpy = null;
});
