import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { describe, expect, test } from "vitest";

const repoRoot = resolve(__dirname, "../..");

function readJson(path: string) {
  return JSON.parse(readFileSync(resolve(repoRoot, path), "utf8"));
}

function parseCsp(csp: string): Record<string, string[]> {
  return Object.fromEntries(
    csp
      .split(";")
      .map((directive) => directive.trim())
      .filter(Boolean)
      .map((directive) => {
        const [name, ...sources] = directive.split(/\s+/);
        return [name, sources];
      }),
  );
}

describe("Tauri security config", () => {
  test("ships with a baseline content security policy for Tauri IPC and assets", () => {
    const config = readJson("src-tauri/tauri.conf.json");
    const csp = config.app?.security?.csp;

    expect(csp).toEqual(expect.any(String));
    const directives = parseCsp(csp);

    expect(directives["default-src"]).toContain("'self'");
    expect(directives["connect-src"]).toEqual(
      expect.arrayContaining(["ipc:", "http://ipc.localhost"]),
    );
    expect(directives["img-src"]).toEqual(
      expect.arrayContaining(["'self'", "asset:", "http://asset.localhost", "data:", "blob:"]),
    );
    expect(directives["style-src"]).toEqual(expect.arrayContaining(["'self'", "'unsafe-inline'"]));
    expect(directives["font-src"]).toEqual(expect.arrayContaining(["'self'", "data:"]));
  });
});
