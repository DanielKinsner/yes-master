import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { describe, expect, it } from "vitest";

const css = readFileSync(resolve(process.cwd(), "src/LandingPage.css"), "utf8");

describe("landing page responsive CSS", () => {
  it("does not letterbox the desktop hero image", () => {
    expect(css).not.toContain("object-fit: contain");
    expect(css).not.toContain("max-aspect-ratio: 16 / 9");
  });

  it("uses a live tablet hero instead of the desktop-only image map", () => {
    expect(css).toContain("@media (min-width: 681px) and (max-width: 1080px)");
    expect(css).toContain("@media (min-width: 1081px) and (max-aspect-ratio: 5 / 3)");
    expect(css).toContain(".landing-hero-hotspots {\n    display: none;");
    expect(css).toContain(".landing-hero-copy {\n    display: block;");
  });
});
