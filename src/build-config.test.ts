// Audit B-01 — deterministic frontend config selection.
//
// Vite resolves `vite.config.js` BEFORE `vite.config.ts`. The old
// `tsc -b` composite build EMITTED vite.config.js / vite.config.d.ts into the
// repo root, and .gitignore hid them — so a dirty local checkout could build,
// test, and preview against a stale compiled config while CI used the
// TypeScript source. Every Vite/Vitest entry point must therefore select
// vite.config.ts explicitly, typechecking must emit nothing, and the emitted
// shadow configs must be neither present nor ignorable.

import { existsSync, readFileSync } from "node:fs";
import { resolve } from "node:path";
import { describe, expect, it } from "vitest";

const root = process.cwd();
const read = (p: string): string => readFileSync(resolve(root, p), "utf8");

describe("deterministic Vite config selection (audit B-01)", () => {
  it("every Vite/Vitest script selects vite.config.ts explicitly", () => {
    const scripts = (
      JSON.parse(read("package.json")) as {
        scripts: Record<string, string>;
      }
    ).scripts;
    for (const name of ["dev", "build", "preview", "test", "test:watch"]) {
      expect(
        scripts[name],
        `script "${name}" must pass --config vite.config.ts`,
      ).toContain("--config vite.config.ts");
    }
    // The build must typecheck first and never route through `tsc -b`
    // (build mode is what emitted the shadow configs).
    expect(scripts.build).toContain("npm run typecheck");
    expect(scripts.build).not.toContain("tsc -b");
    expect(scripts.typecheck).toBe(
      "tsc --noEmit -p tsconfig.json && tsc --noEmit -p tsconfig.node.json",
    );
  });

  it("helper scripts that spawn vite preview select vite.config.ts too", () => {
    for (const file of [
      "scripts/verify-headless.mjs",
      "scripts/capture-landing-assets.mjs",
    ]) {
      const source = read(file);
      const previewSpawns = source.match(/"preview"/g) ?? [];
      expect(previewSpawns.length).toBeGreaterThan(0);
      expect(
        source,
        `${file} must pass --config vite.config.ts to vite preview`,
      ).toContain("vite.config.ts");
    }
  });

  it("typechecking emits nothing and uses no composite project references", () => {
    const nodeConfig = read("tsconfig.node.json");
    expect(nodeConfig).toContain('"noEmit": true');
    expect(nodeConfig).not.toContain('"composite"');
    expect(read("tsconfig.json")).not.toContain('"references"');
  });

  it("emitted shadow configs are neither present nor ignorable", () => {
    // Only actual ignore RULES count — an explanatory comment may name the
    // files.
    const ignoreRules = read(".gitignore")
      .split(/\r?\n/)
      .map((line) => line.trim())
      .filter((line) => line && !line.startsWith("#"));
    expect(ignoreRules).not.toContain("vite.config.js");
    expect(ignoreRules).not.toContain("vite.config.d.ts");
    // With the ignore entries gone, a re-emitted shadow config would show up
    // as an untracked file — and this makes it a red test as well.
    expect(existsSync(resolve(root, "vite.config.js"))).toBe(false);
    expect(existsSync(resolve(root, "vite.config.d.ts"))).toBe(false);
  });
});
