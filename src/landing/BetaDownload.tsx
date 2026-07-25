import {
  INSTALL_GUIDE_URL,
  detectPlatform,
  formatArtifactSize,
  resolveRelease,
  type DetectedPlatform,
  type ResolvedRelease,
} from "./release-config";

// U5. The download is rendered from resolved release state, never from a
// hard-coded URL. `release` and `platform` are props purely so tests can drive
// every state without stubbing globals; production always uses the defaults.
interface BetaDownloadProps {
  release?: ResolvedRelease;
  platform?: DetectedPlatform;
}

const CTA_CLASSES =
  "inline-flex min-h-12 w-full items-center justify-center gap-2 rounded-lg bg-gradient-to-b from-cta-light to-cta-deep px-6 py-3 font-extrabold text-[#1c0d00] shadow-lg shadow-cta-deep/25 sm:w-auto";

// Deliberately shares no gradient, shadow, or CTA colour with the live button.
// An inactive control that looks clickable is the failure this unit exists to
// prevent; a tooltip is not a substitute for saying why on screen.
const INACTIVE_CLASSES =
  "inline-flex min-h-12 w-full cursor-not-allowed items-center justify-center rounded-lg border border-white/15 bg-white/5 px-6 py-3 font-extrabold text-muted sm:w-auto";

export default function BetaDownload({
  release = resolveRelease(),
  platform = detectPlatform(),
}: BetaDownloadProps = {}) {
  return (
    <div className="mx-auto mt-8 max-w-xl" data-release-state={release.state}>
      {release.available ? (
        <div className="flex flex-col items-center gap-3 sm:flex-row sm:justify-center">
          {release.downloads.map((download) => {
            const likely = download.id === platform;
            return (
              <a
                key={download.id}
                href={download.url}
                data-platform={download.id}
                target="_blank"
                rel="noreferrer"
                className={
                  likely ? `${CTA_CLASSES} ring-2 ring-brand` : CTA_CLASSES
                }
              >
                {download.label}
                <span className="text-xs font-bold opacity-70">
                  {formatArtifactSize(download.sizeBytes)}
                </span>
                {likely ? (
                  <span className="sr-only">
                    — this matches the system you are browsing from
                  </span>
                ) : null}
              </a>
            );
          })}
        </div>
      ) : (
        <div className="flex flex-col items-center gap-3">
          <button
            type="button"
            disabled
            aria-describedby="beta-download-reason"
            className={INACTIVE_CLASSES}
          >
            Download the free beta
          </button>
          <p
            id="beta-download-reason"
            role="status"
            className="text-sm font-bold text-ink"
          >
            {release.reason}
          </p>
        </div>
      )}

      <p className="mt-3 text-sm font-bold text-ink">
        No email required · Windows + universal Mac · Hosted on GitHub Releases
      </p>

      <p className="mx-auto mt-2 max-w-lg text-xs leading-relaxed text-muted">
        There is no Linux build, and iPhone and Android are not currently
        available.
      </p>

      <p className="mx-auto mt-2 max-w-lg text-xs leading-relaxed text-muted">
        Early beta installers are not yet backed by paid publisher certificates,
        so Windows or macOS may ask you to confirm that you trust the download.{" "}
        <a
          href={INSTALL_GUIDE_URL}
          target="_blank"
          rel="noreferrer"
          className="font-bold text-ink underline"
        >
          Install help
        </a>
      </p>

      <p className="mt-2 text-xs font-semibold text-muted">
        <a
          href={release.secondary.url}
          target="_blank"
          rel="noreferrer"
          className="underline"
        >
          {release.secondary.label}
        </a>
      </p>
    </div>
  );
}
