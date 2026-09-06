export interface ExportEncodingChoice {
  format: "wav" | "mp3";
  bitrate: number;
  onFormat: (format: "wav" | "mp3") => void;
  onBitrate: (bitrate: number) => void;
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
          onChange={(e) => choice.onFormat(e.target.value as "wav" | "mp3")}
        >
          <option value="wav">WAV</option>
          <option value="mp3">MP3</option>
        </select>
      </label>
      {choice.format === "mp3" && (
        <label className="field">
          <span>Bitrate</span>
          <select
            aria-label="MP3 bitrate"
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
    </div>
  );
}
