#!/usr/bin/env node
import { mkdir, writeFile } from "node:fs/promises";
import path from "node:path";
import { launchHeadless, runtimeStamp } from "./lib/headless-browser.mjs";

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

// `requiresNavAnchor` (added U6): the section must exist on the page, but not
// every section earns a nav slot. #mobile deliberately lost its link — a parked,
// unobtainable surface does not get top-level navigation — while still being
// required to exist, because the one date-free sentence it carries is the
// honest answer to a visitor who has seen the phone screens.
const requiredSections = [
  {
    id: "top",
    requiresNavAnchor: true,
    requiresMobileNavAnchor: true,
    requiresMobilePageAnchor: true,
  },
  {
    id: "how",
    requiresNavAnchor: true,
    requiresMobileNavAnchor: false,
    requiresMobilePageAnchor: false,
  },
  {
    id: "standard",
    requiresNavAnchor: false,
    requiresMobileNavAnchor: false,
    requiresMobilePageAnchor: false,
  },
  {
    id: "advanced",
    requiresNavAnchor: true,
    requiresMobileNavAnchor: false,
    requiresMobilePageAnchor: true,
  },
  {
    id: "sound",
    requiresNavAnchor: false,
    requiresMobileNavAnchor: false,
    requiresMobilePageAnchor: false,
  },
  {
    id: "album",
    requiresNavAnchor: true,
    requiresMobileNavAnchor: false,
    requiresMobilePageAnchor: false,
  },
  {
    id: "beta",
    requiresNavAnchor: true,
    requiresMobileNavAnchor: false,
    requiresMobilePageAnchor: false,
  },
  {
    id: "mobile",
    requiresNavAnchor: false,
    requiresMobileNavAnchor: false,
    requiresMobilePageAnchor: false,
  },
  {
    id: "get-started",
    requiresNavAnchor: true,
    requiresMobileNavAnchor: true,
    requiresMobilePageAnchor: true,
  },
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

// Browser runtime is settled in scripts/lib/headless-browser.mjs (U3). A
// missing browser exits nonzero there — this lane is a gate and must not skip.
const browser = await launchHeadless();
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
      // A BROKEN image is one the browser finished with and got nothing from:
      // `complete` true, `naturalWidth` zero. An image that is merely not
      // loaded YET is `complete === false` — which is the normal, correct state
      // of a below-fold `loading="lazy"` asset (U7) and was being reported as
      // broken here. Lazy images are separately proven to load, below.
      brokenImages: Array.from(document.images)
        .filter((image) => image.complete && image.naturalWidth === 0)
        .map((image) => image.currentSrc || image.src),
      lazyImages: Array.from(document.images)
        .filter((image) => image.loading === "lazy")
        .map((image) => image.getAttribute("src")),
      sections: required.map((id) => ({ id, present: Boolean(document.getElementById(id)) })),
      bodyHasExpectedCopy: [
        "see exactly what it did",
        "One engine",
        "Stop chasing the master",
        // U6 replaced "Same engine, headed to iPhone & Android" (a schedule
        // the product never committed to) with the one permitted date-free
        // sentence. These four anchor the rewritten hierarchy.
        "Finished mix in",
        "It reads the track before it touches it",
        "A record, not a folder of files",
        "What you are actually agreeing to",
      ].every((text) => body.textContent?.includes(text)),
      // U5 / S-A1 / S-I1. Evaluated against the REAL deployed page, so this is
      // the browser-headless instance of "a visitor arrives before a verified
      // release exists". S-A2 (after one exists) has no real state to observe
      // and stays an injected-state case in BetaDownload.test.tsx.
      release: (() => {
        const host = document.querySelector("[data-release-state]");
        const state = host?.getAttribute("data-release-state") ?? null;
        const deadLinks = Array.from(document.querySelectorAll("a[href]"))
          .map((link) => link.getAttribute("href") ?? "")
          .filter((href) => href.includes("/releases/latest"));

        const inactive = host?.querySelector("button[disabled]") ?? null;
        const describedBy = inactive?.getAttribute("aria-describedby") ?? null;
        const reasonEl = describedBy
          ? document.getElementById(describedBy)
          : null;
        const reasonRect = reasonEl?.getBoundingClientRect();
        const reasonStyle = reasonEl ? getComputedStyle(reasonEl) : null;

        return {
          state,
          deadLinks,
          activeDownloads: host?.querySelectorAll("a[data-platform]").length ?? 0,
          inactivePresent: Boolean(inactive),
          reasonText: reasonEl?.textContent?.trim() ?? null,
          // "Visible" means occupying space and painted — a reason rendered
          // into a zero-height or clipped node is a tooltip with extra steps.
          reasonVisible: Boolean(
            reasonEl &&
              reasonRect &&
              reasonRect.width > 1 &&
              reasonRect.height > 1 &&
              reasonStyle?.display !== "none" &&
              reasonStyle?.visibility !== "hidden" &&
              Number(reasonStyle?.opacity ?? "1") > 0.1,
          ),
          mobileHonest: Boolean(
            body.textContent?.includes(
              "iPhone and Android are not currently available",
            ) && body.textContent?.includes("no Linux build"),
          ),
        };
      })(),
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
      if (section.requiresNavAnchor && !visibleNavTargets.includes(href)) {
        failures.push(`${width}x${height}: desktop nav missing visible ${href} link`);
      }
    } else if (section.requiresMobileNavAnchor && !visibleNavTargets.includes(href)) {
      failures.push(`${width}x${height}: mobile nav missing visible ${href} link`);
    }
  }
  if (width < 640) {
    const phoneHiddenTargets = requiredSections
      .filter(
        (section) => section.requiresNavAnchor && !section.requiresMobileNavAnchor,
      )
      .map((section) => `#${section.id}`);
    const visibleHiddenTargets = phoneHiddenTargets.filter((href) => visibleNavTargets.includes(href));
    if (visibleHiddenTargets.length > 0) {
      failures.push(`${width}x${height}: phone nav exposes desktop-only links ${visibleHiddenTargets.join(", ")}`);
    }
  }

  // U5 — the download must never be a dead link, at any viewport.
  const release = metrics.release;
  if (!release?.state) {
    failures.push(`${width}x${height}: no [data-release-state] host on the page`);
  }
  if ((release?.deadLinks?.length ?? 0) > 0) {
    // The exact defect this unit removes: a link that only breaks in
    // production, where nothing tests it.
    failures.push(
      `${width}x${height}: page links to /releases/latest (${release.deadLinks.join(", ")})`,
    );
  }
  if (!release?.mobileHonest) {
    failures.push(
      `${width}x${height}: platform-support copy does not state the Linux and mobile position`,
    );
  }
  // No release is verified yet, so the live page must be in a closed state.
  // When one is verified this branch flips and S-A2 becomes observable here.
  if (release?.state && release.state !== "verified-public") {
    if (release.activeDownloads > 0) {
      failures.push(
        `${width}x${height}: ${release.activeDownloads} download action(s) rendered while release state is ${release.state}`,
      );
    }
    if (!release.inactivePresent) {
      failures.push(`${width}x${height}: closed state renders no inactive action`);
    }
    if (!release.reasonVisible) {
      failures.push(
        `${width}x${height}: inactive download has no visible, associated reason`,
      );
    }
    if (!release.reasonText || release.reasonText.length < 20) {
      failures.push(
        `${width}x${height}: inactive download reason is missing or too short ("${release.reasonText}")`,
      );
    }
    for (const leak of ["draft", "unverified", "candidate"]) {
      if (release.reasonText?.toLowerCase().includes(leak)) {
        failures.push(
          `${width}x${height}: visitor-facing reason leaks internal state ("${leak}")`,
        );
      }
    }
  }

  // U7: prove the lazy assets actually arrive. Deferring an image is only
  // correct if it still loads when the visitor reaches it — a lazy image that
  // never resolves is a broken image with better manners, and the check above
  // deliberately cannot see it.
  if (width === 1440 || width === 390) {
    await page.evaluate(() =>
      window.scrollTo({ top: document.body.scrollHeight, behavior: "instant" }),
    );
    const lazyState = await page
      .waitForFunction(
        () => {
          const lazy = Array.from(document.images).filter(
            (image) => image.loading === "lazy",
          );
          return lazy.every((image) => image.complete && image.naturalWidth > 0)
            ? lazy.map((image) => ({
                src: image.getAttribute("src"),
                width: image.naturalWidth,
              }))
            : null;
        },
        undefined,
        { timeout: 10_000 },
      )
      .then((handle) => handle.jsonValue())
      .catch(() => null);

    if (!lazyState) {
      failures.push(
        `${width}x${height}: lazy images did not load after scrolling to the bottom`,
      );
    } else {
      metrics.lazyLoaded = lazyState;
    }
    await page.evaluate(() =>
      window.scrollTo({ top: 0, behavior: "instant" }),
    );
  }

  records.push({ width, height, screenshot, metrics });
}

function anchorsForWidth(width) {
  return requiredSections
    .filter((section) => section.requiresNavAnchor)
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

const browserStamp = runtimeStamp(browser);
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
  // SYNTHETIC evidence. A pass here proves layout, copy, and link behavior in a
  // headless browser. It proves nothing about native dialogs, real audio,
  // installer signing, updater install, or how anything sounds.
  evidenceLayer: "browser-headless",
  // Which named scenarios this lane actually observes, and which it cannot.
  // Recorded so a green run is not mistaken for coverage it does not have.
  scenarioCoverage: {
    "S-A1": "observed — live page before a verified release exists",
    "S-I1": "observed — platform-support copy at every viewport incl. 360px",
    "S-A2":
      "NOT observed here — needs a verified release; injected-state case lives in src/landing/BetaDownload.test.tsx, closed by U17",
    "S-B1":
      "NOT observed here — needs a real draft release; injected-state case lives in src/lib/release-readiness.test.ts, closed by U16",
  },
  browser: browserStamp,
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
