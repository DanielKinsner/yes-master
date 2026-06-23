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

        <a
          href="mailto:hello@yesmaster.app"
          className="mt-8 inline-flex flex-col items-center rounded-xl bg-gradient-to-b from-cta-light to-cta-deep px-8 py-4 font-extrabold text-[#1c0d00] shadow-xl shadow-cta-deep/30"
        >
          <span className="font-display text-lg">Download YES Master</span>
          <span className="text-xs font-semibold text-[#1c0d00]/70">
            Works offline. No signup.
          </span>
        </a>
      </div>
    </section>
  );
}
