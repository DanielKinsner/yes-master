// The arrangement, in full, before anyone downloads. Three claims are
// load-bearing and kept word for word: feedback is voluntary, the beta ending
// revokes nothing, and pricing states the settled model only. No countdown,
// no entitlement, no manufactured scarcity.
const terms = [
  {
    mark: "i",
    title: "Free while it is in beta",
    body: "The download is not gated behind an email address, an account, or a trial timer. There is no YES Master account to create.",
  },
  {
    mark: "ii",
    title: "Telling us things is optional",
    body: "Reports go through two structured forms on GitHub if you want to send one. Nothing is held back from anyone who never does.",
  },
  {
    mark: "iii",
    title: "The beta ending does not take it away",
    body: "When the beta closes the installers come down, but a build you already installed keeps working. There is no kill switch and nothing expires on your machine.",
  },
  {
    mark: "iv",
    title: "One purchase, not a subscription",
    body: "At launch YES Master is a one-time purchase: a $29 founder price, $49 standard. The founder window's dates and terms are announced when they are decided.",
  },
];

export default function Terms() {
  return (
    <section id="beta" className="border-t border-rule bg-paper-deep/50 px-5 py-20 sm:px-8 sm:py-28">
      <div className="mx-auto max-w-6xl">
        <div className="grid gap-10 lg:grid-cols-12">
          <div className="lg:col-span-4">
            <p className="eyebrow">05 / The arrangement</p>
            <h2 className="headline mt-4">What you are actually agreeing to.</h2>
            <p className="mt-5 max-w-[26rem] text-[1.05rem] leading-[1.6] text-ink-2">
              It is a time-boxed public beta. Here is the whole deal, before you
              download anything.
            </p>
          </div>

          <dl className="grid gap-x-10 sm:grid-cols-2 lg:col-span-8 lg:pt-1">
            {terms.map((term) => (
              <div key={term.mark} className="grid grid-cols-[2rem_1fr] border-t border-rule py-5">
                <span className="font-display text-[1.15rem] text-ink-3">{term.mark}</span>
                <div>
                  <dt className="text-[1.05rem] font-semibold text-ink">{term.title}</dt>
                  <dd className="mt-1.5 text-[0.95rem] leading-[1.6] text-ink-2">{term.body}</dd>
                </div>
              </div>
            ))}
          </dl>
        </div>
      </div>
    </section>
  );
}
