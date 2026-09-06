import exportReceipt from "../assets/landing/studio/export-receipt.png";
import copy from "./page-copy.json";
import { Capture } from "./StudioElements";

export default function ProofDeck() {
  const e = copy.export;
  return (
    <section id="export" className="studio-section studio-export">
      <div className="studio-shell studio-export-grid">
        <div className="studio-export-copy">
          <p className="eyebrow">{e.eyebrow}</p>
          <h2>
            {e.headline[0]}
            <em>{e.headline[1]}</em>
          </h2>
          <p className="studio-lead">{e.body}</p>
          <p>{e.source_note}</p>
          <dl className="studio-receipt-modes">
            <div>
              <dt>Standard</dt>
              <dd>{e.standard}</dd>
            </div>
            <div>
              <dt>Advanced</dt>
              <dd>{e.advanced}</dd>
            </div>
          </dl>
        </div>
        <Capture
          src={exportReceipt}
          alt="YES Master export receipt for Lay the Money on the Desk, showing delivered measurements, export checks and WAV format"
          width={1351}
          height={1164}
          caption="An actual export receipt. Your final file, measured."
        />
      </div>
    </section>
  );
}
