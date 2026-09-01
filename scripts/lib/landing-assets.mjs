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
import { fileURLToPath } from "node:url";

// fileURLToPath, not URL#pathname: pathname keeps percent-encoding and a
// leading slash ("/C:/Users/SM%20-%20Dan/..."), which path.resolve on Windows
// mangles into "C:\C:\...%20..." the moment the checkout path contains a
// space. Found by U14 running this gate on a real Windows checkout.
export const REPO_ROOT = path.resolve(
  path.dirname(fileURLToPath(import.meta.url)),
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
 * `published: true` means the capture is required to exist, be manifest-bound,
 * and be current against CAPTURE_INPUTS. It is the mechanical regression
 * evidence that the console still renders the way the page claims.
 *
 * 2026-09-01: the page's PLATES are no longer these captures. The owner
 * replaced them with real-session screenshots (`manifest.ownerCaptures`,
 * verified below by hash, size and import). The deterministic set stays
 * because it is the only capture that can be proven current by content: an
 * owner screenshot goes stale silently, a scripted one fails the gate.
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

/**
 * Minimal JPEG SOF read — walks the marker chain to the first start-of-frame
 * and returns its dimensions. Owner captures ship as JPEG; this keeps the
 * verify lane dependency-free for one more integer pair.
 */
export function jpegDimensions(buffer) {
  if (buffer.length < 4 || buffer[0] !== 0xff || buffer[1] !== 0xd8) return null;
  let offset = 2;
  while (offset + 9 < buffer.length) {
    if (buffer[offset] !== 0xff) { offset += 1; continue; }
    const marker = buffer[offset + 1];
    if (marker === 0xd8 || marker === 0x01 || (marker >= 0xd0 && marker <= 0xd7)) {
      offset += 2;
      continue;
    }
    const length = buffer.readUInt16BE(offset + 2);
    // SOF0–SOF3, SOF5–SOF7, SOF9–SOF11, SOF13–SOF15 all carry the frame size.
    if (
      (marker >= 0xc0 && marker <= 0xcf) &&
      marker !== 0xc4 && marker !== 0xc8 && marker !== 0xcc
    ) {
      return {
        height: buffer.readUInt16BE(offset + 5),
        width: buffer.readUInt16BE(offset + 7),
      };
    }
    offset += 2 + length;
  }
  return null;
}

/** PNG or JPEG dimensions, whichever the bytes are. */
export function imageDimensions(buffer) {
  return pngDimensions(buffer) ?? jpegDimensions(buffer);
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
