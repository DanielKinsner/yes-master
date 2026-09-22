// The try-it engine is a checked-in wasm binary built from the desktop's DSP
// and analysis sources by path. Nothing else would notice if those sources
// changed and the binary didn't: the app would master one way and the website
// demo another. This test recomputes the source stamp the build script wrote
// and fails — with the fix — the moment they disagree.
import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { describe, expect, it } from "vitest";
import { computeStamp, TRACKED_SOURCES } from "./engine-stamp";
import stamp from "./engine/sources.stamp.json";

const repo = resolve(__dirname, "../..");
const listedIn = (file: string) => (readFileSync(resolve(repo, file), "utf8").match(/"[^"]+\.(rs|toml)"/g) ?? []).map(s => s.slice(1, -1)).sort();

describe("try-it wasm engine stamp", () => {
  it("was built from the desktop sources currently in the tree", () => {
    const current = computeStamp(repo);
    const drifted = TRACKED_SOURCES.filter(rel => current.sources[rel] !== (stamp.sources as Record<string, string>)[rel]);
    expect(drifted, `These sources changed since the wasm was built: ${drifted.join(", ")}. Run \`npm run build:tryit-wasm\` and commit src/tryit/engine.`).toEqual([]);
    expect(current.stamp).toBe(stamp.stamp);
  });
  it("tracks the same source list as the Rust build script and the npm build script", () => {
    const expected = [...TRACKED_SOURCES].sort();
    expect(listedIn("web/tryit/wasm/build.rs")).toEqual(expected);
    expect(listedIn("scripts/build-tryit-wasm.mjs")).toEqual(expected);
  });
});
