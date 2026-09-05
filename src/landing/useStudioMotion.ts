import { useEffect, useRef } from "react";

// Progressive enhancement: artwork is visible before JS, while loading, and
// when motion is disabled. Each selected image settles only once per visit.
export default function useStudioMotion() {
  const root = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (!root.current || typeof IntersectionObserver === "undefined") return;

    const preference = window.matchMedia("(prefers-reduced-motion: reduce)");
    const targets = Array.from(
      root.current.querySelectorAll<HTMLElement>("[data-studio-reveal]"),
    );
    const seen = new Set<Element>();
    const observer = new IntersectionObserver(
      (entries) => {
        if (preference.matches) return;
        for (const entry of entries) {
          if (!entry.isIntersecting) continue;
          entry.target.classList.add("studio-reveal-enter");
          seen.add(entry.target);
          observer.unobserve(entry.target);
        }
      },
      { threshold: 0.12 },
    );

    const syncPreference = () => {
      observer.disconnect();
      for (const target of targets) {
        if (preference.matches) target.classList.remove("studio-reveal-enter");
        else if (!seen.has(target)) observer.observe(target);
      }
    };
    syncPreference();
    preference.addEventListener("change", syncPreference);
    return () => {
      observer.disconnect();
      preference.removeEventListener("change", syncPreference);
      targets.forEach((target) =>
        target.classList.remove("studio-reveal-enter"),
      );
    };
  }, []);

  return root;
}
