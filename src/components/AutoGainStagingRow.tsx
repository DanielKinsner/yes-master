import type { LoudnessPriority } from "../bindings";
import {
  AUTO_GAIN_STAGING_COPY,
  LOUDNESS_PRIORITIES,
  type AutoGainStagingView,
} from "../lib/auto-gain-staging";

// Auto gain staging (plan Slice 6). Advanced only, beside the gain controls:
// the switch with its Beta tag, its line from the locked copy table, and the
// Loudness priority choice while the switch is on.

export interface AutoGainStagingActions {
  onToggle: () => void;
  onPriority: (priority: LoudnessPriority) => void;
}

export function AutoGainStagingRow({
  view,
  onToggle,
  onPriority,
}: AutoGainStagingActions & { view: AutoGainStagingView }) {
  return (
    <div
      className="ags-row"
      data-state={view.state}
      aria-busy={view.pending || undefined}
    >
      <div className="ags-head">
        <span className="ags-label" id="ags-label">
          {AUTO_GAIN_STAGING_COPY.label}
          <span className="ags-beta">{AUTO_GAIN_STAGING_COPY.tag}</span>
        </span>
        <div className="ags-controls">
          <button
            type="button"
            role="switch"
            className="ags-switch"
            aria-checked={view.enabled && !view.disabled}
            aria-labelledby="ags-label"
            disabled={view.disabled}
            title={view.disabled ? AUTO_GAIN_STAGING_COPY.album : undefined}
            onClick={onToggle}
          >
            <span className="ags-switch-knob" aria-hidden="true" />
          </button>
        </div>
      </div>
      <p
        className={"ags-line" + (view.pending ? " is-pending" : "")}
        role="status"
        aria-live="polite"
      >
        {view.line}
      </p>
      {view.showPriority && (
        <div className="ags-priority">
          <span className="ags-priority-label" id="ags-priority-label">
            {AUTO_GAIN_STAGING_COPY.priority}
          </span>
          <div
            className="compressor-mode-tabs ags-priority-options"
            role="radiogroup"
            aria-labelledby="ags-priority-label"
          >
            {LOUDNESS_PRIORITIES.map((option) => (
              <button
                key={option.value}
                type="button"
                role="radio"
                aria-checked={view.priority === option.value}
                className={
                  "compressor-mode-tab" +
                  (view.priority === option.value ? " is-active" : "")
                }
                onClick={() => onPriority(option.value)}
              >
                {option.label}
              </button>
            ))}
          </div>
        </div>
      )}
    </div>
  );
}
