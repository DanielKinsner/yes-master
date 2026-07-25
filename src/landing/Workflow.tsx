// U6 — the workflow section. Sits directly under the hero because the first
// question a visitor has after "what is it" is "what do I actually do", and
// answering it with three concrete decisions is more persuasive than another
// round of adjectives.
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
      className="border-t border-white/10 bg-night px-5 py-20 sm:px-8 sm:py-24"
    >
      <div className="mx-auto max-w-6xl">
        <div className="mx-auto max-w-2xl text-center">
          <p className="text-xs font-black uppercase tracking-wide text-brand-soft">
            Three decisions
          </p>
          <h2 className="mt-4 font-display text-3xl font-black leading-tight sm:text-5xl">
            Finished mix in. Finished master out.
          </h2>
          <p className="mt-4 text-muted sm:text-lg">
            No session to set up, no chain to build, no queue to wait in.
          </p>
        </div>

        <ol className="mt-12 grid gap-5 md:grid-cols-3">
          {steps.map((item) => (
            <li
              key={item.step}
              className="rounded-2xl border border-line bg-card/70 p-6"
            >
              <p className="font-display text-3xl font-black text-brand-soft">
                {item.step}
              </p>
              <h3 className="mt-3 font-display text-xl font-black">
                {item.title}
              </h3>
              <p className="mt-2 text-sm leading-relaxed text-muted">
                {item.body}
              </p>
            </li>
          ))}
        </ol>

        {/* Audience, in the settled order: independent artists finishing their
            own records first, emerging creators second, working engineers
            third. Stated as who it fits, never as what anyone lacks. */}
        <p className="mx-auto mt-10 max-w-2xl text-center text-sm leading-relaxed text-muted">
          Built for independent artists finishing their own records, for
          creators releasing their first few tracks, and for engineers who want
          the whole console when the track asks for it.
        </p>
      </div>
    </section>
  );
}
