#!/usr/bin/env node
// -----------------------------------------------------------------------------
// U7 — capture the landing page's marketing proof from a deterministic build.
//
//   npm run capture:landing
//
// Builds, serves the build on a free port, drives each canonical /app scenario
// to a settled state, screenshots it, and writes src/assets/landing/manifest.json
// binding every asset to the commit and the capture-input digest it came from.
//
// This is a WRITE command and it is not part of any gate. `verify:landing-assets`
// is the gate; this is how you satisfy it after changing a captured surface.
// -----------------------------------------------------------------------------
import { execFileSync, spawn } from "node:child_process";
import { mkdir, readFile, writeFile } from "node:fs/promises";
import net from "node:net";
import path from "node:path";

import { launchHeadless, runtimeStamp } from "./lib/headless-browser.mjs";
import {
  MANIFEST_PATH,
  REPO_ROOT,
  SHOTS,
  captureInputDigest,
  pngDimensions,
  sha256,
} from "./lib/landing-assets.mjs";

const ASSET_DIR = path.join(REPO_ROOT, "src", "assets", "landing");

async function freePort() {
  return await new Promise((resolve, reject) => {
    const server = net.createServer();
    server.on("error", reject);
    server.listen(0, "127.0.0.1", () => {
      const { port } = server.address();
      server.close(() => resolve(port));
    });
  });
}

function run(command, args) {
  execFileSync(command, args, {
    cwd: REPO_ROOT,
    stdio: "inherit",
    // npm is npm.cmd on Windows; a bare spawn only finds it through a shell
    // (same pattern as verify-headless.mjs).
    shell: process.platform === "win32",
  });
}

async function waitForServer(url, timeoutMs = 30_000) {
  const deadline = Date.now() + timeoutMs;
  while (Date.now() < deadline) {
    try {
      const response = await fetch(url);
      if (response.ok) return;
    } catch {
      // not up yet
    }
    await new Promise((r) => setTimeout(r, 250));
  }
  throw new Error(`preview server never became ready at ${url}`);
}

const SETTLE_MARKERS = {
  ready: (text) => text.includes("READY"),
  album: (text) => /\d+ TRACKS?\b/.test(text),
};

/**
 * Settle rules mirror the headless lane's, with one addition.
 *
 * A capture taken before the waveform's peaks land shows an empty box — honest
 * about that instant, misleading as a marketing image. U11 shipped exactly that
 * bug in its own evidence. The reliable signal is not the "No waveform yet."
 * text (which flickers) but the PLAYHEAD: `[role="slider"][aria-valuenow]`
 * only exists once real peaks have rendered, and it takes ~6s in the preview
 * mock, not the ~800ms the text implies. Album scenarios legitimately have no
 * waveform, so its absence is tolerated rather than fatal.
 */
async function settle(page, marker = "ready") {
  const deadline = Date.now() + 30_000;
  while (Date.now() < deadline) {
    const text = await page.evaluate(() => document.body.innerText);
    if (SETTLE_MARKERS[marker](text)) break;
    await page.waitForTimeout(200);
  }

  await page
    .waitForSelector('[role="slider"][aria-valuenow]', { timeout: 15_000 })
    .catch(() => {});

  // One extra beat so entrance motion (U11) has finished; a half-faded overlay
  // is not what the product looks like.
  await page.waitForTimeout(1_000);
}

async function selectView(page, view) {
  const backToStandard = page.getByRole("button", {
    name: /Back to Standard/i,
  });
  const toAdvanced = page.getByRole("button", { name: "Advanced", exact: true });

  if (view === "standard") {
    if (await backToStandard.count()) {
      await backToStandard.first().click();
      await page.waitForTimeout(600);
    }
  } else if (await toAdvanced.count()) {
    await toAdvanced.first().click();
    await page.waitForTimeout(600);
  }
}

const port = await freePort();
const baseUrl = `http://127.0.0.1:${port}`;

console.log("[capture] building");
run("npm", ["run", "build"]);

console.log(`[capture] serving on ${baseUrl}`);
// --host 127.0.0.1 explicitly: `vite preview` otherwise binds `localhost`,
// which resolves to ::1 on Windows, and the 127.0.0.1 readiness probe then
// times out against a server that started fine (same fix as verify-headless).
const server = spawn(
  "npx",
  // Audit B-01: explicit --config so a stray emitted vite.config.js can
  // never shadow the TypeScript source.
  ["vite", "preview", "--config", "vite.config.ts", "--host", "127.0.0.1", "--port", String(port), "--strictPort"],
  {
    cwd: REPO_ROOT,
    stdio: "ignore",
    // POSIX: detached puts the server in its own process group so the
    // group-kill below can take vite down with its children. Windows: npx is
    // npx.cmd (needs a shell), and teardown goes through taskkill instead.
    detached: process.platform !== "win32",
    shell: process.platform === "win32",
  },
);

let browser;
try {
  await waitForServer(`${baseUrl}/`);
  browser = await launchHeadless();
  await mkdir(ASSET_DIR, { recursive: true });

  const assets = [];
  for (const shot of SHOTS) {
    const page = await browser.newPage();
    await page.setViewportSize(shot.viewport);
    await page.goto(`${baseUrl}/app?scenario=${shot.scenario}`, {
      waitUntil: "networkidle",
    });
    await settle(page, shot.settle);
    await selectView(page, shot.view);
    await settle(page, shot.settle);

    const file = path.join(ASSET_DIR, `${shot.id}.png`);
    await page.screenshot({ path: file, fullPage: false });
    await page.close();

    const bytes = await readFile(file);
    const dimensions = pngDimensions(bytes);
    if (
      dimensions?.width !== shot.viewport.width ||
      dimensions?.height !== shot.viewport.height
    ) {
      throw new Error(
        `${shot.id}: captured ${dimensions?.width}x${dimensions?.height}, expected ${shot.viewport.width}x${shot.viewport.height}`,
      );
    }

    assets.push({
      id: shot.id,
      file: `src/assets/landing/${shot.id}.png`,
      scenario: shot.scenario,
      view: shot.view,
      viewport: shot.viewport,
      dimensions,
      bytes: bytes.length,
      sha256: sha256(bytes),
      // Every captured proof lives below the fold, so it is lazy by policy.
      // Only the hero art and its icons are eager, and they are not captures.
      loading: "lazy",
      alt: shot.alt,
    });
    console.log(
      `[capture] ${shot.id}.png  ${dimensions.width}x${dimensions.height}  ${Math.round(bytes.length / 1024)} KB`,
    );
  }

  let sourceCommit = "unknown";
  try {
    sourceCommit = execFileSync("git", ["rev-parse", "HEAD"], {
      cwd: REPO_ROOT,
      encoding: "utf8",
    }).trim();
  } catch {
    // A capture from a dirty or detached tree is still a capture.
  }

  const manifest = {
    // Provenance, NOT the freshness key. Staleness is decided by
    // captureInputDigest so that unrelated commits do not invalidate proof
    // that is still accurate.
    sourceCommit,
    captureInputDigest: captureInputDigest(),
    browser: runtimeStamp(browser),
    evidenceLayer: "browser-headless",
    note: "Deterministic browser captures are marketing evidence. They prove nothing about installation, real audio, or how anything sounds.",
    assets,
  };

  await writeFile(
    path.join(REPO_ROOT, MANIFEST_PATH),
    `${JSON.stringify(manifest, null, 2)}\n`,
  );
  console.log(`[capture] wrote ${MANIFEST_PATH}`);
} finally {
  if (browser) await browser.close();
  try {
    if (process.platform === "win32") {
      // process.kill(-pid) is a POSIX process-group kill; on Windows it
      // throws, the catch swallowed it, and the preview server was orphaned.
      // taskkill /T takes down the shell AND the vite process holding the
      // port (same reasoning as verify-headless.mjs).
      execFileSync("taskkill", ["/pid", String(server.pid), "/T", "/F"], {
        stdio: "ignore",
      });
    } else {
      process.kill(-server.pid);
    }
  } catch {
    // already gone
  }
}
