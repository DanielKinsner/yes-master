import BetaSignup from "./BetaSignup";
import BetaDownload from "./BetaDownload";

export default function FinalCTA() {
  return (
    <section
      id="get-started"
      className="relative border-t border-white/[0.06] bg-night px-5 py-24 text-center sm:px-8 sm:py-32"
    >
      <div
        aria-hidden="true"
        className="pointer-events-none absolute inset-0 -z-10 bg-[radial-gradient(60%_60%_at_50%_100%,rgba(79,134,247,0.12),transparent_65%)]"
      />
      <div className="mx-auto max-w-2xl">
        <h2 className="font-display text-4xl font-black leading-[1.02] tracking-[-0.03em] sm:text-6xl">
          Stop chasing the master.
        </h2>
        <p className="mt-3 font-display text-xl font-bold text-brand-soft sm:text-2xl">
          This is the one you stop on.
        </p>

        <BetaDownload />

        <div className="mx-auto mt-10 flex max-w-md items-center gap-4 text-xs font-black uppercase tracking-wide text-soft" aria-hidden="true">
          <span className="h-px flex-1 bg-line" />
          Optional email updates
          <span className="h-px flex-1 bg-line" />
        </div>

        <BetaSignup />
        {/* C-22 removed. "Beta testers keep $29 forever" is an entitlement
            promise, not a price: R24 makes the founder window a time-limited
            launch window whose duration and terms are undecided
            (docs/OWNER_INPUT_QUEUE.md row 1). The $29 → $49 model itself is
            settled canon and now lives once, in the beta terms section, rather
            than being restated here as a closing nudge. */}
        <p className="mt-4 text-xs font-semibold text-muted">
          Free during the beta. Mac &amp; Windows.
        </p>
      </div>
    </section>
  );
}
