import standardImg from "../assets/landing/desktop-standard-ui.png";
import advancedImg from "../assets/landing/owner-advanced-session.jpg";
import advancedImg1280 from "../assets/landing/owner-advanced-session-1280.jpg";

// U6 / claim C-07. These numbers are hand-authored and always were. They were
// previously rendered as if they were product output, with nothing marking
// them as illustrative — plausible, but not a measurement of anything. Until
// U7 sources them from a deterministic capture bound to a commit, the block
// carries an explicit "Example" label. A fabricated measurement presented as a
// real one is the exact failure the evidence matrix exists to catch.
const receipt: Array<[string, string, boolean]> = [
  ["Delivered LUFS", "-11.0 LUFS", false],
  ["True Peak", "-0.8 dBTP", false],
  ["Dynamic Range", "8.4 LU", false],
  ["Quality Checks", "All good", true],
  ["Status", "Ready to ship", true],
];

// Presentation (2026-08-18): the two captures get the room — framed like the
// product, side by side — and the receipt reads as the printed slip it is.
// Same claims, same "Example" label ordering (label before first number).
//
// 2026-09-01: the Advanced plate is an OWNER capture of a real session ("The
// Machine Restocks on Friday", meters live, hand-shaped EQ). It is not in the
// asset manifest because it is not a scripted capture; the manifest-bound
// Standard capture stays as the Standard plate.
export default function ProofDeck() {
  return (
    <section id="standard" className="relative bg-night px-5 py-20 sm:px-8 sm:py-28">
      {/* The "Advanced" nav link points here too. */}
      <span id="advanced" className="sr-only" />

      <div className="mx-auto max-w-6xl">
        <div className="max-w-2xl">
          <p className="eyebrow">From fast finish to full control</p>
          <h2 className="mt-4 font-display text-3xl font-black leading-[1.02] tracking-[-0.02em] sm:text-5xl">
            One engine. Two rooms.
          </h2>
          <p className="mt-4 text-muted sm:text-lg">
            Standard is one clean column and one button. Advanced is the whole
            console. They are the same engine, and both hand you the same
            receipt.
          </p>
        </div>

        <div className="mt-12 grid gap-6 lg:grid-cols-2">
          <article className="flex flex-col">
            <p className="text-sm font-extrabold text-brand-soft">Standard — the hero</p>
            <h3 className="mt-2 font-display text-2xl font-black tracking-[-0.01em]">
              One click from finished to mastered.
            </h3>
            {/* C-07 note above applies to the receipt block; this paragraph is
                claim C-05. "True-peak safe, every time" was an absolute
                guarantee the product deliberately does not make — you are
                allowed to overcook a track and get an advisory warning instead
                of a refusal. Narrowed to the fixed format and the ceiling the
                limiter targets, both of which are real. */}
            <p className="mt-3 max-w-[52ch] text-[0.95rem] leading-relaxed text-muted">
              Pick a style. Pick a loudness. Create Master. You get a fixed
              44.1 kHz / 24-bit WAV with the limiter's ceiling set to −1 dBTP —
              and a measurement of what actually came out.
            </p>
            {/* U7: below the fold, so lazy. Intrinsic size is declared to
                reserve the box and avoid a layout shift when it arrives. */}
            <div className="surface-frame mt-6 overflow-hidden rounded-xl p-1.5">
              <img
                src={standardImg}
                alt="YES Master Standard view"
                width={1440}
                height={1000}
                loading="lazy"
                decoding="async"
                className="block w-full rounded-[0.6rem]"
              />
            </div>
          </article>

          <article className="flex flex-col">
            <p className="text-sm font-extrabold text-[#f0b35b]">Advanced — the proof</p>
            <h3 className="mt-2 font-display text-2xl font-black tracking-[-0.01em]">
              When you want the full room.
            </h3>
            <p className="mt-3 max-w-[52ch] text-[0.95rem] leading-relaxed text-muted">
              A real session, meters live. Eight presets, a 7-band EQ,
              compressor modes, width and warmth, live metering, and export
              review with a measured receipt.
            </p>
            <div className="surface-frame mt-6 overflow-hidden rounded-xl p-1.5">
              <img
                src={advancedImg}
                srcSet={`${advancedImg1280} 1280w, ${advancedImg} 2048w`}
                sizes="(min-width: 1024px) 50vw, 100vw"
                alt="YES Master Advanced view during a real session, meters live"
                width={2048}
                height={1151}
                loading="lazy"
                decoding="async"
                className="block w-full rounded-[0.6rem]"
              />
            </div>
          </article>
        </div>

        <article className="surface-card mt-6 grid gap-8 rounded-2xl p-6 sm:p-8 lg:grid-cols-[1fr_minmax(0,380px)] lg:items-center">
          <div>
            <p className="text-sm font-extrabold text-good">Technically checked</p>
            <h3 className="mt-2 font-display text-2xl font-black tracking-[-0.01em]">
              Honest results. You decide.
            </h3>
            <p className="mt-3 max-w-[52ch] text-[0.95rem] leading-relaxed text-muted">
              Every master ships with a receipt: delivered LUFS, true peak,
              dynamic range, and quality checks. No guesswork.
            </p>
          </div>
          <div>
            <p className="text-xs font-black uppercase tracking-[0.16em] text-soft">
              Example receipt
            </p>
            <dl className="mt-2 divide-y divide-white/[0.07] rounded-xl border border-line bg-night/60 px-4 shadow-[inset_0_1px_2px_rgba(0,0,0,0.5)]">
              {receipt.map(([label, value, good]) => (
                <div key={label} className="flex justify-between py-3 text-sm">
                  <dt className="text-muted">{label}</dt>
                  <dd className={`tabular-nums ${good ? "font-semibold text-good" : "text-ink"}`}>
                    {value}
                  </dd>
                </div>
              ))}
            </dl>
          </div>
        </article>
      </div>
    </section>
  );
}
