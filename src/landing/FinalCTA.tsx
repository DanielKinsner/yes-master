import BetaSignup from "./BetaSignup";

export default function FinalCTA() {
  return (
    <section
      id="get-started"
      className="border-t border-white/10 bg-night px-5 py-20 text-center sm:px-8 sm:py-28"
    >
      <div className="mx-auto max-w-2xl">
        <h2 className="font-display text-3xl font-black leading-tight sm:text-5xl">
          Stop chasing the master.
        </h2>
        <p className="mt-3 font-display text-xl font-bold text-brand sm:text-2xl">
          This is the one you stop on.
        </p>

        <BetaSignup />
        <p className="mt-4 text-xs font-semibold text-muted">
          Free during the beta. When it launches it's a one-time $29 (then $49)
          — beta testers keep $29 forever. Mac &amp; Windows.
        </p>
      </div>
    </section>
  );
}
