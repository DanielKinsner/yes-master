import desktopStandardImage from "./assets/landing/desktop-standard-ui.png";
import desktopAdvancedImage from "./assets/landing/desktop-advanced-ui.png";
import iphoneStandardImage from "./assets/landing/iphone-standard-ui.jpg";
import brandIcon from "./assets/landing/yes-master-icon.png";

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

const advancedStories: PresetStory[] = [
  {
    name: "Creative control",
    tone: "Main surface",
    copy: "Shape EQ, tone, warmth, air, stereo width, saturation, limiter feel, and the full preset set while the track keeps playing.",
  },
  {
    name: "Judgment rail",
    tone: "Right rail",
    copy: "Quality checks, delivery profile, compressor mode, format, and warning-aware export review stay visible beside the sound.",
  },
  {
    name: "Album path",
    tone: "Long form",
    copy: "Build album-wide delivery with continuous renders, per-track files, mixed-source handling, and a manifest for the release folder.",
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
          <a href="#advanced">Advanced</a>
          <a href="#mobile">Mobile</a>
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

      <section id="advanced" className="landing-section landing-advanced">
        <div className="landing-advanced-copy">
          <p className="landing-label">Advanced</p>
          <h2>When the track needs hands on the chain.</h2>
          <p>
            Advanced opens the mastering desk without hiding the consequences.
            The main surface stays musical: waveform, A/B audition, presets,
            intensity, EQ, width, warmth, saturation, compressor behavior, and
            limiter feel. The right rail stays honest: source checks, delivery
            profile, format, review warnings, and export decisions.
          </p>
        </div>

        <figure className="landing-advanced-frame">
          <img
            src={desktopAdvancedImage}
            alt="YES Master Advanced desktop interface with waveform, EQ, compressor controls, delivery profile, and export review rail"
          />
          <figcaption>
            Creative controls stay in the workspace. Review, format, and export
            decisions stay in the rail.
          </figcaption>
        </figure>

        <div className="landing-advanced-grid" aria-label="YES Master Advanced capabilities">
          {advancedStories.map((story) => (
            <article className="landing-advanced-card" key={story.name}>
              <span>{story.tone}</span>
              <h3>{story.name}</h3>
              <p>{story.copy}</p>
            </article>
          ))}
        </div>

        <ol className="landing-advanced-flow" aria-label="Advanced workflow">
          <li>
            <span>01</span>
            <strong>Listen</strong>
            Original and Mastered stay locked to the same playhead.
          </li>
          <li>
            <span>02</span>
            <strong>Shape</strong>
            Dial musical changes without leaving the audition surface.
          </li>
          <li>
            <span>03</span>
            <strong>Review</strong>
            Loudness, true peak, dynamic range, and integrity checks stay visible.
          </li>
          <li>
            <span>04</span>
            <strong>Deliver</strong>
            Export a master, or build the album path when the release is bigger
            than one track.
          </li>
        </ol>
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
