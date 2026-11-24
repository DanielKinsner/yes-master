// -----------------------------------------------------------------------------
// U5 — release-aware download state.
//
// WHY THIS FILE IS SHAPED LIKE THIS
//
// The landing page's primary call to action used to be an unconditional link to
// `/releases/latest` (claim C-10 in docs/CAPABILITY_EVIDENCE_MATRIX.md). GitHub
// 404s that URL until a full, non-draft release exists, so the loudest control
// on a public marketing page pointed at nothing. Worse, it *looked* fine: there
// is no build step, test, or deploy check that notices a link which is only
// dead in production.
//
// So release availability is modelled as product state rather than markup. The
// resolver below is pure and total: given metadata it always returns something
// renderable, and every path that is not a fully verified public release
// returns `available: false`. Absent, malformed, stale, and incomplete metadata
// all land in the same place, because the failure that matters to a visitor is
// identical in each case — there is nothing to download.
//
// FIVE INTERNAL STATES, TWO PUBLIC ONES
//
//   Unavailable → DraftProof → CandidatePublished → VerifiedPublic → Withdrawn
//
// Visitors are never shown which of those we are in. Telling a stranger that a
// draft exists, or that verification failed, is build/dev detail and the
// landing brief's hard rules forbid it. The state lives here for the owner and
// for tests; the page renders Available or Unavailable and a calm reason.
//
// ACTIVATION IS NOT AN AGENT DECISION
//
// `RELEASE_METADATA` shipped as `null` until the owner published a release and
// ordered the site opened (2026-11-24 UTC; see the constant below). Changing it
// is what opens, repoints, or closes a public download, so it changes only on
// the owner's word, from a release he has published.
// -----------------------------------------------------------------------------

export const REPO_URL = "https://github.com/DanielKinsner/yes-master";

// The releases *index*, not `/releases/latest`. The index resolves whether or
// not any release exists; `/latest` 404s until one does. Every unavailable
// state routes here, which is the whole point of the unit: the fallback must be
// a page that is real today.
export const RELEASES_INDEX_URL = `${REPO_URL}/releases`;

export const INSTALL_GUIDE_URL = `${REPO_URL}/blob/main/docs/BETA_INSTALL.md`;

/** GitHub's own publication status for the release. */
export type ReleasePublication =
  | "draft"
  | "prerelease"
  | "published"
  | "withdrawn";

export interface ReleaseArtifact {
  /** Direct download URL for the artifact itself, not the release page. */
  url: string;
  sizeBytes: number;
  /** Lowercase hex SHA-256, as printed in SHA256SUMS.txt. */
  sha256: string;
}

export interface ReleaseMetadata {
  version: string;
  publication: ReleasePublication;
  /** The specific release page — used for checksums and notes, never as a download. */
  releaseUrl: string;
  /** Must be `latest`; the shipped app reads GitHub's `/releases/latest` channel. */
  updaterChannel: string;
  /** ISO date the release was published (YYYY-MM-DD). */
  publishedAt: string;
  /**
   * ISO date the *deployed* artifacts were verified end to end. Verification
   * that predates publication verified a different build.
   */
  verifiedAt: string | null;
  /** ISO date the beta window closes: actual publication date plus 56 calendar days. */
  betaEndsAt: string | null;
  artifacts: {
    windowsExe: ReleaseArtifact | null;
    macUniversalDmg: ReleaseArtifact | null;
  };
}

export type ReleaseState =
  | "unavailable"
  | "draft-proof"
  | "candidate-published"
  | "verified-public"
  | "withdrawn";

export type ReleaseReasonCode =
  | "available"
  | "no-release"
  | "malformed"
  | "draft"
  | "unverified"
  | "withdrawn"
  | "beta-ended";

export type PlatformId = "windows" | "mac";

export interface PlatformDownload {
  id: PlatformId;
  label: string;
  url: string;
  sizeBytes: number;
  sha256: string;
}

export interface SecondaryLink {
  label: string;
  url: string;
}

export interface ResolvedRelease {
  /** Internal five-state model. Not rendered. */
  state: ReleaseState;
  available: boolean;
  reasonCode: ReleaseReasonCode;
  /** Visitor-facing, present tense, no dates/versions/build detail. */
  reason: string;
  version: string | null;
  betaEndsAt: string | null;
  /** Empty unless the release is verified public. */
  downloads: PlatformDownload[];
  secondary: SecondaryLink;
  /** Owner/test-facing explanation of every downgrade. Never rendered. */
  diagnostics: string[];
}

// Present tense only, per docs/landing-brief.md "Hard rules for generation".
// "Coming soon", dates, seasons, and version numbers are all out of bounds, so
// these say what is true right now and stop.
const REASON_NOT_OPEN =
  "The YES Master free public beta is being prepared. The download is not open.";
const REASON_WITHDRAWN =
  "YES Master is in a free public beta. There is no build available to download right now.";
const REASON_BETA_ENDED = "The YES Master public beta is closed.";

const WATCH_RELEASES: SecondaryLink = {
  label: "Watch the releases page on GitHub",
  url: RELEASES_INDEX_URL,
};

const ISO_DATE = /^\d{4}-\d{2}-\d{2}$/;
const SHA256 = /^[a-f0-9]{64}$/i;
const ARTIFACT_HOST_PREFIX = `${REPO_URL}/releases/`;

function isIsoDate(value: unknown): value is string {
  if (typeof value !== "string" || !ISO_DATE.test(value)) return false;
  const parsed = Date.parse(`${value}T00:00:00Z`);
  if (Number.isNaN(parsed)) return false;
  // Rejects 2026-02-31, which Date.parse would otherwise roll forward.
  return new Date(parsed).toISOString().slice(0, 10) === value;
}

function isNonEmptyString(value: unknown): value is string {
  return typeof value === "string" && value.trim().length > 0;
}

/**
 * Artifact URLs must live under this repository's releases. A download button
 * that can be pointed at an arbitrary host by a bad edit is a supply-chain
 * hole, not a config option.
 */
function isRepoReleaseUrl(value: unknown): value is string {
  return typeof value === "string" && value.startsWith(ARTIFACT_HOST_PREFIX);
}

function artifactProblems(
  name: string,
  artifact: ReleaseArtifact | null | undefined,
  requiredSuffix: string,
): string[] {
  if (!artifact || typeof artifact !== "object") {
    return [`${name}: missing`];
  }
  const problems: string[] = [];
  if (!isRepoReleaseUrl(artifact.url)) {
    problems.push(`${name}: url is not a ${ARTIFACT_HOST_PREFIX}* download`);
  } else if (!artifact.url.toLowerCase().endsWith(requiredSuffix)) {
    problems.push(`${name}: url does not end in ${requiredSuffix}`);
  }
  if (!Number.isInteger(artifact.sizeBytes) || artifact.sizeBytes <= 0) {
    problems.push(`${name}: sizeBytes is not a positive integer`);
  }
  if (!SHA256.test(String(artifact.sha256))) {
    problems.push(`${name}: sha256 is not a 64-character hex digest`);
  }
  return problems;
}

function unavailable(
  state: ReleaseState,
  reasonCode: ReleaseReasonCode,
  reason: string,
  diagnostics: string[],
  version: string | null = null,
  betaEndsAt: string | null = null,
): ResolvedRelease {
  return {
    state,
    available: false,
    reasonCode,
    reason,
    version,
    betaEndsAt,
    downloads: [],
    secondary: WATCH_RELEASES,
    diagnostics,
  };
}

const RELEASE_TAG = "yes-master-v0.9.2-manual-10";
const RELEASE_DOWNLOAD_BASE = `${REPO_URL}/releases/download/${RELEASE_TAG}`;

/**
 * The release the landing page offers. `null` closes the download.
 *
 * ACTIVATED on the owner's instruction after he published
 * `yes-master-v0.9.2-manual-10` and set it as GitHub's latest release
 * (published 2026-11-24T00:10:34Z, built from `6b3c7bb`).
 *
 * - `publishedAt` is the UTC date of GitHub's publication timestamp, because
 *   `resolveRelease` compares dates in UTC.
 * - `betaEndsAt` is that date plus 56 calendar days (owner decision,
 *   September 14).
 * - `verifiedAt` is the day both installers were downloaded from the public
 *   URLs without credentials and their SHA-256 digests matched the release's
 *   `SHA256SUMS.txt`. It does not claim an installed or listening check.
 * - Sizes and digests are those of the published bytes.
 *
 * Do not edit this to "make the button work" or to point at a newer build.
 * U16 owns verification and U17 owns publication; both are owner actions with
 * evidence requirements in docs/plans/beta-go-no-go.md. An agent changing this
 * without the owner's word is an agent shipping a release.
 */
export const RELEASE_METADATA: ReleaseMetadata | null = {
  version: "0.9.2",
  publication: "published",
  releaseUrl: `${REPO_URL}/releases/tag/${RELEASE_TAG}`,
  updaterChannel: "latest",
  publishedAt: "2026-11-24",
  verifiedAt: "2026-11-24",
  betaEndsAt: "2027-01-19",
  artifacts: {
    windowsExe: {
      url: `${RELEASE_DOWNLOAD_BASE}/YES.Master_0.9.2_x64-setup.exe`,
      sizeBytes: 14_562_600,
      sha256:
        "f251f88612e640ab303131de815aeb71635fc1756b09acf7ca89cd3c3b50bb4a",
    },
    macUniversalDmg: {
      url: `${RELEASE_DOWNLOAD_BASE}/YES.Master_0.9.2_universal.dmg`,
      sizeBytes: 34_121_201,
      sha256:
        "00a8d47e33e4689e9dd20c02ec6ed3fca6317d9f94ed299d96ea99e611b1a469",
    },
  },
};

/**
 * The single source of truth for whether a download exists.
 *
 * Pure and total. `now` is injected so staleness is testable without faking
 * clocks, and so a build is never accidentally time-dependent in a way nobody
 * can reproduce.
 */
export function resolveRelease(
  metadata: ReleaseMetadata | null | undefined = RELEASE_METADATA,
  now: Date = new Date(),
): ResolvedRelease {
  if (metadata === null || metadata === undefined) {
    return unavailable("unavailable", "no-release", REASON_NOT_OPEN, [
      "no release metadata is committed",
    ]);
  }

  if (typeof metadata !== "object") {
    return unavailable("unavailable", "malformed", REASON_NOT_OPEN, [
      "release metadata is not an object",
    ]);
  }

  const shape: string[] = [];
  if (!isNonEmptyString(metadata.version)) shape.push("version is missing");
  if (!isRepoReleaseUrl(metadata.releaseUrl)) {
    shape.push("releaseUrl is not a release page in this repository");
  }
  if (!isIsoDate(metadata.publishedAt)) {
    shape.push("publishedAt is not an ISO date");
  }
  const publications: ReleasePublication[] = [
    "draft",
    "prerelease",
    "published",
    "withdrawn",
  ];
  if (!publications.includes(metadata.publication)) {
    shape.push(`publication "${String(metadata.publication)}" is not a known state`);
  }
  if (shape.length > 0) {
    return unavailable("unavailable", "malformed", REASON_NOT_OPEN, shape);
  }

  if (metadata.publication === "withdrawn") {
    // Withdrawn --> Unavailable in the state model; the visitor-facing text
    // differs because someone who downloaded yesterday deserves a truthful
    // "not right now" rather than "never existed".
    return unavailable(
      "withdrawn",
      "withdrawn",
      REASON_WITHDRAWN,
      ["release is withdrawn"],
      metadata.version,
      metadata.betaEndsAt ?? null,
    );
  }

  if (metadata.publication === "draft") {
    return unavailable(
      "draft-proof",
      "draft",
      REASON_NOT_OPEN,
      ["release is a draft; GitHub's /latest channel cannot see drafts"],
      metadata.version,
      metadata.betaEndsAt ?? null,
    );
  }

  // Published or prerelease. Everything below is what "verified" means; any
  // single failure keeps the download closed.
  const diagnostics: string[] = [];

  if (metadata.publication === "prerelease") {
    diagnostics.push(
      "release is flagged prerelease; the updater's /releases/latest channel skips prereleases",
    );
  }
  if (metadata.updaterChannel !== "latest") {
    diagnostics.push(
      `updaterChannel "${String(metadata.updaterChannel)}" is incoherent with the shipped app, which reads /releases/latest`,
    );
  }
  if (!isIsoDate(metadata.betaEndsAt)) {
    // Owner-blocked (docs/OWNER_INPUT_QUEUE.md). Conservative default: no date
    // means no download, because a time-boxed beta with no stated end is a
    // promise we cannot keep.
    diagnostics.push("betaEndsAt is unset; the beta window has no stated end");
  }
  if (!isIsoDate(metadata.verifiedAt)) {
    diagnostics.push("verifiedAt is unset; the deployed artifacts are unverified");
  } else if (
    isIsoDate(metadata.publishedAt) &&
    (metadata.verifiedAt as string) < metadata.publishedAt
  ) {
    diagnostics.push(
      "verifiedAt predates publishedAt; verification covered an earlier build",
    );
  }

  diagnostics.push(
    ...artifactProblems(
      "windowsExe",
      metadata.artifacts?.windowsExe,
      ".exe",
    ),
    ...artifactProblems(
      "macUniversalDmg",
      metadata.artifacts?.macUniversalDmg,
      ".dmg",
    ),
  );

  if (diagnostics.length > 0) {
    return unavailable(
      "candidate-published",
      "unverified",
      REASON_NOT_OPEN,
      diagnostics,
      metadata.version,
      isIsoDate(metadata.betaEndsAt) ? metadata.betaEndsAt : null,
    );
  }

  const today = now.toISOString().slice(0, 10);
  if ((metadata.betaEndsAt as string) < today) {
    // An expired window returns the model to Unavailable, the same terminal the
    // Withdrawn edge leads to. Stale metadata must never keep a download live.
    return unavailable(
      "unavailable",
      "beta-ended",
      REASON_BETA_ENDED,
      ["betaEndsAt is in the past"],
      metadata.version,
      metadata.betaEndsAt as string,
    );
  }

  const windows = metadata.artifacts.windowsExe as ReleaseArtifact;
  const mac = metadata.artifacts.macUniversalDmg as ReleaseArtifact;

  return {
    state: "verified-public",
    available: true,
    reasonCode: "available",
    reason: "",
    version: metadata.version,
    betaEndsAt: metadata.betaEndsAt,
    downloads: [
      {
        id: "windows",
        label: "Download for Windows",
        url: windows.url,
        sizeBytes: windows.sizeBytes,
        sha256: windows.sha256,
      },
      {
        id: "mac",
        label: "Download for Mac",
        url: mac.url,
        sizeBytes: mac.sizeBytes,
        sha256: mac.sha256,
      },
    ],
    secondary: {
      label: "All downloads, checksums, and release notes",
      url: metadata.releaseUrl,
    },
    diagnostics: [],
  };
}

export type DetectedPlatform = PlatformId | "other";

interface NavigatorLike {
  userAgent?: string;
  platform?: string;
  userAgentData?: { platform?: string };
}

/**
 * Best-effort OS guess. It may *emphasise* one action; it must never redirect,
 * hide the other platform, or pick silently — a Mac user on a borrowed Windows
 * laptop still has to be able to find the Mac build.
 */
export function detectPlatform(
  navigatorLike: NavigatorLike | undefined = typeof navigator === "undefined"
    ? undefined
    : navigator,
): DetectedPlatform {
  if (!navigatorLike) return "other";
  const hint = [
    navigatorLike.userAgentData?.platform,
    navigatorLike.platform,
    navigatorLike.userAgent,
  ]
    .filter(isNonEmptyString)
    .join(" ")
    .toLowerCase();

  // iPhone/iPad report "mac"-ish strings; they are not the Mac desktop build.
  if (/iphone|ipad|ipod|android/.test(hint)) return "other";
  if (/win/.test(hint)) return "windows";
  if (/mac|darwin/.test(hint)) return "mac";
  return "other";
}

export function formatArtifactSize(sizeBytes: number): string {
  const mb = sizeBytes / (1024 * 1024);
  if (mb >= 1024) return `${(mb / 1024).toFixed(1)} GB`;
  return `${Math.round(mb)} MB`;
}
