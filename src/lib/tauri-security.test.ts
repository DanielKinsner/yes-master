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

  test("does not weaken the policy with eval, wildcards, or arbitrary remote origins", () => {
    const config = readJson("src-tauri/tauri.conf.json");
    const directives = parseCsp(config.app?.security?.csp);
    const allowedRemoteOrigins = ["http://ipc.localhost", "http://asset.localhost"];

    for (const source of Object.values(directives).flat()) {
      expect(source).not.toBe("'unsafe-eval'");
      expect(source).not.toContain("*");
      // Bare scheme wildcards (http:/https:) would admit any network origin.
      expect(source).not.toBe("http:");
      expect(source).not.toBe("https:");
      if (/^https?:\/\//.test(source)) {
        expect(allowedRemoteOrigins).toContain(source);
      }
    }
  });
});
