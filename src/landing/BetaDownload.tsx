import { BETA_DOWNLOAD_URL } from "./release-config";

export default function BetaDownload() {
  return (
    <div className="mx-auto mt-8 max-w-xl">
      <a
        href={BETA_DOWNLOAD_URL}
        target="_blank"
        rel="noreferrer"
        className="inline-flex min-h-12 w-full items-center justify-center rounded-lg bg-gradient-to-b from-cta-light to-cta-deep px-6 py-3 font-extrabold text-[#1c0d00] shadow-lg shadow-cta-deep/25 sm:w-auto"
      >
        Download the free beta
      </a>
      <p className="mt-3 text-sm font-bold text-ink">
        No email required · Windows + universal Mac · Hosted on GitHub Releases
      </p>
      <p className="mx-auto mt-2 max-w-lg text-xs leading-relaxed text-muted">
        Early beta installers are not yet backed by paid publisher certificates,
        so Windows or macOS may ask you to confirm that you trust the download.
      </p>
    </div>
  );
}
