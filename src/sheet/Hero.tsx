import type { CSSProperties } from "react";
import { resolveRelease, type ResolvedRelease } from "../landing/release-config";
import Receipt from "./Receipt";

const delay = (ms: number) => ({ "--d": `${ms}ms` } as CSSProperties);

// The headline is one sentence in two voices: the category in the sans, the
// promise in the serif. The receipt sits beside it because a page about a
// tool that shows its work should lead with the work being shown.
export default function Hero({
  release = resolveRelease(),
}: { release?: ResolvedRelease } = {}) {
  return (
    <section id="top" className="relative overflow-hidden px-5 pb-16 pt-14 sm:px-8 sm:pt-20 lg:pb-24 lg:pt-24">
      <div className="mx-auto grid max-w-6xl gap-12 lg:grid-cols-12 lg:items-center lg:gap-8">
        <div className="lg:col-span-7">
          <p className="eyebrow rise" style={delay(0)}>
            Desktop mastering · Windows and macOS
          </p>

          <h1 className="mt-6 text-ink">
            <span
              className="rise block font-sans text-[1.35rem] font-semibold leading-tight tracking-[-0.01em] text-ink-2 sm:text-[1.6rem]"
              style={delay(80)}
            >
              One-click mastering.
            </span>
            <span
              className="rise mt-2 block font-display text-[clamp(3.4rem,2rem+7.5vw,7rem)] leading-[0.95] tracking-[-0.02em] [text-wrap:balance]"
              style={delay(160)}
            >
              Your Endgame Sound.
            </span>
          </h1>

          <p
            className="rise mt-7 max-w-[34rem] text-[1.05rem] leading-[1.6] text-ink-2 sm:text-[1.15rem]"
            style={delay(260)}
          >
            Drop a track, pick a sound, and hear the full mastering chain as you
            listen. It reads what your mix already has and eases its own moves to
            fit. No upload, no waiting, no black box.
          </p>

          <div className="rise mt-9 flex flex-col gap-3 sm:flex-row sm:items-center" style={delay(340)}>
            <a href="#get-started" className="btn-ink hover:btn-ink-hover">
              {release.available ? "Download free beta" : "About the free beta"}
            </a>
            <a href="#advanced" className="btn-line hover:btn-line-hover">
              See the full console
            </a>
          </div>
        </div>

        <div className="flex justify-center lg:col-span-5 lg:justify-end lg:pr-6">
          <Receipt className="settle" style={delay(420)} />
        </div>
      </div>
    </section>
  );
}
