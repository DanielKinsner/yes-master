import type { MasteringSettings, SmartMatchSettings } from "../bindings";

/**
 * A saved preset's settings applied to a song. A Smart Match reference belongs
 * to the song, never to a preset (Smart Match plan, M12): whatever the preset
 * carries is dropped (the backend already strips it on save and on read), and
 * the song's current Smart Match state is kept, absent included.
 */
export function presetSettingsKeepingSmartMatch(
  preset: MasteringSettings,
  current: SmartMatchSettings | null | undefined,
): MasteringSettings {
  const advanced = { ...preset.advanced };
  delete advanced.smart_match;
  if (current) {
    advanced.smart_match = current;
  }
  return { ...preset, advanced };
}
