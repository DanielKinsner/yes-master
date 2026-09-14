// Build the browser try-it engine (web/tryit/wasm) into src/tryit/engine and
// stamp which desktop sources it carries.
//
//   npm run build:tryit-wasm
//
// Needs: rustup target wasm32-unknown-unknown, wasm-bindgen-cli 0.2.128
// (`cargo install wasm-bindgen-cli --version 0.2.128 --locked`). The stamp
// (`src/tryit/engine/sources.stamp.json`) is what `src/tryit/engine-stamp.test.ts`
// checks: when any tracked desktop source changes, the test fails until this
// script is run again, so the demo can never quietly master differently from
// the app. The Rust `build.rs` computes the same hash for `version()`.
import { createHash } from "node:crypto";
import { execFileSync } from "node:child_process";
import { readFileSync, writeFileSync, mkdirSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const repo = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const crate = join(repo, "web", "tryit", "wasm");
const outDir = join(repo, "src", "tryit", "engine");

/** Keep in lockstep with `SOURCES` in web/tryit/wasm/build.rs and `TRACKED_SOURCES` in
 *  src/tryit/engine-stamp.ts (the test's copy; Vercel's checkout drops scripts/). */
export const TRACKED_SOURCES = [
  "src-tauri/src/types.rs",
  "src-tauri/src/dsp.rs",
  "src-tauri/src/export_format.rs",
  "src-tauri/src/analysis.rs",
  "src-tauri/src/deep_analysis.rs",
  "src-tauri/src/confidence.rs",
  "src-tauri/src/guardrails.rs",
  "web/tryit/wasm/src/lib.rs",
  "web/tryit/wasm/src/stubs.rs",
  "web/tryit/wasm/Cargo.toml",
];

const sha256 = (bytes) => createHash("sha256").update(bytes).digest("hex");
const normalize = (bytes) => Buffer.from(bytes.filter((b) => b !== 0x0d));

export function computeStamp(root = repo) {
  const sources = {};
  let joined = "";
  for (const rel of TRACKED_SOURCES) {
    const hash = sha256(normalize(readFileSync(join(root, rel))));
    sources[rel] = hash;
    joined += `${rel}:${hash}\n`;
  }
  return { stamp: sha256(joined).slice(0, 12), sources };
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const run = (cmd, args, cwd) => execFileSync(cmd, args, { cwd, stdio: "inherit" });
  run("cargo", ["build", "--release", "--target", "wasm32-unknown-unknown"], crate);
  mkdirSync(outDir, { recursive: true });
  run("wasm-bindgen", ["--target", "web", "--out-dir", outDir, join(crate, "target", "wasm32-unknown-unknown", "release", "yes_master_web.wasm")], crate);
  const { stamp, sources } = computeStamp();
  writeFileSync(join(outDir, "sources.stamp.json"), JSON.stringify({ stamp, sources }, null, 2) + "\n");
  console.log(`tryit wasm built; source stamp ${stamp}`);
}
