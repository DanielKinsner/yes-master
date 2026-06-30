import { useState, type FormEvent } from "react";
import { SIGNUP_ENDPOINT, SIGNUP_FIELD } from "./signup-config";

type Status = "idle" | "submitting" | "success" | "error";

// Free-beta email capture. POSTs the address to the configured provider
// endpoint. Until SIGNUP_ENDPOINT is set it renders a disabled "opening soon"
// state, so the landing page is shippable before the mailing list exists.
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
        You're on the list — we'll email your download link.
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
          className="inline-flex shrink-0 items-center justify-center rounded-lg bg-gradient-to-b from-cta-light to-cta-deep px-6 py-3 font-extrabold text-[#1c0d00] disabled:opacity-60"
        >
          {notWired
            ? "Sign-up opening soon"
            : status === "submitting"
              ? "Joining…"
              : "Join the free beta"}
        </button>
      </form>
      {status === "error" && (
        <p className="mt-2 text-sm text-[#ff9a8b]">
          Something went wrong — try again, or email hello@yesmaster.app.
        </p>
      )}
    </div>
  );
}
