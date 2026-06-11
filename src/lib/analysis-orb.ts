// src/lib/analysis-orb.ts
//
// Pure geometry/easing for the analysis orb: a point-cloud sphere that
// breathes while analysis runs, then dissolves into the track's real
// waveform. Everything here is deterministic and unit-tested; the canvas
// component only iterates and draws.

export type OrbParticle = {
  /// Unit-sphere position.
  x: number;
  y: number;
  z: number;
  /// Per-particle phase so the noise field doesn't move in lockstep.
  jitter: number;
  /// Waveform slot: horizontal position 0..1 …
  u: number;
  /// … and vertical position inside the bar, -1..1 (scaled by peak height).
  tyUnit: number;
};

/// Small deterministic PRNG so the loading orb and the morph overlay can
/// regenerate the SAME cloud from a fixed seed (visual continuity across
/// the WaveformLoading → WaveformView remount).
export function mulberry32(seed: number): () => number {
  let a = seed >>> 0;
  return () => {
    a = (a + 0x6d2b79f5) | 0;
    let t = Math.imul(a ^ (a >>> 15), 1 | a);
    t = (t + Math.imul(t ^ (t >>> 7), 61 | t)) ^ t;
    return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
  };
}

function gauss(random: () => number): number {
  let u = 0,
    v = 0;
  while (!u) u = random();
  while (!v) v = random();
  return Math.sqrt(-2 * Math.log(u)) * Math.cos(6.283185307179586 * v);
}

export function createOrbParticles(
  count: number,
  random: () => number,
): OrbParticle[] {
  const out: OrbParticle[] = [];
  for (let i = 0; i < count; i++) {
    let x = gauss(random),
      y = gauss(random),
      z = gauss(random);
    const m = Math.hypot(x, y, z) || 1;
    x /= m;
    y /= m;
    z /= m;
    out.push({
      x,
      y,
      z,
      jitter: random() * 6.283,
      u: random() * 0.999999,
      tyUnit: random() * 2 - 1,
    });
  }
  return out;
}

/// Wispy radial displacement (the "ethereal" look): a sum of slow sines in
/// particle-space and time. Bounded well inside (0.7, 1.3) so the cloud
/// never collapses or explodes.
export function orbDisplacement(p: OrbParticle, t: number): number {
  return (
    1 +
    0.17 * Math.sin(3 * p.x + t * 1.1 + p.jitter) * Math.sin(2.3 * p.y - t * 0.8) +
    0.06 * Math.sin(5 * p.z + t * 1.7)
  );
}

/// Vertical morph target in -1..1, scaled by the peak amplitude at the
/// particle's horizontal slot. Mirrors the SVG's centered-bar geometry.
export function waveformTargetY(
  tyUnit: number,
  u: number,
  peaks: readonly number[],
): number {
  if (peaks.length === 0) return 0;
  const idx = Math.min(peaks.length - 1, Math.floor(u * peaks.length));
  return tyUnit * peaks[idx];
}

export function easeInOutQuad(t: number): number {
  const c = Math.max(0, Math.min(1, t));
  return c < 0.5 ? 2 * c * c : 1 - Math.pow(-2 * c + 2, 2) / 2;
}

export const ORB_PARTICLE_COUNT = 1800;
export const ORB_SEED = 92;
export const MORPH_MS = 1400;
