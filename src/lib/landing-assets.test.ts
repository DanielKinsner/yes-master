import { execFileSync } from "node:child_process";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";

import { describe, expect, it } from "vitest";

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
    const head = execFileSync("git", ["rev-parse", "HEAD"], {
      cwd: root,
      encoding: "utf8",
    }).trim();
    const script = readFileSync(
      resolve(root, "scripts/verify-landing-assets.mjs"),
      "utf8",
    );
    expect(script).toContain("captureInputDigest");
    expect(script).not.toMatch(/rev-parse/);
    // sourceCommit is provenance only. It is allowed to be stale relative to
    // HEAD, and this asserts nothing compares them.
    expect(manifest.sourceCommit).toMatch(/^[a-f0-9]{7,40}$/);
    expect(typeof head).toBe("string");
  });

  it("lazy-loads every below-fold capture and declares its intrinsic size", () => {
    const proof = landingSource("ProofDeck.tsx") + landingSource("AlbumProof.tsx");
    const imgTags = proof.match(/<img[\s\S]*?\/>/g) ?? [];
    expect(imgTags.length).toBe(3);
    for (const tag of imgTags) {
      expect(tag, "below-fold proof must be lazy").toContain('loading="lazy"');
      // Without width/height the image has no reserved box and its arrival
      // shoves the page around — the thing lazy loading is supposed to avoid
      // making worse.
      expect(tag).toMatch(/width=\{1440\}/);
      expect(tag).toMatch(/height=\{1000\}/);
      expect(tag).toMatch(/alt="[^"]+"/);
    }
  });

  it("serves a smaller hero to small screens", () => {
    const hero = landingSource("Hero.tsx");
    expect(hero).toContain("srcSet");
    expect(hero).toContain("1280w");
    expect(hero).toContain("2560w");
  });

  it("keeps no mobile UI image on the desktop-beta page", () => {
    // R7. Standing check, not a one-off cleanup.
    const everything = [
      "Hero.tsx",
      "ProofDeck.tsx",
      "CrossPlatform.tsx",
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
