// src/components/HintChip.tsx
//
// One quiet first-run hint. Never blocks input, never dims the screen,
// never traps focus — a small floating note with a dismiss ×.

import type { ReactNode } from "react";

export function HintChip({
  children,
  onDismiss,
  className = "",
}: {
  children: ReactNode;
  onDismiss: () => void;
  className?: string;
}) {
  return (
    <div className={`hint-chip ${className}`.trim()} role="status">
      <span className="hint-chip-text">{children}</span>
      <button
        type="button"
        className="hint-chip-x"
        aria-label="Dismiss first-run tips"
        onClick={onDismiss}
      >
        ×
      </button>
    </div>
  );
}
