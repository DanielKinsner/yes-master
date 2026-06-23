import brandIcon from "../assets/landing/yes-master-icon.png";

const links = [
  { label: "Standard", href: "#standard" },
  { label: "Advanced", href: "#advanced" },
  { label: "Mobile", href: "#mobile" },
];

export default function Nav() {
  return (
    <nav className="fixed inset-x-0 top-0 z-50 flex items-center justify-between gap-3 border-b border-white/10 bg-night/70 px-5 py-3 backdrop-blur-md sm:px-8">
      <a
        href="#top"
        className="flex items-center gap-2 font-display text-lg font-extrabold"
      >
        <img src={brandIcon} alt="" className="h-7 w-7" />
        <span>Y.E.S. Master</span>
      </a>

      <div className="flex items-center gap-4 text-sm text-muted sm:gap-7">
        {/* Section links hide on phones to keep the bar clean. */}
        {links.map((link) => (
          <a
            key={link.href}
            href={link.href}
            className="hidden transition-colors hover:text-ink sm:inline"
          >
            {link.label}
          </a>
        ))}
        <a
          href="#get-started"
          className="rounded-lg bg-gradient-to-b from-cta-light to-cta-deep px-4 py-2 font-extrabold text-[#1c0d00]"
        >
          Join desktop beta
        </a>
      </div>
    </nav>
  );
}
