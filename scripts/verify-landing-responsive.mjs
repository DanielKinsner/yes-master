#!/usr/bin/env node
import { mkdir, readFile, writeFile } from "node:fs/promises";
import path from "node:path";
import { createRequire } from "node:module";
import { launchHeadless, runtimeStamp } from "./lib/headless-browser.mjs";

const require = createRequire(import.meta.url);
// axe-core is injected as a script rather than driven through a wrapper: one
// dependency, no adapter to keep in sync, and the same source CI installs.
const AXE_SOURCE = await readFile(require.resolve("axe-core/axe.min.js"), "utf8");

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
  // S-A3 names 320x568 explicitly and no other entry covered it, so the
  // scenario could not pass as written. It is the smallest width still in
  // real use and the one where a fixed nav plus a CTA runs out of room first.
  [320, 568],
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
// U8 — production smoke mode (for U17). Off by default: it makes real network
// requests to third parties, which is exactly what you want against a deployed
// URL and exactly what you do not want in a hermetic CI lane.
const smoke = process.argv.includes("--smoke");
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

// U8: a request that fails is invisible in the DOM. A dead stylesheet, a 404
// asset, or a blocked font degrades the page silently and no layout assertion
// notices.
const failedRequests = [];
page.on("requestfailed", (request) => {
  const failure = request.failure()?.errorText ?? "unknown";
  // ERR_ABORTED is the browser CANCELLING a request it no longer wants, not a
  // resource that is broken. The responsive hero (U7) produces one every time
  // the viewport changes and a different srcset candidate wins: the in-flight
  // fetch for the old candidate is dropped. Treating a cancellation as a
  // failure would make the responsive-image work itself unshippable.
  if (failure.includes("ERR_ABORTED")) return;
  failedRequests.push({ url: request.url(), failure });
});
page.on("response", (response) => {
  if (response.status() >= 400) {
    failedRequests.push({ url: response.url(), failure: `HTTP ${response.status()}` });
  }
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
        "One-click mastering.",
        "Your Endgame Sound.",
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
      // U8 — quality signals that no layout metric covers.
      quality: (() => {
        const visible = (el) => {
          if (!el) return false;
          const rect = el.getBoundingClientRect();
          const style = getComputedStyle(el);
          return (
            rect.width > 1 &&
            rect.height > 1 &&
            style.display !== "none" &&
            style.visibility !== "hidden" &&
            Number(style.opacity) > 0.05
          );
        };

        // Primary acquisition action must be present and fully on screen at
        // every width — a CTA clipped by the viewport edge is the single most
        // expensive layout bug a landing page can have.
        const navCta = document.querySelector('nav a[href="#get-started"]');
        const navRect = navCta?.getBoundingClientRect();
        const heroCta = document.querySelector(
          '#top a[href="#get-started"]',
        );
        const heroRect = heroCta?.getBoundingClientRect();

        // In-page targets must resolve. A nav link to a section that was
        // renamed scrolls nowhere and looks like nothing happened.
        const inPage = Array.from(document.querySelectorAll('a[href^="#"]'));
        const deadAnchors = inPage
          .map((a) => a.getAttribute("href"))
          .filter((href) => href && href !== "#" && !document.getElementById(href.slice(1)));

        // External links must be absolute https and carry rel on new tabs.
        const external = Array.from(document.querySelectorAll('a[href^="http"]'));
        const unsafeExternal = external
          .filter(
            (a) =>
              !a.getAttribute("href").startsWith("https://") ||
              (a.getAttribute("target") === "_blank" &&
                !(a.getAttribute("rel") || "").includes("noreferrer")),
          )
          .map((a) => a.getAttribute("href"));

        // Heading order: h1 exists, is unique, and no level is skipped.
        const levels = Array.from(
          document.querySelectorAll("h1,h2,h3,h4,h5,h6"),
        ).map((h) => Number(h.tagName.slice(1)));
        const headingJumps = [];
        for (let i = 1; i < levels.length; i += 1) {
          if (levels[i] - levels[i - 1] > 1) {
            headingJumps.push(`h${levels[i - 1]} -> h${levels[i]}`);
          }
        }

        // Touch targets. WCAG 2.5.8 asks 24px; 44px is the Apple/Material
        // guidance and the one that actually feels right one-handed.
        const interactive = Array.from(
          document.querySelectorAll("a[href], button, input, select"),
        ).filter(visible);
        // WCAG 2.5.8 exempts a link that sits INLINE inside a sentence — its
        // size is set by the prose around it and padding it out would wreck the
        // paragraph. Standalone controls get no such exemption.
        const inlineInText = (el) => {
          const parent = el.parentElement;
          if (!parent) return false;
          if (!["P", "LI", "SPAN", "STRONG", "EM"].includes(parent.tagName)) return false;
          return (parent.textContent || "").trim().length >
            (el.textContent || "").trim().length + 8;
        };
        const smallTargets = interactive
          .filter((el) => !inlineInText(el))
          .filter((el) => {
            const rect = el.getBoundingClientRect();
            return Math.min(rect.width, rect.height) < 24;
          })
          .map((el) => `${el.tagName.toLowerCase()}:${(el.textContent || "").trim().slice(0, 24)}`);

        // Audit T-03: containment is measured PER AXIS and reported honestly.
        // Horizontal clipping is a layout bug at any width. Vertical (fold)
        // containment is a different question: the fixed nav CTA is the
        // all-axis acquisition guarantee; whether the HERO CTA must also sit
        // above the fold on short phones is an owner taste/spec decision
        // (docs/OWNER_INPUT_QUEUE.md, 2026-08-31) — recorded, not asserted.
        const axes = (rect) => ({
          horizontal: Boolean(
            rect && rect.left >= -1 && rect.right <= window.innerWidth + 1,
          ),
          vertical: Boolean(
            rect && rect.top >= -1 && rect.bottom <= window.innerHeight + 1,
          ),
        });
        const navAxes = axes(navRect);
        const heroAxes = axes(heroRect);

        return {
          navCtaVisible: visible(navCta),
          navCtaHorizontal: navAxes.horizontal,
          navCtaVertical: navAxes.vertical,
          navCtaAllAxes: navAxes.horizontal && navAxes.vertical,
          heroCtaVisible: visible(heroCta),
          heroCtaHorizontal: heroAxes.horizontal,
          heroCtaVertical: heroAxes.vertical,
          heroCtaAllAxes: heroAxes.horizontal && heroAxes.vertical,
          deadAnchors,
          unsafeExternal,
          headingCount: levels.length,
          h1Count: levels.filter((l) => l === 1).length,
          headingJumps,
          smallTargets,
          hasMainLandmark: Boolean(document.querySelector("main")),
          skipLinkHref:
            document.querySelector('a[href="#main"]')?.getAttribute("href") ??
            null,
          // Beta-state indexability (U8). Checked against release state below.
          robots:
            document
              .querySelector('meta[name="robots"]')
              ?.getAttribute("content") ?? null,
          title: document.title,
          description:
            document
              .querySelector('meta[name="description"]')
              ?.getAttribute("content") ?? null,
          canonical:
            document.querySelector('link[rel="canonical"]')?.getAttribute("href") ??
            null,
        };
      })(),
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
          // Platform honesty (owner, 2026-09-01): the Linux position is stated
          // once, and mobile is not mentioned at all — the one permitted
          // "iPhone and Android are not currently available" sentence was
          // removed as an odd line on a desktop product page.
          platformHonest: Boolean(
            body.textContent?.includes("no Linux build") &&
              !/iphone|android|\bios\b/i.test(body.textContent ?? ""),
          ),
        };
      })(),
    };
  }, requiredSectionIds);

  const missingSections = metrics.sections
    .filter((section) => !section.present)
    .map((section) => section.id);

  // U8 widened the title for search/share. It must still name the product
  // first — a title that leads with a tagline is unrecognisable in a tab strip.
  if (!metrics.title.startsWith("YES Master")) {
    failures.push(`${width}x${height}: title must start with "YES Master", got "${metrics.title}"`);
  }
  if (metrics.horizontalOverflow) {
    failures.push(`${width}x${height}: horizontal overflow (${metrics.scrollWidth} > ${metrics.clientWidth})`);
  }
  if (!metrics.heroPresent) {
    failures.push(`${width}x${height}: missing #top hero section`);
  }
  // The hero fills the viewport up to the composition's own height (1080px,
  // the art's native 16:9 at the 1920px cap — 2026-09-01). Past that it is
  // centred with room around it rather than zoomed to fill.
  const heroFloor = Math.min(height, 1080) * 0.9;
  if (typeof metrics.heroHeight !== "number" || metrics.heroHeight < heroFloor) {
    failures.push(`${width}x${height}: hero height is ${metrics.heroHeight}, expected at least ${Math.round(heroFloor)}`);
  }
  if (metrics.imageFit !== "cover") {
    failures.push(`${width}x${height}: hero image fit is ${metrics.imageFit}, expected cover`);
  }
  if (!metrics.heroHeadline?.includes("Your Endgame Sound")) {
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
  if (!release?.platformHonest) {
    failures.push(
      `${width}x${height}: platform-support copy must state the Linux position and say nothing about mobile`,
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

  // U8 assertions on the quality block.
  //
  // Audit T-03: these say exactly what they measure. The FIXED NAV CTA is the
  // all-axis acquisition guarantee, so it must be fully contained on both
  // axes. The HERO CTA is gated on the horizontal axis only: on short phones
  // it legitimately sits below introductory copy, and whether it must also be
  // above the fold is an owner taste/spec decision recorded in
  // docs/OWNER_INPUT_QUEUE.md (2026-08-31). Its vertical/all-axis containment
  // is still measured and lands in summary.json as evidence for that
  // decision, non-blockingly.
  const q = metrics.quality;
  if (!q.navCtaVisible || !q.navCtaAllAxes) {
    failures.push(
      `${width}x${height}: nav acquisition CTA is not fully contained on both axes`,
    );
  }
  if (!q.heroCtaVisible || !q.heroCtaHorizontal) {
    failures.push(
      `${width}x${height}: hero CTA is horizontally clipped`,
    );
  }
  if (q.deadAnchors.length > 0) {
    failures.push(
      `${width}x${height}: in-page links point at nothing: ${q.deadAnchors.join(", ")}`,
    );
  }
  if (q.unsafeExternal.length > 0) {
    failures.push(
      `${width}x${height}: external links are not https or lack rel on _blank: ${q.unsafeExternal.join(", ")}`,
    );
  }
  if (q.h1Count !== 1) {
    failures.push(`${width}x${height}: expected exactly one h1, found ${q.h1Count}`);
  }
  if (q.headingJumps.length > 0) {
    failures.push(
      `${width}x${height}: heading levels skip (${q.headingJumps.join(", ")})`,
    );
  }
  if (!q.hasMainLandmark) {
    failures.push(`${width}x${height}: no <main> landmark`);
  }
  if (q.skipLinkHref !== "#main") {
    failures.push(`${width}x${height}: no skip link targeting #main`);
  }
  // Only enforced where the pointer is a finger. On a desktop width the input
  // is a mouse, and a 20px nav link is not a defect there — applying a touch
  // rule to a mouse surface produces noise, and noisy gates get muted.
  if (width < 640 && q.smallTargets.length > 0) {
    failures.push(
      `${width}x${height}: touch targets under 24px on a phone width: ${q.smallTargets.join(", ")}`,
    );
  }
  if (!q.title || q.title.length < 12 || q.title.length > 70) {
    failures.push(`${width}x${height}: page title is missing or a bad length ("${q.title}")`);
  }
  if (!q.description || q.description.length < 60) {
    failures.push(`${width}x${height}: meta description missing or too short`);
  }
  if (!q.canonical?.startsWith("https://")) {
    failures.push(`${width}x${height}: canonical URL missing or not https`);
  }
  // Indexability follows the beta state. While no verified release exists the
  // page must not be indexed: search traffic would land on a page that cannot
  // deliver what it describes. U17 removes this when it announces.
  if (release?.state !== "verified-public" && !/noindex/.test(q.robots ?? "")) {
    failures.push(
      `${width}x${height}: release state is ${release?.state} but the page is indexable — expected a noindex robots meta until U17`,
    );
  }
  if (release?.state === "verified-public" && /noindex/.test(q.robots ?? "")) {
    failures.push(
      `${width}x${height}: a verified release is live but the page is still noindex — U17 must remove it`,
    );
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

// ---------------------------------------------------------------------------
// U8 — accessibility scan, keyboard path, and reduced motion.
//
// Scoped to two widths on purpose. axe is not free (~1-2s per run) and its
// findings are overwhelmingly width-independent; running it twelve times would
// buy repetition, not coverage. A desktop and a phone width is where the
// layout-dependent rules (contrast over the hero, target size) actually differ.
// ---------------------------------------------------------------------------
const axeRuns = [];
for (const [width, height] of [
  [1440, 900],
  [390, 844],
]) {
  await page.setViewportSize({ width, height });
  await page.goto(url, { waitUntil: "networkidle" });
  await page.addScriptTag({ content: AXE_SOURCE });
  const result = await page.evaluate(async () =>
    // WCAG 2.0/2.1 A and AA. Best-practice rules are deliberately excluded:
    // they are opinions, and a gate that fails on an opinion gets disabled.
    await window.axe.run(document, {
      runOnly: { type: "tag", values: ["wcag2a", "wcag2aa", "wcag21a", "wcag21aa"] },
    }),
  );

  const blocking = result.violations.filter((violation) =>
    ["serious", "critical"].includes(violation.impact),
  );
  axeRuns.push({
    width,
    height,
    violations: result.violations.map((violation) => ({
      id: violation.id,
      impact: violation.impact,
      help: violation.help,
      nodes: violation.nodes.length,
      targets: violation.nodes.slice(0, 4).map((node) => node.target.join(" ")),
    })),
    blockingCount: blocking.length,
    passes: result.passes.length,
  });

  for (const violation of blocking) {
    failures.push(
      `${width}x${height}: a11y ${violation.impact} — ${violation.id}: ${violation.help} ` +
        `(${violation.nodes.length} node(s): ${violation.nodes
          .slice(0, 3)
          .map((node) => node.target.join(" "))
          .join(" | ")})`,
    );
  }
}

// Keyboard-only acquisition. The skip link must be the first stop and must
// become visible when focused; a skip link that stays invisible is a phantom
// focus stop, which is worse than not having one.
await page.setViewportSize({ width: 1440, height: 900 });
await page.goto(url, { waitUntil: "networkidle" });
await page.keyboard.press("Tab");
const keyboard = await page.evaluate(() => {
  const active = document.activeElement;
  const rect = active?.getBoundingClientRect();
  const style = active ? getComputedStyle(active) : null;
  return {
    tag: active?.tagName ?? null,
    href: active?.getAttribute?.("href") ?? null,
    text: active?.textContent?.trim() ?? null,
    visible: Boolean(rect && rect.width > 1 && rect.height > 1),
    // "Visible focus" means SOMETHING changes. A UA outline counts; nothing
    // does not.
    hasFocusIndicator: Boolean(
      style &&
        (style.outlineStyle !== "none" ||
          Number.parseFloat(style.outlineWidth) > 0 ||
          style.boxShadow !== "none"),
    ),
  };
});
if (keyboard.href !== "#main") {
  failures.push(
    `keyboard: first Tab stop is ${keyboard.tag}[href=${keyboard.href}], expected the skip link`,
  );
}
if (!keyboard.visible) {
  failures.push("keyboard: the skip link is not visible when focused");
}
if (!keyboard.hasFocusIndicator) {
  failures.push("keyboard: focused skip link has no visible focus indicator");
}

// Tab all the way to the acquisition action without losing focus to nothing.
let reachedCta = false;
let lostFocus = null;
for (let i = 0; i < 40 && !reachedCta; i += 1) {
  const state = await page.evaluate(() => {
    const active = document.activeElement;
    return {
      href: active?.getAttribute?.("href") ?? null,
      isBody: active === document.body,
      tag: active?.tagName ?? null,
    };
  });
  if (state.isBody && i > 0) {
    lostFocus = i;
    break;
  }
  if (state.href === "#get-started") reachedCta = true;
  else await page.keyboard.press("Tab");
}
if (!reachedCta) {
  failures.push(
    `keyboard: never reached the acquisition CTA within 40 tab stops${lostFocus ? ` (focus fell to <body> at stop ${lostFocus})` : ""}`,
  );
}

// Reduced motion: the page must still say everything it says. Content that
// only exists once an animation has run is content some visitors never get.
const reducedContext = await browser.newContext({
  viewport: { width: 1440, height: 900 },
  reducedMotion: "reduce",
});
const reducedPage = await reducedContext.newPage();
await reducedPage.goto(url, { waitUntil: "networkidle" });
const reducedText = await reducedPage.evaluate(() => document.body.innerText);
for (const required of [
  "Finished mix in",
  "It reads the track before it touches it",
  "A record, not a folder of files",
  "What you are actually agreeing to",
  "The download is not open",
]) {
  if (!reducedText.includes(required)) {
    failures.push(`reduced-motion: "${required}" is missing with motion disabled`);
  }
}
await reducedPage.screenshot({
  path: path.join(outDir, "1440x900-reduced-motion.png"),
});
await reducedContext.close();

// 200% zoom. WCAG 1.4.4 asks that content stay usable at 200%, and the usual
// answer is "recorded manual check" — which nobody repeats. Zooming the page
// and re-measuring overflow is mechanical, so it runs every time.
await page.setViewportSize({ width: 1440, height: 900 });
await page.goto(url, { waitUntil: "networkidle" });
await page.evaluate(() => {
  document.documentElement.style.zoom = "200%";
});
await page.waitForTimeout(400);
const zoomState = await page.evaluate(() => {
  const doc = document.documentElement;
  const cta = document.querySelector('nav a[href="#get-started"]');
  const rect = cta?.getBoundingClientRect();
  return {
    horizontalOverflow:
      Math.max(doc.scrollWidth, document.body.scrollWidth) > doc.clientWidth + 1,
    scrollWidth: Math.max(doc.scrollWidth, document.body.scrollWidth),
    clientWidth: doc.clientWidth,
    ctaOnScreen: Boolean(rect && rect.left >= -1 && rect.right <= window.innerWidth + 1),
  };
});
if (zoomState.horizontalOverflow) {
  failures.push(
    `200% zoom: horizontal overflow (${zoomState.scrollWidth} > ${zoomState.clientWidth})`,
  );
}
if (!zoomState.ctaOnScreen) {
  failures.push("200% zoom: the acquisition CTA is pushed off screen");
}
await page.evaluate(() => {
  document.documentElement.style.zoom = "";
});

// Production smoke (U17): every outbound link must actually resolve. A release
// or feedback URL that 404s is invisible locally — the markup is perfect and
// the destination is gone.
const linkChecks = [];
if (smoke) {
  await page.goto(url, { waitUntil: "networkidle" });
  const outbound = await page.evaluate(() =>
    [
      ...new Set(
        Array.from(document.querySelectorAll('a[href^="https://"]')).map((a) =>
          a.getAttribute("href"),
        ),
      ),
    ],
  );
  for (const target of outbound) {
    let status = 0;
    let error = null;
    try {
      const response = await fetch(target, { method: "GET", redirect: "follow" });
      status = response.status;
    } catch (cause) {
      error = String(cause?.message ?? cause);
    }
    linkChecks.push({ url: target, status, error });
    if (error || status >= 400) {
      failures.push(
        `smoke: outbound link is dead — ${target} (${error ?? `HTTP ${status}`})`,
      );
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
if (failedRequests.length > 0) {
  failures.push(`failed requests: ${JSON.stringify(failedRequests)}`);
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
  axe: axeRuns,
  keyboard,
  zoom200: zoomState,
  smoke,
  linkChecks,
  matrix: records,
  anchors: anchorRecords,
  consoleMessages,
  pageErrors,
  failedRequests,
  failures,
};

await writeFile(path.join(outDir, "summary.json"), `${JSON.stringify(summary, null, 2)}\n`);

if (failures.length > 0) {
  console.error(`Landing responsive verification failed. Evidence: ${outDir}`);
  for (const failure of failures) console.error(`- ${failure}`);
  process.exit(1);
}

console.log(`Landing responsive verification passed. Evidence: ${outDir}`);
