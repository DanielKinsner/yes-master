import desktopStandardImage from "./assets/landing/desktop-standard-ui.png";
import desktopAdvancedImage from "./assets/landing/desktop-advanced-ui.png";
import heroFullscreenImage from "./assets/landing/hero-control-room-studio.jpg";
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

type FeatureStory = {
  name: string;
  copy: string;
  icon: string;
};

const mobileLeftFeatures: FeatureStory[] = [
  {
    name: "A/B in sync",
    copy: "Original vs Mastered with volume match.",
    icon: "sync",
  },
  {
    name: "Four styles",
    copy: "Balanced, Warm, Open, Punch.",
    icon: "styles",
  },
  {
    name: "Intensity control",
    copy: "Subtle to Pushed.",
    icon: "intensity",
  },
];

const mobileRightFeatures: FeatureStory[] = [
  {
    name: "Real-time meters",
    copy: "LUFS, true peak, and gain reduction.",
    icon: "meters",
  },
  {
    name: "Quality checks",
    copy: "Instant feedback.",
    icon: "checks",
  },
  {
    name: "Create & export",
    copy: "Technically checked.",
    icon: "export",
  },
  {
    name: "No cloud",
    copy: "All on your device. All private.",
    icon: "local",
  },
];

export default function LandingPage() {
  return (
    <main className="landing-page">
      <nav className="landing-nav" aria-label="YES Master">
        <a className="landing-brand" href="#top" aria-label="YES Master home">
          <img src={brandIcon} alt="" />
          <span>Y.E.S. Master</span>
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
          className="landing-hero-scene"
          src={heroFullscreenImage}
          alt=""
          aria-hidden="true"
        />
        <div className="landing-hero-copy">
          <span className="landing-eyebrow">Y.E.S. Master / Your Endgame Sound</span>
          <h1 id="landing-title">
            Your Endgame <span>Sound.</span>
          </h1>
          <p>
            Drop in a real track. Hear the mastering chain in real time. Export
            a technically checked master without your music ever leaving your
            machine.
          </p>
          <div className="landing-actions">
            <a className="landing-button landing-button-primary" href="#get-started">
              Join desktop beta
            </a>
            <a className="landing-button landing-button-secondary" href="#advanced">
              See Advanced control
            </a>
          </div>
          <ul className="landing-proof" aria-label="YES Master highlights">
            <li>
              <strong>Local-first</strong>
              <span>
                Your tracks never leave your machine. No uploads, no cloud, no
                waiting.
              </span>
            </li>
            <li>
              <strong>Real-time control</strong>
              <span>
                Hear every change as you make it. Shape tone, loudness, and
                width by ear.
              </span>
            </li>
            <li>
              <strong>Release-ready</strong>
              <span>
                Technically checked and true-peak safe. Ship a master you can
                trust.
              </span>
            </li>
          </ul>
        </div>
        <a className="landing-next" href="#standard">
          <span>Next: the simple path</span>
          <span>Scroll</span>
        </a>
      </section>

      <section id="mobile" className="landing-mobile-stage" aria-labelledby="mobile-heading">
        <div className="landing-stage-heading">
          <p className="landing-label">The same endgame sound.</p>
          <h2 id="mobile-heading">Master anywhere. Same engine. Same truth.</h2>
          <p>The power of the studio in your pocket.</p>
        </div>

        <div className="landing-mobile-orbit">
          <div className="landing-feature-rail landing-feature-rail-left">
            {mobileLeftFeatures.map((feature) => (
              <article className="landing-feature-row" key={feature.name}>
                <span className={`landing-feature-icon landing-feature-icon-${feature.icon}`} aria-hidden="true" />
                <div>
                  <h3>{feature.name}</h3>
                  <p>{feature.copy}</p>
                </div>
              </article>
            ))}
          </div>

          <div className="landing-phone-stack" aria-label="YES Master mobile companion previews">
            <img
              className="landing-phone landing-phone-main"
              src={iphoneStandardImage}
              alt="YES Master iPhone Standard mastering workflow"
            />
            <div className="landing-phone landing-phone-meter" aria-hidden="true">
              <div className="landing-meter-top">Mastered</div>
              <strong>-11.0</strong>
              <span>LUFS integrated</span>
              <strong>-0.8</strong>
              <span>dBTP true peak</span>
              <div className="landing-meter-wave" />
              <div className="landing-meter-check">Quality checks <b>All good</b></div>
            </div>
          </div>

          <div className="landing-feature-rail landing-feature-rail-right">
            {mobileRightFeatures.map((feature) => (
              <article className="landing-feature-row" key={feature.name}>
                <span className={`landing-feature-icon landing-feature-icon-${feature.icon}`} aria-hidden="true" />
                <div>
                  <h3>{feature.name}</h3>
                  <p>{feature.copy}</p>
                </div>
              </article>
            ))}
          </div>
        </div>
      </section>

      <section id="standard" className="landing-product-deck" aria-labelledby="proof-heading">
        <span id="advanced" className="landing-anchor" aria-hidden="true" />
        <p className="landing-label">From fast finish to full control.</p>
        <h2 id="proof-heading">One engine. Three ways to trust it.</h2>

        <div className="landing-proof-panels">
          <article className="landing-proof-panel landing-proof-panel-standard">
            <span className="landing-panel-icon landing-panel-icon-wave" aria-hidden="true" />
            <p className="landing-panel-kicker">Standard - the hero</p>
            <h3>One click from finished to mastered.</h3>
            <p>
              Pick a style. Pick a loudness. Shape the intensity. Create
              Master. You get a clean 44.1 kHz / 24-bit WAV, true-peak safe,
              every time.
            </p>
            <div className="landing-mini-console" aria-label="Standard preset choices">
              <div className="landing-mini-presets">
                {presetStories.map((preset) => (
                  <span key={preset.name}>{preset.name}</span>
                ))}
              </div>
              <div className="landing-mini-button">Create Master</div>
            </div>
            <figure className="landing-panel-image landing-panel-image-standard">
              <img src={desktopStandardImage} alt="YES Master Standard desktop interface" />
            </figure>
          </article>

          <article className="landing-proof-panel landing-proof-panel-advanced">
            <span className="landing-panel-icon landing-panel-icon-sliders" aria-hidden="true" />
            <p className="landing-panel-kicker">Advanced - the proof</p>
            <h3>When you want the full room.</h3>
            <p>
              Eight presets, a 7-band EQ, compressor modes, width and warmth,
              live metering, delivery formats, and export review with a
              measured receipt.
            </p>
            <figure className="landing-panel-image">
              <img src={desktopAdvancedImage} alt="YES Master Advanced EQ and metering interface" />
            </figure>
          </article>

          <article className="landing-proof-panel landing-proof-panel-checked">
            <span className="landing-panel-icon landing-panel-icon-shield" aria-hidden="true" />
            <p className="landing-panel-kicker">Technically checked</p>
            <h3>Honest results. You decide.</h3>
            <p>
              Every master ships with a receipt: delivered LUFS, true peak,
              dynamic range, and quality checks. No guesswork. No surprises.
            </p>
            <dl className="landing-receipt">
              <div>
                <dt>Delivered LUFS</dt>
                <dd>-11.0 LUFS</dd>
              </div>
              <div>
                <dt>True Peak</dt>
                <dd>-0.8 dBTP</dd>
              </div>
              <div>
                <dt>Dynamic Range</dt>
                <dd>8.4 LU</dd>
              </div>
              <div>
                <dt>Quality Checks</dt>
                <dd>All good</dd>
              </div>
              <div>
                <dt>Status</dt>
                <dd>Ready to ship</dd>
              </div>
            </dl>
          </article>
        </div>
      </section>

      <section id="get-started" className="landing-final">
        <div>
          <h2>Stop chasing the master.</h2>
          <p>This is the one you stop on.</p>
        </div>
        <a className="landing-final-button" href="mailto:hello@yesmaster.app">
          <span>Download YES Master</span>
          <small>Works offline. No signup.</small>
        </a>
      </section>
    </main>
  );
}
