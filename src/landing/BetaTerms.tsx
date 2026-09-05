import copy from "./page-copy.json";
import { SectionHeading } from "./StudioElements";

export default function BetaTerms() {
  const c = copy.beta;
  return (
    <section id="beta" className="studio-section studio-beta">
      <div className="studio-shell studio-beta-grid">
        <div>
          <SectionHeading eyebrow={c.eyebrow} headline={c.headline} />
          <p>{c.body}</p>
          <p className="studio-pricing">{c.pricing}</p>
          <span className="studio-platform">Windows + macOS desktop</span>
        </div>
        <div className="studio-faq-grid">
          {c.faq.map((f) => (
            <details key={f.q} open>
              <summary>
                {f.q}
                <span aria-hidden="true">+</span>
              </summary>
              <p>{f.a}</p>
            </details>
          ))}
        </div>
      </div>
    </section>
  );
}
