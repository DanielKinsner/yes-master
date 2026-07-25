// Test support for U5. Not imported by any application module, so it never
// reaches the bundle. It lives beside release-config.ts on purpose: there must
// be exactly one definition of "a release that should be downloadable", or the
// tests and the resolver can drift apart while both stay green.
import { REPO_URL, type ReleaseMetadata } from "./release-config";

const DOWNLOAD_BASE = `${REPO_URL}/releases/download/v0.9.1`;

/**
 * A complete, verified, publishable release. Every test that wants a failure
 * starts here and removes exactly one thing, so a failure always names its own
 * cause.
 */
export function validRelease(
  overrides: Partial<ReleaseMetadata> = {},
): ReleaseMetadata {
  return {
    version: "0.9.1",
    publication: "published",
    releaseUrl: `${REPO_URL}/releases/tag/v0.9.1`,
    updaterChannel: "latest",
    publishedAt: "2026-08-01",
    verifiedAt: "2026-08-01",
    betaEndsAt: "2026-10-01",
    artifacts: {
      windowsExe: {
        url: `${DOWNLOAD_BASE}/YES.Master_0.9.1_x64-setup.exe`,
        sizeBytes: 96 * 1024 * 1024,
        sha256: "a".repeat(64),
      },
      macUniversalDmg: {
        url: `${DOWNLOAD_BASE}/YES.Master_0.9.1_universal.dmg`,
        sizeBytes: 118 * 1024 * 1024,
        sha256: "b".repeat(64),
      },
    },
    ...overrides,
  };
}

/** Inside the fixture's beta window, so staleness never fires by accident. */
export const DURING_BETA = new Date("2026-08-15T12:00:00Z");
