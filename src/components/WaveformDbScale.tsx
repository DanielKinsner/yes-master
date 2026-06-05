export function WaveformDbScale() {
  // Vertical dB scale at the right edge of the main waveform. The waveform
  // canvas is centered around 0 dB (mid-line), so we render ticks at -6,
  // -12, -18, -24 above AND below the centerline. Pure presentation — does
  // not change layout of the waveform itself (uses absolute positioning).
  const ticks = [0, -6, -12, -18, -24];
  return (
    <div className="wf-db-scale" aria-hidden>
      {ticks.map((db, i) => (
        <span
          key={`top-${db}`}
          className={`wf-db-tick${i === 0 ? " wf-db-tick-center" : ""}`}
        >
          {db === 0 ? "0" : db}
        </span>
      ))}
      {ticks.slice(1).map((db) => (
        <span key={`bot-${db}`} className="wf-db-tick">
          {db}
        </span>
      ))}
    </div>
  );
}
