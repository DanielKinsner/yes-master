export function Toast({
  message,
  tone = "danger",
  onClose,
}: {
  message: string;
  tone?: "danger" | "ok" | "info" | "warn";
  onClose: () => void;
}) {
  return (
    <div className={`toast toast-${tone}`}>
      <span>{message}</span>
      <button type="button" className="toast-close" onClick={onClose} aria-label="Dismiss">
        ×
      </button>
    </div>
  );
}
