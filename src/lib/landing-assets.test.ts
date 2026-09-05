import { execFileSync } from "node:child_process";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";

import { describe, expect, it } from "vitest";
import { renderToStaticMarkup } from "react-dom/server";
import { createElement } from "react";
import LandingPage from "../LandingPage";

// ---------------------------------------------------------------------------
// U7 — marketing proof contract.
//
// `npm run verify:landing-assets` is the real gate; these tests exist so the
// contract is also enforced by `npm test`, which is what runs on every commit
// and in every agent session. A gate nobody remembers to run is a gate that
// fails open.
// ---------------------------------------------------------------------------

const root = process.cwd();
const manifest = JSON.parse(
  readFileSync(resolve(root, "src/assets/landing/manifest.json"), "utf8"),
);

function landingSource(file: string): string {
  return readFileSync(resolve(root, "src/landing", file), "utf8");
}

describe("landing marketing proof (U7)", () => {
  it("passes the standalone asset gate", () => {
    // Runs the real script rather than reimplementing its logic here, so the
    // two cannot disagree about what "current" means.
    const output = execFileSync(
      process.execPath,
      ["scripts/verify-landing-assets.mjs"],
      { cwd: root, encoding: "utf8" },
    );
    expect(output).toContain("Landing asset verification passed");
  });

  it("binds every capture to the inputs it was taken from", () => {
    expect(manifest.captureInputDigest).toMatch(/^[a-f0-9]{64}$/);
    expect(manifest.assets.length).toBeGreaterThanOrEqual(3);
    for (const asset of manifest.assets) {
      expect(asset.sha256, `${asset.id} has no hash`).toMatch(/^[a-f0-9]{64}$/);
      expect(asset.scenario, `${asset.id} has no scenario`).toBeTruthy();
      expect(asset.dimensions.width).toBe(1440);
      expect(asset.dimensions.height).toBe(1000);
    }
  });

  it("labels captures as synthetic evidence", () => {
    // The same discipline as every other lane: a browser capture proves layout
    // and nothing about installation, real audio, or how anything sounds.
    expect(manifest.evidenceLayer).toBe("browser-headless");
    expect(manifest.note).toMatch(/marketing evidence/i);
    expect(manifest.note).toMatch(/installation, real audio/i);
  });

  it("does not use HEAD as the freshness key", () => {
    // Freshness is content-derived on purpose: an unrelated commit must leave
    // valid proof valid, or the gate becomes noise that gets muted.
    const script = readFileSync(
      resolve(root, "scripts/verify-landing-assets.mjs"),
      "utf8",
    );
    expect(script).toContain("captureInputDigest");
    expect(script).not.toMatch(/rev-parse/);
    // sourceCommit is provenance only. It is allowed to be stale relative to
    // HEAD, and this asserts nothing compares them.
    expect(manifest.sourceCommit).toMatch(/^[a-f0-9]{7,40}$/);
  });

  it("binds every owner capture by hash, date and session", () => {
    // 2026-09-01: the page's plates are the owner's real-session screenshots.
    // They cannot be proven current by content, so they are bound by what can
    // be checked — and each one must be the one that was reviewed.
    const owners = manifest.ownerCaptures as Array<Record<string, unknown>>;
    expect(owners.length).toBe(3);
    for (const capture of owners) {
      expect(capture.sha256, `${capture.id} has no hash`).toMatch(
        /^[a-f0-9]{64}$/,
      );
      expect(capture.capturedAt ?? capture.receivedAt).toMatch(
        /^\d{4}-\d{2}-\d{2}$/,
      );
      expect(capture.session, `${capture.id} names no session`).toBeTruthy();
      expect(capture.alt, `${capture.id} has no alt`).toBeTruthy();
    }
    expect(manifest.ownerCapturesNote).toMatch(
      /nothing mechanical proves them current/i,
    );
  });

  it("lazy-loads the published proof with intrinsic sizes and alt text", () => {
    const host = document.createElement("div");
    host.innerHTML = renderToStaticMarkup(createElement(LandingPage));
    const images = Array.from(
      host.querySelectorAll<HTMLImageElement>(".studio-capture-button img"),
    );
    expect(images.length).toBeGreaterThanOrEqual(6);
    for (const image of images) {
      expect(image.getAttribute("loading")).toBe("lazy");
      expect(image.alt.length).toBeGreaterThan(5);
      expect(Number(image.getAttribute("width"))).toBeGreaterThan(0);
      expect(Number(image.getAttribute("height"))).toBeGreaterThan(0);
    }
  });

  it("uses separate studio and laptop layers within the eager image budget", () => {
    const host = document.createElement("div");
    host.innerHTML = renderToStaticMarkup(createElement(LandingPage));
    const hero = host.querySelector("#top")!;
    expect(hero.querySelectorAll("img")).toHaveLength(2);
    expect(hero.querySelector("h1")?.textContent).toContain(
      "One-click mastering.",
    );
    expect(
      hero.querySelector("img.studio-device")?.getAttribute("src"),
    ).toContain("hero-device-standard");
    // A generated blank chassis must never replace the real product capture.
    expect(
      host.querySelector(".studio-laptop-chassis")?.getAttribute("src"),
    ).toContain("advanced-laptop-front");
    expect(
      host.querySelector(".studio-laptop-screen")?.getAttribute("src"),
    ).toContain("advanced-ui.png");
    const eager = manifest.studioArtwork.filter(
      (a: { loading: string }) => a.loading === "eager",
    );
    expect(
      eager.reduce((total: number, a: { bytes: number }) => total + a.bytes, 0),
    ).toBeLessThan(1_500_000);
  });

  it("keeps no mobile UI image on the desktop-beta page", () => {
    // R7. Standing check, not a one-off cleanup.
    const everything = [
      "Hero.tsx",
      "ProofDeck.tsx",
      "AlbumProof.tsx",
      "SoundCharacter.tsx",
      "BetaTerms.tsx",
      "Workflow.tsx",
      "FinalCTA.tsx",
      "Nav.tsx",
    ]
      .map(landingSource)
      .join("\n");
    expect(everything).not.toMatch(/assets\/landing\/[^"']*iphone/i);
    expect(everything).not.toMatch(/assets\/landing\/[^"']*android/i);
  });
});
