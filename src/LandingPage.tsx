import Nav from "./landing/Nav";
import Hero from "./landing/Hero";
import Workflow from "./landing/Workflow";
import ProofDeck from "./landing/ProofDeck";
import SignOff from "./landing/SignOff";
import SoundCharacter from "./landing/SoundCharacter";
import AlbumProof from "./landing/AlbumProof";
import BetaTerms from "./landing/BetaTerms";
import CrossPlatform from "./landing/CrossPlatform";
import FinalCTA from "./landing/FinalCTA";

/*
 * The YES Master landing page, top to bottom.
 *
 * Each line below is a section of the page. Open any one in src/landing/
 * to see (and edit) exactly what it shows. There is no hidden styling —
 * the look lives as Tailwind classes right inside each component, and the
 * brand colors/fonts are defined once in LandingPage.css.
 *
 * ORDER IS THE ARGUMENT (U6). One visitor hierarchy, in this sequence:
 *
 *   Hero            problem and outcome
 *   Workflow        what you actually do — three decisions
 *   ProofDeck       credible product proof, Standard as hero / Advanced as depth
 *   SignOff         the one photograph, full-bleed
 *   SoundCharacter  character controls and adaptive restraint
 *   AlbumProof      depth proof, deliberately LOW — not a second audience
 *   BetaTerms       what the arrangement is, before anyone downloads
 *   CrossPlatform   one date-free line that mobile is not available
 *   FinalCTA        the action
 *
 * CrossPlatform used to sit SECOND, immediately under the hero — a parked,
 * unobtainable product in the page's most valuable position, with six
 * present-tense feature cards. Do not move it back up.
 */
export default function LandingPage() {
  return (
    <div className="min-h-svh bg-night text-ink">
      {/* U8 — skip link. The nav is fixed and comes before everything, so a
          keyboard or screen-reader visitor otherwise tabs the whole bar on
          every page load. Visually hidden until focused, then it is the first
          thing on screen rather than a phantom focus ring. */}
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
      <CrossPlatform />
      <FinalCTA />
      </main>
    </div>
  );
}
