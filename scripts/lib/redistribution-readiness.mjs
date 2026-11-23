// Release readiness of the corresponding-source package is derived from
// evidence. It used to be a hard-coded `false` that waited for two things:
// the owner's LGPL relinking permission and an actual rebuild proof. The
// permission is now `LICENSE` section 7 (approved 2026-11-23), so both can be
// checked instead of asserted. Nothing here approves publication: installed,
// listening and publication evidence stay owner gates in
// docs/plans/beta-go-no-go.md.

// Both fragments must be present in the archived LICENSE. They are the
// section heading and its operative grant; matching is whitespace-insensitive
// so re-wrapping the file cannot hide or fake the permission.
export const RELINK_PERMISSION_MARKERS = [
  'LGPL-COVERED LIBRARY COMPONENTS',
  'modify and rebuild or relink',
];

const collapse = text => String(text ?? '').replace(/\s+/g, ' ');

/** True when the given LICENSE text carries the relinking permission. */
export function hasRelinkPermission(licenseText) {
  const license = collapse(licenseText);
  return RELINK_PERMISSION_MARKERS.every(marker => license.includes(marker));
}

export const PENDING_PERMISSION =
  'LGPL relinking permission is absent from the archived LICENSE';
export const PENDING_REBUILD =
  'Rebuild verification with a modified library (scripts/verify-redistribution-relink.mjs)';

/** Gates no script can establish. Recorded so a ready package never reads as publication approval. */
export const OWNER_GATES = [
  'Final candidate installed, listening and publication evidence (docs/plans/beta-go-no-go.md)',
];

/**
 * Ready only when the archived revision grants the relinking permission AND
 * that exact revision was rebuilt with a modified library. Anything that is
 * not literally `true` counts as missing, so absent fields fail closed.
 */
export function sourceReadiness({ relinkPermission, rebuildVerified } = {}) {
  const pending = [];
  if (relinkPermission !== true) pending.push(PENDING_PERMISSION);
  if (rebuildVerified !== true) pending.push(PENDING_REBUILD);
  return { releaseReady: pending.length === 0, pending };
}
