import { useState, type FormEvent } from "react";
import { SIGNUP_ENDPOINT, SIGNUP_FIELD } from "./signup-config";

type Status = "idle" | "submitting" | "success" | "error";

// Optional free-beta email capture. Downloads never depend on this form.
//
// Until SIGNUP_ENDPOINT is set this renders an inert state with a visible
// reason. Three things changed in U6, all of them claims:
//
//   * "Email updates opening soon" said SOON. docs/landing-brief.md bans
//     roadmap words outright, and no provider is even selected yet
//     (docs/OWNER_INPUT_QUEUE.md row 2) — so "opening" was a schedule promise
//     for something that may never open in that form.
//   * The disabled button kept the full CTA gradient, which is the
//     clickable-looking-disabled-control pattern U5's acceptance forbids. It
//     sat directly beneath a correctly-inert download button, so the page
//     contradicted itself about what "unavailable" looks like.
//   * The success state promised to "save your founder price" — the C-22
//     entitlement whose terms are undecided. Unreachable while unwired, but
//     it would have gone live the moment U4 wired a provider.
export default function BetaSignup() {
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
    return (
      <p className="mx-auto mt-8 max-w-md text-lg font-bold text-good">
        You're on the list.
      </p>
    );
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
          className="w-full rounded-lg border border-white/15 bg-white/5 px-4 py-3 text-ink placeholder:text-muted focus:border-brand focus:outline-none disabled:opacity-60"
        />
        <button
          type="submit"
          disabled={notWired || status === "submitting"}
          aria-describedby={notWired ? "beta-signup-reason" : undefined}
          className={
            notWired
              ? "inline-flex shrink-0 cursor-not-allowed items-center justify-center rounded-lg border border-white/15 bg-white/5 px-6 py-3 font-extrabold text-muted"
              : "inline-flex shrink-0 items-center justify-center rounded-lg bg-gradient-to-b from-cta-light to-cta-deep px-6 py-3 font-extrabold text-[#1c0d00] disabled:opacity-60"
          }
        >
          {notWired
            ? "Email updates"
            : status === "submitting"
              ? "Joining…"
              : "Get beta updates"}
        </button>
      </form>
      {notWired && (
        <p id="beta-signup-reason" className="mt-2 text-sm text-muted">
          Email updates are not open. The download does not depend on this.
        </p>
      )}
      {status === "error" && (
        <p className="mt-2 text-sm text-[#ff9a8b]">
          Something went wrong — try again, or email hello@yesmaster.app.
        </p>
      )}
    </div>
  );
}
