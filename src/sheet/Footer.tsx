import { REPO_URL } from "../landing/release-config";

export default function Footer() {
  return (
    <footer className="border-t border-rule px-5 py-8 sm:px-8">
      <div className="mx-auto flex max-w-6xl flex-col gap-3 text-[0.85rem] text-ink-3 sm:flex-row sm:items-baseline sm:justify-between">
        <span className="font-display text-[1.2rem] text-ink">YES Master</span>
        <span>Your Endgame Sound · Windows and macOS</span>
        <a
          href={REPO_URL}
          target="_blank"
          rel="noreferrer"
          className="inline-flex min-h-11 items-center font-semibold text-ink-2 underline decoration-rule-strong underline-offset-4 hover:text-ink"
        >
          GitHub
        </a>
      </div>
    </footer>
  );
}
