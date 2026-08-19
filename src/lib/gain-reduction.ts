// Per-band compressor gain reduction, as the backend reports it on every
// playback tick: dB, NEGATIVE means reduction, -120 is the "no reduction /
// no signal" sentinel (audio.rs PlaybackTick gr_*_db). Alive pass 1
// (2026-08-19): the numbers were on the wire since Phase 12.2 and shown
// nowhere; these helpers put them on the per-band compressor card.

export const GR_SCALE_DB = 12;
const SILENT_THRESHOLD = -119;

export function grToFill(grDb: number | undefined): number {
  if (grDb === undefined || !Number.isFinite(grDb) || grDb <= SILENT_THRESHOLD) return 0;
  if (grDb >= 0) return 0;
  return Math.min(1, -grDb / GR_SCALE_DB);
}

export function grLabel(grDb: number | undefined): string {
  if (grDb === undefined || !Number.isFinite(grDb) || grDb <= SILENT_THRESHOLD) return "—";
  return `${grDb.toFixed(1)} dB`;
}
