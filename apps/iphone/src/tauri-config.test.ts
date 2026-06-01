import { readFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { describe, expect, it } from "vitest";

interface IphoneTauriConfig {
  build?: {
    devUrl?: string;
  };
  app?: {
    security?: {
      assetProtocol?: {
        enable?: boolean;
        scope?: AssetProtocolScope;
      };
    };
  };
}

interface IphoneCapability {
  permissions?: Array<string | { identifier?: string }>;
}

interface RootPackageJson {
  scripts?: Record<string, string>;
}

type AssetProtocolScope = string[] | { allow?: string[] } | undefined;

function readIphoneJson<T>(relativePath: string): T {
  const currentDir = dirname(fileURLToPath(import.meta.url));
  const configPath = resolve(currentDir, relativePath);
  return JSON.parse(readFileSync(configPath, "utf8")) as T;
}

function readIphoneText(relativePath: string): string {
  const currentDir = dirname(fileURLToPath(import.meta.url));
  return readFileSync(resolve(currentDir, relativePath), "utf8");
}

function readIphoneTauriConfig(): IphoneTauriConfig {
  return readIphoneJson("../src-tauri/tauri.conf.json");
}

function readIphoneDefaultCapability(): IphoneCapability {
  return readIphoneJson("../src-tauri/capabilities/default.json");
}

function readRootPackageJson(): RootPackageJson {
  return readIphoneJson("../../../package.json");
}

function assetScopeEntries(scope: AssetProtocolScope): string[] {
  if (Array.isArray(scope)) return scope;
  return scope?.allow ?? [];
}

function permissionIds(permissions: IphoneCapability["permissions"]): string[] {
  return (
    permissions?.map((permission) =>
      typeof permission === "string" ? permission : permission.identifier ?? "",
    ) ?? []
  );
}

function plistArrayValues(plist: string, key: string): string[] {
  const keyStart = plist.indexOf(`<key>${key}</key>`);
  if (keyStart === -1) return [];
  const arrayStart = plist.indexOf("<array>", keyStart);
  const arrayEnd = plist.indexOf("</array>", arrayStart);
  if (arrayStart === -1 || arrayEnd === -1) return [];
  return Array.from(
    plist
      .slice(arrayStart, arrayEnd)
      .matchAll(/<string>([^<]+)<\/string>/g),
    (match) => match[1],
  );
}

describe("iPhone Tauri config", () => {
  it("allows original imports and temp mastered previews through local asset URLs", () => {
    const assetProtocol = readIphoneTauriConfig().app?.security?.assetProtocol;

    expect(assetProtocol?.enable).toBe(true);
    expect(assetScopeEntries(assetProtocol?.scope)).toEqual(
      expect.arrayContaining(["$DOCUMENT/**", "$TEMP/**"]),
    );
  });

  it("allows iPhone import and export dialogs", () => {
    expect(permissionIds(readIphoneDefaultCapability().permissions)).toEqual(
      expect.arrayContaining(["dialog:allow-open", "dialog:allow-save"]),
    );
  });

  it("serves the iPhone dev app on the network for physical devices", () => {
    expect(readRootPackageJson().scripts?.["iphone:dev"]).toBe(
      "vite --config apps/iphone/vite.config.ts",
    );
  });

  it("uses a cache-busted iPhone dev URL while debugging physical devices", () => {
    expect(readIphoneTauriConfig().build?.devUrl).toContain("ym_dev=B3-static");
  });

  it("locks the iPhone target to portrait orientation", () => {
    const plist = readIphoneText(
      "../src-tauri/gen/apple/yes-master-iphone_iOS/Info.plist",
    );

    expect(plistArrayValues(plist, "UISupportedInterfaceOrientations")).toEqual([
      "UIInterfaceOrientationPortrait",
    ]);
  });
});
