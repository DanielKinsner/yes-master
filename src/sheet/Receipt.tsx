import type { CSSProperties } from "react";

// The receipt is the hero object because it is the product's whole argument:
// YES Master measures the rendered file and shows you. These numbers are
// hand-authored (claim C-07) and say so — the "Example receipt" label reads
// before the first number so nobody mistakes a mock-up for a measurement.
const rows: Array<{ label: string; value: string; pass?: boolean }> = [
  { label: "Delivered LUFS", value: "-11.0 LUFS" },
  { label: "True Peak", value: "-0.8 dBTP" },
  { label: "Dynamic Range", value: "8.4 LU" },
  { label: "Quality Checks", value: "All good", pass: true },
  { label: "Status", value: "Ready to ship", pass: true },
];

export default function Receipt({ className = "", style }: { className?: string; style?: CSSProperties }) {
  return (
    <figure
      className={`receipt w-full max-w-[19rem] px-6 pb-9 pt-6 lg:max-w-[20.5rem] ${className}`}
      style={style}
      aria-label="Example master receipt"
    >
      <div className="flex items-baseline justify-between border-b border-dashed border-rule-strong pb-3">
        <span className="font-display text-[1.35rem] leading-none">YES Master</span>
        <span className="text-[0.66rem] font-semibold uppercase tracking-[0.16em] text-ink-3">
          Master receipt
        </span>
      </div>

      <p className="mt-3 text-[0.7rem] font-semibold uppercase tracking-[0.16em] text-ink-3">
        Example receipt
      </p>

      <dl className="mt-2 divide-y divide-rule">
        {rows.map((row) => (
          <div key={row.label} className="flex items-baseline justify-between gap-4 py-2.5 text-[0.9rem]">
            <dt className="text-ink-2">{row.label}</dt>
            <dd className={`tabular-nums ${row.pass ? "font-semibold text-pass" : "text-ink"}`}>
              {row.value}
            </dd>
          </div>
        ))}
      </dl>

      <figcaption className="mt-4 border-t border-dashed border-rule-strong pt-3 text-[0.72rem] leading-relaxed text-ink-3">
        44.1 kHz / 24-bit WAV · ceiling −1 dBTP
        <br />
        Measured from the rendered file, not the settings.
      </figcaption>
    </figure>
  );
}
