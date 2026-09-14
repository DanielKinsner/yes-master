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
// the page. Lazy so the landing bundle doesn't carry the engine until asked.
const TryItModal = lazy(() => import("./tryit/TryItModal"));

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
      <Suspense fallback={null}>
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
        <Hero onTryIt={openTryIt} />
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
