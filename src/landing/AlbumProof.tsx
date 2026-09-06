import album from "../assets/landing/owner-album-session.jpg";
import copy from "./page-copy.json";
import { Capture } from "./StudioElements";

export default function AlbumProof() {
  const c = copy.album;
  return (
    <section id="album" className="studio-section studio-album">
      <div className="studio-shell">
        <div className="studio-album-editorial">
          <div>
            <p className="eyebrow">{c.eyebrow}</p>
            <h2>
              {c.headline[0]}
              <em>{c.headline[1]}</em>
            </h2>
          </div>
          <div>
            <p className="studio-lead">{c.body}</p>
            <p>
              Export individual masters, a continuous WAV and per-track
              receipts. Save your session and pick up where you left off.
            </p>
            <p className="studio-limits">{c.limits}</p>
          </div>
        </div>
        <details className="studio-album-capture">
          <summary>
            Take a closer look at Album Master{" "}
            <span aria-hidden="true">↗</span>
          </summary>
          <Capture
            src={album}
            alt="YES Master Album Master view"
            width={2048}
            height={1147}
          />
        </details>
      </div>
    </section>
  );
}
