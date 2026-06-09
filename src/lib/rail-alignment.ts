// src/lib/rail-alignment.ts
//
// Seam alignment between the Standard center column and the right rail
// (2026-06-09). Design goal: the rail's card seams sit on the same
// horizontal lines as the center column's — Preview card bottom flush with
// the Intensity card bottom (so Delivery Format tops align with Loudness
// via the shared 1rem column gap), and the export group closing flush with
// the Loudness card's bottom edge.
//
// The two columns are independent scroll containers, so CSS alone cannot
// pin their seams: card heights are content-driven. StandardView measures
// the current geometry (ResizeObserver + center scroll) and this module
// decides — pure function, unit-testable with plain numbers — whether
// alignment is possible and what the two pixel values are. Returning null
// means "fall back to the flex-absorb layout" (Preview soaks up free rail
// height), which is the pre-alignment behavior and always safe.

export type RailAlignmentInput = {
  /** Top edge of the Preview card (viewport px). */
  previewTop: number;
  /** Bottom edge of the center column's Intensity card (viewport px). */
  intensityBottom: number;
  /** Bottom edge of the center column's Loudness card (viewport px). */
  loudnessBottom: number;
  /** Bottom of the rail's content box: rail bottom minus its bottom padding. */
  railContentBottom: number;
  /** Measured height of the Delivery Format card. */
  deliveryHeight: number;
  /** Measured height of the export group (button + any notes). */
  exportHeight: number;
  /** The rail's flex gap between cards (px). */
  railGap: number;
};

export type RailAlignment = {
  /** Fixed height for the Preview card so its bottom = Intensity bottom. */
  previewHeightPx: number;
  /** Bottom margin lifting the export group flush with Loudness bottom. */
  exportMarginBottomPx: number;
};

/**
 * Floor for the Preview card: header + A/B toggle + the meter's own
 * 172px minimum (`.std-rail .master-out .lufs-meter`) + three readout rows.
 * Below this the meter would collapse; alignment is not worth a broken meter.
 */
export const MIN_PREVIEW_HEIGHT_PX = 320;

export function computeRailAlignment(
  input: RailAlignmentInput,
): RailAlignment | null {
  const {
    previewTop,
    intensityBottom,
    loudnessBottom,
    railContentBottom,
    deliveryHeight,
    exportHeight,
    railGap,
  } = input;

  const values = [
    previewTop,
    intensityBottom,
    loudnessBottom,
    railContentBottom,
    deliveryHeight,
    exportHeight,
    railGap,
  ];
  if (values.some((v) => !Number.isFinite(v))) return null;

  const previewHeightPx = intensityBottom - previewTop;
  if (previewHeightPx < MIN_PREVIEW_HEIGHT_PX) return null;

  // Lift the export group so its bottom lands on the Loudness card's bottom.
  // Negative would mean the center column ends below the rail's content box
  // (e.g. a shorter window) — pinning would push the button out of the rail.
  const exportMarginBottomPx = railContentBottom - loudnessBottom;
  if (exportMarginBottomPx < 0) return null;

  // Everything between Preview bottom and the lifted export group must fit:
  // gap + Delivery card + gap + export group. If the window is too short the
  // pinned export would overlap Delivery — fall back instead of overlapping.
  const exportTop = railContentBottom - exportMarginBottomPx - exportHeight;
  const deliveryBottom = previewTop + previewHeightPx + railGap + deliveryHeight;
  if (deliveryBottom + railGap > exportTop) return null;

  return { previewHeightPx, exportMarginBottomPx };
}
