import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { describe, expect, it } from "vitest";

const css = readFileSync(resolve(process.cwd(), "src/LandingPage.css"), "utf8");

describe("landing page responsive CSS", () => {
  it("never stretches the hero image (cover, not fill or contain)", () => {
    // `object-fit: fill` distorts the image to the container's aspect ratio
    // (the old desktop bug); `contain` letterboxes. The hero must use `cover`
    // so it crops without distorting at any viewport ratio.
    expect(css).not.toContain("object-fit: fill");
    expect(css).toMatch(/\.landing-hero-scene \{[\s\S]*?object-fit: cover;/);
  });

  it("shows the real hero copy at desktop instead of a baked image map", () => {
    const desktopBlock =
      css.match(/@media \(min-width: 1081px\) \{([\s\S]*?)\n\}/)?.[1] ?? "";
    // The desktop breakpoint must not hide the real copy or fake interactivity
    // with positioned hotspots over a flat comp image.
    expect(desktopBlock).not.toMatch(/\.landing-hero-copy[\s\S]*?display:\s*none/);
    expect(desktopBlock).not.toContain("landing-hotspot");
  });

  it("uses a live tablet hero with real copy", () => {
    expect(css).toContain("@media (min-width: 681px) and (max-width: 1080px)");
    expect(css).toContain(".landing-hero-copy {\n    display: block;");
  });

  it("keeps proof panel screenshots balanced and uncropped", () => {
    expect(css).toContain("object-fit: contain;");
    expect(css).not.toContain("transform: scale(1.1)");
  });
});
