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

type AssetProtocolScope = string[] | { allow?: string[] } | undefined;

function readIphoneTauriConfig(): IphoneTauriConfig {
  const currentDir = dirname(fileURLToPath(import.meta.url));
  const configPath = resolve(currentDir, "../src-tauri/tauri.conf.json");
  return JSON.parse(readFileSync(configPath, "utf8")) as IphoneTauriConfig;
}

function assetScopeEntries(scope: AssetProtocolScope): string[] {
  if (Array.isArray(scope)) return scope;
  return scope?.allow ?? [];
}

describe("iPhone Tauri config", () => {
  it("allows temp mastered previews through local asset URLs", () => {
    const assetProtocol = readIphoneTauriConfig().app?.security?.assetProtocol;

    expect(assetProtocol?.enable).toBe(true);
    expect(assetScopeEntries(assetProtocol?.scope)).toEqual(
      expect.arrayContaining(["$TEMP/**"]),
    );
  });
});
