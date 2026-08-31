/// One inline toast action (Slice 7b / audit L-03). When `disabled`,
/// `disabledTitle` explains why on hover (mirrors the export controls'
/// in-progress affordance).
export interface ToastAction {
  label: string;
  onClick: () => void;
  disabled?: boolean;
  disabledTitle?: string;
}

export function Toast({
  message,
  tone = "danger",
  onClose,
  actions,
}: {
  message: string;
  tone?: "danger" | "ok" | "info" | "warn";
  onClose: () => void;
  /// Optional inline actions, rendered in order (e.g. the updater failure
  /// toast's "Retry" + "Download manually" pair). The dismiss button is
  /// always present and separate.
  actions?: readonly ToastAction[];
}) {
  return (
    <div className={`toast toast-${tone}`}>
      <span>{message}</span>
      {actions && actions.length > 0 && (
        <span className="toast-actions">
          {actions.map((action) => (
            <button
              key={action.label}
              type="button"
              className="toast-action"
              onClick={action.onClick}
              disabled={action.disabled}
              title={action.disabled ? action.disabledTitle : undefined}
            >
              {action.label}
            </button>
          ))}
        </span>
      )}
      <button type="button" className="toast-close" onClick={onClose} aria-label="Dismiss">
        ×
      </button>
    </div>
  );
}
