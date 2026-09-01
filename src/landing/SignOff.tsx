import plate from "../assets/landing/plate-signoff.jpg";
import plate1500 from "../assets/landing/plate-signoff-1500.jpg";

// The one photograph on the page (owner-generated 2026-09-01), full-bleed
// between the proof and the character controls. Mood, not evidence: it shows
// no UI and makes no claim. Fades into the page at the bottom so the section
// hands off to the next one instead of ending on a hard edge.
export default function SignOff() {
  return (
    <figure className="border-t border-white/[0.06] bg-night">
      <div className="relative overflow-hidden">
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
        <div
          aria-hidden="true"
          className="pointer-events-none absolute inset-x-0 bottom-0 h-24 bg-gradient-to-t from-night to-transparent"
        />
      </div>
      <figcaption className="mx-auto grid max-w-6xl gap-2 px-5 pb-8 pt-4 text-sm leading-relaxed text-muted sm:grid-cols-[7rem_1fr] sm:px-8">
        <span className="font-display text-base font-extrabold text-ink">Sign-off</span>
        <span>
          The last thing a master needs is someone to say it is finished. The
          receipt is how YES Master says it.
        </span>
      </figcaption>
    </figure>
  );
}
