// U6 — what the beta relationship actually is.
//
// Every sentence here is one a visitor would otherwise have to guess at, and
// guessing is where resentment comes from. Three claims are load-bearing:
//
//   * Feedback is voluntary. Nothing is withheld from someone who never files
//     an issue. (R19 / the beta guide say the same thing; they must not
//     disagree.)
//   * The beta end does not revoke anything. Installers come down; an
//     installed build keeps working and there is no kill switch (R26/KTD13).
//     "Beta over" normally means something worse than what this actually does,
//     so saying it plainly is the difference between installing and not.
//   * Pricing states the settled model ONLY. The $29 → $49 split is canon
//     (docs/PRODUCT.md "Distribution & Business Model"). The founder window's
//     dates, duration, and what a beta tester is entitled to are NOT decided
//     (docs/OWNER_INPUT_QUEUE.md row 1), so no entitlement promise and no
//     countdown appears here. Manufactured scarcity is explicitly out.
const terms = [
  {
    title: "Free while it is in beta",
    body: "The download is not gated behind an email address, an account, or a trial timer. There is no YES Master account to create.",
  },
  {
    title: "Telling us things is optional",
    body: "Reports go through two structured forms on GitHub if you want to send one. Nothing is held back from anyone who never does.",
  },
  {
    title: "The beta ending does not take it away",
    body: "When the beta closes the installers come down, but a build you already installed keeps working. There is no kill switch and nothing expires on your machine.",
  },
  {
    title: "One purchase, not a subscription",
    body: "At launch YES Master is a one-time purchase: a $29 founder price, $49 standard. The founder window's dates and terms are announced when they are decided.",
  },
];

export default function BetaTerms() {
  return (
    <section
      id="beta"
      className="border-t border-white/[0.06] bg-night px-5 py-20 sm:px-8 sm:py-28"
    >
      <div className="mx-auto max-w-6xl">
        {/* Presentation (2026-08-18): heading on the left, the four terms as
            a plain two-column list with hairline rules on the right — the
            deal, stated, not four more cards. Same sentences. */}
        <div className="grid gap-10 lg:grid-cols-[minmax(0,4fr)_minmax(0,8fr)]">
          <div>
            <p className="eyebrow">The arrangement</p>
            <h2 className="mt-4 font-display text-3xl font-black leading-[1.02] tracking-[-0.02em] sm:text-5xl">
              What you are actually agreeing to.
            </h2>
            <p className="mt-4 text-muted sm:text-lg">
              It is a time-boxed public beta. Here is the whole deal, before you
              download anything.
            </p>
          </div>

          <dl className="grid gap-x-8 gap-y-7 sm:grid-cols-2 lg:pt-2">
            {terms.map((term) => (
              <div key={term.title} className="border-t border-line pt-4">
                <dt className="font-display text-base font-extrabold text-ink">
                  {term.title}
                </dt>
                <dd className="mt-1.5 text-sm leading-relaxed text-muted">
                  {term.body}
                </dd>
              </div>
            ))}
          </dl>
        </div>
      </div>
    </section>
  );
}
