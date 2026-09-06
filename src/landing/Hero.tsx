import { useRef } from "react";
import studio from "../assets/landing/studio/hero-bg-studio.webp";
import standard from "../assets/landing/studio/hero-device-standard.webp";
import advancedChassis from "../assets/landing/studio/advanced-laptop-front.webp";
import advanced from "../assets/landing/studio/advanced-doors-open.png";
import { resolveRelease, type ResolvedRelease } from "./release-config";
import copy from "./page-copy.json";
import { Capture, Icon } from "./StudioElements";

export default function Hero({
  release = resolveRelease(),
}: { release?: ResolvedRelease } = {}) {
  const advancedDialog = useRef<HTMLDialogElement>(null);
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
            <p className="studio-ownership">{copy.hero.micro}</p>
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
        id="advanced"
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
            <p>{copy.advanced.body}</p>
            <p className="studio-advanced-note">{copy.advanced.ab_note}</p>
            <div className="studio-actions">
              <button
                className="studio-view-button"
                type="button"
                aria-haspopup="dialog"
                aria-controls="advanced-screenshot-dialog"
                onClick={() => advancedDialog.current?.showModal()}
              >
                <Icon kind="expand" />
                View Advanced screenshot
              </button>
              <a href="#export" className="studio-text-link">
                Explore the export <span aria-hidden="true">→</span>
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
            <div className="studio-laptop-screen">
              <Capture
                src={advanced}
                width={1663}
                height={946}
                alt="YES Master Advanced WAV session, viewed head-on on a studio laptop"
                caption="Advanced. Take a closer look."
                dialogRef={advancedDialog}
                dialogId="advanced-screenshot-dialog"
                expandLabel="Expand"
              />
            </div>
          </div>
        </div>
      </section>
    </>
  );
}
