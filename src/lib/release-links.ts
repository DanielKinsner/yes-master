// Fixed manual-recovery origin for the desktop updater (audit L-03). The
// native open_release_page command opens ONLY this URL — it takes no
// argument, so a compromised renderer cannot steer the opener anywhere else.
//
// Must stay byte-identical to the private RELEASES_INDEX_URL constant in
// src-tauri/src/lib.rs and to the landing page's derived releases-index URL
// (src/landing/release-config.ts) — release-readiness.test.ts pins all three.
export const RELEASES_INDEX_URL =
  "https://github.com/DanielKinsner/yes-master/releases";
