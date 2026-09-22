// Source stamp for the checked-in try-it wasm engine. Node-only (used by the
// test); `scripts/build-tryit-wasm.mjs` carries its own copy of the same list
// and algorithm because Vercel's checkout drops `scripts/` (.vercelignore) and
// the production typecheck must not depend on it. The test cross-checks both
// copies against this one.
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import { join } from "node:path";

/** Keep in lockstep with `SOURCES` in web/tryit/wasm/build.rs and `TRACKED_SOURCES` in scripts/build-tryit-wasm.mjs. */
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
] as const;

const sha256 = (bytes: Uint8Array | string) => createHash("sha256").update(bytes).digest("hex");
const normalize = (bytes: Uint8Array) => bytes.filter(b => b !== 0x0d);

export function computeStamp(root: string): { stamp: string; sources: Record<string, string> } {
  const sources: Record<string, string> = {};
  let joined = "";
  for (const rel of TRACKED_SOURCES) {
    const hash = sha256(normalize(readFileSync(join(root, rel))));
    sources[rel] = hash;
    joined += `${rel}:${hash}\n`;
  }
  return { stamp: sha256(joined).slice(0, 12), sources };
}
