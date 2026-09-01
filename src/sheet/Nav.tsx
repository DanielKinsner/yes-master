import { resolveRelease, type ResolvedRelease } from "../landing/release-config";

// The page's running head. A parked surface does not get a slot here, so
// "Mobile" stays out (U6); its one sentence lives low on the page.
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
    <nav className="sticky top-0 z-50 border-b border-rule bg-paper/95 px-5 backdrop-blur-sm sm:px-8">
      <div className="mx-auto flex max-w-6xl items-center justify-between gap-4 py-3">
        <a
          href="#top"
          className="font-display text-[1.45rem] leading-none tracking-[-0.01em] text-ink"
        >
          YES Master
        </a>

        <div className="flex items-center gap-5 sm:gap-8">
          {links.map((link) => (
            <a
              key={link.href}
              href={link.href}
              className="hidden text-[0.9rem] font-medium text-ink-2 transition-colors hover:text-ink md:inline"
            >
              {link.label}
            </a>
          ))}
          <a
            href="#get-started"
            className="btn-ink !min-h-10 !px-4 !py-2 !text-[0.85rem] hover:btn-ink-hover"
          >
            {release.available ? "Download free beta" : "About the beta"}
          </a>
        </div>
      </div>
    </nav>
  );
}
