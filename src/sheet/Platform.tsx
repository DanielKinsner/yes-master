// Mobile, reduced to what the canon permits: one restrained, date-free
// sentence, once (docs/landing-brief.md "Mobile status"). Do not grow it.
export default function Platform() {
  return (
    <section id="mobile" className="border-t border-rule px-5 py-12 sm:px-8">
      <div className="mx-auto grid max-w-6xl gap-3 sm:grid-cols-12 sm:items-baseline">
        <h2 className="font-display text-[1.6rem] leading-none text-ink sm:col-span-5">
          Desktop is the product.
        </h2>
        <p className="text-[0.95rem] leading-[1.6] text-ink-2 sm:col-span-6 sm:col-start-7">
          YES Master runs on Windows and macOS. iPhone and Android are not
          currently available.
        </p>
      </div>
    </section>
  );
}
