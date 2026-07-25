// U6 — character controls and the adaptive-restraint pitch.
//
// Two things this section exists to say precisely, because both are easy to
// overstate into a claim the product cannot back:
//
//   1. Presets are starting points, not genre machines. Every one of them
//      runs through the same loudness and safety stages, so picking a bolder
//      character does not mean opting out of the checks.
//   2. Adaptation is RESTRAINT. The shipped Tier-1 guardrails are reduce-only
//      (docs/PRODUCT.md "Adaptive Mastering") — they ease a preset's moves
//      when the material is already close. It is not an amateur-rescue
//      feature and must never be pitched as one; plenty of tracks arrive
//      already processed on purpose.
const styles = [
  {
    name: "Universal",
    body: "The neutral one. Tightens and lands the loudness without changing the character of the mix.",
  },
  {
    name: "Clarity",
    body: "Opens the top and cleans the low end when a mix reads dull or crowded.",
  },
  {
    name: "Tape",
    body: "Softer transients and a warmer weight, for material that wants to sound less digital.",
  },
  {
    name: "Oomph",
    body: "Low-end confidence and front-to-back push, for tracks that need to feel bigger.",
  },
];

export default function SoundCharacter() {
  return (
    <section
      id="sound"
      className="border-t border-white/10 bg-night px-5 py-20 sm:px-8 sm:py-24"
    >
      <div className="mx-auto max-w-6xl">
        <div className="mx-auto max-w-2xl text-center">
          <p className="text-xs font-black uppercase tracking-wide text-brand-soft">
            Character, then restraint
          </p>
          <h2 className="mt-4 font-display text-3xl font-black leading-tight sm:text-5xl">
            It reads the track before it touches it.
          </h2>
          <p className="mt-4 text-muted sm:text-lg">
            YES Master measures what your mix already has, adds impact where the
            material can take it, and eases its own moves when the track is
            already close. Bring something you have already processed and it
            works with what is there rather than fighting it.
          </p>
        </div>

        <div className="mt-12 grid gap-4 sm:grid-cols-2 lg:grid-cols-4">
          {styles.map((style) => (
            <article
              key={style.name}
              className="rounded-xl border border-line bg-card/60 p-5"
            >
              <h3 className="font-display text-lg font-black text-ink">
                {style.name}
              </h3>
              <p className="mt-2 text-sm leading-relaxed text-muted">
                {style.body}
              </p>
            </article>
          ))}
        </div>

        <div className="mx-auto mt-10 max-w-3xl rounded-2xl border border-line bg-card/50 p-6 text-center">
          <p className="text-sm leading-relaxed text-muted">
            <strong className="font-extrabold text-ink">
              Styles are starting points, not lanes.
            </strong>{" "}
            Every one of them runs through the same loudness landing and the
            same safety stages, and Intensity moves any of them from subtle to
            pushed. Choosing a bolder character never means switching the checks
            off — the app measures what happened either way and tells you.
          </p>
        </div>
      </div>
    </section>
  );
}
