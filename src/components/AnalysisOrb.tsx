// src/components/AnalysisOrb.tsx
//
// The analysis orb: ~2k particles on a breathing point-cloud sphere while
// analysis runs ("orb"), or flying into the track's waveform shape
// ("morph"). Draw-only: the PARENT owns when this mounts/unmounts (incl.
// the morph window timer and interaction cuts). jsdom returns a null 2d
// context — in that case the component renders the canvas and does nothing.

import { useEffect, useRef } from "react";
import {
  MORPH_MS,
  ORB_PARTICLE_COUNT,
  ORB_SEED,
  createOrbParticles,
  easeInOutQuad,
  mulberry32,
  orbDisplacement,
  waveformTargetY,
} from "../lib/analysis-orb";

export function AnalysisOrb({
  phase,
  peaks = null,
}: {
  phase: "orb" | "morph";
  peaks?: readonly number[] | null;
}) {
  const canvasRef = useRef<HTMLCanvasElement | null>(null);

  useEffect(() => {
    const canvas = canvasRef.current;
    const ctx = canvas?.getContext("2d");
    if (!canvas || !ctx) return;

    const dpr = globalThis.devicePixelRatio || 1;
    const particles = createOrbParticles(ORB_PARTICLE_COUNT, mulberry32(ORB_SEED));
    const start = performance.now();
    let frame = 0;

    const draw = (now: number) => {
      frame = requestAnimationFrame(draw);
      if (document.visibilityState === "hidden") return;
      const rect = canvas.getBoundingClientRect();
      if (rect.width <= 0 || rect.height <= 0) return;
      const W = Math.round(rect.width * dpr);
      const H = Math.round(rect.height * dpr);
      if (canvas.width !== W || canvas.height !== H) {
        canvas.width = W;
        canvas.height = H;
      }
      ctx.clearRect(0, 0, W, H);

      const t = (now - start) / 1000;
      const mix = phase === "morph" ? easeInOutQuad((now - start) / MORPH_MS) : 0;
      const cx = W / 2,
        cy = H / 2;
      const R = Math.min(W, H) * 0.36;
      const ry = t * 0.4,
        rx = Math.sin(t * 0.23) * 0.35;
      const cosY = Math.cos(ry),
        sinY = Math.sin(ry);
      const cosX = Math.cos(rx),
        sinX = Math.sin(rx);

      for (const p of particles) {
        const x1 = p.x * cosY + p.z * sinY;
        const z1 = -p.x * sinY + p.z * cosY;
        const y1 = p.y * cosX - z1 * sinX;
        const z2 = p.y * sinX + z1 * cosX;
        const n = orbDisplacement(p, t);
        const persp = 1.7 / (1.7 + z2 * 0.9);
        const ox = cx + x1 * n * R * persp;
        const oy = cy + y1 * n * R * persp;

        let X = ox,
          Y = oy;
        if (mix > 0) {
          const wx = p.u * W;
          const wy = cy + waveformTargetY(p.tyUnit, p.u, peaks ?? []) * H * 0.44;
          X = ox + (wx - ox) * mix;
          Y = oy + (wy - oy) * mix;
        }
        const depth = (1 - z2) / 2;
        // Morph ending: particles thin out as the real waveform fades in.
        const alpha = (0.18 + 0.5 * depth) * (1 - 0.85 * mix * mix);
        ctx.fillStyle = `rgba(140, 176, 255, ${alpha.toFixed(3)})`;
        const s = (0.8 + 0.8 * depth) * dpr;
        ctx.fillRect(X, Y, s, s);
      }
    };
    frame = requestAnimationFrame(draw);
    return () => cancelAnimationFrame(frame);
  }, [phase, peaks]);

  return (
    <canvas
      ref={canvasRef}
      className={"wf-orb" + (phase === "morph" ? " is-morph" : "")}
      aria-hidden="true"
    />
  );
}
