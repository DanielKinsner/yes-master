import standardImg from "../assets/landing/desktop-standard-ui.png";
import advancedImg from "../assets/landing/owner-advanced-session.jpg";
import advancedImg1280 from "../assets/landing/owner-advanced-session-1280.jpg";

// Two rooms, one engine. Both captures are printed like photographs on the
// sheet — a plate, a figure number, a caption — not floated in bezels.
//
// Fig. 1 is the deterministic Standard capture bound to the asset manifest
// (U7). Fig. 2 is an OWNER capture of a real session (2026-09-01: "The
// Machine Restocks on Friday", meters live, a hand-shaped EQ). It is not in
// the manifest because it is not a scripted capture; it is the real thing,
// which for the room that promises "the full console" is the better proof.
export default function Proof() {
  return (
    <section id="standard" className="border-t border-rule px-5 py-20 sm:px-8 sm:py-28">
      <span id="advanced" className="sr-only" />
      <div className="mx-auto max-w-6xl">
        <div className="grid gap-8 lg:grid-cols-12">
          <div className="lg:col-span-5">
            <p className="eyebrow">02 / Proof</p>
            <h2 className="headline mt-4">One engine. Two rooms.</h2>
          </div>
          <p className="max-w-[30rem] text-[1.05rem] leading-[1.6] text-ink-2 lg:col-span-6 lg:col-start-7 lg:pt-2">
            Standard is one clean column and one button. Advanced is the whole
            console. They are the same engine, and both hand you the same
            receipt.
          </p>
        </div>

        {/* Fig. 1 — Standard. Full width; this is the room most people stay in. */}
        <figure className="mt-14">
          <div className="plate overflow-hidden">
            <img
              src={standardImg}
              alt="YES Master Standard view"
              width={1440}
              height={1000}
              loading="lazy"
              decoding="async"
              className="block w-full rounded-[0.35rem]"
            />
          </div>
          <figcaption className="mt-4 grid gap-2 text-[0.9rem] leading-[1.55] text-ink-2 sm:grid-cols-[7rem_1fr]">
            <span className="font-display text-[1.05rem] text-ink">Fig. 1</span>
            <span>
              <strong className="font-semibold text-ink">Standard.</strong> Pick a
              style. Pick a loudness. Create Master. You get a fixed 44.1 kHz /
              24-bit WAV with the limiter's ceiling set to −1 dBTP — and a
              measurement of what actually came out.
            </span>
          </figcaption>
        </figure>

        {/* Fig. 2 — Advanced. Offset right and narrower, the way a second plate
            sits on a spread: still the same sheet, a different room. */}
        <figure className="mt-16 grid gap-6 lg:grid-cols-12 lg:items-end">
          <figcaption className="order-2 grid gap-2 text-[0.9rem] leading-[1.55] text-ink-2 lg:order-1 lg:col-span-4 lg:pb-2">
            <span className="font-display text-[1.05rem] text-ink">Fig. 2</span>
            <span>
              <strong className="font-semibold text-ink">Advanced.</strong> A
              real session, meters live. When you want the full room: eight
              presets, a 7-band EQ, compressor modes, width and warmth, live
              metering, and export review with a measured receipt.
            </span>
          </figcaption>
          <div className="plate order-1 overflow-hidden lg:order-2 lg:col-span-8">
            <img
              src={advancedImg}
              srcSet={`${advancedImg1280} 1280w, ${advancedImg} 2048w`}
              sizes="(min-width: 1024px) 66vw, 100vw"
              alt="YES Master Advanced view during a real session, meters live"
              width={2048}
              height={1151}
              loading="lazy"
              decoding="async"
              className="block w-full rounded-[0.35rem]"
            />
          </div>
        </figure>
      </div>
    </section>
  );
}
