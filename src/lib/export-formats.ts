// Encoding is captured per export, separately from the mastering controls.
export type ExportFormat = "wav" | "mp3" | "flac" | "m4a" | "aac" | "ogg" | "aiff";
export type ExportEncoding =
  | { format: "wav" | "flac" | "aiff" }
  | { format: "mp3" | "m4a" | "aac"; bitrate_kbps: number }
  | { format: "ogg"; quality: number };
export interface DeliveredFormat {
  encoding: ExportEncoding;
  codec: string;
  container: string;
  sample_rate: number;
  channels: number;
  bit_depth: number | null;
  requested_sample_rate: number | null;
  requested_bit_depth: number;
}
export const EXPORT_FORMATS: Record<ExportFormat, { label: string; extension: string; lossy: boolean }> = {
  wav: { label: "WAV", extension: "wav", lossy: false },
  mp3: { label: "MP3", extension: "mp3", lossy: true },
  flac: { label: "FLAC", extension: "flac", lossy: false },
  m4a: { label: "AAC / M4A", extension: "m4a", lossy: true },
  aac: { label: "AAC (ADTS)", extension: "aac", lossy: true },
  ogg: { label: "Ogg Vorbis", extension: "ogg", lossy: true },
  aiff: { label: "AIFF", extension: "aiff", lossy: false },
};
export function exportEncoding(format: ExportFormat, bitrate = format === "mp3" ? 320 : 256, quality = 6): ExportEncoding {
  if (format === "mp3" || format === "m4a" || format === "aac") {
    if (![128, 192, 256, 320].includes(bitrate)) throw new Error("Unsupported export bitrate");
    return { format, bitrate_kbps: bitrate };
  }
  if (format === "ogg") {
    if (![4, 6, 8].includes(quality)) throw new Error("Unsupported Vorbis quality");
    return { format, quality };
  }
  return { format };
}

export function exportApiArgs(encoding: ExportEncoding): [number?, ExportEncoding?] {
  return encoding.format === "wav" ? [] : encoding.format === "mp3" ? [encoding.bitrate_kbps] : [undefined, encoding];
}
export function ensureExportExtension(path: string, format: ExportFormat): string {
  return path.replace(/\.(wav|mp3|flac|m4a|aac|ogg|aif|aiff)$/i, "") + `.${EXPORT_FORMATS[format].extension}`;
}
export function encodingQuality(encoding: ExportEncoding, bits?: number | null): string {
  if ("bitrate_kbps" in encoding) return `${encoding.bitrate_kbps} kbps`;
  if ("quality" in encoding) return `VBR quality ${encoding.quality}`;
  return bits ? `${bits}-bit${bits === 32 ? " float" : ""}` : "Lossless";
}
export function deliveredFormatLabel(facts: DeliveredFormat): string {
  return `${EXPORT_FORMATS[facts.encoding.format].label} · ${encodingQuality(facts.encoding, facts.bit_depth)}`;
}
