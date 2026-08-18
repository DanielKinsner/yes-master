import universalArt from "../assets/presets/universal.png";
import clarityArt from "../assets/presets/clarity.png";
import tapeArt from "../assets/presets/tape.png";
import oomphArt from "../assets/presets/oomph.png";

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
//
// Presentation (2026-08-18): the four Standard styles use the SAME artwork
// and accent wash the app's style tiles use, so the page shows the thing you
// will click rather than describing it. `accent` mirrors the per-preset
// --tile-accent hue in the app.
const styles = [
  {
    name: "Universal",
    art: universalArt,
    accent: "#4f86f7",
    body: "The neutral one. Tightens and lands the loudness without changing the character of the mix.",
  },
  {
    name: "Clarity",
    art: clarityArt,
    accent: "#5fd0d8",
    body: "Opens the top and cleans the low end when a mix reads dull or crowded.",
  },
  {
    name: "Tape",
    art: tapeArt,
    accent: "#e6a24a",
    body: "Softer transients and a warmer weight, for material that wants to sound less digital.",
  },
  {
    name: "Oomph",
    art: oomphArt,
    accent: "#ef5b4f",
    body: "Low-end confidence and front-to-back push, for tracks that need to feel bigger.",
  },
];

export default function SoundCharacter() {
  return (
    <section
      id="sound"
      className="relative border-t border-white/[0.06] bg-night px-5 py-20 sm:px-8 sm:py-28"
    >
      <div className="mx-auto max-w-6xl">
        <div className="max-w-2xl">
          <p className="eyebrow">Character, then restraint</p>
          <h2 className="mt-4 font-display text-3xl font-black leading-[1.02] tracking-[-0.02em] sm:text-5xl">
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
              className="surface-card group relative overflow-hidden rounded-2xl p-5 pt-6 transition-transform duration-200 ease-[cubic-bezier(.2,.8,.2,1)] hover:-translate-y-1"
              style={{ "--tile-accent": style.accent } as React.CSSProperties}
            >
              {/* The same radial accent wash the app's tiles carry. */}
              <span
                aria-hidden="true"
                className="pointer-events-none absolute inset-0 -z-0 opacity-70 transition-opacity duration-200 group-hover:opacity-100"
                style={{
                  background:
                    "radial-gradient(circle at 50% 22%, color-mix(in srgb, var(--tile-accent) 26%, transparent), transparent 58%)",
                }}
              />
              <img
                src={style.art}
                alt=""
                aria-hidden="true"
                width={128}
                height={128}
                loading="lazy"
                decoding="async"
                className="relative mx-auto block h-24 w-24 object-contain mix-blend-screen drop-shadow-[0_0_18px_color-mix(in_srgb,var(--tile-accent)_45%,transparent)] transition-transform duration-200 group-hover:scale-105 sm:h-28 sm:w-28"
              />
              <h3 className="relative mt-4 text-center font-display text-lg font-black text-ink">
                {style.name}
              </h3>
              <p className="relative mt-2 text-center text-sm leading-relaxed text-muted">
                {style.body}
              </p>
            </article>
          ))}
        </div>

        <p className="mx-auto mt-12 max-w-3xl text-center text-[0.95rem] leading-relaxed text-muted">
          <strong className="font-extrabold text-ink">
            Styles are starting points, not lanes.
          </strong>{" "}
          Every one of them runs through the same loudness landing and the
          same safety stages, and Intensity moves any of them from subtle to
          pushed. Choosing a bolder character never means switching the checks
          off — the app measures what happened either way and tells you.
        </p>
      </div>
    </section>
  );
}
