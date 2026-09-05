import universal from "../assets/presets/universal.png";
import clarity from "../assets/presets/clarity.png";
import tape from "../assets/presets/tape.png";
import oomph from "../assets/presets/oomph.png";
import standard from "../assets/landing/studio/standard-ui.png";
import copy from "./page-copy.json";
import { Capture, SectionHeading } from "./StudioElements";

export default function SoundCharacter() {
  const c = copy.sound;
  const images = [universal, clarity, tape, oomph];
  return (
    <section id="sound" className="studio-section studio-sound">
      <div className="studio-shell">
        <SectionHeading {...c} />
        <div className="studio-style-grid">
          {c.styles.map((style, i) => (
            <article
              key={style.name}
              className={`studio-style studio-style-${style.name.toLowerCase()}`}
            >
              {i === 2 && (
                <span className="studio-style-tag">FEATURED STYLE</span>
              )}
              <img
                src={images[i]}
                alt=""
                width="256"
                height="256"
                loading="lazy"
              />
              <h3>{style.name}</h3>
              <p>{style.body}</p>
            </article>
          ))}
        </div>
        <div className="studio-shelf studio-intensity">
          <div>
            <h3>
              {c.intensity_headline[0]}
              <br />
              {c.intensity_headline[1]}
            </h3>
            <p>{c.intensity_body}</p>
          </div>
          <Capture
            src={standard}
            alt="Standard Intensity control"
            width={246}
            height={229.77}
            crop={[2557.4, 690.55, 952.78]}
            sourceSize={[2048, 1153]}
            caption="Intensity, in Standard"
          />
          <div className="studio-adaptive">
            <p className="studio-label">SOURCE-AWARE RESTRAINT</p>
            <h3>{c.adaptive_heading}</h3>
            <p>{c.adaptive_body}</p>
          </div>
        </div>
        <p className="studio-footnote">{c.advanced_note}</p>
      </div>
    </section>
  );
}
