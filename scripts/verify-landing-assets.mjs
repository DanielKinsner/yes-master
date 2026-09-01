#!/usr/bin/env node
// -----------------------------------------------------------------------------
// U7 — the gate. Fails when the landing page's marketing proof stops being true.
//
//   npm run verify:landing-assets
//
// Four ways proof goes wrong, all of them silent without this:
//
//   1. STALE     — a captured surface changed and nobody recaptured.
//   2. MISSING   — the manifest promises an asset that is not on disk.
//   3. ALTERED   — the file on disk is not the file that was captured.
//   4. HEAVY     — eager imagery grew past the budget one import at a time.
//
// This runs offline in milliseconds. It never launches a browser: it compares
// committed bytes against a committed manifest.
// -----------------------------------------------------------------------------
import { readFileSync } from "node:fs";
import path from "node:path";

import {
  EAGER_BUDGET_BYTES,
  MANIFEST_PATH,
  REPO_ROOT,
  SHOTS,
  captureInputDigest,
  imageDimensions,
  pngDimensions,
  sha256,
} from "./lib/landing-assets.mjs";
import { readdirSync } from "node:fs";

const failures = [];

function read(relative) {
  return readFileSync(path.join(REPO_ROOT, relative));
}

let manifest;
try {
  manifest = JSON.parse(read(MANIFEST_PATH).toString("utf8"));
} catch (error) {
  console.error(
    `Landing asset verification failed: cannot read ${MANIFEST_PATH} (${error.message}).\n` +
      "Run `npm run capture:landing` to produce it.",
  );
  process.exit(1);
}

// 1. STALE. Content digest, not HEAD — an unrelated commit must not invalidate
//    proof that is still accurate, or the gate becomes noise people mute.
const digest = captureInputDigest();
if (manifest.captureInputDigest !== digest) {
  failures.push(
    "capture inputs changed since the last capture " +
      `(manifest ${String(manifest.captureInputDigest).slice(0, 12)}…, current ${digest.slice(0, 12)}…). ` +
      "A surface the landing page shows has been edited. Run `npm run capture:landing`.",
  );
}

// Every canonical published shot must be in the manifest. A shot that quietly
// disappears is proof the page stops showing without anyone deciding to.
const published = SHOTS.filter((shot) => shot.published);
for (const shot of published) {
  if (!manifest.assets?.some((asset) => asset.id === shot.id)) {
    failures.push(`manifest is missing the required capture "${shot.id}"`);
  }
}

let eagerBytes = 0;
for (const asset of manifest.assets ?? []) {
  let bytes;
  try {
    // 2. MISSING.
    bytes = read(asset.file);
  } catch {
    failures.push(`${asset.id}: ${asset.file} is in the manifest but not on disk`);
    continue;
  }

  // 3. ALTERED. A hand-edited or re-exported image is not the image that was
  //    captured, whatever its filename says.
  const actual = sha256(bytes);
  if (actual !== asset.sha256) {
    failures.push(
      `${asset.id}: file content does not match the manifest ` +
        `(expected ${asset.sha256.slice(0, 12)}…, found ${actual.slice(0, 12)}…)`,
    );
  }
  if (bytes.length !== asset.bytes) {
    failures.push(
      `${asset.id}: expected ${asset.bytes} bytes, found ${bytes.length}`,
    );
  }

  const dimensions = pngDimensions(bytes);
  if (
    dimensions?.width !== asset.dimensions?.width ||
    dimensions?.height !== asset.dimensions?.height
  ) {
    failures.push(
      `${asset.id}: expected ${asset.dimensions?.width}x${asset.dimensions?.height}, ` +
        `found ${dimensions?.width}x${dimensions?.height}`,
    );
  }

  if (asset.loading === "eager") eagerBytes += bytes.length;
}

// 5. OWNER CAPTURES (2026-09-01). Real-session screenshots the owner chose as
//    the page's plates. Nothing mechanical can prove them current, so the gate
//    proves what it can: the bytes are the bytes that were reviewed, the size
//    the page reserves is the size on disk, and every one is actually imported
//    (a listed capture nothing shows is dead weight, exactly like a shot).
const landingSources = readdirSync(path.join(REPO_ROOT, "src/landing"))
  .filter((file) => file.endsWith(".tsx") && !file.includes(".test."))
  .map((file) => read(`src/landing/${file}`).toString("utf8"))
  .join("\n");
for (const capture of manifest.ownerCaptures ?? []) {
  for (const key of ["id", "file", "sha256", "bytes", "dimensions", "capturedAt", "session", "alt"]) {
    if (capture[key] === undefined || capture[key] === "") {
      failures.push(`owner capture ${capture.id ?? "?"}: missing "${key}"`);
    }
  }
  let bytes;
  try {
    bytes = read(capture.file);
  } catch {
    failures.push(`${capture.id}: ${capture.file} is in the manifest but not on disk`);
    continue;
  }
  const actual = sha256(bytes);
  if (actual !== capture.sha256) {
    failures.push(
      `${capture.id}: file content does not match the manifest ` +
        `(expected ${String(capture.sha256).slice(0, 12)}…, found ${actual.slice(0, 12)}…)`,
    );
  }
  if (bytes.length !== capture.bytes) {
    failures.push(`${capture.id}: expected ${capture.bytes} bytes, found ${bytes.length}`);
  }
  const dimensions = imageDimensions(bytes);
  if (
    dimensions?.width !== capture.dimensions?.width ||
    dimensions?.height !== capture.dimensions?.height
  ) {
    failures.push(
      `${capture.id}: expected ${capture.dimensions?.width}x${capture.dimensions?.height}, ` +
        `found ${dimensions?.width}x${dimensions?.height}`,
    );
  }
  const importPath = capture.file.replace(/^src\/assets\/landing\//, "../assets/landing/");
  if (!landingSources.includes(importPath)) {
    failures.push(`${capture.id}: ${capture.file} is in the manifest but no landing component imports it`);
  }
  if (capture.phoneFile) {
    try {
      read(capture.phoneFile);
    } catch {
      failures.push(`${capture.id}: phone variant ${capture.phoneFile} is missing`);
    }
  }
}

// 4. HEAVY. The budget covers everything the browser fetches before the visitor
//    does anything — captures plus the hero art that is not a capture.
const EAGER_UNCAPTURED = [
  "src/assets/landing/hero-control-room-studio.jpg",
  "src/assets/landing/icon-realtime.png",
  "src/assets/landing/icon-local-first.png",
  "src/assets/landing/icon-release-ready.png",
  "src/assets/landing/yes-master-icon.png",
];
const eagerDetail = [];
for (const file of EAGER_UNCAPTURED) {
  try {
    const bytes = read(file);
    eagerBytes += bytes.length;
    eagerDetail.push(`${path.basename(file)} ${Math.round(bytes.length / 1024)} KB`);
  } catch {
    failures.push(`${file} is listed as eager landing imagery but is missing`);
  }
}

if (eagerBytes > EAGER_BUDGET_BYTES) {
  failures.push(
    `eager landing imagery is ${Math.round(eagerBytes / 1024)} KB, over the ` +
      `${Math.round(EAGER_BUDGET_BYTES / 1024)} KB budget by ` +
      `${Math.round((eagerBytes - EAGER_BUDGET_BYTES) / 1024)} KB. ` +
      "Lazy-load below-fold proof or shrink the hero art; do not raise the budget " +
      "without a documented owner exception.",
  );
}

// No mobile UI image may stand as current desktop-beta proof (R7). This is a
// standing check, not a one-off cleanup: it fails if one is imported again.
const landingSource = [
  "src/landing/Hero.tsx",
  "src/landing/ProofDeck.tsx",
  "src/landing/AlbumProof.tsx",
  "src/landing/SoundCharacter.tsx",
  "src/landing/BetaTerms.tsx",
  "src/landing/Workflow.tsx",
  "src/landing/FinalCTA.tsx",
  "src/landing/Nav.tsx",
]
  .map((file) => {
    try {
      return read(file).toString("utf8");
    } catch {
      return "";
    }
  })
  .join("\n");
for (const banned of ["iphone", "android"]) {
  if (new RegExp(`assets/landing/[^"']*${banned}`, "i").test(landingSource)) {
    failures.push(
      `a ${banned} UI image is imported by the landing page; R7 forbids mobile imagery as desktop-beta proof`,
    );
  }
}

if (failures.length > 0) {
  console.error("Landing asset verification FAILED:");
  for (const failure of failures) console.error(`- ${failure}`);
  process.exit(1);
}

console.log(
  `Landing asset verification passed. ${manifest.assets.length} captures bound to ` +
    `digest ${digest.slice(0, 12)}… (source commit ${String(manifest.sourceCommit).slice(0, 7)}). ` +
    `Eager imagery ${Math.round(eagerBytes / 1024)} KB of ` +
    `${Math.round(EAGER_BUDGET_BYTES / 1024)} KB [${eagerDetail.join(", ")}].`,
);
