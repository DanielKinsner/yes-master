import universal from "../assets/presets/universal.png";
import clarity from "../assets/presets/clarity.png";
import tape from "../assets/presets/tape.png";
import oomph from "../assets/presets/oomph.png";
import tapeScene from "../assets/landing/studio/studio-tape.webp";
import tapeSceneSmall from "../assets/landing/studio/studio-tape-768.webp";
import copy from "./page-copy.json";

export default function SoundCharacter() {
  const images = [universal, clarity, tape, oomph];
  return (
    <div id="sound" className="studio-character-panel">
      <figure className="studio-tape-scene">
        <img
          src={tapeScene}
          srcSet={`${tapeSceneSmall} 768w, ${tapeScene} 1536w`}
          sizes="(max-width: 800px) 100vw, 55vw"
          width="1536"
          height="1024"
          loading="lazy"
          alt="Amber light across a brushed metal tape reel and its rollers"
          data-studio-reveal="photo"
        />
        <figcaption>FIND YOUR SOUND. KEEP YOUR SIGNATURE.</figcaption>
      </figure>
      <div className="studio-style-strip">
        {copy.sound.styles.map((style, i) => (
          <div key={style.name}>
            <img
              src={images[i]}
              alt=""
              width="256"
              height="256"
              loading="lazy"
            />
            <h3>{style.name}</h3>
            <p>{style.body}</p>
          </div>
        ))}
      </div>
    </div>
  );
}
