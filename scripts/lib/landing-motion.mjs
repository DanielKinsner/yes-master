import path from "node:path";

// Observe actual browser animations: finite entrances, no scroll replay,
// immediate opt-out, and readable content with the enhancement unavailable.
export async function verifyStudioMotion(browser, url, outDir, failures) {
  const records = [];
  const targets = [
    ".studio-front-laptop",
    ".studio-tape-scene img",
    ".studio-listening-image",
  ];
  for (const width of [1440, 390]) {
    const context = await browser.newContext({
      viewport: { width, height: 900 },
      reducedMotion: "no-preference",
    });
    const page = await context.newPage();
    await page.addInitScript(() => {
      window.studioMotionStarts = [];
      document.addEventListener("animationstart", (event) => {
        if (event.animationName.startsWith("studio-")) {
          window.studioMotionStarts.push({
            name: event.animationName,
            reveal: event.target.hasAttribute("data-studio-reveal"),
          });
        }
      });
    });
    await page.goto(url, { waitUntil: "networkidle" });
    await page.evaluate(() =>
      Promise.all(document.getAnimations().map((a) => a.finished)),
    );
    const heroStarts = await page.evaluate(
      () => window.studioMotionStarts.filter((e) => !e.reveal).length,
    );
    if (heroStarts !== 2)
      failures.push(
        `motion ${width}: expected the two finite hero entrances, saw ${heroStarts}`,
      );

    for (const selector of targets) {
      const target = page.locator(selector);
      await target.evaluate((el) =>
        el.scrollIntoView({ behavior: "instant", block: "center" }),
      );
      await page.waitForFunction(
        (selector) =>
          document
            .querySelector(selector)
            .classList.contains("studio-reveal-enter"),
        selector,
      );
      const animation = await target.evaluate(async (el) => {
        const animations = el.getAnimations();
        const finite = animations.every(
          (a) => a.effect.getTiming().iterations === 1,
        );
        const visible = Number(getComputedStyle(el).opacity) >= 0.79;
        await Promise.all(animations.map((a) => a.finished));
        return { finite, visible };
      });
      if (!animation.finite || !animation.visible)
        failures.push(
          `motion ${width}: ${selector} loops or hides its artwork`,
        );
    }
    const starts = await page.evaluate(() => window.studioMotionStarts.length);
    await page.evaluate(() => scrollTo({ top: 0, behavior: "instant" }));
    for (const selector of targets)
      await page
        .locator(selector)
        .evaluate((el) =>
          el.scrollIntoView({ behavior: "instant", block: "center" }),
        );
    const idle = await page.evaluate(() => ({
      starts: window.studioMotionStarts.length,
      active: document.getAnimations().length,
    }));
    if (starts !== 5 || idle.starts !== starts || idle.active)
      failures.push(
        `motion ${width}: artwork replays or keeps animating at rest`,
      );

    // An OS preference change must also stop an animation already in flight.
    await page.evaluate(() => scrollTo({ top: 0, behavior: "instant" }));
    await page.reload({ waitUntil: "networkidle" });
    await page
      .locator(targets[1])
      .evaluate((el) =>
        el.scrollIntoView({ behavior: "instant", block: "center" }),
      );
    await page.waitForFunction(
      () =>
        document.querySelector(".studio-tape-scene img").getAnimations()
          .length > 0,
    );
    await page.emulateMedia({ reducedMotion: "reduce" });
    await page.waitForFunction(
      () =>
        document.getAnimations().length === 0 &&
        !document.querySelector(".studio-reveal-enter"),
    );
    for (const selector of targets) {
      await page
        .locator(selector)
        .evaluate((el) =>
          el.scrollIntoView({ behavior: "instant", block: "center" }),
        );
      if (
        await page
          .locator(selector)
          .evaluate(
            (el) =>
              getComputedStyle(el).opacity !== "1" ||
              el.getAnimations().length > 0,
          )
      ) {
        failures.push(
          `motion ${width}: ${selector} did not honor reduced motion`,
        );
      }
    }
    await page.evaluate(() => scrollTo({ top: 0, behavior: "instant" }));
    await page.screenshot({
      path: path.join(outDir, `${width}-motion-disabled.png`),
    });

    // Disable just the observer, keeping the React app available. It must not
    // leave photos hidden or prevent the primary navigation from working.
    await page.addInitScript(() => {
      window.IntersectionObserver = undefined;
    });
    await page.emulateMedia({ reducedMotion: "no-preference" });
    await page.reload({ waitUntil: "networkidle" });
    for (const selector of targets) {
      if (
        await page
          .locator(selector)
          .evaluate((el) => getComputedStyle(el).opacity !== "1")
      )
        failures.push(`motion ${width}: observer fallback hides ${selector}`);
    }
    await page.locator('nav a[href="#get-started"]').click();
    if (new URL(page.url()).hash !== "#get-started")
      failures.push(`motion ${width}: observer fallback blocks navigation`);
    records.push({
      width,
      heroStarts,
      revealStarts: starts - heroStarts,
      idle: idle.active === 0,
      reducedMotion: true,
      observerFallback: true,
    });
    await context.close();
  }
  return records;
}
