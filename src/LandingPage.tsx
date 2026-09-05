import Nav from "./landing/Nav";
import Hero from "./landing/Hero";
import Workflow from "./landing/Workflow";
import ProofDeck from "./landing/ProofDeck";
import SignOff from "./landing/SignOff";
import SoundCharacter from "./landing/SoundCharacter";
import AlbumProof from "./landing/AlbumProof";
import BetaTerms from "./landing/BetaTerms";
import FinalCTA from "./landing/FinalCTA";

// Marketing only. Studio styles are scoped to this shell; native UI and its
// stylesheet, feature flags, release configuration and signup stay independent.
export default function LandingPage() {
  return (
    <div className="studio-site min-h-svh bg-night text-ink">
      <a
        href="#main"
        className="sr-only focus:not-sr-only focus:fixed focus:left-4 focus:top-4 focus:z-[100] focus:rounded-lg focus:bg-brand focus:px-4 focus:py-2 focus:font-extrabold focus:text-night"
      >
        Skip to main content
      </a>
      <Nav />
      <main id="main">
        <Hero />
        <Workflow />
        <ProofDeck />
        <SignOff />
        <SoundCharacter />
        <AlbumProof />
        <BetaTerms />
        <FinalCTA />
      </main>
    </div>
  );
}
