// src/lib/motion.ts
//
// Decorative motion is opt-out at the OS level; every animated surface
// (analysis orb, morph, hero entrance) checks here. jsdom has no
// matchMedia — its absence means "motion allowed".

export function prefersReducedMotion(): boolean {
  return (
    globalThis.matchMedia?.("(prefers-reduced-motion: reduce)")?.matches ?? false
  );
}
