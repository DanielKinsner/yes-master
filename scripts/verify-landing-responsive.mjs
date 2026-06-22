#!/usr/bin/env node
import { mkdir, writeFile } from "node:fs/promises";
import path from "node:path";
import { chromium } from "playwright";

const matrix = [
  [2560, 1440],
  [2048, 1152],
  [1728, 1117],
  [1600, 900],
  [1440, 900],
  [1366, 768],
  [1280, 800],
  [1024, 768],
  [768, 1024],
  [430, 932],
  [390, 844],
  [360, 800],
];

const requiredSections = ["top", "mobile", "standard", "advanced", "get-started"];
const requiredAnchors = ["#standard", "#advanced", "#mobile", "#get-started"];

function option(name) {
  const index = process.argv.indexOf(name);
  return index === -1 ? undefined : process.argv[index + 1];
}

function stamp() {
  return new Date().toISOString().replace(/[:.]/g, "-");
}

function expectedComposition(width, height) {
  if (width <= 680) return "overlay";
  if (width <= 1080) return "overlay";
  return "image-map";
}

const url = option("--url") ?? process.env.LANDING_URL ?? "http://127.0.0.1:5177/";
const outDir =
  option("--out") ?? path.join("test-output", "landing-responsive", stamp());

await mkdir(outDir, { recursive: true });

const failures = [];
const records = [];
const anchorRecords = [];

const browser = await chromium.launch({ channel: "chrome", headless: true });
const page = await browser.newPage();
const consoleMessages = [];
const pageErrors = [];

page.on("console", (message) => {
  if (["error", "warning", "warn"].includes(message.type())) {
    consoleMessages.push({ type: message.type(), text: message.text() });
  }
});
page.on("pageerror", (error) => {
  pageErrors.push(error.message);
});

for (const [width, height] of matrix) {
  await page.setViewportSize({ width, height });
  await page.goto(url, { waitUntil: "networkidle" });

  const screenshot = path.join(outDir, `${width}x${height}.png`);
  await page.screenshot({ path: screenshot, fullPage: false });

  const metrics = await page.evaluate((required) => {
    const doc = document.documentElement;
    const body = document.body;
    const hero = document.querySelector(".landing-hero");
    const heroImage = document.querySelector(".landing-hero-scene");
    const heroCopy = document.querySelector(".landing-hero-copy");
    const hotspots = document.querySelector(".landing-hero-hotspots");
    const nav = document.querySelector(".landing-nav");
    const heroRect = hero?.getBoundingClientRect();
    const copyRect = heroCopy?.getBoundingClientRect();
    const imageStyle = heroImage ? getComputedStyle(heroImage) : null;
    const copyStyle = heroCopy ? getComputedStyle(heroCopy) : null;
    const hotspotStyle = hotspots ? getComputedStyle(hotspots) : null;
    const navStyle = nav ? getComputedStyle(nav) : null;

    return {
      title: document.title,
      viewport: { width: window.innerWidth, height: window.innerHeight },
      scrollWidth: Math.max(doc.scrollWidth, body.scrollWidth),
      clientWidth: doc.clientWidth,
      horizontalOverflow: Math.max(doc.scrollWidth, body.scrollWidth) > doc.clientWidth + 1,
      heroHeight: heroRect?.height ?? null,
      heroWidth: heroRect?.width ?? null,
      imageFit: imageStyle?.objectFit ?? null,
      imagePosition: imageStyle?.objectPosition ?? null,
      copyDisplay: copyStyle?.display ?? null,
      copyRect: copyRect
        ? {
            x: copyRect.x,
            y: copyRect.y,
            width: copyRect.width,
            height: copyRect.height,
            bottom: copyRect.bottom,
          }
        : null,
      hotspotsDisplay: hotspotStyle?.display ?? null,
      navDisplay: navStyle?.display ?? null,
      brokenImages: Array.from(document.images)
        .filter((image) => !image.complete || image.naturalWidth === 0)
        .map((image) => image.currentSrc || image.src),
      sections: required.map((id) => ({ id, present: Boolean(document.getElementById(id)) })),
      bodyHasExpectedCopy: [
        "Your Endgame",
        "Master anywhere",
        "One engine",
        "Stop chasing the master",
      ].every((text) => body.textContent?.includes(text)),
    };
  }, requiredSections);

  const expected = expectedComposition(width, height);
  const missingSections = metrics.sections
    .filter((section) => !section.present)
    .map((section) => section.id);

  if (metrics.title !== "YES Master") {
    failures.push(`${width}x${height}: unexpected title "${metrics.title}"`);
  }
  if (metrics.horizontalOverflow) {
    failures.push(`${width}x${height}: horizontal overflow (${metrics.scrollWidth} > ${metrics.clientWidth})`);
  }
  const expectedFit = expected === "image-map" ? "fill" : "cover";
  if (metrics.imageFit !== expectedFit) {
    failures.push(`${width}x${height}: hero image fit is ${metrics.imageFit}, expected ${expectedFit}`);
  }
  if (metrics.brokenImages.length > 0) {
    failures.push(`${width}x${height}: broken images ${metrics.brokenImages.join(", ")}`);
  }
  if (missingSections.length > 0) {
    failures.push(`${width}x${height}: missing sections ${missingSections.join(", ")}`);
  }
  if (!metrics.bodyHasExpectedCopy) {
    failures.push(`${width}x${height}: expected landing copy missing from document`);
  }
  if (expected === "overlay") {
    if (metrics.copyDisplay === "none") {
      failures.push(`${width}x${height}: overlay breakpoint is hiding live hero copy`);
    }
    if (metrics.hotspotsDisplay !== "none") {
      failures.push(`${width}x${height}: overlay breakpoint still exposes image-map hotspots`);
    }
  } else {
    if (metrics.copyDisplay !== "none") {
      failures.push(`${width}x${height}: wide image-map breakpoint shows duplicate live copy`);
    }
    if (metrics.hotspotsDisplay !== "block") {
      failures.push(`${width}x${height}: wide image-map breakpoint hides hotspots`);
    }
  }

  records.push({ width, height, expected, screenshot, metrics });
}

for (const [width, height] of [
  [1440, 900],
  [390, 844],
]) {
  await page.setViewportSize({ width, height });
  for (const href of requiredAnchors) {
    await page.goto(url, { waitUntil: "networkidle" });
    const clicked = await page.evaluate((targetHref) => {
      const links = Array.from(document.querySelectorAll(`a[href="${targetHref}"]`));
      const link = links.find((candidate) => {
        const rect = candidate.getBoundingClientRect();
        const style = getComputedStyle(candidate);
        return (
          rect.width > 1 &&
          rect.height > 1 &&
          style.display !== "none" &&
          style.visibility !== "hidden" &&
          style.pointerEvents !== "none"
        );
      });
      if (!link) return false;
      link.click();
      return true;
    }, href);

    await page.waitForTimeout(80);
    const anchorState = await page.evaluate((targetHref) => {
      const target = document.getElementById(targetHref.slice(1));
      const rect = target?.getBoundingClientRect();
      return {
        hash: window.location.hash,
        targetTop: rect?.top ?? null,
        targetPresent: Boolean(target),
      };
    }, href);

    anchorRecords.push({ width, height, href, clicked, ...anchorState });

    if (!clicked) {
      failures.push(`${width}x${height}: no usable anchor found for ${href}`);
    }
    if (anchorState.hash !== href) {
      failures.push(`${width}x${height}: clicking ${href} left hash at ${anchorState.hash}`);
    }
    if (!anchorState.targetPresent) {
      failures.push(`${width}x${height}: target missing for ${href}`);
    }
  }
}

await browser.close();

const relevantConsoleMessages = consoleMessages.filter(
  (entry) => !entry.text.includes("favicon"),
);

if (relevantConsoleMessages.length > 0) {
  failures.push(`console warnings/errors: ${JSON.stringify(relevantConsoleMessages)}`);
}
if (pageErrors.length > 0) {
  failures.push(`page errors: ${JSON.stringify(pageErrors)}`);
}

const summary = {
  url,
  outDir,
  matrix: records,
  anchors: anchorRecords,
  consoleMessages,
  pageErrors,
  failures,
};

await writeFile(path.join(outDir, "summary.json"), `${JSON.stringify(summary, null, 2)}\n`);

if (failures.length > 0) {
  console.error(`Landing responsive verification failed. Evidence: ${outDir}`);
  for (const failure of failures) console.error(`- ${failure}`);
  process.exit(1);
}

console.log(`Landing responsive verification passed. Evidence: ${outDir}`);
