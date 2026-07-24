#!/usr/bin/env node
//
// The self-contained headless web verification lane (U3).
//
//   npm run verify:headless
//
// One command, no prerequisites beyond `npm ci` and the pinned browser. It
// builds once, starts its OWN preview server on a free port, waits for the
// server to actually serve, runs the landing and /app suites against it, and
// tears the server down on every exit path -- success, assertion failure,
// thrown error, Ctrl+C, or SIGTERM.
//
// Why a wrapper at all. The landing script used to require a manually started
// server on a hardcoded port, which meant it was never wired into CI and could
// not be a gate. It also defaulted to 127.0.0.1 while `vite preview` binds to
// `localhost` -- which on Windows resolves to ::1 -- so the documented
// invocation failed to connect on the owner's own machine. This wrapper owns
// the server, so neither problem can recur.
//
// SYNTHETIC EVIDENCE. Green here means the web surfaces render, lay out, and
// transition correctly in a pinned headless Chromium. It says nothing about
// native dialogs, real audio, installer signing, updater install, or sound.

import { spawn } from "node:child_process";
import { createServer } from "node:net";
import { mkdir, writeFile } from "node:fs/promises";
import path from "node:path";
import process from "node:process";

const SERVER_READY_TIMEOUT_MS = 60_000;
const SERVER_STOP_TIMEOUT_MS = 10_000;

function option(name) {
  const index = process.argv.indexOf(name);
  return index === -1 ? undefined : process.argv[index + 1];
}
const flag = (name) => process.argv.includes(name);

function stamp() {
  return new Date().toISOString().replace(/[:.]/g, "-");
}

const outDir = option("--out") ?? path.join("test-output", "headless", stamp());
const skipBuild = flag("--skip-build");
const forceFail = option("--force-fail");

/** Ask the OS for a port nobody is using, so parallel runs cannot collide. */
function findFreePort() {
  return new Promise((resolve, reject) => {
    const server = createServer();
    server.unref();
    server.on("error", reject);
    server.listen(0, "127.0.0.1", () => {
      const { port } = server.address();
      server.close(() => resolve(port));
    });
  });
}

function run(command, args, label) {
  return new Promise((resolve, reject) => {
    const child = spawn(command, args, {
      stdio: "inherit",
      shell: process.platform === "win32",
    });
    child.on("error", reject);
    child.on("exit", (code) => {
      if (code === 0) resolve();
      else reject(new Error(`${label} exited with code ${code}`));
    });
  });
}

// ---------------------------------------------------------------------------
// Preview server lifecycle
// ---------------------------------------------------------------------------

let previewChild = null;
let stoppingPreview = false;

/**
 * Kill the preview server AND its children. `child.kill()` alone is not enough
 * on Windows: npm/vite runs under a shell, so killing the shell orphans the
 * node process that actually holds the port. An orphaned preview server is the
 * specific failure this lane is required not to leave behind.
 */
async function stopPreview() {
  const child = previewChild;
  if (!child || child.exitCode !== null) return;
  previewChild = null;
  stoppingPreview = true;

  const exited = new Promise((resolve) => child.once("exit", resolve));

  if (process.platform === "win32") {
    await new Promise((resolve) => {
      const killer = spawn("taskkill", ["/pid", String(child.pid), "/T", "/F"], {
        stdio: "ignore",
      });
      killer.on("exit", resolve);
      killer.on("error", resolve);
    });
  } else {
    try {
      // Negative pid targets the whole process group (see `detached` below).
      process.kill(-child.pid, "SIGTERM");
    } catch {
      try {
        child.kill("SIGTERM");
      } catch {
        /* already gone */
      }
    }
  }

  await Promise.race([
    exited,
    new Promise((resolve) => setTimeout(resolve, SERVER_STOP_TIMEOUT_MS)),
  ]);
}

async function startPreview(port) {
  // Bind explicitly to 127.0.0.1. `vite preview` otherwise binds to `localhost`,
  // which resolves to ::1 on Windows, and every 127.0.0.1 client then fails to
  // connect against a server that looks like it started fine.
  previewChild = spawn(
    "npx",
    ["vite", "preview", "--host", "127.0.0.1", "--port", String(port), "--strictPort"],
    {
      stdio: ["ignore", "pipe", "pipe"],
      shell: process.platform === "win32",
      detached: process.platform !== "win32",
    },
  );

  const serverLog = [];
  previewChild.stdout?.on("data", (chunk) => serverLog.push(String(chunk)));
  previewChild.stderr?.on("data", (chunk) => serverLog.push(String(chunk)));
  previewChild.on("exit", (code) => {
    // A nonzero code during our own teardown is expected -- taskkill /F and
    // SIGTERM both produce one. Only an exit we did not ask for is a problem.
    if (stoppingPreview) return;
    if (code !== 0 && code !== null) {
      console.error(`Preview server exited unexpectedly with code ${code}:`);
      console.error(serverLog.join(""));
    }
  });

  return serverLog;
}

async function waitForServer(baseUrl, serverLog) {
  const deadline = Date.now() + SERVER_READY_TIMEOUT_MS;
  while (Date.now() < deadline) {
    if (previewChild?.exitCode !== null && previewChild?.exitCode !== undefined) {
      throw new Error(
        `preview server died before becoming ready:\n${serverLog.join("")}`,
      );
    }
    try {
      const response = await fetch(`${baseUrl}/`, { redirect: "manual" });
      if (response.ok || response.status === 304) return;
    } catch {
      /* not up yet */
    }
    await new Promise((resolve) => setTimeout(resolve, 250));
  }
  throw new Error(
    `preview server did not answer on ${baseUrl} within ${SERVER_READY_TIMEOUT_MS}ms:\n${serverLog.join("")}`,
  );
}

// Tear down on every abnormal exit path too, not just the happy one.
let cleaningUp = false;
async function cleanupAndExit(code) {
  if (cleaningUp) return;
  cleaningUp = true;
  await stopPreview();
  process.exit(code);
}
process.on("SIGINT", () => void cleanupAndExit(130));
process.on("SIGTERM", () => void cleanupAndExit(143));
process.on("uncaughtException", (error) => {
  console.error(error);
  void cleanupAndExit(1);
});

// ---------------------------------------------------------------------------
// Run
// ---------------------------------------------------------------------------

let exitCode = 0;
const results = [];

try {
  await mkdir(outDir, { recursive: true });

  if (!skipBuild) {
    console.log("[verify:headless] building once...");
    await run("npm", ["run", "build"], "npm run build");
  } else {
    console.log("[verify:headless] --skip-build: reusing the existing dist/");
  }

  const port = await findFreePort();
  const baseUrl = `http://127.0.0.1:${port}`;
  console.log(`[verify:headless] starting preview server on ${baseUrl}`);
  const serverLog = await startPreview(port);
  await waitForServer(baseUrl, serverLog);
  console.log("[verify:headless] preview server is serving");

  const suites = [
    {
      name: "landing",
      script: "scripts/verify-landing-responsive.mjs",
      args: ["--url", `${baseUrl}/`, "--out", path.join(outDir, "landing")],
    },
    {
      name: "app",
      script: "scripts/verify-app-headless.mjs",
      args: [
        "--url",
        baseUrl,
        "--out",
        path.join(outDir, "app"),
        ...(forceFail ? ["--force-fail", forceFail] : []),
      ],
    },
  ];

  for (const suite of suites) {
    console.log(`[verify:headless] running ${suite.name} suite`);
    try {
      await run("node", [suite.script, ...suite.args], suite.name);
      results.push({ suite: suite.name, ok: true });
    } catch (error) {
      // Run every suite even after one fails: a single command should report
      // the whole picture, not just the first thing that broke.
      results.push({ suite: suite.name, ok: false, error: String(error.message) });
      exitCode = 1;
    }
  }
} catch (error) {
  console.error(`[verify:headless] ${error?.message ?? error}`);
  results.push({ suite: "harness", ok: false, error: String(error?.message ?? error) });
  exitCode = 1;
} finally {
  await stopPreview();
}

await writeFile(
  path.join(outDir, "summary.json"),
  `${JSON.stringify(
    {
      evidenceLayer: "browser-headless",
      note: "Synthetic browser evidence. Proves layout/journeys/copy. Proves nothing about native dialogs, audio, signing, updater install, or sound.",
      outDir,
      results,
    },
    null,
    2,
  )}\n`,
);

if (exitCode === 0) {
  console.log(`[verify:headless] PASSED. Evidence: ${outDir}`);
} else {
  console.error(`[verify:headless] FAILED. Evidence: ${outDir}`);
  for (const result of results.filter((entry) => !entry.ok)) {
    console.error(`- ${result.suite}: ${result.error}`);
  }
}

process.exit(exitCode);
