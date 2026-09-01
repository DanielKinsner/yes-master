import heroBg from "../assets/landing/hero-control-room-studio.jpg";
import heroBg1280 from "../assets/landing/hero-control-room-studio-1280.jpg";
import iconLocal from "../assets/landing/icon-local-first.png";
import iconRealtime from "../assets/landing/icon-realtime.png";
import iconRelease from "../assets/landing/icon-release-ready.png";
import { resolveRelease, type ResolvedRelease } from "./release-config";

// The three proof points. Edit the words here; the layout stays the same.
const points = [
  {
    icon: iconRealtime,
    title: "Real-time, every tweak",
    body: "The full chain runs as you listen — no upload, no reprocessing wait.",
  },
  {
    icon: iconLocal,
    title: "Simple by default, deep when you want it",
    body: "Master in one move, or open the full metering and album tools when you're ready.",
  },
  {
    icon: iconRelease,
    title: "No black box",
    body: "A pass/fail receipt shows your LUFS, true-peak, and dynamic range — push it hard and still see the truth.",
  },
];

// The hero CTA is an in-page anchor, so it can never be a dead download — but
// a button reading "Download free beta" while the download is closed is still
// a promise the next section has to take back. U5 made availability real
// state; the label follows it.
export default function Hero({
  release = resolveRelease(),
}: { release?: ResolvedRelease } = {}) {
  return (
    <section id="top" className="relative isolate min-h-svh overflow-hidden">
      {/* Background: the owner-generated 4K console render of the real UI
          (2026-08-18). object-cover crops, never stretches; anchored LEFT so
          the dark studio wall stays behind the copy and the crop, when there
          is one, takes the right-hand VU meters rather than the console. */}
      {/* U7: two variants so a phone does not download the full-width render
          to draw it 390px wide. The master art lives beside these as
          *-source.jpg, imported by nothing and shipped in no bundle. */}
      <img
        src={heroBg}
        srcSet={`${heroBg1280} 1280w, ${heroBg} 2560w`}
        sizes="100vw"
        alt=""
        aria-hidden="true"
        fetchPriority="high"
        className="absolute inset-0 -z-20 h-full w-full object-cover object-[38%_center] lg:object-[22%_center]"
      />
      {/* Readability veils. Phones: a bottom-up scrim (text sits over the
          whole image there). Desktop: a left-to-right veil that backs the copy
          column and fades out before the console, plus a floor fade into the
          page so the section hands off cleanly. */}
      <div
        aria-hidden="true"
        className="absolute inset-0 -z-10 bg-gradient-to-t from-night/95 via-night/60 to-night/40 lg:hidden"
      />
      <div
        aria-hidden="true"
        className="absolute inset-0 -z-10 hidden bg-gradient-to-r from-night via-night/70 via-35% to-transparent to-62% lg:block"
      />
      <div
        aria-hidden="true"
        className="absolute inset-x-0 bottom-0 -z-10 h-40 bg-gradient-to-t from-night to-transparent"
      />

      {/* Copy column. A normal block (so its width can never exceed the
          screen), full width on phones, ~45% on desktop, sitting over the
          dark wall to the left of the console. */}
      <div className="relative flex min-h-svh w-full flex-col justify-start gap-5 px-5 pt-24 pb-16 sm:px-8 lg:w-[48%] lg:max-w-[600px] lg:pb-16 lg:pl-16 lg:pt-28 xl:max-w-[640px] xl:pl-24">
        <span className="eyebrow">YES Master</span>

        {/* Two beats, one line each. At lg+ the sentences are pinned with
            whitespace-nowrap so the headline can never rag into the orphaned
            "mastering." / "Sound." it produced at the old 3.5rem — the type is
            sized (2.3rem @ lg, 2.85rem @ xl) to keep 5-10% headroom on the
            longer line at every width from 1024 up, measured in the WIDEST
            fallback face, so a narrower Segoe/SF only gains slack. On
            phones it wraps normally with text-balance. The YES acronym is
            carried by title case alone — a colour shift and a glow were both
            tried and both announced it. */}
        <h1 className="text-balance font-display text-[2.4rem] font-black leading-[1.02] tracking-[-0.03em] sm:text-[3rem] lg:whitespace-nowrap lg:text-[2.3rem] xl:text-[2.85rem]">
          <span className="block">One-click mastering.</span>
          <span className="mt-1 block text-brand-soft">Your Endgame Sound.</span>
        </h1>

        <p className="max-w-[440px] text-base leading-snug text-muted lg:text-[1.05rem]">
          Drop a track, pick a sound, and hear the full mastering chain as you
          listen. It reads what your mix already has and eases its own moves to
          fit. No upload, no waiting, no black box.
        </p>

        <ul className="flex flex-col gap-3">
          {points.map((point) => (
            <li key={point.title} className="relative max-w-[400px] pl-[54px]">
              <img
                src={point.icon}
                alt=""
                aria-hidden="true"
                className="absolute left-0 top-0 h-10 w-10"
              />
              <p className="text-[0.9rem] leading-snug text-muted">
                <strong className="font-extrabold text-ink">
                  {point.title}
                </strong>{" "}
                — {point.body}
              </p>
            </li>
          ))}
        </ul>

        <div className="mt-2 flex flex-col gap-3 sm:flex-row">
          <a
            href="#get-started"
            className="btn-cta w-full hover:btn-cta-hover sm:w-auto"
          >
            {release.available ? "Download free beta" : "About the free beta"}
          </a>
          <a
            href="#advanced"
            className="btn-ghost w-full hover:btn-ghost-hover sm:w-auto"
          >
            See Advanced control
          </a>
        </div>
      </div>
    </section>
  );
}
