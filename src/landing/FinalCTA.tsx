import BetaSignup from "./BetaSignup";
import BetaDownload from "./BetaDownload";
import copy from "./page-copy.json";
import { resolveRelease, INSTALL_GUIDE_URL } from "./release-config";
import { Icon } from "./StudioElements";

export default function FinalCTA() {
  const c = copy.closing;
  const release = resolveRelease();
  return (
    <section id="get-started" className="studio-closing">
      <div className="studio-shell">
        <p className="eyebrow">{c.eyebrow}</p>
        <h2>
          {c.headline[0]}
          <em>{c.headline[1]}</em>
        </h2>
        <p className="studio-lead">{c.body}</p>
        <div className="studio-actions">
          <a href="#beta-availability" className="btn-cta">
            {release.available ? c.primary_available : c.primary_unavailable}
            <Icon kind="arrow" />
          </a>
          <a href="#how" className="btn-ghost">
            {c.secondary}
          </a>
        </div>
        <p className="studio-footnote">{c.micro}</p>
        <div id="beta-availability" className="studio-availability">
          <BetaDownload />
          <div className="studio-signup">
            <p className="studio-label">OPTIONAL EMAIL UPDATES</p>
            <BetaSignup />
          </div>
        </div>
        <footer className="studio-footer">
          <a href="#top" className="studio-wordmark">
            YES Master
          </a>
          <div>
            <a href="#how">How it works</a>
            <a href="#advanced">Advanced</a>
            <a href="#album">Album</a>
            <a href="#beta">Beta details</a>
            <a href={INSTALL_GUIDE_URL} target="_blank" rel="noreferrer">
              Install help
            </a>
          </div>
          <p>Music sounds brighter here.</p>
        </footer>
      </div>
    </section>
  );
}
