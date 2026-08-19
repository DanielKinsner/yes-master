// Meter ballistics for the live MASTER OUT meters (alive pass 1, 2026-08-19).
// Pure: no React, no timers. Callers pass `performance.now()`.
//
// A real meter rises instantly and FALLS at a controlled rate; a peak-hold
// pip sits at the highest value seen, waits `holdMs`, then sinks. Values are
// dB; `METER_SILENT` (-120) is the backend's silence sentinel
// (audio.rs SILENCE_DBFS) — treated as "no input": everything decays,
// nothing rises. The state carries `lastMs` so a step can compute `dt`
// without owning a clock.

export const METER_SILENT = -120;
const SILENT_THRESHOLD = -119;

export type MeterState = {
  display: number;
  hold: number;
  holdUntilMs: number;
  lastMs: number;
};

export type MeterConfig = {
  fallDbPerSec: number;
  holdMs: number;
  holdFallDbPerSec: number;
};

export const DEFAULT_METER_CONFIG: MeterConfig = {
  fallDbPerSec: 24,
  holdMs: 1000,
  holdFallDbPerSec: 12,
};

export function stepMeter(
  prev: MeterState | null,
  inputDb: number,
  nowMs: number,
  cfg: MeterConfig = DEFAULT_METER_CONFIG,
): MeterState {
  const input =
    Number.isFinite(inputDb) && inputDb > SILENT_THRESHOLD ? inputDb : METER_SILENT;
  if (!prev) {
    return { display: input, hold: input, holdUntilMs: nowMs + cfg.holdMs, lastMs: nowMs };
  }
  const dtSec = Math.max(0, (nowMs - prev.lastMs) / 1000);

  // Display: instant rise, rate-limited fall, never below the input.
  const fallen = prev.display - cfg.fallDbPerSec * dtSec;
  const display = Math.max(input, fallen, METER_SILENT);

  // Hold: re-arm on a new high; otherwise wait out holdMs, then sink.
  let hold = prev.hold;
  let holdUntilMs = prev.holdUntilMs;
  if (input >= prev.hold) {
    hold = input;
    holdUntilMs = nowMs + cfg.holdMs;
  } else if (nowMs > prev.holdUntilMs) {
    const overshootSec = (nowMs - Math.max(prev.holdUntilMs, prev.lastMs)) / 1000;
    hold = Math.max(input, prev.hold - cfg.holdFallDbPerSec * overshootSec, METER_SILENT);
  }
  return { display, hold, holdUntilMs, lastMs: nowMs };
}
