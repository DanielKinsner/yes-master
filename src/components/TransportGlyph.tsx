// One play/pause glyph for every transport disc (Standard's `.std-play`,
// Advanced's `.play-btn`). Both views used to render different Unicode
// characters (► / ❚❚ vs ▶ / ⏸) that fall back to whatever font the OS has,
// so the two discs never matched each other — or themselves across
// Windows and macOS. An inline SVG is the same pixels everywhere.
//
// The triangle is nudged right of centre on purpose: a geometrically centred
// play triangle reads as sitting too far left (optical centring).
export function PlayPauseGlyph({
  playing,
  size = 22,
}: {
  playing: boolean;
  size?: number;
}) {
  return (
    <svg
      className={"transport-glyph" + (playing ? " is-pause" : " is-play")}
      width={size}
      height={size}
      viewBox="0 0 24 24"
      aria-hidden="true"
      focusable="false"
    >
      {playing ? (
        <>
          <rect x="6" y="5" width="4.2" height="14" rx="1.2" fill="currentColor" />
          <rect x="13.8" y="5" width="4.2" height="14" rx="1.2" fill="currentColor" />
        </>
      ) : (
        <path d="M8.2 5.2 L18.6 12 L8.2 18.8 Z" fill="currentColor" strokeLinejoin="round" />
      )}
    </svg>
  );
}
