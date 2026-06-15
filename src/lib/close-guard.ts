// S6.8: predicate for the window close-guard. Quitting mid-render/-export would
// silently discard in-flight work, so the close handler confirms first — but
// only when something is actually running. Kept pure so it is unit-testable
// without a Tauri window.
export interface InFlightWork {
  isExporting: boolean;
  isRendering: boolean;
  albumRendering: boolean;
}

export function shouldConfirmClose(work: InFlightWork): boolean {
  return work.isExporting || work.isRendering || work.albumRendering;
}
