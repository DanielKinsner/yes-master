import Nav from "./landing/Nav";
import Hero from "./landing/Hero";
import Workflow from "./landing/Workflow";
import ProofDeck from "./landing/ProofDeck";
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
      <Nav />
      <Hero />
      <Workflow />
      <ProofDeck />
      <SoundCharacter />
      <AlbumProof />
      <BetaTerms />
      <CrossPlatform />
      <FinalCTA />
    </div>
  );
}
