export function EmptyState({ onAdd }: { onAdd: () => void }) {
  return (
    <div className="empty-state">
      <div className="empty-state-glyph" aria-hidden>
        <svg width="64" height="64" viewBox="0 0 64 64" fill="none">
          <defs>
            <linearGradient id="emptyglow" x1="0" y1="0" x2="0" y2="1">
              <stop offset="0" stopColor="#6fa3ff" />
              <stop offset="1" stopColor="#2a6bf2" />
            </linearGradient>
          </defs>
          <circle cx="32" cy="32" r="28" stroke="url(#emptyglow)" strokeWidth="2" opacity="0.5" />
          <path
            d="M14 32h4l2-12 4 24 4-18 4 14 4-10 4 8 4-6 4 4h4"
            stroke="url(#emptyglow)"
            strokeWidth="2"
            strokeLinecap="round"
            strokeLinejoin="round"
            fill="none"
          />
        </svg>
      </div>
      <h1>Drop audio, analyze, export.</h1>
      <p>
        YES Master masters one track or a full album. Universal-first — no genre
        wizard, no jargon walls.
      </p>
      <button type="button" className="primary" onClick={onAdd}>
        Import audio
      </button>
      <p className="empty-foot">
        Supports WAV · AIFF · FLAC · MP3 · M4A · AAC · OGG · Opus.
      </p>
    </div>
  );
}
