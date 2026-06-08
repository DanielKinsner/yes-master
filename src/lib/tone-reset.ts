import type { MasteringSettings } from "../bindings";

// "Fast reset" for the Visual EQ + Intensity area (the macros row): bring
// intensity back to its 50% neutral and flatten the seven EQ bands to 0 dB,
// leaving every other creative choice (preset, gains, delivery profile,
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
  };
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
    settings.eq_sparkle_db === 0
  );
}
