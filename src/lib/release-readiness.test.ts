import { readFileSync } from "node:fs";

import { describe, expect, it } from "vitest";

function readText(path: string): string {
  return readFileSync(new URL(path, import.meta.url), "utf8");
}

const workflow = readText("../../.github/workflows/release.yml");
const windowsSigning = readText("../../src-tauri/tauri.windows-signing.conf.json");

describe("zero-cost beta release contract", () => {
  it("builds one universal macOS artifact instead of separate architecture releases", () => {
    expect(workflow.match(/platform: macos-latest/g)).toHaveLength(1);
    expect(workflow).toContain('label: macOS (Universal)');
    expect(workflow).toContain('args: "--target universal-apple-darwin"');
    expect(workflow).not.toContain('args: "--target aarch64-apple-darwin"');
    expect(workflow).not.toContain('args: "--target x86_64-apple-darwin"');
  });

  it("requires free updater signing while leaving paid OS signing optional", () => {
    expect(workflow).toContain("name: Require updater signing secrets");
    expect(workflow).toContain("TAURI_SIGNING_PRIVATE_KEY");
    expect(workflow).toContain("TAURI_SIGNING_PRIVATE_KEY_PASSWORD");
    expect(workflow).toContain("--config src-tauri/tauri.updater.conf.json");

    expect(workflow).toContain("env.APPLE_CERTIFICATE != ''");
    expect(workflow).toContain("env.AZURE_CLIENT_ID != ''");
  });

  it("keeps the GitHub latest channel discoverable by the updater", () => {
    expect(workflow).toContain("prerelease: false");
    expect(workflow).not.toContain("github.event.inputs.prerelease");
  });

  it("uses the current Azure Artifact Signing CLI when paid signing is added later", () => {
    expect(workflow).toContain("cargo install artifact-signing-cli");
    expect(workflow).not.toContain("cargo install trusted-signing-cli");
    expect(windowsSigning).toContain("artifact-signing-cli");
    expect(windowsSigning).not.toContain("trusted-signing-cli");
  });

  it("audits the draft before the owner can publish an incomplete release", () => {
    expect(workflow).toContain("name: Audit draft release assets");
    expect(workflow).toContain("latest.json");
    expect(workflow).toContain(".dmg");
    expect(workflow).toContain(".msi");
    expect(workflow).toContain(".sig");
  });
});
