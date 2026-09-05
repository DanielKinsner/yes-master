import studio from "../assets/landing/studio/hero-bg-studio.webp";
import standard from "../assets/landing/studio/hero-device-standard.webp";
import advancedChassis from "../assets/landing/studio/advanced-laptop-front.webp";
import advanced from "../assets/landing/studio/advanced-ui.png";
import { resolveRelease, type ResolvedRelease } from "./release-config";
import copy from "./page-copy.json";
import { Icon } from "./StudioElements";

export default function Hero({
  release = resolveRelease(),
}: { release?: ResolvedRelease } = {}) {
  return (
    <>
      <section id="top" className="studio-hero">
        <img
          className="studio-backdrop"
          src={studio}
          width="1672"
          height="941"
          alt=""
          aria-hidden="true"
          fetchPriority="high"
        />
        <div className="studio-hero-inner">
          <div className="studio-hero-copy">
            <p className="eyebrow">{copy.hero.eyebrow}</p>
            <h1>
              <span>{copy.hero.headline[0]}</span>
              <em>{copy.hero.headline[1]}</em>
            </h1>
            <p className="studio-lead">{copy.hero.body}</p>
            <div className="studio-actions">
              <a href="#get-started" className="btn-cta">
                <Icon kind="download" />
                {release.available
                  ? copy.hero.primary_available
                  : copy.hero.primary_unavailable}
              </a>
              <button
                className="btn-ghost studio-demo"
                type="button"
                aria-disabled="true"
                aria-describedby="demo-note"
              >
                <Icon kind="play" />
                Watch demo
              </button>
            </div>
            <p id="demo-note" className="studio-demo-note">
              Demo video not available yet.{" "}
              <a href="#how">
                See how it works <span aria-hidden="true">↗</span>
              </a>
            </p>
            <ul className="studio-hero-points">
              <li>
                <Icon kind="bolt" />
                <div>
                  <strong>
                    Real-time,
                    <br />
                    every tweak
                  </strong>
                  <p>The full chain runs as you listen.</p>
                </div>
              </li>
              <li>
                <Icon kind="sliders" />
                <div>
                  <strong>
                    Simple by default,
                    <br />
                    deep when you want
                  </strong>
                  <p>Start with Standard. Open the full tools.</p>
                </div>
              </li>
              <li>
                <Icon kind="eye" />
                <div>
                  <strong>No black box</strong>
                  <p>See loudness, true peak and dynamics.</p>
                </div>
              </li>
            </ul>
            <p className="studio-signature">
              BETTER MIXES.
              <br />
              LOUDER STORIES.
            </p>
          </div>
          <img
            className="studio-device"
            src={standard}
            width="1448"
            height="1086"
            alt="YES Master Standard interface on a studio laptop"
            fetchPriority="high"
          />
        </div>
      </section>
      <section
        className="studio-advanced-hero"
        aria-labelledby="advanced-hero-title"
      >
        <img
          className="studio-backdrop"
          src={studio}
          width="1672"
          height="941"
          alt=""
          aria-hidden="true"
        />
        <div className="studio-hero-inner">
          <div className="studio-hero-copy">
            <p className="eyebrow">GO FURTHER</p>
            <h2 id="advanced-hero-title">
              Advanced mode.
              <br />
              Total control.
            </h2>
            <p>
              For those who want to dive deeper. Shape tone, dynamics, width and
              warmth, with real-time feedback and custom delivery options.
            </p>
            <div className="studio-actions">
              <a href="#advanced" className="btn-ghost">
                Explore Advanced <Icon kind="arrow" />
              </a>
              <a href="#export" className="studio-text-link">
                Learn more <span aria-hidden="true">→</span>
              </a>
            </div>
          </div>
          <div className="studio-front-laptop" data-studio-reveal="device">
            <img
              className="studio-laptop-chassis"
              src={advancedChassis}
              width="1536"
              height="1024"
              alt=""
              aria-hidden="true"
              loading="lazy"
            />
            <img
              className="studio-laptop-screen"
              src={advanced}
              width="2048"
              height="1129"
              alt="YES Master Advanced interface, viewed head-on on a studio laptop"
              loading="lazy"
            />
          </div>
          <p className="studio-side-note" aria-hidden="true">
            SAME SOUND. MORE POSSIBILITIES.
          </p>
        </div>
      </section>
      <div className="studio-benefits studio-shell">
        <div>
          <Icon kind="disc" />
          <p>
            <strong>Made for your music</strong>
            <span>From the first comparison to the final WAV.</span>
          </p>
        </div>
        <div>
          <Icon kind="levels" />
          <p>
            <strong>Your sound, in focus</strong>
            <span>Shape the character. Keep your signature.</span>
          </p>
        </div>
        <div>
          <Icon kind="screen" />
          <p>
            <strong>No upload. All local.</strong>
            <span>Your audio stays on your machine.</span>
          </p>
        </div>
        <div>
          <Icon kind="sliders" />
          <p>
            <strong>Room to go deeper</strong>
            <span>Track and album mastering in one place.</span>
          </p>
        </div>
      </div>
    </>
  );
}
