import { useEffect, useRef, type ReactNode } from "react";

export function ChromeDialog({
  title,
  eyebrow,
  onClose,
  children,
}: {
  title: string;
  eyebrow: string;
  onClose: () => void;
  children: ReactNode;
}) {
  const titleId = `chrome-dialog-${title.toLowerCase()}`;
  const dialogRef = useRef<HTMLElement | null>(null);
  const openerRef = useRef<HTMLElement | null>(null);
  const onCloseRef = useRef(onClose);
  useEffect(() => {
    onCloseRef.current = onClose;
  });

  const closeAndRestore = () => {
    onCloseRef.current();
    // Restore only after an actual close, never during StrictMode cleanup or
    // a parent rerender. All three close routes share this behavior.
    window.setTimeout(() => {
      if (!dialogRef.current?.isConnected && openerRef.current?.isConnected) {
        openerRef.current.focus();
      }
    }, 0);
  };

  useEffect(() => {
    const active = document.activeElement;
    if (active instanceof HTMLElement && !dialogRef.current?.contains(active)) {
      openerRef.current = active;
    }
    dialogRef.current?.focus();
    const onKeyDown = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        closeAndRestore();
        return;
      }
      if (e.key !== "Tab" || !dialogRef.current) return;
      const dialog = dialogRef.current;
      const controls = Array.from(dialog.querySelectorAll<HTMLElement>(
        'button, a[href], input, select, textarea, summary, [tabindex]',
      )).filter((element) => element.tabIndex >= 0
        && !element.matches(":disabled")
        && !element.closest('[hidden], [inert], [aria-hidden="true"]')
        && getComputedStyle(element).display !== "none"
        && getComputedStyle(element).visibility !== "hidden");
      e.preventDefault();
      const index = controls.indexOf(document.activeElement as HTMLElement);
      const next = e.shiftKey
        ? index <= 0 ? controls.length - 1 : index - 1
        : index < 0 || index === controls.length - 1 ? 0 : index + 1;
      (controls[next] ?? dialog).focus();
    };
    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, []);
  return (
    <div
      className="chrome-dialog-backdrop"
      role="presentation"
      onClick={closeAndRestore}
    >
      <section
        ref={dialogRef}
        tabIndex={-1}
        className="chrome-dialog"
        role="dialog"
        aria-modal="true"
        aria-labelledby={titleId}
        onClick={(e) => e.stopPropagation()}
      >
        <header className="chrome-dialog-head">
          <div>
            <div className="chrome-dialog-eyebrow">{eyebrow}</div>
            <h2 id={titleId}>{title}</h2>
          </div>
          <button
            type="button"
            className="icon-tile"
            onClick={closeAndRestore}
            aria-label={`Close ${title}`}
            title={`Close ${title}`}
          >
            <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={2} strokeLinecap="round" strokeLinejoin="round">
              <path d="M18 6 6 18" />
              <path d="m6 6 12 12" />
            </svg>
          </button>
        </header>
        {children}
      </section>
    </div>
  );
}
