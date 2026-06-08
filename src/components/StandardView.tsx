// src/components/StandardView.tsx
import type { Preset } from "../bindings";
import {
  STANDARD_STYLES,
  presetToStyle,
  styleToPreset,
} from "../lib/standard-mapping";

export function StyleTiles({
  preset,
  onSelect,
}: {
  preset: Preset;
  onSelect: (preset: Preset) => void;
}) {
  const activeStyle = presetToStyle(preset);
  return (
    <div className="std-tiles" role="group" aria-label="Style">
      {STANDARD_STYLES.map((s) => (
        <button
          key={s.id}
          type="button"
          className={"std-tile" + (s.id === activeStyle ? " is-active" : "")}
          data-tone={s.tone}
          aria-pressed={s.id === activeStyle}
          onClick={() => onSelect(styleToPreset(s.id))}
        >
          <span className="std-tile-label">{s.label}</span>
          <span className="std-tile-subtitle">{s.subtitle}</span>
        </button>
      ))}
    </div>
  );
}
