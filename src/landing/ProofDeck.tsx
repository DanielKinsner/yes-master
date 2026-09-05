import advanced from "../assets/landing/studio/advanced-ui.png";

import copy from "./page-copy.json";
import { Capture, DetailList, SectionHeading } from "./StudioElements";

export default function ProofDeck() {
  const c = copy.advanced;
  const e = copy.export;
  return (
    <>
      <section id="advanced" className="studio-section">
        <div className="studio-shell">
          <p className="eyebrow">{c.eyebrow}</p>
          <div className="studio-live-grid">
            <div>
              <h2>
                {c.headline[0]}
                <em>{c.headline[1]}</em>
              </h2>
              <p className="studio-lead">{c.body}</p>
              <div className="studio-pills">
                <span>Same-playhead A/B</span>
                <span>Live metering</span>
              </div>
            </div>
            <div>
              <Capture
                src={advanced}
                alt="YES Master Advanced playback and meters"
                width={910}
                height={332.09}
                crop={[1402.32, 232.12, 43.14]}
                caption="Hear the difference. Keep your place."
              />
              <p className="studio-capture-note">
                {c.ab_note} Volume Match is optional and off by default.
              </p>
            </div>
          </div>
          <div className="studio-control-grid">
            {c.details.map((d, i) => (
              <article className="studio-shelf" key={d.title}>
                <p className="studio-label">
                  0{i + 1} / {i === 0 ? "TONE" : "DYNAMICS"}
                </p>
                <h3>{d.title}</h3>
                <p>{d.body}</p>
                <Capture
                  src={advanced}
                  alt={
                    i === 0
                      ? "Seven-band visual EQ"
                      : "Preset, Manual and Off compressor modes"
                  }
                  width={i === 0 ? 712 : 319}
                  height={i === 0 ? 280.72 : 190.31}
                  crop={
                    i === 0
                      ? [2613.22, 1065.45, 1084.59]
                      : [1856, 1531.56, 496.62]
                  }
                />
              </article>
            ))}
          </div>
          <div className="studio-pills studio-support">
            {c.support.map((s) => (
              <span key={s}>{s}</span>
            ))}
          </div>
        </div>
      </section>
      <section id="export" className="studio-section studio-export">
        <div className="studio-shell">
          <SectionHeading eyebrow={e.eyebrow} headline={e.headline} />
          <div className="studio-export-grid">
            <div>
              <p className="studio-lead">{e.body}</p>
              <DetailList items={e.details} />
              <p>{e.source_note}</p>
            </div>
            <figure className="studio-receipt studio-shelf">
              <p className="studio-label">
                YES MASTER <span aria-hidden="true">↗</span>
              </p>
              <h3>Export receipt.</h3>
              <figcaption>
                Report fields, illustrated.
                <br />
                Actual measurements populate after rendering.
              </figcaption>
              <dl>
                {[
                  ["Delivered loudness", "LUFS"],
                  ["True peak", "dBTP"],
                  ["Loudness range", "LU"],
                  ["Quality checks", "Measured result"],
                ].map(([label, unit]) => (
                  <div key={label}>
                    <dt>{label}</dt>
                    <dd>{unit}</dd>
                  </div>
                ))}
              </dl>
              <p>Your final file. The facts behind the sound.</p>
            </figure>
          </div>
          <div className="studio-export-modes">
            <div>
              <h3>
                <span>Standard</span> · A clean finish.
              </h3>
              <p>{e.standard}</p>
            </div>
            <div>
              <h3>
                <em>Advanced</em> · The detail behind it.
              </h3>
              <p>{e.advanced}</p>
            </div>
          </div>
        </div>
      </section>
    </>
  );
}
