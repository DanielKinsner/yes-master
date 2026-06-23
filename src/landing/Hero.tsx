import heroBg from "../assets/landing/hero-control-room-studio.jpg";
import iconLocal from "../assets/landing/icon-local-first.png";
import iconRealtime from "../assets/landing/icon-realtime.png";
import iconRelease from "../assets/landing/icon-release-ready.png";

// The three proof points. Edit the words here; the layout stays the same.
const points = [
  {
    icon: iconLocal,
    title: "Local-first",
    body: "Your tracks never leave your machine. No uploads, no cloud, no waiting.",
  },
  {
    icon: iconRealtime,
    title: "Real-time control",
    body: "Hear every change as you make it. Shape tone, loudness, and width by ear.",
  },
  {
    icon: iconRelease,
    title: "Release-ready",
    body: "Technically checked and true-peak safe. Ship a master you can trust.",
  },
];

export default function Hero() {
  return (
    <section
      id="top"
      className="relative isolate flex min-h-svh items-center overflow-hidden"
    >
      {/* Background: the studio + app photo. object-cover crops, never stretches. */}
      <img
        src={heroBg}
        alt=""
        aria-hidden="true"
        className="absolute inset-0 -z-20 h-full w-full object-cover object-[62%_center]"
      />
      {/* Readability scrim: a soft veil on phones, left-weighted on desktop so
          the image breathes on the right while the text stays legible. */}
      <div
        aria-hidden="true"
        className="absolute inset-0 -z-10 bg-gradient-to-t from-night/90 via-night/55 to-night/40 md:bg-gradient-to-r md:from-night/85 md:via-night/30 md:to-transparent"
      />

      {/* Copy column. Full width on phones, ~half on desktop. */}
      <div className="mx-auto flex w-full max-w-7xl flex-col gap-6 px-5 pt-28 pb-16 sm:px-8 md:min-h-[72svh] md:justify-center md:py-0 lg:w-[54%] lg:pl-12">
        <span className="text-xs font-black uppercase tracking-wide text-brand-soft">
          Y.E.S. Master / Instantly master your track
        </span>

        <h1 className="font-display text-[2.6rem] font-black leading-[0.9] tracking-tight sm:text-6xl lg:text-7xl">
          Your Endgame <span className="block text-brand">Sound.</span>
        </h1>

        <ul className="flex flex-col gap-4">
          {points.map((point) => (
            <li key={point.title} className="flex items-center gap-3">
              <img
                src={point.icon}
                alt=""
                aria-hidden="true"
                className="h-11 w-11 shrink-0"
              />
              <p className="text-[15px] leading-snug text-muted">
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
            Join desktop beta
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
