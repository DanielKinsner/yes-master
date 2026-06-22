import desktopStandardImage from "./assets/landing/desktop-standard-ui.png";
import iphoneStandardImage from "./assets/landing/iphone-standard-ui.jpg";
import brandIcon from "../src-tauri/icons/32x32.png";

type PresetStory = {
  name: string;
  tone: string;
  copy: string;
};

const presetStories: PresetStory[] = [
  {
    name: "Universal",
    tone: "Balanced",
    copy: "A clean, release-minded starting point for most tracks.",
  },
  {
    name: "Clarity",
    tone: "Open",
    copy: "Lift presence and detail without turning the whole mix brittle.",
  },
  {
    name: "Tape",
    tone: "Warm",
    copy: "Round the edges and bring the track closer to a finished record.",
  },
  {
    name: "Oomph",
    tone: "Punch",
    copy: "Add weight and impact when the song needs a firmer push.",
  },
];

export default function LandingPage() {
  return (
    <main className="landing-page">
      <nav className="landing-nav" aria-label="YES Master">
        <a className="landing-brand" href="#top" aria-label="YES Master home">
          <img src={brandIcon} alt="" />
          <span>YES Master</span>
        </a>
        <div className="landing-nav-links">
          <a href="#standard">Standard</a>
          <a href="#mobile">Mobile</a>
          <a href="#advanced">Advanced</a>
          <a className="landing-nav-cta" href="#get-started">
            Join desktop beta
          </a>
        </div>
      </nav>

      <section id="top" className="landing-hero" aria-labelledby="landing-title">
        <img
          className="landing-hero-media"
          src={desktopStandardImage}
          alt=""
          aria-hidden="true"
        />
        <div className="landing-hero-shade" aria-hidden="true" />
        <div className="landing-hero-copy">
          <h1 id="landing-title">YES Master</h1>
          <p>
            Local-first mastering for musicians who want to hear the change,
            make the call, and export a technically checked master without
            sending private audio to the cloud.
          </p>
          <div className="landing-actions">
            <a className="landing-button landing-button-primary" href="#get-started">
              Join desktop beta
            </a>
            <a className="landing-button landing-button-secondary" href="#standard">
              See Standard mode
            </a>
          </div>
          <ul className="landing-proof" aria-label="YES Master highlights">
            <li>Mac and Windows desktop</li>
            <li>Original/Mastered audition at the same playhead</li>
            <li>Local audio, measured exports, no source overwrite by default</li>
          </ul>
        </div>
        <a className="landing-next" href="#standard">
          <span>Next: the simple path</span>
          <span>Scroll</span>
        </a>
      </section>

      <section id="standard" className="landing-section landing-standard">
        <div className="landing-section-copy">
          <p className="landing-label">Standard</p>
          <h2>Three decisions. One finished master.</h2>
          <p>
            Pick a style, set intensity, choose Low, Medium, or High loudness,
            then create a fixed 44.1 kHz / 24-bit WAV at a -1 dBTP ceiling.
            Standard is the fast path for finishing the track without pretending
            the technical details disappeared.
          </p>
        </div>
        <figure className="landing-product-frame">
          <img src={desktopStandardImage} alt="YES Master Standard desktop interface" />
        </figure>
      </section>

      <section className="landing-section landing-presets" aria-labelledby="preset-heading">
        <div className="landing-section-copy">
          <p className="landing-label">Styles</p>
          <h2 id="preset-heading">Four ways to finish the feel.</h2>
        </div>
        <div className="landing-preset-grid">
          {presetStories.map((preset) => (
            <article className="landing-preset" key={preset.name}>
              <span>{preset.tone}</span>
              <h3>{preset.name}</h3>
              <p>{preset.copy}</p>
            </article>
          ))}
        </div>
      </section>

      <section id="mobile" className="landing-section landing-mobile">
        <div className="landing-mobile-media">
          <img src={iphoneStandardImage} alt="YES Master iPhone companion interface" />
        </div>
        <div className="landing-section-copy">
          <p className="landing-label">Mobile companions</p>
          <h2>The simple workflow belongs on the phone too.</h2>
          <p>
            The iPhone and Android companion apps reuse the same mastering
            engine, shaped around the Standard flow: load a track, compare
            Original and Mastered, choose the style, set intensity, pick
            loudness, and create the master.
          </p>
          <p>
            Desktop still leads the release. Mobile extends the same local,
            listen-first promise into a smaller, faster surface.
          </p>
        </div>
      </section>

      <section id="advanced" className="landing-section landing-advanced">
        <div className="landing-section-copy">
          <p className="landing-label">Advanced</p>
          <h2>Open the control room when the track asks for it.</h2>
          <p>
            Advanced keeps creative sound in the main surface and judgment in
            the rail: visual EQ, width, warmth, compressor modes, delivery
            profiles, quality checks, export review, and album rendering paths.
          </p>
        </div>
        <div className="landing-feature-list">
          <article>
            <h3>Hear it live</h3>
            <p>Controls respond during playback, and A/B comparison preserves the playhead.</p>
          </article>
          <article>
            <h3>See the consequences</h3>
            <p>Rendered output is checked for loudness, true peak, dynamic range, and integrity.</p>
          </article>
          <article>
            <h3>Export with intent</h3>
            <p>Use safe Standard defaults or open Advanced delivery settings for the target.</p>
          </article>
        </div>
      </section>

      <section id="get-started" className="landing-final">
        <h2>Finish the song. Keep the source safe.</h2>
        <p>
          YES Master is built for private, real-world mastering on your own
          machine, from quick Standard exports to deeper Advanced review.
        </p>
        <a className="landing-button landing-button-primary" href="mailto:hello@yesmaster.app">
          Request access
        </a>
      </section>
    </main>
  );
}
