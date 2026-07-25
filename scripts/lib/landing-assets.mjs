// -----------------------------------------------------------------------------
// U7 — the marketing-proof contract, shared by the capture and verify scripts.
//
// The problem this solves: a screenshot on a marketing page has no expiry. The
// two desktop UI images on the landing page were captured 2026-06-28 and then
// U9, U10 and U11 changed the console underneath them — accessible names,
// warning ownership, a modal review gate, the sticky export cluster, the title
// clamp. Nothing failed. Nothing could fail. The page just quietly showed a
// build that no longer exists.
//
// So "is this proof current?" becomes a mechanical question with a mechanical
// answer: a digest over an EXPLICIT, COMMITTED FILE LIST of every input that
// can change what a capture looks like. Change one of them without recapturing
// and the verify lane fails.
//
// Why a file list and not `git HEAD`: HEAD advances constantly for reasons that
// cannot affect a screenshot — docs, tests, the Rust crate, CI config. Tying
// freshness to HEAD would cry wolf on every commit until nobody believed it.
// Tying it to content means an unrelated commit leaves valid proof valid, which
// is exactly what the acceptance asks for.
// -----------------------------------------------------------------------------
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import path from "node:path";

export const REPO_ROOT = path.resolve(
  path.dirname(new URL(import.meta.url).pathname),
  "..",
  "..",
);

/**
 * Every committed file that can alter how a captured surface looks.
 *
 * ADDING A CAPTURED SURFACE MEANS ADDING ITS INPUTS HERE. That is deliberate
 * friction: the list is the whole mechanism, and a capture whose inputs are not
 * listed is proof that can go stale silently — the exact failure this replaces.
 */
export const CAPTURE_INPUTS = [
  // The shell and every style rule. U9, U10 and U11 all landed here.
  "src/App.tsx",
  "src/App.css",
  // Every non-test component. The list is exhaustive rather than "the ones I
  // think matter", because deciding case by case is how a surface silently
  // stops being covered — and a component that renders nothing visible costs
  // one line here and nothing at runtime.
  "src/components/AdvancedPanel.tsx",
  "src/components/AlbumExportReceipt.tsx",
  "src/components/AlbumPanel.tsx",
  "src/components/AnalysisOrb.tsx",
  "src/components/ChromeDialog.tsx",
  "src/components/EmptyState.tsx",
  "src/components/ExportReceiptCard.tsx",
  "src/components/FirstRunOverlay.tsx",
  "src/components/HintChip.tsx",
  "src/components/Knob.tsx",
  "src/components/PresetIcon.tsx",
  "src/components/RightRail.tsx",
  "src/components/SettingsGroup.tsx",
  "src/components/SignalChain.tsx",
  "src/components/StandardView.tsx",
  "src/components/Toast.tsx",
  "src/components/VisualEqPanel.tsx",
  "src/components/Waveform.tsx",
  "src/components/WaveformDbScale.tsx",
  "src/components/fields.tsx",
  // The deterministic state the captures are taken against. A scenario edit
  // changes what is on screen just as surely as a component edit.
  "src/lib/preview-mock.ts",
  // The capture procedure itself: viewport, driving, and settle rules decide
  // the image as much as the app does.
  "scripts/capture-landing-assets.mjs",
  "scripts/lib/landing-assets.mjs",
];

/** Eager landing imagery budget. Optional audio proof is excluded by policy. */
export const EAGER_BUDGET_BYTES = 1_500_000;

export const MANIFEST_PATH = "src/assets/landing/manifest.json";

const LAPTOP = { width: 1440, height: 1000 };

/**
 * Canonical capture scenarios.
 *
 * `published: true` means the asset is imported by the landing page and is
 * therefore subject to the freshness and budget gates. A capture that nothing
 * imports is dead weight in the repo, so there are none.
 */
export const SHOTS = [
  {
    id: "desktop-standard-ui",
    scenario: "clean",
    view: "standard",
    viewport: LAPTOP,
    settle: "ready",
    published: true,
    alt: "YES Master Standard view",
  },
  {
    id: "desktop-advanced-ui",
    scenario: "clean",
    view: "advanced",
    viewport: LAPTOP,
    settle: "ready",
    published: true,
    alt: "YES Master Advanced view",
  },
  {
    id: "desktop-album-ui",
    scenario: "album-4",
    view: "advanced",
    viewport: LAPTOP,
    settle: "album",
    published: true,
    alt: "YES Master Album Master view with four tracks",
  },
];

export function sha256(buffer) {
  return createHash("sha256").update(buffer).digest("hex");
}

/**
 * Content digest over CAPTURE_INPUTS. Order-independent by construction, so a
 * reordered list does not read as a change.
 */
export function captureInputDigest(root = REPO_ROOT) {
  const hash = createHash("sha256");
  for (const relative of [...CAPTURE_INPUTS].sort()) {
    let contents;
    try {
      contents = readFileSync(path.join(root, relative));
    } catch {
      // A listed input that no longer exists means a captured surface was
      // renamed or deleted. Silently skipping it would shrink the covered set
      // without anyone deciding to, so this is loud.
      throw new Error(
        `capture input "${relative}" is listed in CAPTURE_INPUTS but does not exist. ` +
          "If the file moved, update the list; if the surface is gone, remove it deliberately.",
      );
    }
    // Normalize line endings: this repo is edited from both Windows and WSL,
    // and a CRLF round-trip is not a visual change.
    const normalized = Buffer.from(
      contents.toString("utf8").replace(/\r\n/g, "\n"),
      "utf8",
    );
    hash.update(relative).update("\0").update(sha256(normalized)).update("\n");
  }
  return hash.digest("hex");
}

/** Minimal PNG header read — avoids a dependency for two integers. */
export function pngDimensions(buffer) {
  const isPng =
    buffer.length > 24 &&
    buffer.readUInt32BE(0) === 0x89504e47 &&
    buffer.readUInt32BE(4) === 0x0d0a1a0a;
  if (!isPng) return null;
  return { width: buffer.readUInt32BE(16), height: buffer.readUInt32BE(20) };
}
