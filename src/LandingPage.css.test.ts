import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { describe, expect, it } from "vitest";

const css = readFileSync(resolve(process.cwd(), "src/LandingPage.css"), "utf8");
const desktopHeroBlock = css.match(
  /@media \(min-width: 1081px\) \{[\s\S]*?\.landing-hero-scene \{([\s\S]*?)\n  \}/,
)?.[1];

describe("landing page responsive CSS", () => {
  it("does not letterbox the desktop hero image", () => {
    expect(desktopHeroBlock).toContain("object-fit: fill;");
    expect(desktopHeroBlock).not.toContain("object-fit: contain");
    expect(css).not.toContain("max-aspect-ratio: 16 / 9");
    expect(css).not.toContain("@media (min-width: 1081px) and (max-aspect-ratio: 5 / 3)");
  });

  it("uses a live tablet hero instead of the desktop-only image map", () => {
    expect(css).toContain("@media (min-width: 681px) and (max-width: 1080px)");
    expect(css).toContain(".landing-hero-hotspots {\n    display: none;");
    expect(css).toContain(".landing-hero-copy {\n    display: block;");
  });

  it("keeps proof panel screenshots balanced and uncropped", () => {
    expect(css).toContain(".landing-proof-panel {\n  position: relative;\n  overflow: hidden;\n  display: flex;");
    expect(css).toContain("object-fit: contain;");
    expect(css).not.toContain("transform: scale(1.1)");
    expect(css).toContain(".landing-panel-image-standard {\n  margin-top: 18px;\n  aspect-ratio: 16 / 9;");
  });
});
