# Landing tightening — review branch

Owner requested this pass on 2026-09-05, for review before any merge to main.

- Lead with music and the listener's control; retain Your Endgame Sound as brand.
- Name artists, producers and anyone making music in the hero.
- Remove the benefits strip and duplicate Advanced explanation.
- Combine Standard workflow, four styles and source-aware restraint.
- Keep the actual export receipt as supporting proof, with the Standard/Advanced distinction.
- Shorten Album Master and remove the sign-off section. Preserve studio atmosphere.
- Leave audio examples for later. Preserve all download/signup/release gates.

## Advanced image provenance

Built-in image generation, edit mode. The owner explicitly requested generation
and title cleanup using their new WAV screenshot. The selected display is a
presentation derivative, not an unchanged screenshot or evidence of audio output.
Generative editing can alter fine pixels; the original is retained for comparison.
The original WAV metadata is present in both the supplied reference and the edit.

Original: `assets/doors-open-source.png` (2867 × 1629).
Selected asset: `src/assets/landing/studio/advanced-doors-open.png` (1663 × 946).
Exact original/output hashes and sizes are bound in the landing asset manifest.
The existing head-on laptop chassis is reused, with this display as a separate
HTML image layer. Site copy and controls remain HTML.

### Final generation prompt

Use case: text-localization. Edit target: the supplied YES Master screenshot. This is a precise text cleanup for a website product image, NOT a redesign. Preserve the complete screenshot, original aspect ratio (2867x1629), all controls, waveforms, metering, colors, panels, tiny text and numbers, and their exact positions. Only change the selected track name in TWO places: (1) large heading near top left of the main panel, currently 'Doors Open (Remastered) (1)' followed by music-note glyph: make it exactly 'Doors Open', retaining the original font and baseline; (2) highlighted second track row in the left sidebar, currently the same long name: make it exactly 'Doors Open', retaining its original font and baseline. Remove the music-note glyph after those two names along with the '(Remastered) (1)' suffixes. Fill the vacated text area with the corresponding flat original panel background. Do not change the WAV · 48 KHZ · STEREO · 3:25 label or the 3:25 duration below the sidebar name. Do not add or alter other text, labels, controls, numeric values, waveforms or features. No cropping, perspective change, laptop, framing, lighting effects, or additional decoration. Output the full flat screenshot at the highest fidelity available.

## Review evidence

Compared production at 436245d with this branch using bundled Chromium, reduced motion, decoded images and closed optional detail panels. No native code, DSP, release configuration or production domain
changes are part of this pass.

| View | Before height | After height | Reduction | Visible words before / after |
|---|---:|---:|---:|---:|
| Desktop, 1440px | 9,343px | 4,775px | 48.9% | 1,233 / 687 |
| Mobile, 390px | 12,254px | 6,672px | 45.6% | 1,225 / 683 |

Local evidence (ignored by git):
- `test-output/tightening/before/` and `after/`: hero, whole page and section renders at 1440px and 390px. No horizontal overflow. Visually reviewed hero, Advanced, Standard/styles, receipt and Album compositions.
- `npm test`: 841/841 tests across 83 files passed, including marketing copy and image provenance gates.
- `npm run build`: passed TypeScript and production build.
- `npm run verify:headless`: the app suite passed all 31 scenario/viewport checks. The initial landing run found one obsolete assertion requiring the brand inside the H1, rather than its new hero eyebrow. All other layout, accessibility and interaction checks passed. Evidence: `test-output/headless/2026-09-06T00-15-32-197Z/`.
- After correcting that copy assertion, `npm run verify:landing -- --url http://127.0.0.1:5175` passed all 13 viewports (320–2560px), keyboard/lightbox checks, 200% zoom, accessibility scans, reduced motion and observer fallback. Evidence: `test-output/landing-responsive/2026-09-06T00-18-12-719Z/`. The aggregate file from the earlier run is intentionally left intact; it records that earlier assertion failure, not the corrected rerun.

Review URL on this machine: http://127.0.0.1:5175/.
Branch: `codex/landing-tightening`, isolated from the owner's dirty native checkout.
No push, production deployment or merge to main was performed.

## Owner approval and interaction polish — 2026-09-05

The owner reviewed the page, approved it as the replacement and authorized merge
and publication after a tasteful bolder/animate pass. The preceding no-push and
review-only statements record the original review checkpoint, not the current
authorization.

- Added a clear View Advanced screenshot button beside the Advanced copy. Both
  this action and the laptop image open the same full-resolution viewer; Escape
  restores focus to the actual trigger.
- Made Standard's disclosure a visible blue outlined button with explicit
  View/Hide wording and a directional chevron. Screenshot overlays now label
  expansion on both modes. Minimum button height is 46px.
- Added only interaction feedback: brief icon/colour transitions and a 220ms
  opacity/4px opening transition for the Standard screenshot. Reduced motion
  disables these; existing finite artwork entrances remain unchanged.
- Retained the approved layout, artwork, editable copy and current release gates.
