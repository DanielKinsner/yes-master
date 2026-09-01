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
      expect(capture.sha256, `${capture.id} has no hash`).toMatch(/^[a-f0-9]{64}$/);
      expect(capture.capturedAt).toMatch(/^\d{4}-\d{2}-\d{2}$/);
      expect(capture.session, `${capture.id} names no session`).toBeTruthy();
      expect(capture.alt, `${capture.id} has no alt`).toBeTruthy();
    }
    expect(manifest.ownerCapturesNote).toMatch(/nothing mechanical proves them current/i);
  });

  it("lazy-loads every below-fold plate and declares its intrinsic size", () => {
    const proof = landingSource("ProofDeck.tsx") + landingSource("AlbumProof.tsx");
    const imgTags = proof.match(/<img[\s\S]*?\/>/g) ?? [];
    expect(imgTags.length).toBe(3);
    const owners = manifest.ownerCaptures as Array<{
      file: string;
      dimensions: { width: number; height: number };
    }>;
    for (const tag of imgTags) {
      expect(tag, "below-fold proof must be lazy").toContain('loading="lazy"');
      expect(tag).toMatch(/alt="[^"]+"/);
      // Without width/height the image has no reserved box and its arrival
      // shoves the page around — the thing lazy loading is supposed to avoid
      // making worse. The reserved box must be the file's real size.
      const width = Number(tag.match(/width=\{(\d+)\}/)?.[1]);
      const height = Number(tag.match(/height=\{(\d+)\}/)?.[1]);
      expect(
        owners.some((o) => o.dimensions.width === width && o.dimensions.height === height),
        `no owner capture is ${width}x${height}`,
      ).toBe(true);
    }
    // Nothing in the manifest is dead weight: every owner capture is on the page.
    for (const capture of owners) {
      expect(proof).toContain(capture.file.replace("src/assets/landing/", "../assets/landing/"));
    }
  });

  it("serves a smaller hero to small screens", () => {
    const hero = landingSource("Hero.tsx");
    // Two variants: a 1280w one for phones and the full-width master. The
    // master's width follows the source art (2560 for the original photo,
    // 1672 for the 2026-08-18 console render), so assert the shape — a phone
    // variant plus a strictly wider one — rather than a magic number.
    expect(hero).toContain("srcSet");
    expect(hero).toContain("1280w");
    const widths = [...hero.matchAll(/(\d{3,4})w\b/g)].map((m) => Number(m[1]));
    expect(Math.max(...widths)).toBeGreaterThan(1280);
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
