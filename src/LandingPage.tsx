import Nav from "./sheet/Nav";
import Hero from "./sheet/Hero";
import Chain from "./sheet/Chain";
import Workflow from "./sheet/Workflow";
import Proof from "./sheet/Proof";
import Sound from "./sheet/Sound";
import Album from "./sheet/Album";
import Terms from "./sheet/Terms";
import Platform from "./sheet/Platform";
import GetStarted from "./sheet/GetStarted";
import Footer from "./sheet/Footer";

/*
 * The YES Master landing page — "the delivery sheet" — top to bottom.
 *
 * Each line is a section; open it in src/sheet/ to see and edit exactly what
 * it shows. The look lives as Tailwind classes inside each section, and the
 * paper/ink palette, type, and shared utilities are defined once in
 * LandingPage.css.
 *
 * ORDER IS THE ARGUMENT (U6). One visitor hierarchy, unchanged in sequence:
 *
 *   Hero        the promise, with the receipt as the object
 *   Chain       the eight stages, in the app's order
 *   Workflow    what you actually do — three decisions
 *   Proof       Standard as the room, Advanced as the console
 *   Sound       character controls and adaptive restraint
 *   Album       depth proof, deliberately low
 *   Terms       the arrangement, before anyone downloads
 *   Platform    one date-free line that mobile is not available
 *   GetStarted  the action
 */
export default function LandingPage() {
  return (
    <div className="min-h-svh bg-paper text-ink">
      <a
        href="#main"
        className="sr-only focus:not-sr-only focus:fixed focus:left-4 focus:top-4 focus:z-[100] focus:rounded-full focus:bg-ink focus:px-4 focus:py-2 focus:font-semibold focus:text-paper-white"
      >
        Skip to main content
      </a>
      <Nav />
      <main id="main">
        <Hero />
        <Chain />
        <Workflow />
        <Proof />
        <Sound />
        <Album />
        <Terms />
        <Platform />
        <GetStarted />
      </main>
      <Footer />
    </div>
  );
}
