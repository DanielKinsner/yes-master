import standard from "../assets/landing/studio/standard-ui.png";
import copy from "./page-copy.json";
import { Capture, DetailList, SectionHeading } from "./StudioElements";

export default function Workflow() {
  const c = copy.workflow;
  return (
    <section id="how" className="studio-section studio-workflow">
      <div className="studio-shell">
        <SectionHeading {...c} />
        <div id="standard" className="studio-workflow-grid">
          <Capture
            src={standard}
            alt="YES Master Standard view"
            width={2048}
            height={1153}
            caption="Standard. Three decisions, all in one view."
          />
          <DetailList items={c.steps} />
        </div>
        <div className="studio-shelf studio-specs">
          <div>
            <h3>{c.support_title}</h3>
            <p>{c.support_body}</p>
          </div>
          <dl>
            <div>
              <dt>Sample rate</dt>
              <dd>
                44.1 <small>kHz</small>
              </dd>
            </div>
            <div>
              <dt>WAV output</dt>
              <dd>24-bit</dd>
            </div>
            <div>
              <dt>Limiter ceiling setting</dt>
              <dd>
                −1 <small>dBTP</small>
              </dd>
            </div>
          </dl>
        </div>
      </div>
    </section>
  );
}
