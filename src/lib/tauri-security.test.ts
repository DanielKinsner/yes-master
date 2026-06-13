import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { describe, expect, test } from "vitest";

const repoRoot = resolve(__dirname, "../..");
const BASELINE_CSP =
  "default-src 'self'; img-src 'self' asset: data: blob:; style-src 'self' 'unsafe-inline'; font-src 'self' data:";

function readJson(path: string) {
  return JSON.parse(readFileSync(resolve(repoRoot, path), "utf8"));
}

describe("Tauri security config", () => {
  test("ships with a baseline content security policy", () => {
    const config = readJson("src-tauri/tauri.conf.json");

    expect(config.app?.security?.csp).not.toBeNull();
    expect(config.app?.security?.csp).toBe(BASELINE_CSP);
  });
});
