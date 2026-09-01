import { useState, type FormEvent } from "react";
import {
  INSTALL_GUIDE_URL,
  detectPlatform,
  formatArtifactSize,
  resolveRelease,
  type DetectedPlatform,
  type ResolvedRelease,
} from "../landing/release-config";
import { SIGNUP_ENDPOINT, SIGNUP_FIELD } from "../landing/signup-config";

// The action. Download state is resolved, never hard-coded (U5): every path
// that is not a verified public release renders an inert control with its
// reason printed beside it, and an inert control shares nothing visual with
// a live one. The email form is optional and the download never depends on it.

function Download({
  release,
  platform,
}: {
  release: ResolvedRelease;
  platform: DetectedPlatform;
}) {
  return (
    <div className="mt-10" data-release-state={release.state}>
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
                className={`btn-ink w-full hover:btn-ink-hover sm:w-auto ${likely ? "ring-2 ring-accent ring-offset-2 ring-offset-paper-deep" : ""}`}
              >
                {download.label}
                <span className="text-[0.8rem] font-medium opacity-70">
                  {formatArtifactSize(download.sizeBytes)}
                </span>
                {likely ? (
                  <span className="sr-only">— this matches the system you are browsing from</span>
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
            className="btn-inert w-full sm:w-auto"
          >
            Download the free beta
          </button>
          <p id="beta-download-reason" role="status" className="text-[0.95rem] font-semibold text-ink">
            {release.reason}
          </p>
        </div>
      )}

      <p className="mt-5 text-[0.9rem] font-semibold text-ink">
        No email required · Windows + universal Mac · Hosted on GitHub Releases
      </p>
      <p className="mx-auto mt-2 max-w-lg text-[0.8rem] leading-[1.6] text-ink-3">
        Desktop only — there is no Linux build.
      </p>
      <p className="mx-auto mt-2 max-w-lg text-[0.8rem] leading-[1.6] text-ink-3">
        Early beta installers are not yet backed by paid publisher certificates,
        so Windows or macOS may ask you to confirm that you trust the download.{" "}
        <a
          href={INSTALL_GUIDE_URL}
          target="_blank"
          rel="noreferrer"
          className="font-semibold text-ink underline decoration-rule-strong underline-offset-4 hover:decoration-ink"
        >
          Install help
        </a>
      </p>
      <p className="mt-2">
        <a
          href={release.secondary.url}
          target="_blank"
          rel="noreferrer"
          className="inline-flex min-h-11 items-center justify-center px-3 py-2 text-[0.8rem] font-semibold text-ink-2 underline decoration-rule-strong underline-offset-4 hover:text-ink"
        >
          {release.secondary.label}
        </a>
      </p>
    </div>
  );
}

type Status = "idle" | "submitting" | "success" | "error";

function Signup() {
  const [email, setEmail] = useState("");
  const [status, setStatus] = useState<Status>("idle");
  const notWired = SIGNUP_ENDPOINT === "";

  async function onSubmit(e: FormEvent) {
    e.preventDefault();
    if (notWired || status === "submitting") return;
    setStatus("submitting");
    try {
      const res = await fetch(SIGNUP_ENDPOINT, {
        method: "POST",
        headers: { "Content-Type": "application/x-www-form-urlencoded" },
        body: `${SIGNUP_FIELD}=${encodeURIComponent(email)}`,
      });
      setStatus(res.ok ? "success" : "error");
    } catch {
      setStatus("error");
    }
  }

  if (status === "success") {
    return <p className="mx-auto mt-8 max-w-md text-[1.05rem] font-semibold text-pass">You're on the list.</p>;
  }

  return (
    <div className="mx-auto mt-8 w-full max-w-md">
      <form onSubmit={onSubmit} className="flex w-full flex-col gap-3 sm:flex-row">
        <input
          type="email"
          required
          value={email}
          onChange={(e) => setEmail(e.target.value)}
          placeholder="you@email.com"
          disabled={notWired || status === "submitting"}
          aria-label="Email address"
          className="min-h-12 w-full rounded-full border border-rule-strong bg-paper-white px-5 text-ink placeholder:text-ink-3 focus:border-ink focus:outline-none disabled:opacity-60"
        />
        <button
          type="submit"
          disabled={notWired || status === "submitting"}
          aria-describedby={notWired ? "beta-signup-reason" : undefined}
          className={notWired ? "btn-inert shrink-0" : "btn-line shrink-0 hover:btn-line-hover disabled:opacity-60"}
        >
          {notWired ? "Email updates" : status === "submitting" ? "Joining…" : "Get beta updates"}
        </button>
      </form>
      {notWired && (
        <p id="beta-signup-reason" className="mt-2 text-[0.85rem] text-ink-3">
          Email updates are not open. The download does not depend on this.
        </p>
      )}
      {status === "error" && (
        <p className="mt-2 text-[0.85rem] text-[#b3402f]">
          Something went wrong — try again, or email hello@yesmaster.app.
        </p>
      )}
    </div>
  );
}

export default function GetStarted({
  release = resolveRelease(),
  platform = detectPlatform(),
}: { release?: ResolvedRelease; platform?: DetectedPlatform } = {}) {
  return (
    <section id="get-started" className="border-t border-rule bg-paper-deep px-5 py-24 text-center sm:px-8 sm:py-32">
      <div className="mx-auto max-w-3xl">
        <p className="eyebrow">06 / The action</p>
        <h2 className="mt-5 font-display text-[clamp(2.75rem,1.6rem+5.5vw,5.5rem)] leading-[0.98] tracking-[-0.02em] text-ink [text-wrap:balance]">
          Stop chasing the master.
        </h2>
        <p className="mt-4 font-display text-[1.5rem] leading-tight text-ink-2 sm:text-[1.8rem]">
          This is the one you stop on.
        </p>

        <Download release={release} platform={platform} />

        <div className="mx-auto mt-12 flex max-w-md items-center gap-4 text-[0.7rem] font-semibold uppercase tracking-[0.16em] text-ink-3" aria-hidden="true">
          <span className="h-px flex-1 bg-rule" />
          Optional email updates
          <span className="h-px flex-1 bg-rule" />
        </div>

        <Signup />

        <p className="mt-5 text-[0.85rem] font-medium text-ink-2">Free during the beta. Mac &amp; Windows.</p>
      </div>
    </section>
  );
}
