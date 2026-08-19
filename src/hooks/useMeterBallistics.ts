import { useRef } from "react";
import {
  DEFAULT_METER_CONFIG,
  METER_SILENT,
  stepMeter,
  type MeterConfig,
  type MeterState,
} from "../lib/meter-ballistics";

/// Runs the pure meter ballistics once per render against `valueDb`.
/// `active=false` (idle/stopped) resets the state and reports nothing, so a
/// stopped meter never shows a stale hold. The playback tick cadence
/// (~20 Hz) is the clock; no timers are started here, and nothing about
/// audition, playhead, or render timing is touched.
export function useMeterBallistics(
  valueDb: number | undefined,
  active: boolean,
  cfg: MeterConfig = DEFAULT_METER_CONFIG,
): { display: number | undefined; hold: number | undefined } {
  const stateRef = useRef<MeterState | null>(null);
  if (!active) {
    stateRef.current = null;
    return { display: undefined, hold: undefined };
  }
  const now = typeof performance !== "undefined" ? performance.now() : Date.now();
  const next = stepMeter(stateRef.current, valueDb ?? METER_SILENT, now, cfg);
  stateRef.current = next;
  return {
    display: next.display > -119 ? next.display : undefined,
    hold: next.hold > -119 ? next.hold : undefined,
  };
}
