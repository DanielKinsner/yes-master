export const AUDIO_EXTENSIONS = ["wav", "mp3", "m4a", "aac", "flac", "ogg"] as const;

export type AudioExtension = (typeof AUDIO_EXTENSIONS)[number];

export const AUDIO_DIALOG_FILTER = {
  name: "Audio",
  extensions: [...AUDIO_EXTENSIONS],
};

export const SUPPORTED_FORMATS_COPY = AUDIO_EXTENSIONS.map((ext) =>
  ext.toUpperCase(),
).join(" · ");

export function supportedAudioExtensionFromName(name: string): AudioExtension | null {
  const ext = name.split(".").pop()?.toLowerCase();
  if (!ext) return null;
  return AUDIO_EXTENSIONS.includes(ext as AudioExtension) ? (ext as AudioExtension) : null;
}
