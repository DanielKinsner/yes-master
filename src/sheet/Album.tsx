import albumImg from "../assets/landing/desktop-album-ui.png";

// Album sits low on purpose: proof that the depth is real, not a second
// audience. Exclusions are stated because a mastering engineer reading
// "album" will otherwise assume DDP and gapless guarantees v1 does not make.
const points = [
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

export default function Album() {
  return (
    <section id="album" className="border-t border-rule px-5 py-20 sm:px-8 sm:py-28">
      <div className="mx-auto max-w-6xl">
        <div className="grid gap-8 lg:grid-cols-12">
          <div className="lg:col-span-5">
            <p className="eyebrow">04 / Depth, when the work needs it</p>
            <h2 className="headline mt-4">A record, not a folder of files.</h2>
          </div>
          <p className="max-w-[30rem] text-[1.05rem] leading-[1.6] text-ink-2 lg:col-span-6 lg:col-start-7 lg:pt-2">
            Album Master lives in Advanced. Your tracks come out sounding like one
            deliberate record — and nothing is silently altered to get them there.
          </p>
        </div>

        <div className="mt-14 grid gap-10 lg:grid-cols-12 lg:items-start">
          <figure className="lg:col-span-7">
            <div className="plate overflow-hidden">
              <img
                src={albumImg}
                alt="YES Master Album Master view with four tracks"
                width={1440}
                height={1000}
                loading="lazy"
                decoding="async"
                className="block w-full rounded-[0.35rem]"
              />
            </div>
            <figcaption className="mt-4 grid gap-2 text-[0.9rem] leading-[1.55] text-ink-2 sm:grid-cols-[7rem_1fr]">
              <span className="font-display text-[1.05rem] text-ink">Fig. 3</span>
              <span>Album Master with four tracks following one album decision.</span>
            </figcaption>
          </figure>

          <dl className="border-t border-rule lg:col-span-5">
            {points.map((item) => (
              <div key={item.title} className="border-b border-rule py-5">
                <dt className="text-[1.05rem] font-semibold text-ink">{item.title}</dt>
                <dd className="mt-1.5 text-[0.95rem] leading-[1.6] text-ink-2">{item.body}</dd>
              </div>
            ))}
          </dl>
        </div>

        <p className="mt-10 max-w-[44rem] text-[0.85rem] leading-[1.6] text-ink-3">
          Album Master delivers audio, renders, and receipts. It does not produce
          DDP images, cue sheets, or ISRC metadata, and it does not guarantee
          gapless playback.
        </p>
      </div>
    </section>
  );
}
