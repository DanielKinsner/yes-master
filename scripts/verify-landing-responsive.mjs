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

const requiredSections = [
  { id: "top", requiresMobileNavAnchor: true, requiresMobilePageAnchor: true },
  { id: "mobile", requiresMobileNavAnchor: false, requiresMobilePageAnchor: false },
  { id: "standard", requiresMobileNavAnchor: false, requiresMobilePageAnchor: false },
  { id: "advanced", requiresMobileNavAnchor: false, requiresMobilePageAnchor: true },
  { id: "get-started", requiresMobileNavAnchor: true, requiresMobilePageAnchor: true },
];
const requiredSectionIds = requiredSections.map((section) => section.id);

function option(name) {
  const index = process.argv.indexOf(name);
  return index === -1 ? undefined : process.argv[index + 1];
}

function stamp() {
  return new Date().toISOString().replace(/[:.]/g, "-");
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
    const hero = document.getElementById("top");
    const heroImage = hero?.querySelector('img[aria-hidden="true"]');
    const heroHeadline = hero?.querySelector("h1");
    const heroCopy = heroHeadline?.closest("div");
    const nav = document.querySelector("nav");
    const heroRect = hero?.getBoundingClientRect();
    const copyRect = heroCopy?.getBoundingClientRect();
    const imageStyle = heroImage ? getComputedStyle(heroImage) : null;
    const copyStyle = heroCopy ? getComputedStyle(heroCopy) : null;
    const navStyle = nav ? getComputedStyle(nav) : null;
    const sectionLinks = Array.from(nav?.querySelectorAll('a[href^="#"]') ?? []).map((link) => {
      const rect = link.getBoundingClientRect();
      const style = getComputedStyle(link);
      return {
        href: link.getAttribute("href"),
        label: link.textContent?.trim() ?? "",
        visible:
          rect.width > 1 &&
          rect.height > 1 &&
          style.display !== "none" &&
          style.visibility !== "hidden",
      };
    });

    return {
      title: document.title,
      viewport: { width: window.innerWidth, height: window.innerHeight },
      scrollWidth: Math.max(doc.scrollWidth, body.scrollWidth),
      clientWidth: doc.clientWidth,
      horizontalOverflow: Math.max(doc.scrollWidth, body.scrollWidth) > doc.clientWidth + 1,
      heroHeight: heroRect?.height ?? null,
      heroWidth: heroRect?.width ?? null,
      heroPresent: Boolean(hero),
      heroHeadline: heroHeadline?.textContent ?? null,
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
      navDisplay: navStyle?.display ?? null,
      navPosition: navStyle?.position ?? null,
      navLinks: sectionLinks,
      brokenImages: Array.from(document.images)
        .filter((image) => !image.complete || image.naturalWidth === 0)
        .map((image) => image.currentSrc || image.src),
      sections: required.map((id) => ({ id, present: Boolean(document.getElementById(id)) })),
      bodyHasExpectedCopy: [
        "see exactly what it did",
        "One engine",
        "Stop chasing the master",
        "Same engine",
      ].every((text) => body.textContent?.includes(text)),
    };
  }, requiredSectionIds);

  const missingSections = metrics.sections
    .filter((section) => !section.present)
    .map((section) => section.id);

  if (metrics.title !== "YES Master") {
    failures.push(`${width}x${height}: unexpected title "${metrics.title}"`);
  }
  if (metrics.horizontalOverflow) {
    failures.push(`${width}x${height}: horizontal overflow (${metrics.scrollWidth} > ${metrics.clientWidth})`);
  }
  if (!metrics.heroPresent) {
    failures.push(`${width}x${height}: missing #top hero section`);
  }
  if (typeof metrics.heroHeight !== "number" || metrics.heroHeight < height * 0.9) {
    failures.push(`${width}x${height}: hero height is ${metrics.heroHeight}, expected at least ${Math.round(height * 0.9)}`);
  }
  if (metrics.imageFit !== "cover") {
    failures.push(`${width}x${height}: hero image fit is ${metrics.imageFit}, expected cover`);
  }
  if (!metrics.heroHeadline?.includes("Master your track in real time")) {
    failures.push(`${width}x${height}: hero headline missing expected copy`);
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
  if (metrics.copyDisplay === "none" || !metrics.copyRect || metrics.copyRect.width < 240) {
    failures.push(`${width}x${height}: live hero copy is not visible`);
  }
  if (metrics.navDisplay !== "flex" || metrics.navPosition !== "fixed") {
    failures.push(`${width}x${height}: landing nav is ${metrics.navPosition}/${metrics.navDisplay}, expected fixed/flex`);
  }
  const visibleNavTargets = metrics.navLinks.filter((link) => link.visible).map((link) => link.href);
  for (const section of requiredSections) {
    const href = `#${section.id}`;
    if (width >= 640) {
      if (!visibleNavTargets.includes(href)) {
        failures.push(`${width}x${height}: desktop nav missing visible ${href} link`);
      }
    } else if (section.requiresMobileNavAnchor && !visibleNavTargets.includes(href)) {
      failures.push(`${width}x${height}: mobile nav missing visible ${href} link`);
    }
  }
  if (width < 640) {
    const phoneHiddenTargets = requiredSections
      .filter((section) => !section.requiresMobileNavAnchor)
      .map((section) => `#${section.id}`);
    const visibleHiddenTargets = phoneHiddenTargets.filter((href) => visibleNavTargets.includes(href));
    if (visibleHiddenTargets.length > 0) {
      failures.push(`${width}x${height}: phone nav exposes desktop-only links ${visibleHiddenTargets.join(", ")}`);
    }
  }

  records.push({ width, height, screenshot, metrics });
}

function anchorsForWidth(width) {
  return requiredSections
    .filter((section) => width >= 640 || section.requiresMobilePageAnchor)
    .map((section) => `#${section.id}`);
}

for (const [width, height] of [
  [1440, 900],
  [390, 844],
]) {
  await page.setViewportSize({ width, height });
  for (const href of anchorsForWidth(width)) {
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
