import { spawnSync } from "node:child_process";
import { resolve } from "node:path";
import { describe, expect, test } from "vitest";

const repoRoot = resolve(__dirname, "../..");
const systemRoot = process.env.SystemRoot ?? "C:\\Windows";
const powershell = resolve(systemRoot, "System32/WindowsPowerShell/v1.0/powershell.exe");

function runVerifyFast(args: string[]) {
  const system32 = resolve(systemRoot, "System32");

  return spawnSync(
    powershell,
    ["-NoProfile", "-ExecutionPolicy", "Bypass", "-File", "scripts/verify-fast.ps1", ...args],
    {
      cwd: repoRoot,
      encoding: "utf8",
      env: {
        ...process.env,
        ANDROID_HOME: "",
        JAVA_HOME: "",
        PATH: system32,
      },
    },
  );
}

describe("verify-fast.ps1 Android prerequisite handling", () => {
  test("fails the Android lane when prerequisites are missing by default", () => {
    const result = runVerifyFast(["-Lane", "android"]);
    const output = `${result.stdout}\n${result.stderr}`;

    expect(result.status).not.toBe(0);
    expect(output).toContain("Android lane unavailable; missing:");
    expect(output).toContain("Pass -AllowAndroidSkip");
    expect(output).not.toContain("android lane passed.");
  });

  test("allows an explicit non-green Android skip", () => {
    const result = runVerifyFast(["-Lane", "android", "-AllowAndroidSkip"]);
    const output = `${result.stdout}\n${result.stderr}`;

    expect(result.status).toBe(0);
    expect(output).toContain("Skipping Android lane by explicit -AllowAndroidSkip; missing:");
    expect(output).toContain("android lane completed with skipped lane(s): android.");
    expect(output).not.toContain("android lane passed.");
  });
});
