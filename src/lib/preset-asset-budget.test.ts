import { statSync } from "node:fs";
import { resolve } from "node:path";
import { describe, expect, test } from "vitest";

const repoRoot = resolve(__dirname, "../..");
const presetFiles = [
  "clarity.png",
  "loud.png",
  "oomph.png",
  "punch.png",
  "spatial.png",
  "tape.png",
  "universal.png",
  "warmth.png",
];

describe("preset artwork asset budget", () => {
  test("keeps preset PNGs small enough for the installer bundle", () => {
    const sizes = presetFiles.map((name) =>
      statSync(resolve(repoRoot, "src/assets/presets", name)).size,
    );

    for (const size of sizes) {
      expect(size).toBeLessThanOrEqual(700 * 1024);
    }
    expect(sizes.reduce((sum, size) => sum + size, 0)).toBeLessThanOrEqual(
      5 * 1024 * 1024,
    );
  });
});
