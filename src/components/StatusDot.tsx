export function StatusDot({
  tone,
  label,
}: {
  tone: "idle" | "ok" | "warn" | "bad";
  label: string;
}) {
  return (
    <span className={`status-dot-row status-dot-${tone}`} title={label}>
      <span className="status-dot-glyph" aria-hidden />
      <span className="status-dot-label">{label}</span>
    </span>
  );
}
