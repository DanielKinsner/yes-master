import { readFileSync } from "node:fs";
import { resolve } from "node:path";

import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";

import LandingPage from "../LandingPage";

// ---------------------------------------------------------------------------
// U6 — landing narrative and marketing truth.
//
// Copy review alone cannot stop an untrue sentence from shipping; the sentence
// looks fine in a diff and nothing fails. These are the assertions that fail.
//
// Rendered with renderToStaticMarkup ON PURPOSE. It produces the markup with no
// client JavaScript, no effects, and no animation at all — so every claim these
// tests read is one a visitor gets whether or not motion runs, which is the
// "essential content still understandable when animations do not run" half of
// the S-A1/S-A2/S-I1 acceptance.
// ---------------------------------------------------------------------------

const html = renderToStaticMarkup(<LandingPage />);
const text = html
  .replace(/<[^>]+>/g, " ")
  .replace(/&amp;/g, "&")
  .replace(/&#x27;|&#39;/g, "'")
  .replace(/&quot;/g, '"')
  .replace(/\s+/g, " ")
  .trim();

const matrix = readFileSync(
  resolve(process.cwd(), "docs/CAPABILITY_EVIDENCE_MATRIX.md"),
  "utf8",
);
const brief = readFileSync(
  resolve(process.cwd(), "docs/landing-brief.md"),
  "utf8",
);

function positionOf(needle: string): number {
  const at = text.indexOf(needle);
  expect(at, `expected the page to contain "${needle}"`).toBeGreaterThan(-1);
  return at;
}

describe("landing hierarchy (U6)", () => {
  it("orders the page as one visitor hierarchy", () => {
    // Problem/outcome → what you do → proof → character → depth → the deal →
    // platform reality → action. The order IS the argument; asserting section
    // presence alone would let someone shuffle it back.
    const order = [
      "Your Endgame Sound.",
      "Finished mix in. Finished master out.",
      "One engine. Two rooms.",
      "It reads the track before it touches it.",
      "A record, not a folder of files.",
      "What you are actually agreeing to.",
      "Desktop is the product.",
      "Stop chasing the master.",
    ];
    const positions = order.map(positionOf);
    for (let i = 1; i < positions.length; i += 1) {
      expect(
        positions[i],
        `"${order[i]}" must come after "${order[i - 1]}"`,
      ).toBeGreaterThan(positions[i - 1]);
    }
  });

  it("keeps Album as lower-page depth proof, not a headline promise", () => {
    // Album-minded creators are inside the primary audience, not a second one.
    // Leading with a record-length workflow misrepresents what most visitors
    // came to do, which is finish one track.
    expect(positionOf("A record, not a folder of files.")).toBeGreaterThan(
      positionOf("One engine. Two rooms."),
    );
    expect(positionOf("A record, not a folder of files.")).toBeGreaterThan(
      positionOf("It reads the track before it touches it."),
    );
    expect(text).toContain("Album Master lives in Advanced");
  });

  it("demotes mobile out of the position it used to occupy", () => {
    // It was the SECOND section on the page — a parked, unobtainable product
    // in the most valuable slot, with six present-tense feature cards.
    expect(positionOf("Desktop is the product.")).toBeGreaterThan(
      positionOf("What you are actually agreeing to."),
    );
    expect(html).not.toContain('href="#mobile"');
  });
});

describe("prohibited claims (U6)", () => {
  it("makes no roadmap, schedule, or availability promise about mobile", () => {
    for (const banned of [
      /coming soon/i,
      /coming to your pocket/i,
      /coming after launch/i,
      /headed to/i,
      /\bin the coming\b/i,
      /download (?:for )?(?:iphone|android|ios)/i,
      /app store|google play/i,
      // "Soon" anywhere, not just "coming soon". The newsletter button read
      // "Email updates opening soon" and slipped past a /coming soon/ scan
      // while being exactly the roadmap word the brief bans — and for a
      // feature with no provider selected at all.
      /\bsoon\b/i,
      /\bshortly\b/i,
      /\b(?:next|later) (?:month|year|quarter)\b/i,
    ]) {
      expect(text, `page reintroduced ${banned}`).not.toMatch(banned);
    }

    // Exactly one restrained sentence, per docs/landing-brief.md "Mobile
    // status" — which permits this as the single deliberate exception to the
    // no-roadmap rule.
    expect(text).toContain("iPhone and Android are not currently available");
    expect(text.match(/iPhone and Android are not currently available/g))
      .toHaveLength(1);

    // The phone screenshot is gone. R7 forbids a mobile UI image standing as
    // desktop-beta proof, and that asset has no capture-commit binding (C-17).
    expect(html).not.toContain("iphone-standard-ui");
  });

  it("promises no entitlement and manufactures no scarcity", () => {
    // C-22: "beta testers keep $29 forever" is an entitlement promise whose
    // terms are undecided (owner queue row 1).
    for (const banned of [
      /keep \$29 forever/i,
      /forever/i,
      /limited spots|only \d+ (?:spots|seats|licen[cs]es)/i,
      /act now|hurry|don't miss|last chance/i,
      /ends in \d/i,
    ]) {
      expect(text, `page reintroduced ${banned}`).not.toMatch(banned);
    }
  });

  it("guarantees no outcome and grades nobody's ability", () => {
    for (const banned of [
      /professional(?:ly)? (?:results?|sound|master)/i,
      /\bwe guarantee\b/i,
      /guaranteed (?:results?|quality|loudness|master)/i,
      /\bbroke\b/i,
      /amateur/i,
      /nonprofessional|non-professional/i,
      /bedroom producer/i,
      /fix your (?:bad )?mix/i,
      /true-peak safe, every time/i,
    ]) {
      expect(text, `page reintroduced ${banned}`).not.toMatch(banned);
    }
  });

  it("prints no version number and no build detail", () => {
    expect(text).not.toMatch(/\bv?\d+\.\d+\.\d+\b/);
    for (const banned of [/commit|sha-|nightly|changelog|roadmap/i]) {
      expect(text).not.toMatch(banned);
    }
  });
});

describe("required claims (U6)", () => {
  it("makes the adaptive-restraint pitch without an amateur-rescue frame", () => {
    // Restraint, not rescue: plenty of tracks arrive already processed on
    // purpose, and the shipped Tier-1 guardrails are reduce-only.
    expect(text).toContain("reads what your mix already has");
    expect(text).toContain("eases its own moves");
    expect(text).toContain("already processed");
    expect(text).toMatch(/works with what is there rather than fighting it/i);
    // "Rescue" framing would read as a verdict on the visitor's mix.
    expect(text).not.toMatch(/rescue|salvage|repair your/i);
  });

  it("addresses the audiences in the settled order", () => {
    const independent = positionOf("independent artists");
    const emerging = positionOf("first few tracks");
    const engineers = positionOf("engineers who want the whole console");
    expect(emerging).toBeGreaterThan(independent);
    expect(engineers).toBeGreaterThan(emerging);
  });

  it("presents styles as starting points under one safety system", () => {
    expect(text).toContain("Styles are starting points, not lanes.");
    expect(text).toMatch(/same loudness landing and the same safety stages/i);
    expect(text).toMatch(
      /Choosing a bolder character never means switching the checks off/i,
    );
    for (const style of ["Universal", "Clarity", "Tape", "Oomph"]) {
      expect(text).toContain(style);
    }
  });

  it("states the beta arrangement, including the part people assume is worse", () => {
    // Voluntary feedback.
    expect(text).toMatch(/Telling us things is optional/i);
    expect(text).toMatch(/Nothing is held back from anyone who never does/i);

    // Non-revoking end (R26/KTD13). "Beta over" normally means something worse
    // than what this actually does, so it is said plainly.
    expect(text).toMatch(/a build you already installed keeps working/i);
    expect(text).toMatch(/no kill switch/i);
    expect(text).toMatch(/nothing expires on your machine/i);

    // Ungated download.
    expect(text).toMatch(/not gated behind an email address/i);
    expect(text).toMatch(/no YES Master account/i);
  });

  it("states the settled price model and stops there", () => {
    expect(text).toContain("$29");
    expect(text).toContain("$49");
    expect(text).toMatch(/one-time purchase/i);
    expect(text).toMatch(/not a subscription/i);
    // The window exists but its terms do not, so the page says exactly that.
    expect(text).toMatch(
      /founder window's dates and terms are announced when they are decided/i,
    );
    expect(text).not.toMatch(/\bbeta (?:testers|users) (?:get|keep)\b/i);
  });

  it("narrows the Standard export claim to what the app actually does", () => {
    // C-05. The absolute guarantee is gone; the fixed format and the ceiling
    // the limiter targets are both real (docs/PRODUCT.md).
    expect(text).toContain("44.1 kHz / 24-bit WAV");
    expect(text).toContain("−1 dBTP");
  });

  it("labels the illustrative receipt as an example", () => {
    // C-07. These numbers are hand-authored. Rendering them unmarked, in a
    // block that looks exactly like product output, is a fabricated
    // measurement — the failure the evidence matrix exists to catch.
    const label = positionOf("Example receipt");
    const firstNumber = positionOf("-11.0 LUFS");
    expect(label).toBeLessThan(firstNumber);
  });
});

describe("claim/evidence binding (U6)", () => {
  // "A claim with no row is a defect, not an oversight" — docs/landing-brief.md.
  const landingClaims = [
    "C-01", "C-02", "C-03", "C-04", "C-05", "C-06", "C-07", "C-08",
    "C-11", "C-12", "C-13", "C-14", "C-15", "C-16", "C-17", "C-18",
    "C-20", "C-21", "C-22", "C-25", "C-27", "C-28", "C-29", "C-30",
    "C-31", "C-32", "C-33", "C-34",
  ];

  it("binds every claim the page makes to a matrix row with an evidence source", () => {
    for (const id of landingClaims) {
      const row = matrix
        .split("\n")
        .find((line) => line.startsWith(`| ${id} |`));
      expect(row, `matrix has no row for ${id}`).toBeDefined();

      // A row with an empty Status or Evidence cell is a row that proves
      // nothing, which is worse than a missing row because it looks done.
      const cells = (row as string).split("|").map((cell) => cell.trim());
      expect(cells.length, `${id} row is missing columns`).toBeGreaterThanOrEqual(7);
      expect(cells[4], `${id} has no status`).not.toBe("");
      // Terse is allowed — C-08's source is legitimately "As C-04." What is
      // not allowed is an empty cell, or a placeholder pretending to be one.
      expect(cells[5].length, `${id} has no evidence source`).toBeGreaterThan(3);
      expect(cells[5], `${id} evidence is a placeholder`).not.toMatch(
        /^(?:tbd|todo|n\/a|-+|obviously true)$/i,
      );
    }
  });

  it("leaves no owner-blocked claim asserted on the page", () => {
    // C-22 (entitlement) and C-23 (beta end date) are Owner rows. Their
    // conservative default is silence, and the copy tests above enforce it —
    // this pins the *reason*, so removing the queue row does not quietly make
    // the claim publishable.
    expect(matrix).toMatch(/C-22[\s\S]{0,400}\*\*Owner\*\*/);
    expect(matrix).toMatch(/C-23[\s\S]{0,400}\*\*Owner\*\*/);
  });

  it("keeps the brief and the page telling the same story", () => {
    // The brief is the source the next copy pass reads. If it still described
    // a page that no longer exists, the next pass would rebuild the old one.
    expect(brief).toContain("## Mobile status");
    expect(brief).toContain("**iPhone and Android are parked.**");
    expect(brief).toContain("## Page order");
  });
});
