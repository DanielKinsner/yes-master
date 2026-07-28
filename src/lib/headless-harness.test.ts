import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { describe, expect, it } from "vitest";

const source = readFileSync(resolve(process.cwd(), "scripts/verify-headless.mjs"), "utf8");

describe("headless verification harness", () => {
  it("launches pinned Node CLIs without a Windows command shell", () => {
    expect(source).not.toMatch(/shell:\s*(?:true|process\.platform)/);
    expect(source).toContain("process.env.npm_execpath");
    expect(source).toContain('import.meta.resolve("vite")');
  });

  it("still terminates the complete Windows preview process tree", () => {
    expect(source).toContain('spawn("taskkill"');
    expect(source).toContain('["/pid", String(child.pid), "/T", "/F"]');
  });
});
