// Single source of truth for the headless browser runtime (U3).
//
// DECISION (2026-07-24, U3 of the public beta quality plan): this lane uses
// Playwright's BUNDLED CHROMIUM, not the installed Google Chrome channel.
//
// Why. The landing script previously launched `chromium.launch({ channel:
// "chrome" })`, which requires a real Google Chrome install on the machine.
// That is fine on the owner's desktop and useless as a gate: the version is
// whatever Chrome happens to have auto-updated to that week, and a CI runner or
// a fresh clone has no Chrome at all. Bundled Chromium is pinned by the
// `playwright` dependency in package.json, so the browser version moves only
// when the lockfile moves — the same property that makes the rest of the
// verification lanes reproducible.
//
// Verified before adopting: the full landing assertion suite passes identically
// under both runtimes (Chrome 150.0.7871.186 and bundled Chromium
// 149.0.7827.55) against the same build. Evidence is in the go/no-go ledger.
//
// The cost is one setup step — `npx playwright install --with-deps chromium` —
// which CI now runs explicitly and the local docs name.
//
// A MISSING BROWSER MUST FAIL. This lane is a gate. A gate that quietly skips
// itself when a dependency is absent is worse than no gate, because it reports
// green. Every caller here exits nonzero rather than degrading.

import { chromium } from "playwright";

/// Recorded in evidence output so a reader can tell which runtime produced a
/// result without re-deriving it from the script.
export const BROWSER_RUNTIME = "playwright-bundled-chromium";

export const INSTALL_HINT = "npx playwright install --with-deps chromium";

/**
 * Launch the pinned headless browser, or exit nonzero with an actionable
 * message. Never returns a null/undefined browser and never resolves to a
 * "skipped" state.
 */
export async function launchHeadless() {
  try {
    return await chromium.launch({ headless: true });
  } catch (error) {
    console.error(
      "FAILED to launch Playwright's bundled Chromium. This lane is a gate and",
    );
    console.error("cannot be skipped, so this is a hard failure, not a warning.");
    console.error(`  ${error?.message ?? error}`);
    console.error("");
    console.error("Install the pinned browser with:");
    console.error(`  ${INSTALL_HINT}`);
    process.exit(1);
  }
}

/**
 * Report the launched browser's version. Used to stamp evidence output so a
 * stale or unexpected runtime is visible in the summary rather than implied.
 */
export function runtimeStamp(browser) {
  return {
    runtime: BROWSER_RUNTIME,
    version: browser.version(),
  };
}
