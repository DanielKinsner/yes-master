// U6 — mobile, reduced to what the canon actually permits.
//
// This section used to be the SECOND thing on the page: "Coming to your
// pocket", "Same engine, headed to iPhone & Android", "coming after launch", a
// phone screenshot, and six present-tense feature cards. Three problems, all
// recorded in docs/CAPABILITY_EVIDENCE_MATRIX.md:
//
//   C-15  "coming after launch" is a schedule the product has not committed to.
//   C-16  Six feature cards read as a product a visitor can obtain. They
//         cannot obtain it. The engine sharing is true; the availability
//         implication is not.
//   C-17  A mobile screenshot sitting next to desktop beta proof is forbidden
//         by R7, and this asset has no capture-commit binding. Removed rather
//         than relabelled — U7 owns asset provenance and can reinstate a bound,
//         labelled version if it earns its place.
//
// docs/landing-brief.md "Mobile status" allows exactly one restrained,
// date-free sentence, and calls it the single deliberate exception to the
// no-roadmap rule: saying nothing at all would be less honest than "not yet"
// for a visitor who has seen the phone screens. Anything more is out of bounds.
// Do not grow this section back.
export default function CrossPlatform() {
  return (
    <section
      id="mobile"
      className="border-t border-white/[0.06] bg-night px-5 py-14 sm:px-8"
    >
      <div className="mx-auto max-w-2xl text-center">
        <h2 className="font-display text-xl font-black leading-tight sm:text-2xl">
          Desktop is the product.
        </h2>
        <p className="mt-3 text-sm leading-relaxed text-muted">
          YES Master runs on Windows and macOS. iPhone and Android are not
          currently available.
        </p>
      </div>
    </section>
  );
}
