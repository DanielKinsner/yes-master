export function formatDuration(sec: number | null | undefined): string {
  if (sec == null || !Number.isFinite(sec)) return "";

  const total = Math.max(0, Math.floor(sec));
  const minutes = Math.floor(total / 60);
  const seconds = total % 60;

  return `${minutes}:${seconds.toString().padStart(2, "0")}`;
}
