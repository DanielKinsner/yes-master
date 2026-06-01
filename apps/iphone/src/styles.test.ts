import { readFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { describe, expect, it } from "vitest";

function readIphoneStyles() {
  const currentDir = dirname(fileURLToPath(import.meta.url));
  return readFileSync(resolve(currentDir, "styles.css"), "utf8");
}

function cssBlock(css: string, selector: string) {
  let start = css.indexOf(`${selector} {`);
  if (start < 0) start = css.indexOf(selector);
  if (start < 0) return "";
  const open = css.indexOf("{", start);
  if (open < 0) return "";
  let depth = 0;
  let close = -1;
  for (let index = open; index < css.length; index += 1) {
    if (css[index] === "{") depth += 1;
    if (css[index] === "}") depth -= 1;
    if (depth === 0) {
      close = index;
      break;
    }
  }
  return open >= 0 && close >= 0 ? css.slice(open + 1, close) : "";
}

describe("iPhone styles", () => {
  it("uses the desktop brand palette for the phone shell", () => {
    const css = readIphoneStyles();

    expect(css).toContain("#030712");
    expect(css).toContain("#060b17");
    expect(css).toContain("#4d8bff");
    expect(css).toContain("#4da2ff");
    expect(css).toContain("#ffb86b");
    expect(css).toContain("color-scheme: dark");
  });

  it("draws a static dotted hero import target", () => {
    const css = readIphoneStyles();

    expect(css).toContain(".hero-orb::before");
    expect(css).toContain(".hero-orb::after");
    expect(css).toContain("box-shadow:");
    expect(css).toContain("repeating-radial-gradient");
    expect(css).not.toContain("@keyframes hero-ring-spin");
  });

  it("keeps decorative hero layers from blocking import taps", () => {
    const css = readIphoneStyles();
    const heroRingBlock = cssBlock(css, ".hero-orb::before,\n.hero-orb::after");
    const heroButtonBlock = cssBlock(css, ".hero-action-button");

    expect(heroRingBlock).toContain("pointer-events: none");
    expect(heroButtonBlock).toMatch(/z-index: [1-9]/);
  });

  it("makes the empty hero circle the import hotspot without pill chrome", () => {
    const css = readIphoneStyles();
    const heroButtonBlock = cssBlock(css, ".hero-action-button");
    const emptyHeroButtonBlock = cssBlock(
      css,
      ".hero-orb.is-empty .hero-action-button",
    );
    const emptyHeroLabelBlock = cssBlock(
      css,
      ".hero-orb.is-empty .hero-action-button span:last-child",
    );

    expect(heroButtonBlock).toContain("background: transparent");
    expect(heroButtonBlock).toContain("border: 0");
    expect(heroButtonBlock).toContain("box-shadow: none");
    expect(emptyHeroButtonBlock).toContain("height: min(64vw, 240px)");
    expect(emptyHeroButtonBlock).toContain("width: min(64vw, 240px)");
    expect(emptyHeroLabelBlock).toContain("position: absolute");
    expect(emptyHeroLabelBlock).toContain("top: 67%");
    expect(emptyHeroLabelBlock).toContain("transform: translate(-50%, -50%)");
  });

  it("uses a simple upload mark instead of center preset artwork", () => {
    const css = readIphoneStyles();
    const uploadGlyphBlock = cssBlock(css, ".hero-upload-glyph");

    expect(css).not.toContain(".preset-hero-art");
    expect(uploadGlyphBlock).toContain("color: var(--cta)");
    expect(uploadGlyphBlock).toContain("pointer-events: none");
    expect(uploadGlyphBlock).toContain("top: 45%");
    expect(css).toContain(".hero-upload-glyph::before");
    expect(css).toContain(".hero-upload-glyph::after");
  });

  it("uses the desktop spectrum glyph and an empty-state brand headline", () => {
    const css = readIphoneStyles();
    const brandMarkBlock = cssBlock(css, ".brand-mark");
    const heroHeadlineBlock = cssBlock(css, ".hero-copy h1");

    expect(brandMarkBlock).toContain("color: var(--accent-bright)");
    expect(brandMarkBlock).toContain("drop-shadow");
    expect(heroHeadlineBlock).toContain("font-weight: 700");
    expect(heroHeadlineBlock).not.toContain("clamp(");
  });

  it("keeps the mastered preview button as a visible play button", () => {
    const css = readIphoneStyles();
    const loadedHeroButtonBlock = cssBlock(
      css,
      ".hero-orb.has-track .hero-action-button",
    );

    expect(loadedHeroButtonBlock).toContain("radial-gradient");
    expect(loadedHeroButtonBlock).toContain("rgba(20, 87, 232, 0.96)");
    expect(loadedHeroButtonBlock).toContain("border-radius: 999px");
  });

  it("uses compact checkbox options near the hero instead of lower toggles", () => {
    const css = readIphoneStyles();

    expect(css).toContain(".hero-option-row");
    expect(css).toContain(".check-option");
    expect(css).toContain(".check-box");
    expect(css).not.toContain(".toggle-stack");
    expect(css).not.toContain(".toggle-row");
    expect(css).not.toContain(".switch");
  });

  it("keeps hero checkbox options unframed inside the import card", () => {
    const css = readIphoneStyles();
    const checkOptionBlock = cssBlock(css, ".check-option");

    expect(checkOptionBlock).toContain("background: transparent");
    expect(checkOptionBlock).toContain("border: 0");
    expect(checkOptionBlock).not.toContain("linear-gradient");
    expect(checkOptionBlock).not.toContain("border: 1px");
    expect(checkOptionBlock).not.toContain("border-radius:");
  });

  it("respects iOS safe areas around the phone shell", () => {
    const css = readIphoneStyles();

    expect(css).toContain("100dvh");
    expect(css).toContain("env(safe-area-inset-top)");
    expect(css).toContain("env(safe-area-inset-right)");
    expect(css).toContain("env(safe-area-inset-bottom)");
    expect(css).toContain("env(safe-area-inset-left)");
  });

  it("keeps the import hero compact enough to reveal presets", () => {
    const css = readIphoneStyles();

    expect(css).toContain("max(12px, env(safe-area-inset-top))");
    expect(css).toContain("min-height: 32px");
    expect(css).toContain("min-height: min(330px, calc(100dvh - 118px))");
    expect(css).toContain("min-height: min(246px, calc(100dvh - 172px))");
    expect(css).toContain("height: min(64vw, 240px)");
    expect(css).toContain("min-height: 70px");
  });

  it("styles loaded-track metadata as compact chips", () => {
    const css = readIphoneStyles();
    const chipRowBlock = cssBlock(css, ".track-chip-row");
    const chipBlock = cssBlock(css, ".track-chip-row span");

    expect(chipRowBlock).toContain("flex-wrap: wrap");
    expect(chipBlock).toContain("border-radius: 999px");
    expect(chipBlock).toContain("font-size: 0.65rem");
  });

  it("shows a polished Master Ready sheet after export", () => {
    const css = readIphoneStyles();

    expect(css).toContain(".export-ready-sheet");
    expect(css).toContain(".export-ready-mark");
    expect(css).toContain(".export-ready-stats");
    expect(css).toContain("@keyframes export-ready-enter");
  });

  it("renders a compact waveform preview in the audition panel", () => {
    const css = readIphoneStyles();

    expect(css).toContain(".mini-waveform");
    expect(css).toContain("grid-template-columns: repeat(44");
    expect(css).toContain("@keyframes waveform-loading");
  });

  it("uses desktop-like preset art blending and restrained brand typography", () => {
    const css = readIphoneStyles();
    const toneCardBlock = cssBlock(css, ".tone-card");
    const toneArtBlock = cssBlock(css, ".tone-card img");
    const trackLabelBlock = cssBlock(css, ".track-label");
    const customExportLabelBlock = cssBlock(css, ".custom-export-panel label");
    const processingStepsBlock = cssBlock(css, ".processing-steps");

    expect(toneCardBlock).toContain("grid-template-columns: 48px minmax(0, 1fr)");
    expect(toneArtBlock).toContain("mix-blend-mode: screen");
    expect(toneArtBlock).toContain("brightness(0.95)");
    expect(css).not.toMatch(/font-weight:\s*(850|900)/);
    expect(trackLabelBlock).toContain("letter-spacing: 0.12em");
    expect(customExportLabelBlock).toContain("letter-spacing: 0.1em");
    expect(processingStepsBlock).toContain("letter-spacing: 0.1em");
  });

  it("animates the import and analysis processing sheet responsibly", () => {
    const css = readIphoneStyles();

    expect(css).toContain(".processing-scrim");
    expect(css).toContain(".processing-card");
    expect(css).toContain("@keyframes processing-fade");
    expect(css).toContain("@keyframes processing-card-enter");
    expect(css).toContain("@keyframes processing-spin");
    expect(css).toContain("@keyframes processing-slide");
    expect(css).toContain("@media (prefers-reduced-motion: reduce)");
  });
});
