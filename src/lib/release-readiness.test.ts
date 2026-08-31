import { readFileSync } from "node:fs";

import { describe, expect, it } from "vitest";

import {
  RELEASE_METADATA,
  RELEASES_INDEX_URL,
  detectPlatform,
  resolveRelease,
  type ReleaseMetadata,
} from "../landing/release-config";
import { DURING_BETA, validRelease } from "../landing/release-fixture";
import { RELEASES_INDEX_URL as DESKTOP_RELEASES_INDEX_URL } from "./release-links";

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

describe("launch truth stays mechanically sticky (audit D-01)", () => {
  // Only the few truths whose drift would cause UNSAFE ACTION are pinned —
  // acting on "listening still pending" could retune approved sound; a
  // resurrected idea-backlog entry could rebuild a shipped system.
  it("owner listening is recorded as approved, not pending", () => {
    const product = readText("../../docs/PRODUCT.md");
    expect(product).not.toMatch(/owner listening remains pending/i);
    expect(product).toContain("approved 2026-08-25");
  });

  it("the adaptive compressor stays described as built but gated off", () => {
    const product = readText("../../docs/PRODUCT.md");
    expect(product).toMatch(/gated off/i);
  });

  it("shipped updater and navigation-machine entries are gone from the idea backlog", () => {
    const backlog = readText("../../docs/IDEAS_BACKLOG.md");
    expect(backlog).not.toContain("**Auto-updater**");
    expect(backlog).not.toContain("navigation state machine");
  });

  it("capability rows C-05, C-07, C-25, and C-26 are Applied", () => {
    const matrix = readText("../../docs/CAPABILITY_EVIDENCE_MATRIX.md");
    for (const row of ["C-05", "C-07", "C-25", "C-26"]) {
      const line = matrix
        .split("\n")
        .find((candidate) => candidate.includes(`| ${row} |`));
      expect(line, `row ${row} missing from the capability matrix`).toBeDefined();
      expect(line, `row ${row} must be Applied`).toContain("Applied");
    }
  });

  it("presets are approved: further retuning requires a new listening note", () => {
    const betaGuide = readText("../../docs/BETA_TESTING.md");
    expect(betaGuide).not.toContain("still being tuned by ear");
  });
});

describe("RustSec gate fails closed on unsound advisories (audit S-02)", () => {
  it("all three CI cargo-audit commands deny unsound with only the exact Linux GTK3 exception", () => {
    // Plain `cargo audit` treats unsoundness advisories as warnings — CI
    // stayed green while RUSTSEC-2026-0190 (anyhow) sat in all three locks.
    // The gate must deny unsound, with exactly one narrow, removable
    // exception: the Linux-only GTK3 path (RUSTSEC-2024-0429), because Linux
    // is deferred for this beta.
    const ci = readText("../../.github/workflows/ci.yml");
    const auditCommands = ci
      .split("\n")
      .filter(
        (line) =>
          line.includes("cargo audit") && !line.trim().startsWith("#"),
      );
    expect(auditCommands).toHaveLength(3);
    for (const command of auditCommands) {
      expect(command).toContain("--deny unsound");
      expect(command).toContain("--ignore RUSTSEC-2024-0429");
      expect(command).toContain("--file");
    }
  });
});

describe("updater manual-recovery origin (audit L-03)", () => {
  it("pins the fixed Releases URL identically in frontend, Rust, and landing sources", () => {
    // The desktop open_release_page command must only ever open this URL.
    // Three sources carry it: the frontend copy constant, the private Rust
    // constant, and the landing page's derived releases-index. If any of
    // them drifts, manual recovery could point somewhere else.
    expect(DESKTOP_RELEASES_INDEX_URL).toBe(
      "https://github.com/DanielKinsner/yes-master/releases",
    );
    expect(DESKTOP_RELEASES_INDEX_URL).toBe(RELEASES_INDEX_URL);

    const rustLib = readText("../../src-tauri/src/lib.rs");
    expect(rustLib).toContain(
      `RELEASES_INDEX_URL: &str = "${DESKTOP_RELEASES_INDEX_URL}"`,
    );
  });
});

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
    // that would send a visitor to a release that does not exist. U1 opened it
    // as "Remove (as unconditional)"; U5 removed it, so the assertion now pins
    // the *resolution* instead of the pending status. The row itself must
    // survive — deleting it would erase the record that the page ever shipped
    // a dead CTA, and the resolver below is what keeps it removed.
    expect(claimMatrix).toContain("releases/latest");
    expect(claimMatrix).toMatch(/C-10[\s\S]{0,900}Removed \(U5/);
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
    // 2026-08-25 owner decision: the standing listening gates are approved;
    // the go/no-go keeps ONE final by-ear spot-check on the installed
    // candidate. This pin moved with the decision — it still guards against
    // the listening line disappearing entirely.
    expect(goNoGo).toContain(
      "One final by-ear check on the installed candidate",
    );
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

// ---------------------------------------------------------------------------
// U5 — release state model.
//
// These are behaviour tests over the resolver, not static invariants. The one
// thing they all defend: the download activates for exactly one input — a
// fully verified, published, in-window release with both platform artifacts —
// and every deviation from that closes it. Absent, malformed, stale, and
// incomplete are not distinct outcomes for a visitor; they are all "nothing to
// download", and treating them differently is how a dead CTA ships.
// ---------------------------------------------------------------------------

describe("release state model (U5)", () => {
  // Negative control. Every rejection test below removes one field from this
  // fixture, so if the fixture itself did not resolve to available, all of them
  // would pass while proving nothing.
  it("activates for a complete, verified, in-window release", () => {
    const resolved = resolveRelease(validRelease(), DURING_BETA);
    expect(resolved.state).toBe("verified-public");
    expect(resolved.available).toBe(true);
    expect(resolved.downloads.map((d) => d.id)).toEqual(["windows", "mac"]);
    expect(resolved.diagnostics).toEqual([]);
  });

  it("ships closed: no release metadata is committed", () => {
    // U16 verifies and U17 publishes. An agent populating this constant is an
    // agent shipping a release, so the shipped default is pinned here.
    expect(RELEASE_METADATA).toBeNull();
    const resolved = resolveRelease();
    expect(resolved.available).toBe(false);
    expect(resolved.reasonCode).toBe("no-release");
    expect(resolved.secondary.url).toBe(RELEASES_INDEX_URL);
    expect(resolved.secondary.url).not.toContain("/releases/latest");
  });

  it("rejects an active state without the beta end date", () => {
    // Owner-blocked in docs/OWNER_INPUT_QUEUE.md. A time-boxed beta with no
    // stated end is a promise that cannot be kept, so it stays closed.
    const resolved = resolveRelease(
      validRelease({ betaEndsAt: null }),
      DURING_BETA,
    );
    expect(resolved.available).toBe(false);
    expect(resolved.state).toBe("candidate-published");
    expect(resolved.diagnostics.join(" ")).toContain("betaEndsAt is unset");
  });

  it("rejects an active state without the expected platform artifacts", () => {
    const cases: Array<[string, ReleaseMetadata, RegExp]> = [
      [
        "no Windows installer",
        validRelease({
          artifacts: {
            windowsExe: null,
            macUniversalDmg: validRelease().artifacts.macUniversalDmg,
          },
        }),
        /windowsExe: missing/,
      ],
      [
        "no Mac disk image",
        validRelease({
          artifacts: {
            windowsExe: validRelease().artifacts.windowsExe,
            macUniversalDmg: null,
          },
        }),
        /macUniversalDmg: missing/,
      ],
      [
        "an artifact hosted outside this repository",
        validRelease({
          artifacts: {
            ...validRelease().artifacts,
            windowsExe: {
              url: "https://cdn.example.com/YES-Master-setup.exe",
              sizeBytes: 1024,
              sha256: "c".repeat(64),
            },
          },
        }),
        /windowsExe: url is not a/,
      ],
      [
        "a missing checksum",
        validRelease({
          artifacts: {
            ...validRelease().artifacts,
            macUniversalDmg: {
              url: `${validRelease().artifacts.macUniversalDmg!.url}`,
              sizeBytes: 1024,
              sha256: "not-a-digest",
            },
          },
        }),
        /macUniversalDmg: sha256/,
      ],
    ];

    for (const [name, metadata, expected] of cases) {
      const resolved = resolveRelease(metadata, DURING_BETA);
      expect(resolved.available, `${name} should not activate`).toBe(false);
      expect(resolved.downloads).toEqual([]);
      expect(resolved.diagnostics.join(" ")).toMatch(expected);
    }
  });

  it("rejects an active state without a coherent updater channel", () => {
    // The shipped app reads GitHub's /releases/latest. A prerelease is invisible
    // to that channel, so publishing one would give downloaders an app that can
    // never update itself.
    const prerelease = resolveRelease(
      validRelease({ publication: "prerelease" }),
      DURING_BETA,
    );
    expect(prerelease.available).toBe(false);
    expect(prerelease.diagnostics.join(" ")).toContain("prerelease");

    const wrongChannel = resolveRelease(
      validRelease({ updaterChannel: "nightly" }),
      DURING_BETA,
    );
    expect(wrongChannel.available).toBe(false);
    expect(wrongChannel.diagnostics.join(" ")).toContain("incoherent");
  });

  it("rejects verification that predates the build it claims to cover", () => {
    const resolved = resolveRelease(
      validRelease({ publishedAt: "2026-08-01", verifiedAt: "2026-07-30" }),
      DURING_BETA,
    );
    expect(resolved.available).toBe(false);
    expect(resolved.diagnostics.join(" ")).toContain("predates");
  });

  it("keeps drafts closed and never leaks that a draft exists", () => {
    // GitHub's /latest channel cannot see drafts (KTD3). S-B1: creating a draft
    // must not open the landing page.
    const resolved = resolveRelease(
      validRelease({ publication: "draft" }),
      DURING_BETA,
    );
    expect(resolved.state).toBe("draft-proof");
    expect(resolved.available).toBe(false);
    expect(resolved.downloads).toEqual([]);
    expect(resolved.reason.toLowerCase()).not.toContain("draft");
    expect(resolved.diagnostics.join(" ")).toContain("draft");
  });

  it("closes the download once the beta window has passed", () => {
    const resolved = resolveRelease(
      validRelease(),
      new Date("2027-01-01T00:00:00Z"),
    );
    expect(resolved.available).toBe(false);
    expect(resolved.reasonCode).toBe("beta-ended");
    expect(resolved.state).toBe("unavailable");
  });

  it("treats malformed metadata exactly like absent metadata", () => {
    const malformed = [
      undefined,
      "0.9.1" as unknown as ReleaseMetadata,
      { version: "0.9.1" } as unknown as ReleaseMetadata,
      validRelease({ publication: "shipped" as never }),
      validRelease({ publishedAt: "2026-02-31" }),
      validRelease({ releaseUrl: "https://example.com/downloads" }),
    ];
    for (const metadata of malformed) {
      const resolved = resolveRelease(metadata as never, DURING_BETA);
      expect(resolved.available).toBe(false);
      expect(resolved.downloads).toEqual([]);
      expect(resolved.diagnostics.length).toBeGreaterThan(0);
    }
  });

  it("never renders internal state as visitor-facing copy", () => {
    const states: Array<ReleaseMetadata | null> = [
      null,
      validRelease({ publication: "draft" }),
      validRelease({ publication: "prerelease" }),
      validRelease({ verifiedAt: null }),
      validRelease({ publication: "withdrawn" }),
    ];
    for (const metadata of states) {
      const { reason } = resolveRelease(metadata, DURING_BETA);
      expect(reason.length).toBeGreaterThan(0);
      // Present tense, no roadmap, no build detail — docs/landing-brief.md.
      for (const banned of [
        /draft/i,
        /prerelease/i,
        /unverified/i,
        /candidate/i,
        /coming soon/i,
        /\bwill be\b/i,
        /\d+\.\d+\.\d+/,
      ]) {
        expect(reason, `reason leaked ${banned}`).not.toMatch(banned);
      }
    }
  });

  it("guesses the platform without ever excluding the other one", () => {
    expect(detectPlatform({ platform: "Win32" })).toBe("windows");
    expect(detectPlatform({ userAgentData: { platform: "Windows" } })).toBe(
      "windows",
    );
    expect(detectPlatform({ platform: "MacIntel" })).toBe("mac");
    expect(detectPlatform({ userAgent: "Mozilla/5.0 (Macintosh; ...)" })).toBe(
      "mac",
    );
    expect(detectPlatform({ platform: "Linux x86_64" })).toBe("other");
    // A navigator with no usable signal must resolve to "other". Passing
    // `undefined` here would NOT test that: an explicit `undefined` argument
    // triggers the default parameter, which falls back to the REAL jsdom
    // navigator — whose user agent embeds the host OS, so the assertion
    // passed on Linux and inverted on Windows (found by U14's first
    // real-Windows frontend run, 2026-07-27).
    expect(detectPlatform({})).toBe("other");
    expect(detectPlatform({ platform: "", userAgent: "" })).toBe("other");

    // iPads report a Mac-shaped platform string. Handing a tablet visitor the
    // desktop .dmg as "your" download would be a lie in the honest direction's
    // opposite.
    expect(detectPlatform({ platform: "iPad", userAgent: "Macintosh" })).toBe(
      "other",
    );
    expect(
      detectPlatform({ userAgent: "Mozilla/5.0 (Linux; Android 14)" }),
    ).toBe("other");

    // Detection can only ever be a hint: the resolver hands over both
    // downloads regardless of what it returns.
    const resolved = resolveRelease(validRelease(), DURING_BETA);
    expect(resolved.downloads).toHaveLength(2);
  });
});
