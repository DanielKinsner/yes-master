import { useEffect, useRef, useState } from "react";
import { createPortal } from "react-dom";
import type { AlbumRenderReport } from "../lib/api";
import { api } from "../lib/api";
import { AlbumExportReceipt } from "./AlbumExportReceipt";
import "./AlbumExportCompletion.css";

/** One receipt, beside the export workflow, automatically visible on success.
 * Nonmodal: completion never interrupts editing or takes keyboard focus. */
export function AlbumExportCompletion({ report }: { report: AlbumRenderReport }) {
  const success = report.status.status === "done";
  const [open, setOpen] = useState(success);
  const [revealError, setRevealError] = useState<string | null>(null);
  const panel = useRef<HTMLElement>(null);
  const trigger = useRef<HTMLButtonElement>(null);
  useEffect(() => { setOpen(success); setRevealError(null); }, [report.job_id, success]);
  const showFiles = async () => {
    setRevealError(null);
    try {
      await api.openOutput(report.album_wav_path);
    } catch {
      setRevealError("Couldn't open the export folder. Your saved file path is shown below.");
    }
  };
  const close = () => {
    if (panel.current?.contains(document.activeElement)) trigger.current?.focus();
    setOpen(false);
  };
  if (!success) return report.status.status === "cancelled" ? <AlbumExportReceipt report={report} /> : null;
  return <>
    <div className="album-completion-status" role="status">Album exported</div>
    <button ref={trigger} type="button" className="ghost-btn" aria-expanded={open}
      aria-controls="album-completion" onClick={() => setOpen(!open)}>View Album receipt</button>
    {open && createPortal(<section ref={panel} id="album-completion" className="album-completion"
      aria-label="Album export receipt" onKeyDown={e => { if (e.key === "Escape") { e.stopPropagation(); close(); } }}>
      <header><h2>Album export complete</h2><button type="button" className="ghost-btn"
        onClick={close} aria-label="Close Album receipt">Close</button></header>
      <div className="album-completion-actions">
        <button type="button" className="ghost-btn" onClick={() => void showFiles()}>Show files</button>
      </div>
      {revealError && <p role="alert">{revealError}</p>}
      <AlbumExportReceipt key={report.job_id} report={report} expanded />
    </section>, document.body)}
  </>;
}
