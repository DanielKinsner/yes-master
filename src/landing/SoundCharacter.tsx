import universal from "../assets/presets/universal.png";
import clarity from "../assets/presets/clarity.png";
import tape from "../assets/presets/tape.png";
import oomph from "../assets/presets/oomph.png";
import standard from "../assets/landing/studio/standard-ui.png";
import tapeScene from "../assets/landing/studio/studio-tape.webp";
import tapeSceneSmall from "../assets/landing/studio/studio-tape-768.webp";
import copy from "./page-copy.json";
import { Capture } from "./StudioElements";

export default function SoundCharacter() {
  const c = copy.sound;
  const images = [universal, clarity, tape, oomph];
  return (
    <section id="sound" className="studio-section studio-sound">
      <div className="studio-shell">
        <div className="studio-sound-editorial">
          <div className="studio-sound-intro">
            <p className="eyebrow">{c.eyebrow}</p>
            <h2>
              {c.headline[0]}
              <em>{c.headline[1]}</em>
            </h2>
            <p className="studio-lead">{c.body}</p>
            <p className="studio-editorial-note">
              <span aria-hidden="true">01 — 04</span> A character for the sound
              in your head.
            </p>
          </div>
          <figure className="studio-tape-scene">
            <img
              src={tapeScene}
              srcSet={`${tapeSceneSmall} 768w, ${tapeScene} 1536w`}
              sizes="(max-width: 800px) 100vw, 55vw"
              width="1536"
              height="1024"
              loading="lazy"
              alt="Amber light across a brushed metal tape reel and its rollers"
            />
            <figcaption>CHARACTER YOU CAN FEEL.</figcaption>
          </figure>
        </div>
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
