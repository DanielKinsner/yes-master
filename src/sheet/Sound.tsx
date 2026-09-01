import type { CSSProperties } from "react";
import universalArt from "../assets/presets/universal.png";
import clarityArt from "../assets/presets/clarity.png";
import tapeArt from "../assets/presets/tape.png";
import oomphArt from "../assets/presets/oomph.png";

// The four Standard styles, as a specimen list rather than four boxes. The
// tile is the actual dark tile you click in the app; the accent bar is the
// app's per-style hue. Two claims are kept exact on purpose: styles are
// starting points under one safety system, and adaptation is restraint, not
// rescue (docs/PRODUCT.md "Adaptive Mastering").
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

export default function Sound() {
  return (
    <section id="sound" className="border-t border-rule bg-paper-deep/50 px-5 py-20 sm:px-8 sm:py-28">
      <div className="mx-auto max-w-6xl">
        <div className="grid gap-8 lg:grid-cols-12">
          <div className="lg:col-span-5">
            <p className="eyebrow">03 / Character, then restraint</p>
            <h2 className="headline mt-4">It reads the track before it touches it.</h2>
          </div>
          <p className="max-w-[30rem] text-[1.05rem] leading-[1.6] text-ink-2 lg:col-span-6 lg:col-start-7 lg:pt-2">
            YES Master measures what your mix already has, adds impact where the
            material can take it, and eases its own moves when the track is
            already close. Bring something you have already processed and it
            works with what is there rather than fighting it.
          </p>
        </div>

        <ul className="mt-14 border-t border-rule">
          {styles.map((style) => (
            <li
              key={style.name}
              className="grid grid-cols-[4.5rem_1fr] items-center gap-5 border-b border-rule py-5 sm:grid-cols-[5rem_11rem_1fr_3rem] sm:gap-8"
              style={{ "--swatch": style.accent } as CSSProperties}
            >
              <span className="flex h-[4.5rem] w-[4.5rem] items-center justify-center rounded-lg bg-screen shadow-[inset_0_1px_0_rgba(255,255,255,0.06)] sm:h-20 sm:w-20">
                <img
                  src={style.art}
                  alt=""
                  aria-hidden="true"
                  width={128}
                  height={128}
                  loading="lazy"
                  decoding="async"
                  className="h-12 w-12 object-contain sm:h-14 sm:w-14"
                />
              </span>
              <h3 className="font-display text-[1.9rem] leading-none text-ink sm:text-[2.2rem]">
                {style.name}
              </h3>
              <p className="col-span-2 max-w-[44ch] text-[0.95rem] leading-[1.6] text-ink-2 sm:col-span-1">
                {style.body}
              </p>
              <span
                aria-hidden="true"
                className="hidden h-10 w-1.5 justify-self-end rounded-full sm:block"
                style={{ background: "var(--swatch)" }}
              />
            </li>
          ))}
        </ul>

        <p className="mt-10 max-w-[44rem] text-[0.95rem] leading-[1.6] text-ink-2">
          <strong className="font-semibold text-ink">Styles are starting points, not lanes.</strong>{" "}
          Every one of them runs through the same loudness landing and the same
          safety stages, and Intensity moves any of them from subtle to pushed.
          Choosing a bolder character never means switching the checks off — the
          app measures what happened either way and tells you.
        </p>
      </div>
    </section>
  );
}
