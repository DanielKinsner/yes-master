import { readFileSync } from "node:fs";
import { resolve } from "node:path";

import { describe, expect, it } from "vitest";

import { resolveRelease } from "./release-config";

// ---------------------------------------------------------------------------
// U8 — document head and beta-state indexability.
//
// The headless lane checks these against a rendered page. These run in
// `npm test`, with no browser, so a bad edit to index.html fails in the same
// second it is made rather than at the end of a 3-minute browser lane.
// ---------------------------------------------------------------------------

const html = readFileSync(resolve(process.cwd(), "index.html"), "utf8");

function meta(name: string): string | null {
  const byName = html.match(
    new RegExp(`<meta\\s+name="${name}"\\s+content="([^"]*)"`, "i"),
  );
  if (byName) return byName[1];
  const byProperty = html.match(
    new RegExp(`<meta\\s+property="${name}"\\s+content="([^"]*)"`, "i"),
  );
  return byProperty ? byProperty[1] : null;
}

describe("landing document head (U8)", () => {
  it("names the product first in a title of usable length", () => {
    const title = html.match(/<title>([^<]*)<\/title>/)?.[1] ?? "";
    expect(title.startsWith("YES Master")).toBe(true);
    expect(title.length).toBeGreaterThan(12);
    // Search results truncate past ~60-70 characters, and a tab strip shows
    // far less. A title that only makes sense in full is a title nobody reads.
    expect(title.length).toBeLessThanOrEqual(70);
  });

  it("carries a description and a canonical URL", () => {
    expect(meta("description")?.length ?? 0).toBeGreaterThan(60);
    expect(html).toMatch(/<link rel="canonical" href="https:\/\/[^"]+"/);
  });

  it("carries coherent share metadata", () => {
    for (const property of ["og:type", "og:title", "og:description", "og:url", "og:image"]) {
      expect(meta(property), `missing ${property}`).toBeTruthy();
    }
    expect(meta("twitter:card")).toBe("summary_large_image");
    // The share image is served from /public so social crawlers can fetch it
    // without it ever entering the page's eager image budget (U7).
    expect(meta("og:image")).toContain("/og-card.jpg");
    expect(meta("og:title")).toContain("YES Master");
  });

  it("stays out of search until there is something to download", () => {
    // Mechanically tied to release state, not left as a note somebody has to
    // remember. Indexing a page whose download is closed sends people to a
    // page that cannot give them the thing it describes.
    const release = resolveRelease();
    const robots = meta("robots") ?? "";
    if (release.available) {
      expect(
        robots,
        "a verified release is live — U17 must remove the noindex",
      ).not.toMatch(/noindex/);
    } else {
      expect(
        robots,
        "no verified release exists — the page must not be indexable yet",
      ).toMatch(/noindex/);
    }
  });

  it("keeps the favicon wired", () => {
    expect(html).toMatch(/<link rel="icon"[^>]+href="[^"]+"/);
  });
});
