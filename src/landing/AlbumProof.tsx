import albumImg from "../assets/landing/owner-album-session.jpg";
import albumImg1280 from "../assets/landing/owner-album-session-1280.jpg";

// U6 — Album sits LOW on the page on purpose.
//
// It is proof that the depth is real, not the headline promise and not a
// second audience. Album-minded creators are already inside the primary
// audience; leading with a record-length workflow would misrepresent what most
// visitors are here to do, which is finish one track.
//
// The promise wording is owner-defined (docs/PRODUCT.md "Album Master"), and
// the exclusions are stated because a mastering engineer reading "album" will
// otherwise assume DDP and gapless guarantees that v1 does not make.
const guarantees = [
  {
    title: "One loudness across the record",
    body: "Tracks land together instead of each being mastered in isolation.",
  },
  {
    title: "One delivery format",
    body: "Mixed source rates are resolved to a single album-wide format, and mixed mono/stereo sources to one channel count.",
  },
  {
    title: "A receipt per track",
    body: "Every track reports what its own render measured. Nothing is averaged away to make the set look tidy.",
  },
  {
    title: "Per-track override",
    body: "Follow the album decision or override any single track, and the page says which tracks you changed.",
  },
];

export default function AlbumProof() {
  return (
    <section
      id="album"
      className="relative border-t border-white/[0.06] bg-night px-5 py-20 sm:px-8 sm:py-28"
    >
      <div
        aria-hidden="true"
        className="pointer-events-none absolute inset-0 -z-10 bg-[radial-gradient(60%_50%_at_100%_50%,rgba(79,134,247,0.07),transparent_60%)]"
      />
      <div className="mx-auto max-w-6xl">
        {/* Presentation (2026-08-18): copy + guarantees on the left, the real
            album capture framed on the right, so the section reads as one
            composition rather than a heading, a grid, then a picture. */}
        <div className="grid gap-10 lg:grid-cols-[minmax(0,5fr)_minmax(0,7fr)] lg:items-center">
          <div>
            <p className="eyebrow">Depth, when the work needs it</p>
            <h2 className="mt-4 font-display text-3xl font-black leading-[1.02] tracking-[-0.02em] sm:text-5xl">
              A record, not a folder of files.
            </h2>
            <p className="mt-4 text-muted sm:text-lg">
              Album Master lives in Advanced. Your tracks come out sounding like
              one deliberate record — and nothing is silently altered to get them
              there.
            </p>

            <dl className="mt-8 grid gap-x-6 gap-y-5 sm:grid-cols-2">
              {guarantees.map((item) => (
                <div key={item.title} className="border-t border-line pt-4">
                  <dt className="font-display text-base font-extrabold text-ink">
                    {item.title}
                  </dt>
                  <dd className="mt-1 text-sm leading-relaxed text-muted">
                    {item.body}
                  </dd>
                </div>
              ))}
            </dl>
          </div>

          {/* 2026-09-01: an OWNER capture of Album Master in a real session
              (manifest `ownerCaptures`). Lazy — it is well below the fold. */}
          <div className="surface-frame overflow-hidden rounded-2xl p-1.5">
            <img
              src={albumImg}
              srcSet={`${albumImg1280} 1280w, ${albumImg} 2048w`}
              sizes="(min-width: 1024px) 58vw, 100vw"
              alt="YES Master Album Master view during a real session, meters live"
              width={2048}
              height={1147}
              loading="lazy"
              decoding="async"
              className="block w-full rounded-[0.85rem]"
            />
          </div>
        </div>

        <p className="mx-auto mt-10 max-w-2xl text-center text-xs leading-relaxed text-muted">
          Album Master delivers audio, renders, and receipts. It does not
          produce DDP images, cue sheets, or ISRC metadata, and it does not
          guarantee gapless playback.
        </p>
      </div>
    </section>
  );
}
