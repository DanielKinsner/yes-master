import iphoneImg from "../assets/landing/iphone-standard-ui.jpg";

// Compact feature list shown beside the phone.
const features = [
  { title: "A/B in sync", body: "Original vs Mastered with volume match." },
  { title: "Four styles", body: "Balanced, Warm, Open, Punch." },
  { title: "Intensity control", body: "Subtle to pushed." },
  { title: "Real-time meters", body: "LUFS, true peak, and gain reduction." },
  { title: "Quality checks", body: "Instant feedback." },
  { title: "No cloud", body: "All on your device. All private." },
];

export default function CrossPlatform() {
  return (
    <section
      id="mobile"
      className="border-t border-white/10 bg-night px-5 py-20 sm:px-8 sm:py-24"
    >
      <div className="mx-auto max-w-6xl">
        <div className="mx-auto max-w-2xl text-center">
          <p className="text-xs font-black uppercase tracking-wide text-brand-soft">
            The same endgame sound
          </p>
          <h2 className="mt-4 font-display text-3xl font-black leading-tight sm:text-5xl">
            Master anywhere. Same engine. Same truth.
          </h2>
          <p className="mt-4 text-muted sm:text-lg">
            The power of the studio in your pocket.
          </p>
        </div>

        <div className="mt-12 grid items-center gap-10 md:grid-cols-2 md:gap-14">
          <div className="flex justify-center">
            <img
              src={iphoneImg}
              alt="YES Master running on iPhone"
              className="w-56 rounded-[2rem] border border-white/15 shadow-2xl sm:w-64"
            />
          </div>

          <ul className="grid gap-3 sm:grid-cols-2">
            {features.map((feature) => (
              <li
                key={feature.title}
                className="rounded-xl border border-line bg-card/60 p-4"
              >
                <h3 className="font-display text-base font-extrabold text-ink">
                  {feature.title}
                </h3>
                <p className="mt-1 text-sm text-muted">{feature.body}</p>
              </li>
            ))}
          </ul>
        </div>
      </div>
    </section>
  );
}
