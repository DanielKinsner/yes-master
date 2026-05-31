import { readFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { describe, expect, it } from "vitest";

function readIphoneStyles() {
  const currentDir = dirname(fileURLToPath(import.meta.url));
  return readFileSync(resolve(currentDir, "styles.css"), "utf8");
}

describe("iPhone styles", () => {
  it("uses the desktop brand palette for the phone shell", () => {
    const css = readIphoneStyles();

    expect(css).toContain("#07080d");
    expect(css).toContain("#10131a");
    expect(css).toContain("#4d8bff");
    expect(css).toContain("#6fa3ff");
    expect(css).toContain("#ffb86b");
    expect(css).toContain("color-scheme: dark");
  });

  it("animates a dotted hero import ring", () => {
    const css = readIphoneStyles();

    expect(css).toContain(".hero-orb::before");
    expect(css).toContain(".hero-orb::after");
    expect(css).toContain("border: 2px dashed");
    expect(css).toContain("animation: hero-ring-spin");
    expect(css).toContain("@keyframes hero-ring-spin");
  });

  it("respects iOS safe areas around the phone shell", () => {
    const css = readIphoneStyles();

    expect(css).toContain("100dvh");
    expect(css).toContain("env(safe-area-inset-top)");
    expect(css).toContain("env(safe-area-inset-right)");
    expect(css).toContain("env(safe-area-inset-bottom)");
    expect(css).toContain("env(safe-area-inset-left)");
  });
});
