import { useRef, type CSSProperties } from "react";

export function Icon({ kind }: { kind: string }) {
  const paths: Record<string, string> = {
    download: "M12 3v12m-4-4 4 4 4-4M5 17v4h14v-4",
    play: "m8 4 12 8-12 8Z",
    bolt: "m14 2-9 12h6l-1 8 9-12h-6Z",
    sliders: "M3 6h18M3 12h18M3 18h18M8 3v6m8 0v6m-6 0v6",
    eye: "M2 12s4-7 10-7 10 7 10 7-4 7-10 7S2 12 2 12Z M15 12a3 3 0 1 1-6 0 3 3 0 0 1 6 0",
    arrow: "M4 12h15m-6-6 6 6-6 6",
    screen: "M3 4h18v14H3ZM8 22h8m-4-4v4",
    levels: "M4 16v5m5-12v12m6-17v17m6-21v21",
    disc: "M21 6c0 2-4 4-9 4S3 8 3 6s4-4 9-4 9 2 9 4ZM3 6v12c0 2 4 4 9 4s9-2 9-4V6M3 12c0 2 4 4 9 4s9-2 9-4",
    folder: "M3 7h7l2-3h9v17H3Z",
  };
  return (
    <svg
      aria-hidden="true"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="1.5"
      strokeLinecap="round"
      strokeLinejoin="round"
    >
      <path d={paths[kind] ?? paths.arrow} />
    </svg>
  );
}

export function SectionHeading({
  eyebrow,
  headline,
  body,
}: {
  eyebrow: string;
  headline: string[];
  body?: string;
}) {
  return (
    <header className="studio-section-heading">
      <div>
        <p className="eyebrow">{eyebrow}</p>
        <h2>
          {headline[0]}
          <em>{headline[1]}</em>
        </h2>
      </div>
      {body && <p className="studio-lead">{body}</p>}
    </header>
  );
}

export function DetailList({
  items,
}: {
  items: { title: string; body: string }[];
}) {
  return (
    <ol className="studio-detail-list">
      {items.map((item, i) => (
        <li key={item.title}>
          <span>{String(i + 1).padStart(2, "0")}</span>
          <div>
            <h3>{item.title}</h3>
            <p>{item.body}</p>
          </div>
        </li>
      ))}
    </ol>
  );
}

// Crop viewports preserve the supplied image bytes. Full screenshots can be
// inspected with a keyboard or touch; no UI is redrawn or simulated.
export function Capture({
  src,
  alt,
  width,
  height,
  crop,
  sourceSize = [2048, 1129],
  fullSrc,
  caption = "Explore the interface",
}: {
  src: string;
  alt: string;
  width: number;
  height: number;
  crop?: [number, number, number];
  sourceSize?: [number, number];
  fullSrc?: string;
  caption?: string;
}) {
  const dialog = useRef<HTMLDialogElement>(null);
  const cropStyle = crop
    ? {
        width: `${(crop[0] / width) * 100}%`,
        left: `${(-crop[1] / width) * 100}%`,
        top: `${(-crop[2] / height) * 100}%`,
      }
    : undefined;
  return (
    <figure className="studio-capture">
      <button
        type="button"
        className="studio-capture-button"
        aria-label={`Enlarge ${alt}`}
        onClick={() => dialog.current?.showModal()}
      >
        <span
          className={crop ? "studio-crop" : "studio-flat"}
          style={{ aspectRatio: `${width} / ${height}` } as CSSProperties}
        >
          <img
            src={src}
            alt={alt}
            width={crop ? sourceSize[0] : width}
            height={crop ? sourceSize[1] : height}
            style={cropStyle}
            loading="lazy"
          />
        </span>
        <span className="studio-enlarge" aria-hidden="true">
          ↗
        </span>
      </button>
      <figcaption>
        {caption} <span aria-hidden="true">↗</span>
      </figcaption>
      <dialog
        ref={dialog}
        className="studio-lightbox"
        aria-label={alt}
        onClick={(e) => {
          if (e.target === e.currentTarget) dialog.current?.close();
        }}
      >
        <form method="dialog">
          <button className="btn-ghost" autoFocus>
            Close image <span aria-hidden="true">×</span>
          </button>
        </form>
        <div className="studio-lightbox-scroll">
          <img src={fullSrc ?? src} alt={alt} loading="lazy" />
        </div>
        <p>Product screenshot. Scroll to inspect details.</p>
      </dialog>
    </figure>
  );
}
