import albumImg from "../assets/landing/desktop-album-ui.png";

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
      className="border-t border-white/10 bg-night px-5 py-20 sm:px-8 sm:py-24"
    >
      <div className="mx-auto max-w-6xl">
        <div className="mx-auto max-w-2xl text-center">
          <p className="text-xs font-black uppercase tracking-wide text-brand-soft">
            Depth, when the work needs it
          </p>
          <h2 className="mt-4 font-display text-3xl font-black leading-tight sm:text-5xl">
            A record, not a folder of files.
          </h2>
          <p className="mt-4 text-muted sm:text-lg">
            Album Master lives in Advanced. Your tracks come out sounding like
            one deliberate record — and nothing is silently altered to get them
            there.
          </p>
        </div>

        <div className="mt-12 grid gap-4 sm:grid-cols-2">
          {guarantees.map((item) => (
            <article
              key={item.title}
              className="rounded-xl border border-line bg-card/60 p-5"
            >
              <h3 className="font-display text-base font-extrabold text-ink">
                {item.title}
              </h3>
              <p className="mt-1 text-sm leading-relaxed text-muted">
                {item.body}
              </p>
            </article>
          ))}
        </div>

        {/* U7: a real capture of the album-4 scenario from the current build,
            bound to the manifest. Lazy — it is well below the fold. */}
        <img
          src={albumImg}
          alt="YES Master Album Master view with four tracks"
          width={1440}
          height={1000}
          loading="lazy"
          decoding="async"
          className="mt-10 w-full rounded-xl border border-line"
        />

        <p className="mx-auto mt-8 max-w-2xl text-center text-xs leading-relaxed text-muted">
          Album Master delivers audio, renders, and receipts. It does not
          produce DDP images, cue sheets, or ISRC metadata, and it does not
          guarantee gapless playback.
        </p>
      </div>
    </section>
  );
}
