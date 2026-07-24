import { readFileSync } from "node:fs";

import { describe, expect, it } from "vitest";

function readText(path: string): string {
  return readFileSync(new URL(path, import.meta.url), "utf8").replace(
    /\r\n/g,
    "\n",
  );
}

function readMaybe(path: string): string | null {
  try {
    return readText(path);
  } catch {
    return null;
  }
}

const workflow = readText("../../.github/workflows/release.yml");
const preflightJobHeader = workflow
  .split("\n  preflight:\n")[1]
  .split("\n    steps:\n")[0];
const releaseJobHeader = workflow
  .split("\n  release:\n")[1]
  .split("\n    steps:\n")[0];
const windowsSigning = readText("../../src-tauri/tauri.windows-signing.conf.json");
const updaterOverlay = JSON.parse(
  readText("../../src-tauri/tauri.updater.conf.json"),
) as Record<string, unknown>;
const tauriConfig = JSON.parse(
  readText("../../src-tauri/tauri.conf.json"),
) as { plugins?: { updater?: { pubkey?: unknown } } };

const DISCARDED_BOOTSTRAP_PUBKEY =
  "dW50cnVzdGVkIGNvbW1lbnQ6IG1pbmlzaWduIHB1YmxpYyBrZXk6IEM5OEJBMEIzQzYyRURDRUUKUldUdTNDN0dzNkNMeVR5VVhUeXJERVVST05nZG5XSXFzK3RDZTBjZFgray81WXpxc2d5eDR3eGEK";

describe("zero-cost beta release contract", () => {
  it("builds one universal macOS artifact instead of separate architecture releases", () => {
    expect(workflow.match(/platform: macos-latest/g)).toHaveLength(1);
    expect(workflow).toContain('label: macOS (Universal)');
    expect(workflow).toContain('args: "--target universal-apple-darwin"');
    expect(workflow).not.toContain('args: "--target aarch64-apple-darwin"');
    expect(workflow).not.toContain('args: "--target x86_64-apple-darwin"');
  });

  it("requires free updater signing while leaving paid OS signing optional", () => {
    expect(workflow).toContain("name: Require updater signing secrets");
    expect(workflow).toContain("TAURI_SIGNING_PRIVATE_KEY");
    expect(workflow).toContain("TAURI_SIGNING_PRIVATE_KEY_PASSWORD");
    expect(workflow).toContain("--config src-tauri/tauri.updater.conf.json");

    expect(workflow).toContain("env.APPLE_SIGNING_ENABLED == 'true'");
    expect(workflow).toContain("env.AZURE_SIGNING_ENABLED == 'true'");
    expect(updaterOverlay).not.toHaveProperty("//");
  });

  it("exposes signing secrets only to steps that need them", () => {
    expect(preflightJobHeader).not.toContain(
      "TAURI_SIGNING_PRIVATE_KEY: ${{ secrets.TAURI_SIGNING_PRIVATE_KEY }}",
    );
    expect(releaseJobHeader).not.toContain(
      "TAURI_SIGNING_PRIVATE_KEY: ${{ secrets.TAURI_SIGNING_PRIVATE_KEY }}",
    );
    expect(releaseJobHeader).not.toContain(
      "AZURE_CLIENT_SECRET: ${{ secrets.AZURE_CLIENT_SECRET }}",
    );
    expect(releaseJobHeader).toContain("APPLE_SIGNING_ENABLED");
    expect(releaseJobHeader).toContain("AZURE_SIGNING_ENABLED");
  });

  it("keeps the GitHub latest channel discoverable by the updater", () => {
    expect(workflow).toContain("prerelease: false");
    expect(workflow).not.toContain("github.event.inputs.prerelease");
  });

  it("uses the current Azure Artifact Signing CLI when paid signing is added later", () => {
    expect(workflow).toContain("cargo install artifact-signing-cli");
    expect(workflow).not.toContain("cargo install trusted-signing-cli");
    expect(windowsSigning).toContain("artifact-signing-cli");
    expect(windowsSigning).not.toContain("trusted-signing-cli");
  });

  it("audits the draft before the owner can publish an incomplete release", () => {
    expect(workflow).toContain("name: Audit draft release assets");
    expect(workflow).toContain("latest.json");
    expect(workflow).toContain(".dmg");
    expect(workflow).toContain(".msi");
    expect(workflow).toContain(".sig");
    expect(workflow).toContain("macOS updater signature (.app.tar.gz.sig)");
    expect(workflow).toContain("Windows MSI updater signature (.msi.sig)");
    expect(workflow).toContain("Windows NSIS updater signature (setup.exe.sig)");
    expect(workflow).toContain("Updater manifest is missing macOS");
    expect(workflow).toContain("Updater manifest is missing Windows");
    expect(workflow).toContain('Where-Object Name -ne "SHA256SUMS.txt"');
  });

  it("bakes in the permanent updater public key instead of the discarded bootstrap", () => {
    const pubkey = tauriConfig.plugins?.updater?.pubkey;
    expect(typeof pubkey).toBe("string");
    expect(pubkey).not.toBe(DISCARDED_BOOTSTRAP_PUBKEY);

    const decoded = Buffer.from(pubkey as string, "base64").toString("utf8");
    expect(decoded).toContain("minisign public key");
    expect(decoded).toContain("RW");
  });
});

// ---------------------------------------------------------------------------
// U1 — product canon and evidence contract (2026-07-24 quality program).
//
// These are STATIC INVARIANTS, not behavior tests. They exist because the canon
// reconciliation U1 performed is the kind of thing that rots silently: someone
// re-adds a present-tense mobile claim to the landing brief, or the two agent
// instruction files drift, and nothing fails until a visitor reads an untrue
// sentence on a public page. Each assertion below pins one reconciliation.
// ---------------------------------------------------------------------------

const agentsMd = readText("../../AGENTS.md");
const claudeMd = readText("../../CLAUDE.md");
const landingBrief = readText("../../docs/landing-brief.md");
const productDoc = readText("../../docs/PRODUCT.md");
const goNoGo = readText("../../docs/plans/beta-go-no-go.md");
const ownerQueue = readText("../../docs/OWNER_INPUT_QUEUE.md");
const claimMatrix = readText("../../docs/CAPABILITY_EVIDENCE_MATRIX.md");

describe("product canon and evidence contract (U1)", () => {
  it("keeps the two agent instruction files byte-identical", () => {
    // AGENTS.md is what Codex reads and CLAUDE.md is what Claude reads. Drift
    // between them means two agents working the same repo from different rules.
    expect(claudeMd).toBe(agentsMd);
  });

  it("names exactly one authoritative source for each beta question", () => {
    // Three beta documents exist. Before U1 nothing said how they rank, so a
    // session could resume from executed history and redo shipped work.
    expect(agentsMd).toContain("Which beta document is authoritative");
    expect(agentsMd).toMatch(
      /2026-07-07-beta-execution-plan\.md` is \*\*executed history/,
    );
    expect(agentsMd).toMatch(
      /2026-07-24-001-feat-public-beta-quality-plan\.md` is the \*\*active\s*\n?\s*forward queue/,
    );
    expect(agentsMd).toMatch(
      /beta-go-no-go\.md` is the \*\*live release gate/,
    );

    // Platform status -> PRODUCT.md. Release state + owner gates -> go/no-go.
    // Undecided questions (incl. the beta end date) -> the owner input queue.
    expect(agentsMd).toContain("docs/OWNER_INPUT_QUEUE.md");
    expect(agentsMd).toContain("docs/CAPABILITY_EVIDENCE_MATRIX.md");
    expect(ownerQueue).toContain("Beta end date");
  });

  it("does not leave landing-page scope described as an open owner decision", () => {
    // Stale parenthetical: the Non-Negotiables, PRODUCT.md "Public Surface",
    // and D16 all already put the landing page in scope.
    expect(agentsMd).not.toContain(
      "whether the landing page is in agent scope is an open",
    );
    expect(agentsMd).toContain("The landing page **is** in agent scope");
  });

  it("keeps the landing brief free of present-tense mobile availability claims", () => {
    expect(landingBrief).toContain("## Mobile status");
    expect(landingBrief).toContain("**iPhone and Android are parked.**");

    // The scan covers the brief's CLAIM-BEARING sections only. Three parts of
    // this document state rules rather than making claims — the header
    // blockquote, "Mobile status", and "Hard rules for generation" — and all
    // three necessarily QUOTE banned phrasing in order to prohibit it. Scanning
    // them would fail on the prohibition itself. They are pinned separately
    // below, so deleting a prohibition to dodge the scan does not work either.
    const ruleStatingSections = ["Mobile status", "Hard rules for generation"];
    const copySource = landingBrief
      .split(/^## /m)
      .slice(1) // drop the title + header blockquote (rule text)
      .filter(
        (section) =>
          !ruleStatingSections.some((name) => section.startsWith(name)),
      )
      .join("\n");
    expect(copySource).toContain("Feature pillars");
    expect(copySource).toContain("What's on screen");

    const bannedInCopySource = [
      /companions on the same engine/i,
      /coming soon/i,
      /\bcoming (?:after|to)\b/i,
      /download (?:for |on )?(?:iphone|android|ios)/i,
      /available on (?:the )?app store/i,
      /google play/i,
    ];
    for (const pattern of bannedInCopySource) {
      expect(
        pattern.test(copySource),
        `landing brief copy source reintroduced a mobile availability claim matching ${pattern}`,
      ).toBe(false);
    }

    // The prohibition list itself must survive. If someone deletes it, the
    // scoped scan above would stop protecting anything.
    const mobileStatus = landingBrief.slice(
      landingBrief.indexOf("## Mobile status"),
    );
    expect(mobileStatus).toContain("It may not:");
    expect(mobileStatus).toMatch(/give a date, a season, a release order/);
    expect(landingBrief).toContain(
      "**No roadmap, no \"coming soon,\" no version numbers, no build/dev detail.**",
    );
  });

  it("binds every public claim to the evidence matrix", () => {
    expect(landingBrief).toContain("docs/CAPABILITY_EVIDENCE_MATRIX.md");

    // The matrix must still cover the three claim classes U1 audited. If a
    // section is deleted wholesale the matrix stops being truth control.
    expect(claimMatrix).toContain("## A — Product capability claims");
    expect(claimMatrix).toContain("## B — Platform and availability claims");
    expect(claimMatrix).toContain("## C — Pricing and beta-promise claims");

    // The dead-CTA row is the single most load-bearing entry: it is the claim
    // that would send a visitor to a release that does not exist.
    expect(claimMatrix).toContain("releases/latest");
    expect(claimMatrix).toMatch(/C-10[\s\S]{0,600}Remove \(as unconditional\)/);
  });

  it("keeps product policy desktop-first with mobile parked", () => {
    expect(productDoc).toContain("The v1 public push is desktop-first");
    expect(productDoc).toMatch(
      /phones go live when the owner\s*\n?\s*judges them ready/,
    );
  });

  it("retains the required release metadata in the go/no-go gate", () => {
    // Losing any of these turns a checkbox back into an unevidenced claim.
    expect(goNoGo).toContain("Update path proven end-to-end, once.");
    expect(goNoGo).toContain("one-sitting listening runbook is executed and signed off");
    expect(goNoGo).toContain("Exact-commit evidence ledger");

    for (const column of [
      "Commit SHA",
      "Platform / toolchain",
      "Artifact / version",
      "Command or procedure",
      "Evidence layer",
      "Evidence location",
    ]) {
      expect(goNoGo, `evidence ledger lost the "${column}" column`).toContain(
        column,
      );
    }

    // Evidence layers are not interchangeable; the ledger must keep saying so.
    for (const layer of [
      "browser-headless",
      "native-synthetic",
      "installed-machine",
      "owner-listening",
      "production-smoke",
    ]) {
      expect(goNoGo).toContain(layer);
    }

    // The candidate freeze has to declare its own state or a parallel agent
    // session cannot tell whether `main` is frozen.
    expect(goNoGo).toMatch(/\*\*Freeze status: (NOT IN FORCE|IN FORCE)/);
  });

  it("keeps owner-blocked questions in the queue instead of guessed in docs", () => {
    expect(ownerQueue).toContain("Never block on an owner decision");
    expect(ownerQueue).toContain("Conservative default in place");

    // Each seeded question must still carry a conservative default. A row that
    // loses its default is a row that will get guessed.
    for (const question of [
      "Founder-window dates",
      "Newsletter provider",
      "Beta end date",
      "Public beta announcement date",
    ]) {
      const row = ownerQueue
        .split("\n")
        .find((line) => line.includes(question) && line.startsWith("|"));
      expect(row, `owner queue lost the "${question}" row`).toBeDefined();
      expect(
        (row as string).split("|").length,
        `"${question}" row is missing columns`,
      ).toBeGreaterThanOrEqual(6);
    }
  });

  it("has not created a fourth competing release checklist", () => {
    // R17 says the evidence ledger lives in the existing go/no-go artifact.
    expect(readMaybe("../../docs/RELEASE_CHECKLIST.md")).toBeNull();
    expect(readMaybe("../../docs/plans/release-evidence-ledger.md")).toBeNull();
  });
});
