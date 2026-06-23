import standardImg from "../assets/landing/desktop-standard-ui.png";
import advancedImg from "../assets/landing/desktop-advanced-ui.png";

const receipt: Array<[string, string, boolean]> = [
  ["Delivered LUFS", "-11.0 LUFS", false],
  ["True Peak", "-0.8 dBTP", false],
  ["Dynamic Range", "8.4 LU", false],
  ["Quality Checks", "All good", true],
  ["Status", "Ready to ship", true],
];

export default function ProofDeck() {
  return (
    <section id="standard" className="bg-night px-5 py-20 sm:px-8 sm:py-24">
      {/* The "Advanced" nav link points here too. */}
      <span id="advanced" className="sr-only" />

      <div className="mx-auto max-w-6xl">
        <div className="mx-auto max-w-2xl text-center">
          <p className="text-xs font-black uppercase tracking-wide text-brand-soft">
            From fast finish to full control
          </p>
          <h2 className="mt-4 font-display text-3xl font-black leading-tight sm:text-5xl">
            One engine. Three ways to trust it.
          </h2>
        </div>

        <div className="mt-12 grid gap-5 lg:grid-cols-3">
          <article className="flex flex-col rounded-2xl border border-line bg-card/70 p-6">
            <p className="text-sm font-extrabold text-brand-soft">Standard — the hero</p>
            <h3 className="mt-3 font-display text-2xl font-black">
              One click from finished to mastered.
            </h3>
            <p className="mt-3 text-sm leading-relaxed text-muted">
              Pick a style. Pick a loudness. Create Master. You get a clean
              44.1 kHz / 24-bit WAV, true-peak safe, every time.
            </p>
            <img
              src={standardImg}
              alt="YES Master Standard view"
              className="mt-5 rounded-lg border border-line"
            />
          </article>

          <article className="flex flex-col rounded-2xl border border-line bg-card/70 p-6">
            <p className="text-sm font-extrabold text-[#f0b35b]">Advanced — the proof</p>
            <h3 className="mt-3 font-display text-2xl font-black">
              When you want the full room.
            </h3>
            <p className="mt-3 text-sm leading-relaxed text-muted">
              Eight presets, a 7-band EQ, compressor modes, width and warmth,
              live metering, and export review with a measured receipt.
            </p>
            <img
              src={advancedImg}
              alt="YES Master Advanced view"
              className="mt-5 rounded-lg border border-line"
            />
          </article>

          <article className="flex flex-col rounded-2xl border border-line bg-card/70 p-6">
            <p className="text-sm font-extrabold text-good">Technically checked</p>
            <h3 className="mt-3 font-display text-2xl font-black">
              Honest results. You decide.
            </h3>
            <p className="mt-3 text-sm leading-relaxed text-muted">
              Every master ships with a receipt: delivered LUFS, true peak,
              dynamic range, and quality checks. No guesswork.
            </p>
            <dl className="mt-5 divide-y divide-white/10 rounded-lg border border-line bg-night/40 px-4">
              {receipt.map(([label, value, good]) => (
                <div key={label} className="flex justify-between py-3 text-sm">
                  <dt className="text-muted">{label}</dt>
                  <dd className={good ? "font-semibold text-good" : "text-ink"}>
                    {value}
                  </dd>
                </div>
              ))}
            </dl>
          </article>
        </div>
      </div>
    </section>
  );
}
