import album from "../assets/landing/owner-album-session.jpg";
import standard from "../assets/landing/studio/standard-ui.png";
import copy from "./page-copy.json";
import { Capture, Icon, SectionHeading } from "./StudioElements";

export default function AlbumProof() {
  const c = copy.album;
  return (
    <section id="album" className="studio-section studio-album">
      <div className="studio-shell">
        <SectionHeading {...c} />
        <div className="studio-album-grid">
          <figure className="studio-album-tracks">
            {[1, 2, 3, 4].map((n) => (
              <div
                className={`studio-track ${n === 3 ? "studio-override" : ""}`}
                key={n}
              >
                <b>0{n}</b>
                <div className="studio-track-wave" aria-hidden="true">
                  <img
                    src={standard}
                    alt=""
                    width={2048}
                    height={1153}
                    loading="lazy"
                  />
                </div>
                <span>
                  {n === 3 ? "Your settings · Override" : "Follow album"}
                </span>
              </div>
            ))}
            <figcaption>Album delivery, illustrated.</figcaption>
          </figure>
          <div className="studio-shelf studio-folder">
            <div className="studio-folder-title">
              <Icon kind="folder" />
              <div>
                <h3>Everything, together.</h3>
                <p className="studio-label">YOUR ALBUM’S EXPORT FOLDER</p>
              </div>
            </div>
            <dl>
              {[
                ["Individual WAVs", "A master for each track"],
                ["Continuous WAV", "The record, together"],
                ["Manifest", "Delivery details"],
                ["Per-track receipts", "See each result"],
              ].map(([k, v]) => (
                <div key={k}>
                  <dt>{k}</dt>
                  <dd>{v}</dd>
                </div>
              ))}
            </dl>
          </div>
        </div>
        <div className="studio-album-details">
          {c.details.map((d) => (
            <article key={d.title}>
              <h3>{d.title}</h3>
              <p>{d.body}</p>
            </article>
          ))}
        </div>
        <p className="studio-limits">{c.limits}</p>
        <details className="studio-album-capture">
          <summary>
            Take a closer look at Album Master <span aria-hidden="true">↗</span>
          </summary>
          <Capture
            src={album}
            alt="YES Master Album Master view"
            width={2048}
            height={1147}
          />
        </details>
        <div className="studio-shelf studio-project">
          <div>
            <h3>{c.project_title}</h3>
            <p>{c.project_body}</p>
          </div>
          <p className="studio-label">
            .AMS.JSON PROJECTS
            <br />
            SAVE AND REOPEN SESSIONS
          </p>
        </div>
      </div>
    </section>
  );
}
