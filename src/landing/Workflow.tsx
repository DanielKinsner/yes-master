import standard from "../assets/landing/studio/standard-ui.png";
import copy from "./page-copy.json";
import { Capture, DetailList, Icon, SectionHeading } from "./StudioElements";
import SoundCharacter from "./SoundCharacter";

export default function Workflow() {
  const c = copy.workflow;
  return (
    <section id="how" className="studio-section studio-workflow">
      <div className="studio-shell">
        <SectionHeading {...c} />
        <div id="standard" className="studio-workflow-grid">
          <SoundCharacter />
          <div>
            <DetailList items={c.steps} />
            <aside className="studio-restraint">
              <h3>{copy.sound.adaptive_heading}</h3>
              <p>{copy.sound.adaptive_body}</p>
            </aside>
          </div>
        </div>
        <details className="studio-standard-capture studio-album-capture">
          <summary className="studio-view-button">
            <Icon kind="screen" />
            <span className="studio-disclosure-show">
              View Standard screenshot
            </span>
            <span className="studio-disclosure-hide">
              Hide Standard screenshot
            </span>
            <span className="studio-disclosure-chevron" aria-hidden="true">
              <Icon kind="chevron" />
            </span>
          </summary>
          <p>
            {copy.sound.body} Standard exports a 44.1 kHz / 24-bit WAV, with the
            limiter ceiling set to −1 dBTP. Custom formats live in Advanced.
          </p>
          <Capture
            src={standard}
            alt="YES Master Standard view"
            width={2048}
            height={1153}
            caption="Style, Intensity and loudness. All in one view."
            expandLabel="Expand screenshot"
          />
        </details>
      </div>
    </section>
  );
}
