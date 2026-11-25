import type { SmartMatchView } from "../lib/smart-match";
import {
  SMART_MATCH_BAND_LABELS,
  SMART_MATCH_COPY,
  displayReferenceName,
} from "../lib/smart-match";
import { NumberField } from "./fields";

// Smart Match (plan Slice 7). One row: the label, the Beta tag, the controls
// and the result line from the locked copy table. Standard shows the row;
// Advanced adds the six-band strip and the Match amount slider.

export interface SmartMatchActions {
  onChoose: () => void;
  onToggle: () => void;
  onRemove: () => void;
}

export function SmartMatchRow({
  view,
  onChoose,
  onToggle,
  onRemove,
  className = "",
}: SmartMatchActions & { view: SmartMatchView; className?: string }) {
  const album = view.state === "album_disabled";
  const hasReference = view.name !== null && !album;
  const busy = view.measuring;
  return (
    <div
      className={`smart-match-row ${className}`.trim()}
      aria-busy={busy || undefined}
      data-state={view.state}
    >
      <div className="smart-match-head">
        <span className="smart-match-label">
          {SMART_MATCH_COPY.label}
          <span className="smart-match-beta">{SMART_MATCH_COPY.tag}</span>
        </span>
        <div className="smart-match-controls">
          {!hasReference ? (
            <button
              type="button"
              className="smart-match-choose"
              disabled={album || busy}
              onClick={onChoose}
            >
              {SMART_MATCH_COPY.choose}
            </button>
          ) : (
            <>
              <span
                className="smart-match-name"
                // The full name on hover and on keyboard focus.
                title={view.name ?? undefined}
                tabIndex={0}
                aria-label={`Reference: ${view.name}`}
              >
                {displayReferenceName(view.name ?? "")}
              </span>
              <button
                type="button"
                role="switch"
                className="smart-match-switch"
                aria-checked={view.enabled}
                aria-label={SMART_MATCH_COPY.label}
                disabled={busy}
                onClick={onToggle}
              >
                <span className="smart-match-switch-knob" aria-hidden="true" />
              </button>
              <button
                type="button"
                className="smart-match-change"
                disabled={busy}
                onClick={onChoose}
              >
                {SMART_MATCH_COPY.change}
              </button>
              <button
                type="button"
                className="smart-match-remove"
                aria-label={SMART_MATCH_COPY.remove}
                title={SMART_MATCH_COPY.remove}
                disabled={busy}
                onClick={onRemove}
              >
                <span aria-hidden="true">×</span>
              </button>
            </>
          )}
        </div>
      </div>
      <p
        className={
          "smart-match-line" +
          (view.partial ? " is-partial" : "") +
          (view.pending ? " is-pending" : "")
        }
        role="status"
        aria-live="polite"
        title={view.line}
      >
        {view.line}
      </p>
    </div>
  );
}

function signed(db: number): string {
  if (Math.abs(db) < 0.05) return "0.0";
  return `${db > 0 ? "+" : "−"}${Math.abs(db).toFixed(1)}`;
}

/// Advanced: the row, the six-band strip (per band: the gap to the reference
/// and how much of it the match closes), the Match amount slider and the
/// strip captions.
export function SmartMatchCard({
  view,
  onChoose,
  onToggle,
  onRemove,
  onAmount,
}: SmartMatchActions & { view: SmartMatchView; onAmount: (amount: number) => void }) {
  const album = view.state === "album_disabled";
  const bands = view.bands;
  return (
    <section
      id="jump-smart-match"
      className="rail-section rail-card-smart-match"
      aria-label={SMART_MATCH_COPY.label}
    >
      <div className="rail-card-body">
        <SmartMatchRow
          view={view}
          onChoose={onChoose}
          onToggle={onToggle}
          onRemove={onRemove}
          className="is-advanced"
        />
        {!album && (
          <>
            <table className="smart-match-strip" aria-label="Smart Match bands">
              <thead>
                <tr>
                  <th scope="col">
                    <span className="sr-only">Band</span>
                  </th>
                  {SMART_MATCH_BAND_LABELS.map((label) => (
                    <th key={label} scope="col">
                      {label}
                    </th>
                  ))}
                </tr>
              </thead>
              <tbody>
                <tr>
                  <th scope="row">Gap</th>
                  {SMART_MATCH_BAND_LABELS.map((label, b) => {
                    const gap = bands?.[b]?.gap_db;
                    return (
                      <td key={label} aria-label={gap == null ? `${label} gap unknown` : `${label} gap ${signed(gap)} dB`}>
                        {gap == null ? "—" : signed(gap)}
                      </td>
                    );
                  })}
                </tr>
                <tr>
                  <th scope="row">Closed</th>
                  {SMART_MATCH_BAND_LABELS.map((label, b) => {
                    const band = bands?.[b];
                    const correction = band?.correction_db;
                    return (
                      <td
                        key={label}
                        className={band?.capped ? "is-capped" : undefined}
                        aria-label={
                          correction == null
                            ? `${label} correction unknown`
                            : `${label} correction ${signed(correction)} dB${band?.capped ? ", limited by a cap" : ""}`
                        }
                      >
                        {correction == null ? "—" : signed(correction)}
                      </td>
                    );
                  })}
                </tr>
              </tbody>
            </table>
            <NumberField
              label={SMART_MATCH_COPY.amountLabel}
              value={view.amount}
              min={0}
              max={1}
              step={0.05}
              format={(v) => `${Math.round(v * 100)}%`}
              disabled={view.name === null || view.measuring}
              onChange={(v) => onAmount(v ?? 0.5)}
            />
            <p className="smart-match-caption">{SMART_MATCH_COPY.stripCaption}</p>
            <p className="smart-match-caption">{SMART_MATCH_COPY.stripCaptionDynamics}</p>
          </>
        )}
      </div>
    </section>
  );
}
