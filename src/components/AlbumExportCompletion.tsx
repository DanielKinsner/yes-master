import { useEffect, useRef, useState } from "react";
import { createPortal } from "react-dom";
import type { AlbumRenderReport } from "../lib/api";
import { AlbumExportReceipt } from "./AlbumExportReceipt";
import "./AlbumExportCompletion.css";

/** One receipt, beside the export workflow, automatically visible on success.
 * Nonmodal: completion never interrupts editing or takes keyboard focus. */
export function AlbumExportCompletion({ report }: { report: AlbumRenderReport }) {
  const success = report.status.status === "done";
  const [open, setOpen] = useState(success);
  const panel = useRef<HTMLElement>(null);
  const trigger = useRef<HTMLButtonElement>(null);
  useEffect(() => setOpen(success), [report.job_id, success]);
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
      <AlbumExportReceipt key={report.job_id} report={report} expanded />
    </section>, document.body)}
  </>;
}
