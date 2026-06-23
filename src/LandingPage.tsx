import Nav from "./landing/Nav";
import Hero from "./landing/Hero";
import CrossPlatform from "./landing/CrossPlatform";
import ProofDeck from "./landing/ProofDeck";
import FinalCTA from "./landing/FinalCTA";

/*
 * The YES Master landing page, top to bottom.
 *
 * Each line below is a section of the page. Open any one in src/landing/
 * to see (and edit) exactly what it shows. There is no hidden styling —
 * the look lives as Tailwind classes right inside each component, and the
 * brand colors/fonts are defined once in LandingPage.css.
 */
export default function LandingPage() {
  return (
    <div className="min-h-svh bg-night text-ink">
      <Nav />
      <Hero />
      <CrossPlatform />
      <ProofDeck />
      <FinalCTA />
    </div>
  );
}
