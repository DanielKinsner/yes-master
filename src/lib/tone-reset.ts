import type { MasteringSettings } from "../bindings";
import { EQ_BAND_DEFAULTS } from "../bindings";

// "Fast reset" for the Visual EQ + Intensity area (the macros row): bring
// intensity back to its 50% neutral, flatten the seven EQ bands to 0 dB and
// put their frequencies back on the chain defaults (2026-08-18), leaving every other creative choice (preset, gains, delivery profile,
// volume match, advanced/compressor) exactly as the user left it. Kept as a
// pure helper so the reset's scope is pinned by tests, and so the hook can
// apply it as ONE settings mutation = one undo step.

/// Neutral intensity — mirrors `DEFAULT_SETTINGS.intensity` in
/// useTrackMaster and the Intensity knob's `defaultValue`, so "reset" and
/// "double-click the knob" agree on what neutral means.
export const TONE_DEFAULT_INTENSITY = 0.5;

export function resetToneSettings(settings: MasteringSettings): MasteringSettings {
  return {
    ...settings,
    intensity: TONE_DEFAULT_INTENSITY,
    eq_sub_db: 0,
    eq_low_db: 0,
    eq_low_mid_db: 0,
    eq_mid_db: 0,
    eq_high_mid_db: 0,
    eq_high_db: 0,
    eq_sparkle_db: 0,
    eq_bands: { ...EQ_BAND_DEFAULTS },
  };
}

/// True when every band sits on its default frequency (or the field is
/// absent, which the engine reads as the defaults).
export function eqBandsAreDefault(settings: MasteringSettings): boolean {
  const b = settings.eq_bands;
  if (!b) return true;
  return (Object.keys(EQ_BAND_DEFAULTS) as (keyof typeof EQ_BAND_DEFAULTS)[]).every(
    (k) => b[k] === EQ_BAND_DEFAULTS[k],
  );
}

/// True when the tone area is already at neutral — used to disable the reset
/// button so it can't fire a no-op edit (which would otherwise push an empty
/// undo step and re-dispatch the chain for nothing).
export function isToneFlat(settings: MasteringSettings): boolean {
  return (
    settings.intensity === TONE_DEFAULT_INTENSITY &&
    settings.eq_sub_db === 0 &&
    settings.eq_low_db === 0 &&
    settings.eq_low_mid_db === 0 &&
    settings.eq_mid_db === 0 &&
    settings.eq_high_mid_db === 0 &&
    settings.eq_high_db === 0 &&
    settings.eq_sparkle_db === 0 &&
    eqBandsAreDefault(settings)
  );
}
