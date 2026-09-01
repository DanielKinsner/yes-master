import plate from "../assets/landing/plate-signoff.jpg";
import plate1500 from "../assets/landing/plate-signoff-1500.jpg";

// The one photograph on the page. Full-bleed between the proof and the
// styles, printed the way a plate sits across a spread: edge to edge, a
// hairline above and below, the caption set back inside the margin.
//
// Owner-generated (2026-09-01) to the sheet's brief: dusk, one lamp, the
// monitors soft, a sheet of paper and a hand about to sign. It is a mood
// image, not product evidence — it shows no UI and makes no claim.
export default function Plate() {
  return (
    <figure className="border-t border-rule">
      <div className="relative overflow-hidden bg-screen">
        <img
          src={plate}
          srcSet={`${plate1500} 1500w, ${plate} 3000w`}
          sizes="100vw"
          alt="A desk at dusk: one lamp lit, studio monitors out of focus, a hand holding a pencil over a printed sheet."
          width={3000}
          height={1000}
          loading="lazy"
          decoding="async"
          className="block h-[15rem] w-full object-cover object-[78%_60%] sm:h-auto sm:aspect-[3/1] sm:object-center"
        />
      </div>
      <figcaption className="mx-auto grid max-w-6xl gap-2 px-5 py-5 text-[0.9rem] leading-[1.55] text-ink-2 sm:grid-cols-[7rem_1fr] sm:px-8">
        <span className="font-display text-[1.05rem] text-ink">Sign-off</span>
        <span>The last thing a master needs is someone to say it is finished. The receipt is how YES Master says it.</span>
      </figcaption>
    </figure>
  );
}
