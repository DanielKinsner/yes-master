import { readFileSync } from "node:fs";

import { describe, expect, it } from "vitest";

type VersionSource = {
  label: string;
  version: string;
};

const STRICT_SEMVER = /^\d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?(?:\+[0-9A-Za-z.-]+)?$/;

function readText(path: string): string {
  return readFileSync(new URL(path, import.meta.url), "utf8");
}

function readJsonVersion(path: string): string {
  const parsed = JSON.parse(readText(path)) as { version?: unknown };
  expect(typeof parsed.version, `${path} must expose a string version`).toBe("string");
  return parsed.version as string;
}

function readCargoPackageVersion(path: string): string {
  const match = readText(path).match(/^version\s*=\s*"([^"]+)"/m);
  expect(match, `${path} must expose a package version`).not.toBeNull();
  return match![1];
}

function readGradleVersionName(path: string): string {
  const match = readText(path).match(/^\s*versionName\s*=\s*"([^"]+)"/m);
  expect(match, `${path} must expose defaultConfig.versionName`).not.toBeNull();
  return match![1];
}

function readPlistString(path: string, key: string): string {
  const pattern = new RegExp(`<key>${key}</key>\\s*<string>([^<]+)</string>`);
  const match = readText(path).match(pattern);
  expect(match, `${path} must expose ${key}`).not.toBeNull();
  return match![1];
}

function expectStrictSemver(source: VersionSource): void {
  expect(source.version, `${source.label} must be strict semver`).toMatch(STRICT_SEMVER);
}

describe("release version coherence", () => {
  it("keeps desktop manifests aligned on a non-placeholder semver", () => {
    const desktopSources: VersionSource[] = [
      { label: "package.json", version: readJsonVersion("../../package.json") },
      { label: "src-tauri/tauri.conf.json", version: readJsonVersion("../../src-tauri/tauri.conf.json") },
      { label: "src-tauri/Cargo.toml", version: readCargoPackageVersion("../../src-tauri/Cargo.toml") },
    ];

    for (const source of desktopSources) {
      expectStrictSemver(source);
      expect(source.version, `${source.label} must not keep the placeholder version`).not.toBe("0.0.0");
    }

    const [expected, ...rest] = desktopSources;
    for (const source of rest) {
      expect(source.version, `${source.label} must match ${expected.label}`).toBe(expected.version);
    }
  });

  it("keeps mobile release versions semver-parseable without forcing desktop equality", () => {
    const mobileSources: VersionSource[] = [
      {
        label: "apps/android-native/app/build.gradle.kts versionName",
        version: readGradleVersionName("../../apps/android-native/app/build.gradle.kts"),
      },
      {
        label: "apps/iphone-native/YESMasterNative/Info.plist CFBundleShortVersionString",
        version: readPlistString(
          "../../apps/iphone-native/YESMasterNative/Info.plist",
          "CFBundleShortVersionString",
        ),
      },
    ];

    for (const source of mobileSources) {
      expectStrictSemver(source);
    }
  });
});
