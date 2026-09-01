// U6 — the workflow section. Sits directly under the hero because the first
// question a visitor has after "what is it" is "what do I actually do", and
// answering it with three concrete decisions is more persuasive than another
// round of adjectives.
//
// Presentation (2026-08-18): a three-beat timeline — big numerals on a
// hairline rail — instead of three identical boxes. Same words.
const steps = [
  {
    step: "01",
    title: "Drop the track in",
    body: "Import your finished mix. Analysis runs on your machine and the waveform is playing while you decide.",
  },
  {
    step: "02",
    title: "Make three decisions",
    body: "A style, a loudness, and how hard to push it. The whole chain re-runs live as you move, so you are choosing by ear, not by guessing.",
  },
  {
    step: "03",
    title: "Create Master",
    body: "You get a 44.1 kHz / 24-bit WAV with the limiter's ceiling at −1 dBTP, plus a receipt measuring what the render actually did.",
  },
];

export default function Workflow() {
  return (
    <section
      id="how"
      className="relative border-t border-white/[0.06] bg-night px-5 py-20 sm:px-8 sm:py-28"
    >
      {/* Ambient: a faint cobalt wash so the section isn't flat black. */}
      <div
        aria-hidden="true"
        className="pointer-events-none absolute inset-0 -z-10 bg-[radial-gradient(70%_50%_at_50%_0%,rgba(79,134,247,0.08),transparent_60%)]"
      />
      <div className="mx-auto max-w-6xl">
        <div className="max-w-2xl">
          <p className="eyebrow">Three decisions</p>
          {/* Two sentences, one line each (owner, 2026-09-01): the natural wrap
              broke after the second "Finished", which reads as a stutter. */}
          <h2 className="mt-4 font-display text-3xl font-black leading-[1.02] tracking-[-0.02em] sm:text-5xl">
            <span className="block sm:whitespace-nowrap">Finished mix in.</span>
            <span className="block sm:whitespace-nowrap">Finished master out.</span>
          </h2>
          <p className="mt-4 text-muted sm:text-lg">
            No session to set up, no chain to build, no queue to wait in.
          </p>
        </div>

        <ol className="relative mt-14 grid gap-10 md:grid-cols-3 md:gap-8">
          {/* The rail the three beats sit on (desktop only). */}
          <span
            aria-hidden="true"
            className="absolute left-0 right-0 top-[1.15rem] hidden h-px bg-gradient-to-r from-brand/60 via-line to-transparent md:block"
          />
          {steps.map((item) => (
            <li key={item.step} className="relative md:pt-10">
              <span
                aria-hidden="true"
                className="absolute left-0 top-0 hidden h-[2.3rem] w-[2.3rem] items-center justify-center rounded-full border border-brand/40 bg-night text-xs font-black text-brand-soft shadow-[0_0_0_6px_#07080c,0_0_24px_rgba(79,134,247,0.35)] md:flex"
              >
                {item.step}
              </span>
              <p className="font-display text-4xl font-black tracking-[-0.03em] text-brand-soft md:hidden">
                {item.step}
              </p>
              <h3 className="mt-3 font-display text-xl font-black md:mt-0">
                {item.title}
              </h3>
              <p className="mt-2 max-w-[34ch] text-[0.95rem] leading-relaxed text-muted">
                {item.body}
              </p>
            </li>
          ))}
        </ol>

        {/* Audience, in the settled order: independent artists finishing their
            own records first, emerging creators second, working engineers
            third. Stated as who it fits, never as what anyone lacks. */}
        <p className="mt-14 max-w-2xl text-sm leading-relaxed text-muted">
          Built for independent artists finishing their own records, for
          creators releasing their first few tracks, and for engineers who want
          the whole console when the track asks for it.
        </p>
      </div>
    </section>
  );
}
