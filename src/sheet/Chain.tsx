import type { CSSProperties } from "react";

// The app's own signal chain, in the app's own order — the eight stages that
// run live while audio plays. It is the one piece of decoration on the page,
// and it is not decoration: it is the thing you are buying.
const stages = ["Source", "EQ", "Warmth", "Air", "Comp", "Width", "Saturation", "Limiter"];

export default function Chain() {
  return (
    <div className="border-y border-rule bg-paper-deep/60 px-5 py-6 sm:px-8">
      <div className="mx-auto max-w-6xl">
        <p className="eyebrow">The chain, as it runs while you listen</p>
        {/* Phones: two rows of four, no rail (eight labels cannot share one
            390px line). From sm up: one row on a rail that draws left to
            right once on load. */}
        <ol className="relative mt-5 grid grid-cols-4 gap-x-3 gap-y-5 sm:flex sm:items-start sm:justify-between sm:gap-2">
          <span
            aria-hidden="true"
            className="draw absolute left-0 right-0 top-[5px] hidden h-px bg-rule-strong sm:block"
            style={{ "--d": "300ms" } as CSSProperties}
          />
          {stages.map((stage, i) => (
            <li
              key={stage}
              className="rise relative flex flex-col items-start gap-2"
              style={{ "--d": `${500 + i * 90}ms` } as CSSProperties}
            >
              <span
                aria-hidden="true"
                className={`block h-[11px] w-[11px] rounded-full border ${
                  i === stages.length - 1
                    ? "border-accent bg-accent"
                    : "border-rule-strong bg-paper"
                }`}
              />
              <span className="text-[0.72rem] font-semibold uppercase tracking-[0.12em] text-ink-2">
                {stage}
              </span>
            </li>
          ))}
        </ol>
      </div>
    </div>
  );
}
