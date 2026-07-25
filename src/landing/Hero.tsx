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
      {/* Background: the studio + app photo. object-cover crops, never stretches. */}
      {/* U7: two variants so a phone does not download a 2560px photograph to
          render it 390px wide. The master art lives beside this one as
          *-source.jpg, imported by nothing and shipped in no bundle. */}
      <img
        src={heroBg}
        srcSet={`${heroBg1280} 1280w, ${heroBg} 2560w`}
        sizes="100vw"
        alt=""
        aria-hidden="true"
        fetchPriority="high"
        className="absolute inset-0 -z-20 h-full w-full object-cover object-[62%_center]"
      />
      {/* Readability veil — phones only (text sits over the whole image there).
          On desktop there is NO gradient; the studio image's own dark left
          backs the copy. */}
      <div
        aria-hidden="true"
        className="absolute inset-0 -z-10 bg-gradient-to-t from-night/90 via-night/55 to-night/40 md:hidden"
      />

      {/* Copy column. A normal block (so its width can never exceed the
          screen), full width on phones, ~half on desktop. */}
      <div className="relative flex min-h-svh w-full flex-col justify-start gap-6 px-5 pt-28 pb-20 sm:px-8 lg:w-[54%] lg:max-w-[600px] lg:pl-16">
        <span className="text-xs font-black uppercase tracking-wide text-brand-soft">
          YES Master
        </span>

        <h1 className="font-display text-[2.6rem] font-black leading-[0.9] tracking-tight sm:text-6xl lg:text-7xl">
          Master your track in real time
          <span className="block text-brand">— and see exactly what it did.</span>
        </h1>

        <p className="max-w-[440px] text-base leading-snug text-muted sm:text-lg">
          Drop a track, pick a sound, and hear the full mastering chain as you
          listen. It reads what your mix already has and eases its own moves to
          fit. No upload, no waiting, no black box.
        </p>

        <ul className="flex flex-col gap-5">
          {points.map((point) => (
            <li key={point.title} className="relative max-w-[360px] pl-[72px]">
              <img
                src={point.icon}
                alt=""
                aria-hidden="true"
                className="absolute left-0 top-0 h-14 w-14"
              />
              <p className="text-base leading-snug text-muted">
                <strong className="font-extrabold text-ink">
                  {point.title}
                </strong>{" "}
                — {point.body}
              </p>
            </li>
          ))}
        </ul>

        <div className="mt-2 flex flex-col gap-3 sm:flex-row md:mt-auto">
          <a
            href="#get-started"
            className="inline-flex w-full items-center justify-center rounded-lg bg-gradient-to-b from-cta-light to-cta-deep px-5 py-3 font-extrabold text-[#1c0d00] shadow-lg shadow-cta-deep/30 sm:w-auto"
          >
            {release.available ? "Download free beta" : "About the free beta"}
          </a>
          <a
            href="#advanced"
            className="inline-flex w-full items-center justify-center rounded-lg border border-white/15 bg-white/5 px-5 py-3 font-extrabold text-ink sm:w-auto"
          >
            See Advanced control
          </a>
        </div>
      </div>
    </section>
  );
}
