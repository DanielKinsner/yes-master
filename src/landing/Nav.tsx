import brandIcon from "../assets/landing/yes-master-icon.png";

import { resolveRelease, type ResolvedRelease } from "./release-config";

// U6. "Mobile" is deliberately NOT a nav item any more: a parked, unobtainable
// surface does not get top-level navigation. It keeps its section and its one
// date-free sentence further down the page.
const links = [
  { label: "How it works", href: "#how" },
  { label: "Advanced", href: "#advanced" },
  { label: "Album", href: "#album" },
  { label: "Beta", href: "#beta" },
];

export default function Nav({
  release = resolveRelease(),
}: { release?: ResolvedRelease } = {}) {
  return (
    <nav className="fixed inset-x-0 top-0 z-50 flex items-center justify-between gap-3 border-b border-white/[0.07] bg-night/60 px-5 py-3 shadow-[0_1px_0_rgba(0,0,0,0.5)] backdrop-blur-xl sm:px-8">
      <a
        href="#top"
        className="flex items-center gap-2.5 font-display text-lg font-extrabold tracking-[-0.01em]"
      >
        <img src={brandIcon} alt="" className="h-7 w-7 drop-shadow-[0_0_8px_rgba(122,166,255,0.45)]" />
        <span>YES Master</span>
      </a>

      <div className="flex items-center gap-4 text-sm text-muted sm:gap-7">
        {/* Section links hide on phones to keep the bar clean. */}
        {links.map((link) => (
          <a
            key={link.href}
            href={link.href}
            className="hidden font-semibold transition-colors hover:text-ink sm:inline"
          >
            {link.label}
          </a>
        ))}
        <a
          href="#get-started"
          className="btn-cta !px-4 !py-2 text-sm hover:btn-cta-hover"
        >
          {/* Anchors to the section that tells the truth about availability,
              so this is never a dead download — but it must not promise one
              either while the release is closed (U5). */}
          {release.available ? "Download free beta" : "About the beta"}
        </a>
      </div>
    </nav>
  );
}
