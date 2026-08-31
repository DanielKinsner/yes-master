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

import { mkdir, readFile, writeFile } from "node:fs/promises";
import { createRequire } from "node:module";
import path from "node:path";
import { launchHeadless, runtimeStamp } from "./lib/headless-browser.mjs";

// Audit A-02: the committed axe-core build (pinned by the lockfile) is the
// scanner — never a CDN copy, so the gate is reproducible offline.
const axeSource = await readFile(
  createRequire(import.meta.url).resolve("axe-core/axe.min.js"),
  "utf8",
);

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

// U10(c) reachability targets. The two the unit names explicitly, because both
// live at the far end of a scroll container in a short viewport: Delivery
// Format is the last rail card above the sticky export group, and Export Album
// is the album mode's terminal action.
const DELIVERY_FORMAT = {
  label: "Delivery Format card",
  selector: ".rail-card-format",
};
const EXPORT_ALBUM = {
  label: "Export Album button",
  selector: "button",
  text: "Export Album",
};
const EXPORT_MASTER = {
  label: "Export action",
  selector: "button.right-rail-export",
};

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
    scenarioId: "S-D1",
    purpose:
      "S-D1 (no track): first-run state offers import and no impossible export.",
    viewports: [LAPTOP, MIN_DESKTOP],
    settle: "empty",
    mustContain: ["Drop audio. Hear it mastered.", "Import audio"],
    mustNotContain: ["READY"],
    mustNotHaveControls: ["Export With Review"],
  },
  {
    name: "clean",
    scenarioId: "S-F1",
    purpose:
      "S-F1 (Advanced, clean): seeded single-track project reaches a ready, warning-free state.",
    viewports: [LAPTOP, MIN_DESKTOP],
    settle: "ready",
    mustContain: ["READY", "Original", "Mastered"],
    mustNotContain: ["Drop audio. Hear it mastered."],
    mustHaveControls: ["Export With Review", "Original", "Mastered"],
    mustReach: [DELIVERY_FORMAT, EXPORT_MASTER],
    // U11: carries the quality-verdict badge; the reduced-motion pass proves
    // the verdict still reads with the attention cue switched off.
    reducedMotionVariant: true,
    // Audit A-02: loaded Advanced is a permanently axe-gated state.
    axe: true,
  },
  {
    // Audit U-01. The static CSS test used to read the FIRST `.right-rail-tools`
    // source block (opaque) while the LAST unconditional block won the cascade
    // with `background: transparent` — a false green. Only the browser's
    // computed style is authoritative, so this scenario proves opacity at the
    // exact moment rail content genuinely sits behind the sticky surface.
    name: "clean",
    label: "clean-tools-overlap",
    scenarioId: "S-F1",
    purpose:
      "S-F1 (sticky TOOLS, audit U-01): with the rail scrolled so a section " +
      "genuinely intersects the sticky TOOLS surface, its computed background " +
      "must be fully opaque and own the overlapped pixels.",
    viewports: [LAPTOP, MIN_DESKTOP],
    settle: "ready",
    beforeScreenshot: toolsOverlapProbe,
  },
  {
    name: "warning",
    scenarioId: "S-F1",
    purpose:
      "S-F1 (warnings): a warning and a critical both reach the review surface.",
    // Audit A-02: the open warning receipt is axe-gated at BOTH supported
    // desktop sizes, so the minimum viewport joined the laptop one here.
    viewports: [LAPTOP, MIN_DESKTOP],
    settle: "ready",
    drive: exportWithReview,
    mustContainAfterDrive: [
      "Dynamic range is low",
      "True peak exceeds the delivery ceiling",
    ],
    // U11: opens the review gate, so this covers the overlay entrance in both
    // motion modes — the warning must be readable either way.
    reducedMotionVariant: true,
    // Audit A-02: scanned while the receipt is OPEN (the drive leaves it up),
    // so the modal's own accessibility is what the scan measures.
    axe: true,
  },
  {
    name: "long-copy",
    scenarioId: "S-F1",
    purpose:
      "S-F1 (long copy): pathological filename length must not overflow or hide controls.",
    viewports: [LAPTOP, MIN_DESKTOP],
    settle: "ready",
    mustContain: ["READY"],
    mustHaveControls: ["Export With Review"],
    // U10(c): long copy is exactly what pushes the lower rail cards down, so
    // this is where "Delivery Format still reachable" is worth asking.
    mustReach: [DELIVERY_FORMAT, EXPORT_MASTER],
  },
  {
    name: "export-success",
    scenarioId: "S-F3",
    purpose: "S-F3 (write): save dialog returns a path; the export completes.",
    viewports: [LAPTOP],
    settle: "ready",
    drive: exportWithReview,
    mustContainAfterDrive: ["preview-master.wav"],
  },
  {
    name: "export-cancel",
    scenarioId: "S-F3",
    purpose:
      "S-F3 (no write): save dialog cancelled; the UI must NOT claim success.",
    viewports: [LAPTOP],
    settle: "ready",
    drive: exportWithReview,
    // U10: a cancelled export must leave no trace of success — not the output
    // path, and not any of the language a completed export uses. Claiming a
    // file was written when the user cancelled is the single worst thing this
    // surface could do.
    mustNotContainAfterDrive: [
      "preview-master.wav",
      "Reveal in file manager",
      "Ready to ship",
    ],
  },
  {
    name: "album-1",
    scenarioId: "S-F2",
    purpose:
      "S-F2 (one track): album mode with a single track — the degenerate case.",
    viewports: [LAPTOP, MIN_DESKTOP],
    settle: "album",
    mustContain: ["ALBUM", "1 TRACK"],
    mustReach: [DELIVERY_FORMAT, EXPORT_ALBUM],
  },
  {
    name: "album-4",
    scenarioId: "S-F2",
    purpose:
      "S-F2 (four tracks): follow/override states, ordering, and common settings.",
    viewports: [LAPTOP, MIN_DESKTOP],
    settle: "album",
    mustContain: ["ALBUM", "4 TRACKS", "ALBUM FLOW", "01 - Preview Track 1.wav"],
    mustReach: [DELIVERY_FORMAT, EXPORT_ALBUM],
    // U11: the album sequence arc and its settle animation live here.
    reducedMotionVariant: true,
  },
  {
    name: "album-12",
    scenarioId: "S-F2",
    purpose:
      "S-F2 (twelve tracks): scrolling, reorder boundaries, selection retention.",
    viewports: [LAPTOP, MIN_DESKTOP],
    settle: "album",
    mustContain: ["ALBUM", "12 TRACKS", "12 - Preview Track 12.wav"],
    checkAlbumRows: 12,
    // U10(c): twelve rows is the case that pushes the album's terminal action
    // furthest down the rail.
    mustReach: [DELIVERY_FORMAT, EXPORT_ALBUM],
  },
  {
    name: "album-long",
    scenarioId: "S-F2",
    purpose:
      "S-F2 (long names): full-name discovery must not depend on clipped labels.",
    viewports: [LAPTOP, MIN_DESKTOP],
    settle: "album",
    mustContain: ["ALBUM", "12 TRACKS"],
    checkAlbumRows: 12,
    mustReach: [DELIVERY_FORMAT, EXPORT_ALBUM],
  },
  {
    name: "album-warning",
    scenarioId: "S-F2",
    purpose: "S-F2 (warnings): album mode where export checks surface a warning.",
    viewports: [LAPTOP],
    settle: "album",
    mustContain: ["ALBUM", "4 TRACKS"],
  },
  {
    // S-E1 (plan scenario family). Proving unit U10; closing owner U15, whose
    // listening pass judges what this cannot: how the switch SOUNDS.
    // Reuses the `clean` preview state but is a distinct named case, so it
    // carries its own label for screenshots and evidence records.
    name: "clean",
    label: "S-E1-rapid-ab",
    scenarioId: "S-E1",
    purpose:
      "S-E1: analyze, then switch Original->Mastered, Mastered->Original, and " +
      "rapidly in both directions with Volume Match off and on. Playhead is " +
      "preserved; no directional stall, stale readiness error, or inconsistent state.",
    viewports: [LAPTOP],
    settle: "ready",
    drive: rapidAbSwitching,
    // A stale readiness error must not surface after the switches resolved.
    mustNotContainAfterDrive: ["Mastered preview is still preparing"],
    assert: async (page, report) => {
      const run = await abRun(page);
      if (!run) {
        report("S-E1: the A/B run recorded nothing — the driver did not execute");
        return;
      }
      if (!run.volumeMatchToggled) {
        report(
          "S-E1: Volume Match was never toggled, so only half the matrix ran — " +
            "the control was not found",
        );
      }
      // Without this the whole scenario is vacuous: a transport pinned at 0
      // can never dip to 0, so every playhead assertion below would pass on a
      // completely dead app.
      if (typeof run.startPlayhead !== "number" || run.startPlayhead <= 0.05) {
        report(
          `S-E1: playback never started (playhead ${run.startPlayhead}) — the ` +
            "switching run proves nothing about playhead preservation",
        );
      }
      if (run.samples.length < 6) {
        report(
          `S-E1: only ${run.samples.length} of 6 switches completed — a direction stalled`,
        );
      }
      if (run.samples.some((value) => value === null)) {
        report(
          "S-E1: the playhead was unreadable during switching — the waveform " +
            "slider lost its aria-valuenow",
        );
      }
      // The playhead must survive the switch. A reset to zero is the specific
      // failure the scenario names ("no near-zero dip"): switching source is
      // not supposed to rewind the track.
      const dipped = run.samples.filter(
        (value) => typeof value === "number" && value < 0.05,
      );
      if (run.startPlayhead !== null && run.startPlayhead > 0.05 && dipped.length > 0) {
        report(
          `S-E1: playhead dipped to ~0 during switching (started at ${run.startPlayhead}, ` +
            `saw ${JSON.stringify(dipped)}) — the switch rewound the track`,
        );
      }
      // Each switch must land on the side it was asked for. A null or "false"
      // here is a stalled direction: the click was accepted, the state was not.
      run.pressedStates.forEach((state, index) => {
        if (state !== "true") {
          report(
            `S-E1: switch ${index + 1} (to ${index % 2 === 0 ? "Mastered" : "Original"}) ` +
              `did not take — aria-pressed was ${state}`,
          );
        }
      });
    },
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

/**
 * Audit A-02 — WCAG 2.0/2.1 A+AA axe scan of the live app state.
 *
 * Violations fail the lane. Incomplete results are persisted as review
 * evidence, not passes, and never silently discarded. A scan that runs zero
 * rules proves nothing and is itself a failure. No rule is suppressed here
 * to green the lane — a real violation gets a focused fix commit instead.
 */
async function runAxeScan(page, report, contextLabel) {
  await page.evaluate(axeSource);
  const results = await page.evaluate(async () =>
    window.axe.run(document, {
      runOnly: {
        type: "tag",
        values: ["wcag2a", "wcag2aa", "wcag21a", "wcag21aa"],
      },
    }),
  );
  const compact = (list) =>
    list.map((rule) => ({
      id: rule.id,
      impact: rule.impact ?? null,
      nodes: rule.nodes.length,
      targets: rule.nodes.slice(0, 5).map((node) => node.target.join(" ")),
      failureSummary:
        rule.nodes[0]?.failureSummary ?? rule.description ?? rule.id,
    }));
  const payload = {
    totalRules:
      results.passes.length +
      results.incomplete.length +
      results.violations.length,
    passes: results.passes.length,
    violations: compact(results.violations),
    incomplete: compact(results.incomplete),
  };
  if (payload.totalRules === 0) {
    report(`axe scan at ${contextLabel} ran zero rules — the scan proves nothing`);
  }
  for (const violation of payload.violations) {
    report(
      `axe violation "${violation.id}" (${violation.impact}, ${violation.nodes} node(s)) ` +
        `at ${contextLabel}: ${violation.failureSummary} [${violation.targets.join(", ")}]`,
    );
  }
  return payload;
}

/**
 * Audit U-01 — the sticky TOOLS surface, measured where it matters.
 *
 * Never model CSS precedence as "last matching text block wins": specificity,
 * `!important`, media conditions, and source order all participate, so only
 * `getComputedStyle` on a live page is authoritative. This probe:
 *   1. walks the rail's scroll range until a `.rail-section` box genuinely
 *      intersects the `.right-rail-tools` box (no overlap = the probe proved
 *      nothing, which is itself a failure);
 *   2. requires the computed background alpha at that moment to be >= 0.999 —
 *      a merely translucent surface is not a pass;
 *   3. asks `elementFromPoint` at an actual intersection point and requires
 *      the topmost element to be TOOLS or a descendant of it.
 * The returned payload lands in summary.json beside the screenshot taken in
 * the SAME frame, so the image and the measurements describe one state.
 */
async function toolsOverlapProbe(page, report) {
  const result = await page.evaluate(() => {
    const rail = document.querySelector(".right-rail");
    const tools = document.querySelector(".right-rail-tools");
    if (!rail || !tools) return { found: false };

    const sections = Array.from(
      document.querySelectorAll(".right-rail .rail-section"),
    );
    const intersect = (a, b) => {
      const left = Math.max(a.left, b.left);
      const right = Math.min(a.right, b.right);
      const top = Math.max(a.top, b.top);
      const bottom = Math.min(a.bottom, b.bottom);
      // Require a meaningful overlap, not a 1px rounding kiss.
      return right - left > 4 && bottom - top > 4
        ? { left, right, top, bottom }
        : null;
    };

    let overlap = null;
    let overlapCount = 0;
    const maxScroll = rail.scrollHeight - rail.clientHeight;
    for (let scrollTop = 0; scrollTop <= maxScroll; scrollTop += 8) {
      rail.scrollTop = scrollTop;
      const toolsRect = tools.getBoundingClientRect();
      const hits = sections
        .map((section) => intersect(section.getBoundingClientRect(), toolsRect))
        .filter(Boolean);
      if (hits.length > 0) {
        overlap = hits[0];
        overlapCount = hits.length;
        break;
      }
    }

    const style = getComputedStyle(tools);
    const backgroundColor = style.backgroundColor;
    // Computed colors normalize to rgb(...) (alpha 1) or rgba(..., a).
    const match = backgroundColor.match(/^rgba?\(([^)]+)\)$/);
    const parts = match ? match[1].split(",").map((p) => parseFloat(p)) : [];
    const alpha = parts.length === 4 ? parts[3] : match ? 1 : 0;

    let sampledPoint = null;
    let topmostElement = null;
    let toolsTopmost = null;
    if (overlap) {
      const x = (overlap.left + overlap.right) / 2;
      const y = (overlap.top + overlap.bottom) / 2;
      const topmost = document.elementFromPoint(x, y);
      sampledPoint = { x: Math.round(x), y: Math.round(y) };
      topmostElement = topmost
        ? `${topmost.tagName}.${String(topmost.className).slice(0, 80)}`
        : "nothing";
      toolsTopmost = !!topmost && (topmost === tools || tools.contains(topmost));
    }

    return {
      found: true,
      railScrollTop: rail.scrollTop,
      maxScroll,
      overlapCount,
      backgroundColor,
      alpha,
      sampledPoint,
      topmostElement,
      toolsTopmost,
    };
  });

  if (!result.found) {
    report(
      "tools overlap probe: .right-rail or .right-rail-tools not found — the rail did not render",
    );
    return result;
  }
  if (result.overlapCount === 0) {
    report(
      `no rail section ever intersected the sticky TOOLS surface across the full ` +
        `scroll range (max ${result.maxScroll}px) — the probe proved nothing at this viewport`,
    );
  }
  if (result.alpha < 0.999) {
    report(
      `sticky TOOLS computed background is not opaque: ${result.backgroundColor} ` +
        `(alpha ${result.alpha}) — scrolled rail content can paint through it`,
    );
  }
  if (result.overlapCount > 0 && !result.toolsTopmost) {
    report(
      `at the sampled overlap point the topmost element is ${result.topmostElement}, ` +
        `not the TOOLS surface — content paints or hits above it`,
    );
  }
  return result;
}

/**
 * S-E1 — rapid Original/Mastered switching, both directions, Volume Match off
 * and on.
 *
 * The scenario the plan names is not "does the toggle work" — that is covered
 * by a single switch. It is what happens when a mastering engineer does what
 * they actually do: flip back and forth quickly while listening, with Volume
 * Match in either state. The failure modes it is looking for are a playhead
 * that resets to zero, a direction that stalls (one way works, the other
 * sticks), and a stale readiness error surfacing after the switch resolved.
 *
 * So: start playback, then flip 6 times alternating direction, toggling Volume
 * Match halfway through, sampling the transport clock throughout. The
 * scenario's assertions check what the run left behind.
 */
async function rapidAbSwitching(page) {
  // The waveform mounts a moment AFTER the READY marker, and it is what
  // publishes the playhead. Waiting for the marker alone samples a slider that
  // does not exist yet and reports "unreadable" about a healthy app.
  await page.waitForSelector('[role="slider"][aria-valuenow]', {
    timeout: SETTLE_TIMEOUT_MS,
  });

  // Play is an icon button: its accessible name is an aria-label, and matching
  // on text content finds nothing. A silently-missed click would leave the
  // playhead pinned at 0 and make every "did it dip to zero" assertion vacuous,
  // so this resolves by ROLE AND NAME and the scenario asserts playback
  // actually started.
  await page
    .getByRole("button", { name: "Play", exact: true })
    .first()
    .click()
    .catch(() => {});

  // The playhead is published as aria-valuenow on the waveform slider — a real
  // number the app already exposes, not a test-only hook. A null here means the
  // slider is missing, and the scenario asserts on that rather than shrugging.
  const readPlayhead = () =>
    page.evaluate(() => {
      const el = document.querySelector('[role="slider"][aria-valuenow]');
      if (!el) return null;
      const raw = el.getAttribute("aria-valuenow");
      const value = raw === null ? NaN : Number(raw);
      return Number.isFinite(value) ? value : null;
    });

  const original = page.locator("button", { hasText: /^Original$/ }).first();
  const mastered = page.locator("button", { hasText: /^Mastered$/ }).first();

  // Wait for the transport to actually move before switching. "Rapid A/B while
  // playing" is the scenario; doing it against a stopped transport would prove
  // something easier and less useful.
  let startPlayhead = await readPlayhead();
  const moveDeadline = Date.now() + 5_000;
  while (Date.now() < moveDeadline) {
    const now = await readPlayhead();
    if (typeof now === "number" && now > 0.05) {
      startPlayhead = now;
      break;
    }
    await page.waitForTimeout(150);
  }
  const samples = [];
  const pressedStates = [];
  let volumeMatchToggled = false;

  for (let i = 0; i < 6; i += 1) {
    // Halfway through, flip Volume Match so both halves of the matrix are
    // exercised inside one continuous switching run.
    if (i === 3) {
      const vm = page.locator("button", { hasText: /Volume Match/ }).first();
      if ((await vm.count()) > 0) {
        await vm.click().catch(() => {});
        volumeMatchToggled = true;
      }
    }
    const wantMastered = i % 2 === 0;
    const target = wantMastered ? mastered : original;
    if ((await target.count()) === 0) break;
    await target.click().catch(() => {});
    // Deliberately short — "rapid" is the point. Long enough for React to
    // commit, far shorter than the mock's 2.5s landing window, so switches
    // genuinely overlap in-flight work.
    await page.waitForTimeout(120);
    samples.push(await readPlayhead());
    pressedStates.push(
      await page.evaluate((want) => {
        const label = want ? "Mastered" : "Original";
        const btn = Array.from(document.querySelectorAll("button")).find(
          (b) => (b.textContent ?? "").trim() === label,
        );
        return btn ? btn.getAttribute("aria-pressed") : null;
      }, wantMastered),
    );
  }

  // Handed to the scenario's assertions via the page, so the checks live with
  // the scenario definition rather than buried in the driver.
  await page.evaluate(
    (payload) => {
      window.__abRun = payload;
    },
    { startPlayhead, samples, pressedStates, volumeMatchToggled },
  );
}

/** Read back what rapidAbSwitching recorded, for scenario assertions. */
const abRun = (page) => page.evaluate(() => window.__abRun ?? null);

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
 * U10(c) — REACHABILITY, which is a different question from the two this lane
 * already answered.
 *
 * `mustContainAfterDrive` (textContent) proves a thing EXISTS. `horizontal
 * overflow` proves the page does not spill sideways. Neither proves a user at
 * the minimum supported size can actually GET to a control: a card can exist,
 * inside a scroll container, under a sticky footer, clipped by an ancestor
 * with `overflow: hidden` and no scrollport of its own — present in the DOM,
 * unreachable in the app.
 *
 * So this scrolls the element into view the way a user would and then asks the
 * platform three questions that a purely-DOM check cannot:
 *   1. does it have a real box (non-zero width and height)?
 *   2. after scrolling, is that box inside the viewport?
 *   3. is it the topmost thing at its own centre — i.e. nothing is covering it?
 *
 * (3) is the one that matters most, because a control hidden under a sticky
 * export bar looks perfect in every DOM assertion ever written.
 */
async function reachability(page, target) {
  return page.evaluate(({ selector, text }) => {
    const candidates = Array.from(document.querySelectorAll(selector));
    const el = text
      ? candidates.find((node) =>
          (node.textContent ?? "").replace(/\s+/g, " ").includes(text),
        )
      : candidates[0];
    if (!el) return { found: false };

    el.scrollIntoView({ block: "center", inline: "nearest", behavior: "instant" });

    const rect = el.getBoundingClientRect();
    const vw = document.documentElement.clientWidth;
    const vh = document.documentElement.clientHeight;
    const hasBox = rect.width > 0 && rect.height > 0;
    const inViewport =
      rect.top >= 0 && rect.left >= 0 && rect.bottom <= vh && rect.right <= vw;

    // Probe the centre of the element. If something else is painted there, the
    // control is covered and a click would land on the coverer.
    //
    // Reachable means the topmost element is the control ITSELF or something
    // INSIDE it (its label, its icon). An ANCESTOR coming back topmost is a
    // failure, not a pass: that is what a `::after { inset: 0 }` sheet over a
    // sticky export group looks like — the pseudo-element is not a node, so
    // elementFromPoint reports the element that generated it, which happens to
    // be the button's own parent. Accepting `topmost.contains(el)` here made
    // the check pass while Playwright's click on the same button timed out.
    const cx = rect.left + rect.width / 2;
    const cy = rect.top + rect.height / 2;
    const topmost = document.elementFromPoint(cx, cy);
    const notCovered = !!topmost && (topmost === el || el.contains(topmost));

    return {
      found: true,
      hasBox,
      inViewport,
      notCovered,
      rect: {
        top: Math.round(rect.top),
        bottom: Math.round(rect.bottom),
        height: Math.round(rect.height),
      },
      viewport: { width: vw, height: vh },
      coveredBy: notCovered
        ? null
        : `${topmost?.tagName ?? "nothing"}.${topmost?.className ?? ""}`.slice(0, 120),
    };
  }, target);
}

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
// Audit A-02 — one row per completed axe scan; checked against the expected
// matrix after the run so a silently skipped, duplicated, or empty scan is a
// failure, not a quiet gap.
const axeRuns = [];
const browser = await launchHeadless();

/** Record a failure with everything needed to act on it without re-running. */
function fail({ scenario, route, viewport, screenshot, message }) {
  failures.push(
    `[scenario=${scenario}] [route=${route}] [viewport=${viewport}] ${message}\n    screenshot: ${screenshot}`,
  );
}

for (const scenario of SCENARIOS) {
  // A scenario may reuse a preview state under its own name (e.g. the S-E1
  // rapid-A/B case drives the `clean` state), so evidence is keyed by label.
  const label = scenario.label ?? scenario.name;
  // U11 — a scenario may additionally be run with prefers-reduced-motion
  // forced, producing the reduced-motion equivalent screenshot the unit's
  // acceptance asks for. The assertions are identical: reduced motion is
  // supposed to change how a state ARRIVES, never which state you arrive at,
  // so a scenario that only passes with motion enabled is a real failure.
  const motionModes = scenario.reducedMotionVariant
    ? ["no-preference", "reduce"]
    : ["no-preference"];
  for (const motion of motionModes) {
  const motionSuffix = motion === "reduce" ? "-reduced-motion" : "";
  for (const [width, height] of scenario.viewports) {
    const viewportLabel = `${width}x${height}`;
    const route = `/app?scenario=${scenario.name}`;
    const url = `${baseUrl}${route}`;
    const screenshot = path.join(
      outDir,
      `${label}${motionSuffix}-${viewportLabel}.png`,
    );

    const context = await browser.newContext({
      viewport: { width, height },
      reducedMotion: motion,
    });
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

      // The waveform's peaks land ~800ms AFTER analysis reports READY (the
      // preview mock reproduces the real decode gap on purpose). Screenshotting
      // the instant READY appears therefore captured "No waveform yet." in a
      // ready-looking window — which is honest about that instant and
      // misleading as evidence: anyone reviewing these images for the U11
      // visual pass would report an empty waveform as a defect.
      // Non-fatal: a scenario whose analysis is deliberately still pending
      // (the album cases) has no waveform to wait for, and that is not an
      // error.
      await page
        .waitForSelector('[role="slider"][aria-valuenow]', { timeout: 2_500 })
        .catch(() => {});

      if (scenario.drive) {
        await scenario.drive(page);
        text = await bodyText(page);
      }

      const report = (message) =>
        fail({
          scenario: label,
          route,
          viewport: viewportLabel,
          screenshot,
          message,
        });

      // General pre-screenshot hook: runs after settle/drive but BEFORE the
      // screenshot, so its measurements and the captured image describe the
      // same frame (e.g. the U-01 tools-overlap probe leaves the rail
      // scrolled to the exact overlap it measured). Its serializable payload
      // travels into summary.json on this scenario's record. The legacy
      // window.__abRun payload for `assert` scenarios is kept until a
      // separate migration.
      let preScreenshotEvidence = null;
      if (scenario.beforeScreenshot) {
        preScreenshotEvidence = await scenario.beforeScreenshot(page, report);
      }

      // Audit A-02 — axe scan of this exact settled/driven state, composed
      // WITH (never instead of) any pre-screenshot collector above. Runs only
      // in the normal-motion pass so the coverage ledger counts each
      // state/viewport exactly once.
      let axe = null;
      if (scenario.axe && motion === "no-preference") {
        axe = await runAxeScan(page, report, `${label}@${viewportLabel}`);
        axeRuns.push({
          label,
          viewport: viewportLabel,
          totalRules: axe.totalRules,
        });
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

      // Deliberate self-test hook.
      if (forceFailScenario === label || forceFailScenario === scenario.name) {
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

      // U10(c) — reachability at the supported minimum size.
      const reachRecords = [];
      for (const target of scenario.mustReach ?? []) {
        const result = await reachability(page, target);
        reachRecords.push({ label: target.label, ...result });
        if (!result.found) {
          report(
            `"${target.label}" not found (selector ${target.selector}${
              target.text ? ` containing "${target.text}"` : ""
            })`,
          );
          continue;
        }
        if (!result.hasBox) {
          report(`"${target.label}" has no layout box — it renders to nothing`);
        }
        if (!result.inViewport) {
          report(
            `"${target.label}" cannot be scrolled into view at ${viewportLabel}: ` +
              `box top=${result.rect.top} bottom=${result.rect.bottom} in a ${result.viewport.height}px viewport`,
          );
        }
        if (!result.notCovered) {
          report(
            `"${target.label}" is covered at its centre by ${result.coveredBy} — a click would not reach it`,
          );
        }
      }

      // Scenario-specific assertions that need more than text matching.
      let driverPayload = null;
      if (scenario.assert) {
        await scenario.assert(page, report);
        // Whatever the driver recorded travels into the evidence file, so a
        // pass is a measurement someone can read rather than the absence of a
        // complaint.
        driverPayload = await page
          .evaluate(() => window.__abRun ?? null)
          .catch(() => null);
      }
      if (consoleMessages.length > 0) {
        report(`console errors/warnings: ${JSON.stringify(consoleMessages)}`);
      }
      if (pageErrors.length > 0) {
        report(`page errors: ${JSON.stringify(pageErrors)}`);
      }

      records.push({
        scenario: label + motionSuffix,
        scenarioId: scenario.scenarioId ?? null,
        previewState: scenario.name,
        motion,
        purpose: scenario.purpose,
        route,
        viewport: viewportLabel,
        screenshot,
        metrics,
        reachability: reachRecords,
        preScreenshotEvidence,
        axe,
        driverPayload,
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
}

// Audit A-02 — the axe coverage ledger. Every expected state/viewport must
// have been scanned exactly once with a positive rule total. Task 8 extends
// this matrix with the loaded-Standard scenario at both viewports.
const EXPECTED_AXE_MATRIX = [
  ["clean", "1440x900"],
  ["clean", "1360x740"],
  ["warning", "1440x900"],
  ["warning", "1360x740"],
];
for (const [expectedLabel, expectedViewport] of EXPECTED_AXE_MATRIX) {
  const hits = axeRuns.filter(
    (run) => run.label === expectedLabel && run.viewport === expectedViewport,
  );
  if (hits.length !== 1) {
    failures.push(
      `[axeCoverage] expected exactly one axe scan for ${expectedLabel}@${expectedViewport}, got ${hits.length}`,
    );
  } else if (!(hits[0].totalRules > 0)) {
    failures.push(
      `[axeCoverage] the axe scan for ${expectedLabel}@${expectedViewport} ran zero rules`,
    );
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
      // Audit A-02 — which states were axe-scanned and with how many rules;
      // per-scan violations/incomplete detail lives on each scenario record.
      axeCoverage: axeRuns,
      consoleAllowlist: CONSOLE_ALLOWLIST.map((entry) => ({
        pattern: String(entry.pattern),
        reason: entry.reason,
      })),
      // U10(d) — which plan scenario families this run actually covered, so
      // "the set is closed" can be checked against a file instead of asserted
      // from memory. Scenario IDs map to the traceability table in
      // docs/plans/2026-07-24-001-feat-public-beta-quality-plan.md.
      scenarioCoverage: Object.fromEntries(
        [...new Set(records.map((r) => r.scenarioId).filter(Boolean))]
          .sort()
          .map((id) => [
            id,
            records
              .filter((r) => r.scenarioId === id)
              .map((r) => `${r.scenario}@${r.viewport}`),
          ]),
      ),
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
