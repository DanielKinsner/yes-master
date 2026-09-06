/** A shared studio signature, with the source name kept readable in every view. */
export function TrackIdentity({ name, standard = false }: { name: string; standard?: boolean }) {
  return (
    <div className="track-identity">
      <span className="track-identity-mark" aria-hidden="true">
        <svg width="28" height="28" viewBox="0 0 24 24" fill="currentColor">
          <path d="M4 6h2v12H4zM8 10h2v8H8zM12 4h2v16h-2zM16 8h2v10h-2zM20 12h2v6h-2z" />
        </svg>
      </span>
      <h1 className={standard ? "std-title" : "track-title"} title={name}>{name}</h1>
    </div>
  );
}
