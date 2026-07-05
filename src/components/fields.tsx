// Shared field primitives for the right-rail cards. Extracted from App.tsx
// (consolidated backlog B4.2); used by AdvancedPanel.tsx and the Standard
// EQUALIZER block in App.tsx.
import { useEffect, useRef, useState } from "react";

export function PanelResetButton({
  label,
  onClick,
  disabled = false,
}: {
  label: string;
  onClick: () => void;
  disabled?: boolean;
}) {
  return (
    <button
      type="button"
      className="panel-reset-button"
      aria-label={label}
      title={label}
      disabled={disabled}
      onClick={(e) => {
        e.preventDefault();
        e.stopPropagation();
        if (disabled) return;
        onClick();
      }}
    >
      ↺
    </button>
  );
}

// Always-on slider for required dB trim values (input gain, output gain).
// No "Auto" affordance — the value is always present (default 0 dB), so the
// slider is always active and double-click resets to 0.
export function GainField({
  label,
  value,
  onChange,
}: {
  label: string;
  value: number;
  onChange: (db: number) => void;
}) {
  const [draft, setDraft] = useState<string | null>(null);
  const inputRef = useRef<HTMLInputElement | null>(null);
  useEffect(() => {
    if (
      draft !== null &&
      inputRef.current &&
      document.activeElement !== inputRef.current
    ) {
      setDraft(null);
    }
  }, [value, draft]);
  const commitDraft = (raw: string) => {
    const parsed = parseFloat(raw);
    if (!Number.isFinite(parsed)) {
      setDraft(null);
      return;
    }
    const clamped = Math.max(-24, Math.min(24, parsed));
    if (clamped !== value) onChange(clamped);
    setDraft(null);
  };
  return (
    <div className="adv-field">
      <span className="adv-label">{label}</span>
      <div className="adv-control">
        <input
          type="range"
          min={-24}
          max={24}
          step={0.1}
          value={value}
          onChange={(e) => onChange(parseFloat(e.target.value))}
          onDoubleClick={() => onChange(0)}
          aria-label={label}
          title="Double-click to reset to 0 dB"
        />
        <span className="adv-value">
          {value > 0 ? "+" : ""}{value.toFixed(1)} dB
        </span>
        <input
          ref={inputRef}
          type="number"
          className="adv-number"
          min={-24}
          max={24}
          step={0.1}
          value={draft !== null ? draft : value}
          onChange={(e) => setDraft(e.target.value)}
          onBlur={(e) => commitDraft(e.target.value)}
          onKeyDown={(e) => {
            if (e.key === "Enter") {
              commitDraft((e.target as HTMLInputElement).value);
              (e.target as HTMLInputElement).blur();
            } else if (e.key === "Escape") {
              setDraft(null);
              (e.target as HTMLInputElement).blur();
            }
          }}
          aria-label={`${label} value`}
          title="Type a value or double-click slider to reset to 0 dB"
        />
      </div>
    </div>
  );
}

export function NumberField({
  label,
  value,
  min,
  max,
  step,
  format,
  autoLabel = "Auto",
  autoReadout,
  sliderAutoValue,
  showAutoReset = false,
  disabled = false,
  onChange,
}: {
  label: string;
  value: number | null;
  min: number;
  max: number;
  step: number;
  format: (v: number) => string;
  autoLabel?: string;
  autoReadout?: string;
  // F10 (owner smoke): where the slider THUMB parks while value is null.
  // Without it the thumb sat at `min` on Auto — for Width that rendered as
  // "current width is 0", so dragging to 0.05 looked like a tiny increase
  // when it actually replaced the preset's ~1.11 baseline with near-mono.
  // Pass the backend-resolved effective auto value (preset baseline after
  // the adaptive guard) so the thumb tells the truth.
  sliderAutoValue?: number;
  // F10: opt-in visible "↺ AUTO" chip while a value is engaged — the
  // double-click-to-Auto gesture is invisible to first-time users, which is
  // why "sliding back to 0" felt like the only way back (it isn't Auto).
  showAutoReset?: boolean;
  disabled?: boolean;
  onChange: (v: number | null) => void;
}) {
  const effective = value ?? sliderAutoValue ?? min;
  // Same draft-while-editing pattern as Slider so the user can type "1." or
  // "-" mid-value without the parent re-formatting on every keystroke.
  const [draft, setDraft] = useState<string | null>(null);
  const inputRef = useRef<HTMLInputElement | null>(null);
  useEffect(() => {
    if (
      draft !== null &&
      inputRef.current &&
      document.activeElement !== inputRef.current
    ) {
      setDraft(null);
    }
  }, [value, draft]);
  const commitDraft = (raw: string) => {
    const parsed = parseFloat(raw);
    if (!Number.isFinite(parsed)) {
      setDraft(null);
      return;
    }
    const clamped = Math.max(min, Math.min(max, parsed));
    if (clamped !== value) onChange(clamped);
    setDraft(null);
  };
  return (
    <div
      className={
        "adv-field " +
        (value === null ? "is-auto " : "") +
        (disabled ? "is-disabled" : "")
      }
    >
      <span className="adv-label">
        {label}
        {value === null && (
          <span className="adv-auto-pill">{autoLabel.toUpperCase()}</span>
        )}
        {showAutoReset && value !== null && !disabled && (
          <button
            type="button"
            className="adv-auto-reset"
            title={`Reset to ${autoLabel}`}
            aria-label={`Reset ${label} to ${autoLabel}`}
            onClick={() => onChange(null)}
          >
            ↺ {autoLabel.toUpperCase()}
          </button>
        )}
      </span>
      <div className="adv-control">
        <input
          type="range"
          min={min}
          max={max}
          step={step}
          value={effective}
          disabled={disabled}
          // Always live: dragging an Auto slider engages it at the dragged
          // value instead of staying greyed out. Double-click reverts to Auto.
          onChange={(e) => onChange(parseFloat(e.target.value))}
          onDoubleClick={() => onChange(null)}
          aria-label={label}
          title={
            value === null
              ? `Drag to engage. Double-click to leave it on ${autoLabel}.`
              : `Drag or type a value. Double-click slider to reset to ${autoLabel}.`
          }
        />
        <span
          className="adv-value"
          title={
            value === null && autoReadout
              ? `${autoLabel}: ${autoReadout}`
              : undefined
          }
        >
          {value === null ? (
            <>
              {autoLabel}
              {autoReadout && (
                <span className="adv-auto-readout"> · {autoReadout}</span>
              )}
            </>
          ) : (
            format(value)
          )}
        </span>
        <input
          ref={inputRef}
          type="number"
          className="adv-number"
          min={min}
          max={max}
          step={step}
          disabled={disabled}
          value={draft !== null ? draft : value ?? ""}
          aria-label={`${label} value`}
          placeholder={autoLabel.toLowerCase()}
          onChange={(e) => {
            if (e.target.value === "") {
              onChange(null);
              setDraft(null);
            } else {
              setDraft(e.target.value);
            }
          }}
          onBlur={(e) => commitDraft(e.target.value)}
          onKeyDown={(e) => {
            if (e.key === "Enter") {
              commitDraft((e.target as HTMLInputElement).value);
              (e.target as HTMLInputElement).blur();
            } else if (e.key === "Escape") {
              setDraft(null);
              (e.target as HTMLInputElement).blur();
            }
          }}
          title={
            value === null
              ? `Type a number to engage, or leave blank for ${autoLabel}.`
              : `Type a value or clear to reset to ${autoLabel}.`
          }
        />
      </div>
    </div>
  );
}

export function SelectField({
  label,
  value,
  options,
  onChange,
}: {
  label: string;
  value: number | null;
  options: { value: number | null; label: string }[];
  onChange: (v: number | null) => void;
}) {
  return (
    <div className="adv-field">
      <span className="adv-label">{label}</span>
      <select
        className="adv-select"
        aria-label={label}
        value={value === null ? "" : String(value)}
        onChange={(e) => {
          const v = e.target.value;
          onChange(v === "" ? null : Number(v));
        }}
      >
        {options.map((o) => (
          <option key={o.label} value={o.value === null ? "" : String(o.value)}>
            {o.label}
          </option>
        ))}
      </select>
    </div>
  );
}
