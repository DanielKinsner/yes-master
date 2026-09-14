import Nav from "./landing/Nav";
import Hero from "./landing/Hero";
import Workflow from "./landing/Workflow";
import ProofDeck from "./landing/ProofDeck";
import SoundCharacter from "./landing/SoundCharacter";
import AlbumProof from "./landing/AlbumProof";
import BetaTerms from "./landing/BetaTerms";
import FinalCTA from "./landing/FinalCTA";
import useStudioMotion from "./landing/useStudioMotion";
import { lazy, Suspense, useCallback, useState } from "react";

// "Try it on your mix": the Standard chain compiled to wasm, in a modal over
// the page. Lazy so the landing bundle doesn't carry the engine until asked;
// hovering or focusing the hero button starts the download early, and the
// fallback below is visible so a slow connection never reads as a dead click.
const loadTryIt = () => import("./tryit/TryItModal");
const TryItModal = lazy(loadTryIt);
const warmTryIt = () => { void loadTryIt().catch(() => {}); };

function TryItLoading() {
  return (
    <div role="status" aria-live="polite" style={{ position: "fixed", inset: 0, zIndex: 100, display: "grid", placeItems: "center", background: "rgba(4, 6, 10, 0.62)" }}>
      <p style={{ margin: 0, padding: "12px 18px", borderRadius: 999, border: "1px solid rgba(158, 176, 214, 0.2)", background: "#0d1017", color: "#eef1f8", font: "600 14px/1.4 Inter, system-ui, sans-serif" }}>
        Loading the demo…
      </p>
    </div>
  );
}

// Marketing only. Studio styles are scoped to this shell; native UI and its
// stylesheet, feature flags, release configuration and signup stay independent.
export default function LandingPage() {
  const root = useStudioMotion();
  const [tryItOpen, setTryItOpen] = useState(false);
  const openTryIt = useCallback(() => setTryItOpen(true), []);
  const closeTryIt = useCallback(() => setTryItOpen(false), []);
  return (
    <>
    {tryItOpen && (
      <Suspense fallback={<TryItLoading />}>
        <TryItModal onClose={closeTryIt} />
      </Suspense>
    )}
    <div ref={root} className={"studio-site min-h-svh bg-night text-ink" + (tryItOpen ? " is-tryit-open" : "")} aria-hidden={tryItOpen || undefined}>
      <a
        href="#main"
        className="sr-only focus:not-sr-only focus:fixed focus:left-4 focus:top-4 focus:z-[100] focus:rounded-lg focus:bg-brand focus:px-4 focus:py-2 focus:font-extrabold focus:text-night"
      >
        Skip to main content
      </a>
      <Nav />
      <main id="main">
        <Hero onTryIt={openTryIt} onTryItIntent={warmTryIt} />
        <Workflow />
        <ProofDeck />
        <SoundCharacter />
        <AlbumProof />
        <BetaTerms />
        <FinalCTA />
      </main>
    </div>
    </>
  );
}
