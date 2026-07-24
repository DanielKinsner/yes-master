#!/usr/bin/env node
//
// Deterministic /app preview journeys (U3).
//
// SYNTHETIC EVIDENCE. A pass here proves that the desktop UI renders, lays out,
// and transitions correctly in a headless browser against the preview mock. It
// proves NOTHING about native file dialogs, real audio, installer signing,
// updater installation, or how any of it sounds. Those need the installed
// machine and owner listening layers (U15/U16).
//
// Each scenario below is a NAMED test: a fixed preview state (`?scenario=`),
// the viewports it must survive, the visible state expected, and the console
// behavior permitted. An unrecognized preview command or listen channel emits a
// `[preview-mock] unhandled ...` warning, and this lane fails on any warning —
// so the mock contract cannot silently rot.

import { mkdir, writeFile } from "node:fs/promises";
import path from "node:path";
import { launchHeadless, runtimeStamp } from "./lib/headless-browser.mjs";

function option(name) {
  const index = process.argv.indexOf(name);
  return index === -1 ? undefined : process.argv[index + 1];
}

function stamp() {
  return new Date().toISOString().replace(/[:.]/g, "-");
}

const baseUrl = (
  option("--url") ??
  process.env.PREVIEW_URL ??
  "http://127.0.0.1:5177"
).replace(/\/$/, "");
const outDir =
  option("--out") ?? path.join("test-output", "app-headless", stamp());

// A forced-failure switch, used to prove the lane actually fails (and reports
// viewport/route/scenario/screenshot) rather than only ever passing.
const forceFailScenario = option("--force-fail");

// The supported minimum desktop size is 1360x740 (APP_BEHAVIOR.md, resolved
// 2026-07-08). Every scenario is checked there as well as at a common laptop
// size, because "fits at 1440x900" has never been the interesting question.
const MIN_DESKTOP = [1360, 740];
const LAPTOP = [1440, 900];

// The preview's staged analysis runs 5 x 800ms. Settling is detected by
// polling for a terminal state rather than sleeping a fixed amount, but this
// bounds how long we are willing to wait for one.
const SETTLE_TIMEOUT_MS = 20_000;

/**
 * Scenario table. `mustContain` / `mustNotContain` are matched against the
 * whitespace-collapsed innerText of <body>.
 */
const SCENARIOS = [
  {
    name: "empty",
    purpose: "True first-run state: no session, no track, empty state visible.",
    viewports: [LAPTOP, MIN_DESKTOP],
    settle: "empty",
    mustContain: ["Drop audio. Hear it mastered.", "Import audio"],
    mustNotContain: ["READY"],
    mustNotHaveControls: ["Export With Review"],
  },
  {
    name: "clean",
    purpose: "Seeded single-track project analyses to a ready, warning-free state.",
    viewports: [LAPTOP, MIN_DESKTOP],
    settle: "ready",
    mustContain: ["READY", "Original", "Mastered"],
    mustNotContain: ["Drop audio. Hear it mastered."],
    mustHaveControls: ["Export With Review", "Original", "Mastered"],
  },
  {
    name: "warning",
    purpose:
      "Export checks report a warning and a critical; both reach the review surface.",
    viewports: [LAPTOP],
    settle: "ready",
    drive: exportWithReview,
    mustContainAfterDrive: [
      "Dynamic range is low",
      "True peak exceeds the delivery ceiling",
    ],
  },
  {
    name: "long-copy",
    purpose:
      "Pathological filename length must not overflow or hide controls at the minimum size.",
    viewports: [LAPTOP, MIN_DESKTOP],
    settle: "ready",
    mustContain: ["READY"],
    mustHaveControls: ["Export With Review"],
  },
  {
    name: "export-success",
    purpose: "Save dialog returns a path; the export completes.",
    viewports: [LAPTOP],
    settle: "ready",
    drive: exportWithReview,
    mustContainAfterDrive: ["preview-master.wav"],
  },
  {
    name: "export-cancel",
    purpose:
      "Save dialog is cancelled; the UI must NOT claim the export succeeded.",
    viewports: [LAPTOP],
    settle: "ready",
    drive: exportWithReview,
    mustNotContainAfterDrive: ["preview-master.wav"],
  },
  {
    name: "album-1",
    purpose: "Album mode with a single track — the degenerate album case.",
    viewports: [LAPTOP, MIN_DESKTOP],
    settle: "album",
    mustContain: ["ALBUM", "1 TRACK"],
  },
  {
    name: "album-4",
    purpose: "Album mode, four tracks, one overriding the album settings.",
    viewports: [LAPTOP, MIN_DESKTOP],
    settle: "album",
    mustContain: ["ALBUM", "4 TRACKS", "ALBUM FLOW", "01 - Preview Track 1.wav"],
  },
  {
    name: "album-12",
    purpose:
      "Album mode, twelve tracks — scrolling, reorder boundaries, selection retention.",
    viewports: [LAPTOP, MIN_DESKTOP],
    settle: "album",
    mustContain: ["ALBUM", "12 TRACKS", "12 - Preview Track 12.wav"],
    checkAlbumRows: 12,
  },
  {
    name: "album-long",
    purpose:
      "Album mode with long track names and a long album title at the minimum size.",
    viewports: [LAPTOP, MIN_DESKTOP],
    settle: "album",
    mustContain: ["ALBUM", "12 TRACKS"],
    checkAlbumRows: 12,
  },
  {
    name: "album-warning",
    purpose: "Album mode where export checks surface a warning.",
    viewports: [LAPTOP],
    settle: "album",
    mustContain: ["ALBUM", "4 TRACKS"],
  },
];

// ---------------------------------------------------------------------------
// Console policy
// ---------------------------------------------------------------------------
//
// Any console error or warning fails the lane. That is the point: the preview
// mock warns on an unhandled command or listen channel, so a drifted contract
// becomes a red lane instead of noise nobody reads.
//
// The allowlist below is for behavior that is DELIBERATELY unsupported in a
// browser and named as such. Each entry needs a reason.
const CONSOLE_ALLOWLIST = [
  {
    // Favicon is served from /src in dev-shaped HTML; a 404 here says nothing
    // about the app.
    pattern: /favicon/i,
    reason: "favicon 404 in the built preview is unrelated to app behavior",
  },
  {
    // Named native-only commands. The mock logs these at info level with a
    // distinct prefix precisely so "we know" is distinguishable from "nobody
    // looked". They are info, not warn, so they would not fail anyway — listed
    // so the allowlist documents the whole contract in one place.
    pattern: /\[preview-mock\] native-only command, not simulated:/,
    reason: "installing an update cannot be proved in a browser (U16 owns it)",
  },
  {
    pattern: /\[preview-mock\] (open|save)\(\) returned/,
    reason: "native dialog outcomes are scenario-driven, logged at info level",
  },
];

function isAllowedConsole(text) {
  return CONSOLE_ALLOWLIST.some((entry) => entry.pattern.test(text));
}

// ---------------------------------------------------------------------------
// Drivers
// ---------------------------------------------------------------------------

/**
 * Drive the real two-step export: "Export With Review" opens the review gate
 * ("Adjust Settings" / "Export Anyway"), and only the second click reaches the
 * save dialog. Clicking once and asserting on the result would test the gate,
 * not the export.
 */
async function exportWithReview(page) {
  const open = page
    .locator("button", { hasText: /^Export With Review$/ })
    .first();
  if ((await open.count()) === 0) {
    throw new Error("Export With Review button not found");
  }
  await open.click();

  const confirm = page.locator("button", { hasText: /^Export Anyway$/ }).first();
  await waitForControl(page, /^Export Anyway$/, "the export review gate");
  await confirm.click();

  // Render + save dialog + receipt all resolve immediately in the preview, but
  // React commits are async. Wait for the DOM to actually change.
  const before = await documentText(page);
  await waitForText(
    page,
    (text) => text !== before,
    "the export to resolve",
    documentText,
  ).catch(() => {
    // A cancelled export legitimately changes little or nothing. That is the
    // expected outcome for export-cancel, so absence of change is not an error
    // here -- the scenario's assertions decide whether it was correct.
  });
}

async function waitForControl(page, pattern, description) {
  const deadline = Date.now() + SETTLE_TIMEOUT_MS;
  let names = [];
  while (Date.now() < deadline) {
    names = await controlNames(page);
    if (names.some((name) => pattern.test(name))) return;
    await page.waitForTimeout(200);
  }
  throw new Error(
    `${description} never appeared; controls present: ${JSON.stringify(names.slice(-16))}`,
  );
}

// innerText respects layout, so it reflects what is actually laid out and
// visible -- but the desktop right rail is a scroll container, and innerText
// omits content scrolled out of it. Two different questions therefore need two
// different reads:
//
//   bodyText     -> "is this VISIBLE right now" (innerText)
//   documentText -> "does this EXIST in the DOM at all" (textContent)
//
// Using innerText for existence checks silently under-reports every control
// below the rail's fold, which is exactly the mistake this comment exists to
// stop the next person repeating.
const bodyText = (page) =>
  page.evaluate(() => document.body.innerText.replace(/\s+/g, " "));

const documentText = (page) =>
  page.evaluate(() => (document.body.textContent ?? "").replace(/\s+/g, " "));

const controlNames = (page) =>
  page.evaluate(() =>
    Array.from(document.querySelectorAll("button")).map((element) =>
      (element.getAttribute("aria-label") || element.textContent || "").trim(),
    ),
  );

/**
 * Wait until a named terminal marker appears in the page text.
 *
 * Deliberately NOT "wait until the text stops changing": the preview's staged
 * analysis pauses 800ms between stages, so text-stability polling reports
 * "settled" mid-analysis and every downstream assertion then fails against a
 * loading screen. Waiting for an explicit terminal marker is the only honest
 * signal, and a marker that never arrives is a real failure, not a timeout to
 * paper over.
 */
async function waitForMarker(page, marker, label) {
  if (!marker) return bodyText(page);
  const deadline = Date.now() + SETTLE_TIMEOUT_MS;
  let last = "";
  while (Date.now() < deadline) {
    last = await bodyText(page);
    if (marker(last)) return last;
    await page.waitForTimeout(200);
  }
  throw new Error(
    `never reached the "${label}" state within ${SETTLE_TIMEOUT_MS}ms; last text began: ${last.slice(0, 200)}`,
  );
}

/** Wait for a post-interaction condition, or throw with what was seen. */
async function waitForText(page, predicate, description, read = bodyText) {
  const deadline = Date.now() + SETTLE_TIMEOUT_MS;
  let last = "";
  while (Date.now() < deadline) {
    last = await read(page);
    if (predicate(last)) return last;
    await page.waitForTimeout(200);
  }
  throw new Error(
    `${description} did not happen within ${SETTLE_TIMEOUT_MS}ms; last text began: ${last.slice(0, 200)}`,
  );
}

const SETTLE_MARKERS = {
  empty: (text) => text.includes("Drop audio. Hear it mastered."),
  ready: (text) => text.includes("READY"),
  album: (text) => /\d+ TRACKS?\b/.test(text),
};

// ---------------------------------------------------------------------------
// Run
// ---------------------------------------------------------------------------

await mkdir(outDir, { recursive: true });

const failures = [];
const records = [];
const browser = await launchHeadless();

/** Record a failure with everything needed to act on it without re-running. */
function fail({ scenario, route, viewport, screenshot, message }) {
  failures.push(
    `[scenario=${scenario}] [route=${route}] [viewport=${viewport}] ${message}\n    screenshot: ${screenshot}`,
  );
}

for (const scenario of SCENARIOS) {
  for (const [width, height] of scenario.viewports) {
    const viewportLabel = `${width}x${height}`;
    const route = `/app?scenario=${scenario.name}`;
    const url = `${baseUrl}${route}`;
    const screenshot = path.join(
      outDir,
      `${scenario.name}-${viewportLabel}.png`,
    );

    const context = await browser.newContext({ viewport: { width, height } });
    const page = await context.newPage();
    const consoleMessages = [];
    const pageErrors = [];

    page.on("console", (message) => {
      if (!["error", "warning", "warn"].includes(message.type())) return;
      const text = message.text();
      if (isAllowedConsole(text)) return;
      consoleMessages.push({ type: message.type(), text });
    });
    page.on("pageerror", (error) => pageErrors.push(error.message));

    try {
      await page.goto(url, { waitUntil: "networkidle" });
      let text = await waitForMarker(
        page,
        SETTLE_MARKERS[scenario.settle],
        scenario.settle,
      );

      if (scenario.drive) {
        await scenario.drive(page);
        text = await bodyText(page);
      }
      const domText = await documentText(page);
      const controls = await controlNames(page);

      await page.screenshot({ path: screenshot, fullPage: false });

      const metrics = await page.evaluate(() => {
        const doc = document.documentElement;
        return {
          horizontalOverflow:
            Math.max(doc.scrollWidth, document.body.scrollWidth) >
            doc.clientWidth + 1,
          scrollWidth: Math.max(doc.scrollWidth, document.body.scrollWidth),
          clientWidth: doc.clientWidth,
          brokenImages: Array.from(document.images)
            .filter((image) => !image.complete || image.naturalWidth === 0)
            .map((image) => image.currentSrc || image.src),
          // Album rows carry a two-digit position prefix in their label.
          albumRowCount: document.querySelectorAll(
            '[aria-label="Album order controls"]',
          ).length,
          focusableCount: document.querySelectorAll(
            "button:not([disabled]), a[href], input, select, textarea",
          ).length,
        };
      });

      const report = (message) =>
        fail({
          scenario: scenario.name,
          route,
          viewport: viewportLabel,
          screenshot,
          message,
        });

      // Deliberate self-test hook.
      if (forceFailScenario === scenario.name) {
        report("forced assertion failure (--force-fail)");
      }

      if (metrics.horizontalOverflow) {
        report(
          `horizontal overflow (${metrics.scrollWidth} > ${metrics.clientWidth})`,
        );
      }
      if (metrics.brokenImages.length > 0) {
        report(`broken images: ${metrics.brokenImages.join(", ")}`);
      }
      if (metrics.focusableCount === 0) {
        report("no focusable control rendered — the shell did not boot");
      }
      for (const needle of scenario.mustContain ?? []) {
        if (!text.includes(needle)) {
          report(`expected visible text missing: "${needle}"`);
        }
      }
      for (const needle of scenario.mustNotContain ?? []) {
        if (text.includes(needle)) {
          report(`forbidden text present: "${needle}"`);
        }
      }
      for (const name of scenario.mustHaveControls ?? []) {
        if (!controls.includes(name)) {
          report(`expected control missing: "${name}"`);
        }
      }
      for (const name of scenario.mustNotHaveControls ?? []) {
        if (controls.includes(name)) {
          report(`forbidden control present: "${name}"`);
        }
      }
      for (const needle of scenario.mustContainAfterDrive ?? []) {
        if (!domText.includes(needle)) {
          report(`expected text missing after interaction: "${needle}"`);
        }
      }
      for (const needle of scenario.mustNotContainAfterDrive ?? []) {
        if (domText.includes(needle)) {
          report(`forbidden text present after interaction: "${needle}"`);
        }
      }
      if (
        scenario.checkAlbumRows &&
        metrics.albumRowCount !== scenario.checkAlbumRows
      ) {
        report(
          `album row count is ${metrics.albumRowCount}, expected ${scenario.checkAlbumRows}`,
        );
      }
      if (consoleMessages.length > 0) {
        report(`console errors/warnings: ${JSON.stringify(consoleMessages)}`);
      }
      if (pageErrors.length > 0) {
        report(`page errors: ${JSON.stringify(pageErrors)}`);
      }

      records.push({
        scenario: scenario.name,
        purpose: scenario.purpose,
        route,
        viewport: viewportLabel,
        screenshot,
        metrics,
        consoleMessages,
        pageErrors,
      });
    } catch (error) {
      await page
        .screenshot({ path: screenshot, fullPage: false })
        .catch(() => {});
      fail({
        scenario: scenario.name,
        route,
        viewport: viewportLabel,
        screenshot,
        message: `threw: ${error?.message ?? error}`,
      });
    } finally {
      await context.close();
    }
  }
}

const browserStamp = runtimeStamp(browser);
await browser.close();

await writeFile(
  path.join(outDir, "summary.json"),
  `${JSON.stringify(
    {
      baseUrl,
      outDir,
      evidenceLayer: "browser-headless",
      browser: browserStamp,
      consoleAllowlist: CONSOLE_ALLOWLIST.map((entry) => ({
        pattern: String(entry.pattern),
        reason: entry.reason,
      })),
      scenarios: records,
      failures,
    },
    null,
    2,
  )}\n`,
);

if (failures.length > 0) {
  console.error(`App headless verification FAILED. Evidence: ${outDir}`);
  for (const failure of failures) console.error(`- ${failure}`);
  process.exit(1);
}

console.log(
  `App headless verification passed (${records.length} scenario/viewport checks). Evidence: ${outDir}`,
);
