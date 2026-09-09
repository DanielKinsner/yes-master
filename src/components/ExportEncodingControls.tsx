import { EXPORT_FORMATS, type ExportFormat } from "../lib/export-formats";

export interface ExportEncodingChoice {
  format: ExportFormat;
  bitrate: number;
  quality?: number;
  onFormat: (format: ExportFormat) => void;
  onBitrate: (bitrate: number) => void;
  onQuality?: (quality: number) => void;
}

export function ExportEncodingControls({
  choice,
}: {
  choice: ExportEncodingChoice;
}) {
  return (
    <div className="export-encoding-controls">
      <label className="field">
        <span>File format</span>
        <select
          aria-label="Export file format"
          value={choice.format}
          onChange={(e) => choice.onFormat(e.target.value as ExportFormat)}
        >
          {Object.entries(EXPORT_FORMATS).map(([value, item]) => <option key={value} value={value}>{item.label}</option>)}
        </select>
      </label>
      {["mp3", "m4a", "aac"].includes(choice.format) && (
        <label className="field">
          <span>Bitrate</span>
          <select
            aria-label={choice.format === "mp3" ? "MP3 bitrate" : "AAC target bitrate"}
            value={choice.bitrate}
            onChange={(e) => choice.onBitrate(Number(e.target.value))}
          >
            {[320, 256, 192, 128].map((rate) => (
              <option key={rate} value={rate}>
                {rate} kbps
              </option>
            ))}
          </select>
        </label>
      )}
      {choice.format === "ogg" && <label className="field"><span>Quality</span>
        <select aria-label="Vorbis quality" value={choice.quality ?? 6} onChange={(event) => choice.onQuality?.(Number(event.target.value))}>
          {[4,6,8].map(quality => <option key={quality} value={quality}>VBR {quality}</option>)}
        </select>
      </label>}
    </div>
  );
}
