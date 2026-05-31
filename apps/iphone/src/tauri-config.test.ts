import { readFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { describe, expect, it } from "vitest";

interface IphoneTauriConfig {
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

type AssetProtocolScope = string[] | { allow?: string[] } | undefined;

function readIphoneJson<T>(relativePath: string): T {
  const currentDir = dirname(fileURLToPath(import.meta.url));
  const configPath = resolve(currentDir, relativePath);
  return JSON.parse(readFileSync(configPath, "utf8")) as T;
}

function readIphoneTauriConfig(): IphoneTauriConfig {
  return readIphoneJson("../src-tauri/tauri.conf.json");
}

function readIphoneDefaultCapability(): IphoneCapability {
  return readIphoneJson("../src-tauri/capabilities/default.json");
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

describe("iPhone Tauri config", () => {
  it("allows temp mastered previews through local asset URLs", () => {
    const assetProtocol = readIphoneTauriConfig().app?.security?.assetProtocol;

    expect(assetProtocol?.enable).toBe(true);
    expect(assetScopeEntries(assetProtocol?.scope)).toEqual(
      expect.arrayContaining(["$TEMP/**"]),
    );
  });

  it("allows iPhone import and export dialogs", () => {
    expect(permissionIds(readIphoneDefaultCapability().permissions)).toEqual(
      expect.arrayContaining(["dialog:allow-open", "dialog:allow-save"]),
    );
  });
});
