export function Toast({
  message,
  tone = "danger",
  onClose,
  action,
}: {
  message: string;
  tone?: "danger" | "ok" | "info" | "warn";
  onClose: () => void;
  /// Optional inline action (Slice 7b: the updater's "Restart to update").
  /// When `disabled`, `disabledTitle` explains why on hover (mirrors the
  /// export controls' in-progress affordance).
  action?: {
    label: string;
    onClick: () => void;
    disabled?: boolean;
    disabledTitle?: string;
  };
}) {
  return (
    <div className={`toast toast-${tone}`}>
      <span>{message}</span>
      {action && (
        <button
          type="button"
          className="toast-action"
          onClick={action.onClick}
          disabled={action.disabled}
          title={action.disabled ? action.disabledTitle : undefined}
        >
          {action.label}
        </button>
      )}
      <button type="button" className="toast-close" onClick={onClose} aria-label="Dismiss">
        ×
      </button>
    </div>
  );
}
