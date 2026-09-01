// What you actually do. Three decisions, printed the way a sheet prints them:
// big numerals, short instructions.
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
    <section id="how" className="px-5 py-20 sm:px-8 sm:py-28">
      <div className="mx-auto max-w-6xl">
        <div className="grid gap-8 lg:grid-cols-12">
          <div className="lg:col-span-5">
            <p className="eyebrow">01 / Three decisions</p>
            <h2 className="headline mt-4">
              <span className="block">Finished mix in.</span>
              <span className="block">Finished master out.</span>
            </h2>
          </div>
          <p className="max-w-[30rem] text-[1.05rem] leading-[1.6] text-ink-2 lg:col-span-6 lg:col-start-7 lg:pt-2">
            No session to set up, no chain to build, no queue to wait in.
          </p>
        </div>

        <ol className="mt-14 grid gap-10 border-t border-rule pt-10 md:grid-cols-3 md:gap-8">
          {steps.map((item) => (
            <li key={item.step} className="grid grid-cols-[3.5rem_1fr] gap-4 md:block">
              <span className="font-display text-[3.25rem] leading-none text-ink-3 md:text-[4.5rem]">
                {item.step}
              </span>
              <div className="md:mt-5">
                <h3 className="text-[1.15rem] font-semibold text-ink">{item.title}</h3>
                <p className="mt-2 max-w-[32ch] text-[0.95rem] leading-[1.6] text-ink-2">
                  {item.body}
                </p>
              </div>
            </li>
          ))}
        </ol>

        {/* Audience, in the settled order: independent artists first, emerging
            creators second, working engineers third. Who it fits, never what
            anyone lacks. */}
        <p className="mt-16 max-w-[44rem] border-t border-rule pt-6 text-[0.95rem] leading-[1.6] text-ink-2">
          Built for independent artists finishing their own records, for creators
          releasing their first few tracks, and for engineers who want the whole
          console when the track asks for it.
        </p>
      </div>
    </section>
  );
}
