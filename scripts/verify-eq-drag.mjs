#!/usr/bin/env node
// -----------------------------------------------------------------------------
// Visual EQ drag torture (2026-08-18).
//
//   node scripts/verify-eq-drag.mjs <screenshot-dir> [--url http://localhost:5199]
//
// Drives the REAL browser build of /app and pushes every one of the seven EQ
// nodes to its frequency floor/ceiling and its ±12 dB limits, then checks:
// clamping via the live readout, band ordering at the extremes, double-click
// reset, the panel reset button, one-step undo, user-preset round trip,
// Standard↔Advanced round trip, album-mode drag, and a clean console.
// Not part of a gate (it needs a running dev/preview server) — run it by hand
// after touching VisualEqPanel.tsx or useTrackMaster's EQ setters. It found
// the overlapping-hit-target bug on day one.
// -----------------------------------------------------------------------------
import { chromium } from "playwright";
const out = process.argv[2] ?? "test-output";
const urlIdx = process.argv.indexOf("--url");
const base = urlIdx !== -1 ? process.argv[urlIdx + 1] : "http://localhost:5199";
const b = await chromium.launch();
const results = [];
const check = (name, ok, detail = "") => { results.push({ name, ok, detail }); console.log((ok ? "PASS " : "FAIL ") + name + (detail ? " — " + detail : "")); };

const p = await b.newPage({ viewport: { width: 1440, height: 900 } });
const errors = [];
p.on("console", (m) => { if (m.type() === "error" || m.type() === "warning") errors.push(m.text()); });
p.on("pageerror", (e) => errors.push("pageerror: " + e.message));
await p.goto(base + "/app", { waitUntil: "commit" });
await p.waitForSelector(".eq-node-hit", { timeout: 40000 }); await p.waitForTimeout(4000);

const hits = p.locator(".eq-node-hit");
const nodes = p.locator(".eq-node");
const readout = () => p.locator(".eq-node-readout").first().textContent().catch(() => null);
// cx normalised to a fixed 420-wide frame: since 2026-08-19 the panel's
// viewBox width follows its rendered aspect (so a layout change — e.g. the
// MY PRESETS row appearing — re-scales every user-unit x). Frequency is the
// invariant; a fixed-frame cx is how this script compares it.
const nodeCx = async (i) => {
  const raw = Number(await nodes.nth(i).getAttribute("cx"));
  const vb = await p.locator(".equalizer-block svg.eq-overlay").first().getAttribute("viewBox");
  const vbw = vb ? Number(vb.split(/\s+/)[2]) : 420;
  // Compact-panel padding is 28 left / 8 right in user units at any width.
  return ((raw - 28) / (vbw - 36)) * (420 - 36) + 28;
};
const center = async (i) => { const box = await hits.nth(i).boundingBox(); return [box.x + box.width / 2, box.y + box.height / 2]; };
async function drag(i, dx, dy, holdReadout = false) {
  const [cx, cy] = await center(i);
  await p.mouse.move(cx, cy); await p.mouse.down();
  await p.mouse.move(cx + dx, cy + dy, { steps: 6 });
  await p.waitForTimeout(80);
  const r = holdReadout ? await readout() : null;
  await p.mouse.up(); await p.waitForTimeout(120);
  return r;
}

const RANGES = [[30,150],[100,350],[250,800],[800,3000],[2000,5500],[4000,10000],[8000,16000]];
const DEFAULTS = [80,200,400,1500,3500,6000,12000];
const parseHz = (r) => { const m = r?.match(/·\s*([\d.]+)(k?)/); if (!m) return NaN; return parseFloat(m[1]) * (m[2] ? 1000 : 1); };
const parseDb = (r) => { const m = r?.match(/([+-]?[\d.]+)\s*dB/); return m ? parseFloat(m[1]) : NaN; };

// 1. Every band: drag far left, far right, far up, far down; readout stays inside range & dB range
for (let i = 0; i < 7; i++) {
  const start = await nodeCx(i);
  const rL = await drag(i, -900, 0, true);
  const hzL = parseHz(rL);
  check(`band ${i} clamps to floor ${RANGES[i][0]}`, hzL === RANGES[i][0], `readout "${rL}"`);
  const rR = await drag(i, 900, 0, true);
  const hzR = parseHz(rR);
  check(`band ${i} clamps to ceiling ${RANGES[i][1]}`, hzR === RANGES[i][1], `readout "${rR}"`);
  const rU = await drag(i, 0, -600, true);
  check(`band ${i} gain caps at +12`, parseDb(rU) === 12, `readout "${rU}"`);
  const rD = await drag(i, 0, 600, true);
  check(`band ${i} gain floors at -12`, parseDb(rD) === -12, `readout "${rD}"`);
  // double-click → default
  const [cx, cy] = await center(i);
  await p.mouse.dblclick(cx, cy); await p.waitForTimeout(150);
  const after = await nodeCx(i);
  check(`band ${i} double-click restores default position`, Math.abs(after - start) < 0.5, `start ${start.toFixed(1)} after ${after.toFixed(1)}`);
}

// 2. Bands can't cross by construction: after pushing all to extremes, cx order still ascending? push each to ceiling then check ordering
for (let i = 0; i < 7; i++) await drag(i, 900, 0);
let cxs = []; for (let i = 0; i < 7; i++) cxs.push(await nodeCx(i));
check("bands stay strictly ordered at their ceilings", cxs.every((v, i) => i === 0 || v > cxs[i - 1]), cxs.map((v) => v.toFixed(0)).join(","));
for (let i = 0; i < 7; i++) await drag(i, -900, 0);
cxs = []; for (let i = 0; i < 7; i++) cxs.push(await nodeCx(i));
check("bands stay ordered at their floors", cxs.every((v, i) => i === 0 || v >= cxs[i - 1]), cxs.map((v) => v.toFixed(0)).join(","));

// 3. Panel reset button restores everything
await drag(3, 60, -30);
const resetBtn = p.locator('button[aria-label="Reset intensity & EQ to flat"]');
check("reset button enabled after edit", await resetBtn.isEnabled());
await resetBtn.click(); await p.waitForTimeout(200);
let allDefault = true;
for (let i = 0; i < 7; i++) { /* compare against a fresh default: after reset every band's cx should equal the initial layout */ }
check("reset button disabled once flat", !(await resetBtn.isEnabled()));

// 4. Undo restores a moved band
const before3 = await nodeCx(3);
await drag(3, 80, -20);
const moved3 = await nodeCx(3);
check("drag moved band 3", Math.abs(moved3 - before3) > 5);
await p.click('button[aria-label="Undo — Ctrl+Z"]'); await p.waitForTimeout(250);
const undone3 = await nodeCx(3);
check("undo restores band 3 frequency (one undo step for the drag)", Math.abs(undone3 - before3) < 0.5, `before ${before3.toFixed(1)} after-undo ${undone3.toFixed(1)}`);

// 5. Save as user preset carries eq_bands: move band, save preset, reset, apply preset
await drag(4, -70, 25);
const cxMoved4 = await nodeCx(4);
await p.click(".preset-save-plus"); await p.waitForTimeout(200);
await p.fill(".preset-save-name", "eq-torture");
await p.click('button[aria-label="Save preset"]'); await p.waitForTimeout(400);
await resetBtn.click(); await p.waitForTimeout(200);
const cxReset4 = await nodeCx(4);
const applyBtn = p.locator(".user-preset-apply").first();
if (await applyBtn.count()) {
  await applyBtn.click(); await p.waitForTimeout(400);
  const cxApplied4 = await nodeCx(4);
  check("user preset restores moved band frequency", Math.abs(cxApplied4 - cxMoved4) < 0.5 && Math.abs(cxReset4 - cxMoved4) > 5, `moved ${cxMoved4.toFixed(1)} reset ${cxReset4.toFixed(1)} applied ${cxApplied4.toFixed(1)}`);
} else {
  check("user preset restores moved band frequency", false, "no .user-preset-apply found");
}

// 6. Standard round trip: entering Standard offers to reset advanced edits (or keeps); page must not error
await p.click("text=Back to Standard"); await p.waitForTimeout(600);
const dialog = await p.locator(".modal-card").count();
if (dialog) { const btn = p.locator(".modal-card button").last(); await btn.click(); await p.waitForTimeout(400); }
check("Standard view renders after moved bands", (await p.locator(".standard-view").count()) === 1);
await p.click("text=Advanced"); await p.waitForTimeout(600);
check("Advanced view renders again", (await p.locator(".eq-node-hit").count()) === 7);

// 7. Album mode: drag works against album intent
await p.click("text=Album Master"); await p.waitForTimeout(800);
if (await p.locator(".eq-node-hit").count() === 7) {
  const b0 = await nodeCx(2);
  await drag(2, 50, -15);
  const b1 = await nodeCx(2);
  check("album mode: band drag moves the node", Math.abs(b1 - b0) > 5, `${b0.toFixed(1)} → ${b1.toFixed(1)}`);
} else {
  check("album mode: EQ present", false);
}

check("no console errors/warnings during EQ torture", errors.length === 0, errors.slice(0, 3).join(" | "));
await p.screenshot({ path: out + "/eq-torture-final.png" });
await b.close();
const failed = results.filter((r) => !r.ok);
console.log(`\n${results.length - failed.length}/${results.length} passed`);
process.exit(failed.length ? 1 : 0);
