#!/usr/bin/env node
// -----------------------------------------------------------------------------
// U7 — resize/recompress the landing page's ART (not its captures).
//
//   npm run optimize:landing-art
//
// Captures are produced by capture-landing-assets.mjs at their display size and
// need no processing. The hero photograph and the three proof icons are hand-made
// art that shipped at their source resolution: three 400×400 PNGs rendered into
// a 56px box, and a 2560×1440 hero JPEG, together blowing the eager budget.
//
// There is no imagemagick or sharp in this toolchain, and adding an image
// dependency for four files is a poor trade. Chromium is already here, pinned by
// the lockfile, and canvas resampling is deterministic for a fixed browser
// build — the same input produces the same bytes on every run, which is what
// matters for a hashed, manifest-bound asset.
//
// Rerunning this on already-optimized files is a no-op in intent but NOT in
// bytes (re-encoding always drifts), so it checks the source dimensions first
// and skips anything already at target size.
// -----------------------------------------------------------------------------
import { readFile, writeFile } from "node:fs/promises";
import path from "node:path";

import { launchHeadless } from "./lib/headless-browser.mjs";
import { REPO_ROOT, pngDimensions } from "./lib/landing-assets.mjs";

// The hero photograph was 97% of the eager budget on its own. Two variants: a
// recompressed full-size one for desktop, and a half-width one so a phone does
// not download a 2560px image to render it 390px wide.
//
// The SOURCE is a separate pristine file that nothing imports (so it never
// reaches the bundle) and that this script only ever reads. The first version
// of this script wrote the full-size variant back over its own input, which
// made re-running it stack generation loss on the master art — the comment
// claiming otherwise was simply wrong. Keeping the master is worth 1.4 MB of
// repo that ships to nobody.
const HERO = {
  source: "src/assets/landing/hero-control-room-studio-source.jpg",
  variants: [
    { file: "src/assets/landing/hero-control-room-studio.jpg", width: 2560, quality: 0.82 },
    { file: "src/assets/landing/hero-control-room-studio-1280.jpg", width: 1280, quality: 0.82 },
  ],
};

const ART = [
  // Rendered in a 56px box (h-14/w-14). 168px covers a 3× display and is still
  // less than half the shipped width.
  { file: "src/assets/landing/icon-realtime.png", width: 168, type: "image/png" },
  { file: "src/assets/landing/icon-local-first.png", width: 168, type: "image/png" },
  { file: "src/assets/landing/icon-release-ready.png", width: 168, type: "image/png" },
];

const browser = await launchHeadless();
try {
  const page = await browser.newPage();
  await page.goto("about:blank");

  for (const art of ART) {
    const absolute = path.join(REPO_ROOT, art.file);
    const source = await readFile(absolute);
    const before = pngDimensions(source);
    if (before && before.width <= art.width) {
      console.log(`[art] ${path.basename(art.file)} already ${before.width}px — skipped`);
      continue;
    }

    const dataUrl = `data:${art.type};base64,${source.toString("base64")}`;
    const encoded = await page.evaluate(
      async ({ url, width, type }) => {
        const image = new Image();
        image.src = url;
        await image.decode();
        const scale = width / image.naturalWidth;
        const canvas = document.createElement("canvas");
        canvas.width = width;
        canvas.height = Math.round(image.naturalHeight * scale);
        const context = canvas.getContext("2d");
        context.imageSmoothingEnabled = true;
        context.imageSmoothingQuality = "high";
        context.drawImage(image, 0, 0, canvas.width, canvas.height);
        return canvas.toDataURL(type).split(",")[1];
      },
      { url: dataUrl, width: art.width, type: art.type },
    );

    const output = Buffer.from(encoded, "base64");
    await writeFile(absolute, output);
    const after = pngDimensions(output);
    console.log(
      `[art] ${path.basename(art.file)}  ${before?.width}px ${Math.round(source.length / 1024)} KB` +
        `  →  ${after?.width}px ${Math.round(output.length / 1024)} KB`,
    );
  }
  // Hero variants, always encoded from the pristine source above, so repeated
  // runs are idempotent in quality even though the bytes drift.
  const heroSource = await readFile(path.join(REPO_ROOT, HERO.source));
  const heroUrl = `data:image/jpeg;base64,${heroSource.toString("base64")}`;
  for (const variant of HERO.variants) {
    const encoded = await page.evaluate(
      async ({ url, width, quality }) => {
        const image = new Image();
        image.src = url;
        await image.decode();
        const scale = Math.min(1, width / image.naturalWidth);
        const canvas = document.createElement("canvas");
        canvas.width = Math.round(image.naturalWidth * scale);
        canvas.height = Math.round(image.naturalHeight * scale);
        const context = canvas.getContext("2d");
        context.imageSmoothingEnabled = true;
        context.imageSmoothingQuality = "high";
        context.drawImage(image, 0, 0, canvas.width, canvas.height);
        return canvas.toDataURL("image/jpeg", quality).split(",")[1];
      },
      { url: heroUrl, width: variant.width, quality: variant.quality },
    );
    const output = Buffer.from(encoded, "base64");
    await writeFile(path.join(REPO_ROOT, variant.file), output);
    console.log(
      `[art] ${path.basename(variant.file)}  ${variant.width}px  ${Math.round(output.length / 1024)} KB`,
    );
  }
} finally {
  await browser.close();
}
