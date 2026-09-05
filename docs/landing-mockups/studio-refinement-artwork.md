# Studio refinement artwork

Owner-requested creative pass, 2026-09-04. Branch: `codex/studio-marketing-redesign`.
Generated with the built-in `image_gen` tool. No API/CLI fallback or external stock images.

## Selected assets

All files are under `src/assets/landing/studio/`:

- `advanced-laptop-front.webp`: newly generated blank, front-facing graphite laptop (1536 × 1024). The unchanged `advanced-ui.png` is placed as a separate HTML image inside the display. No app pixels were generated, retouched or warped. CSS clips the generated surrounding background to the chassis silhouette.
- `studio-tape.webp` and `studio-tape-768.webp`: decorative reel-machine detail for the sound section.
- `studio-listening.webp` and `studio-listening-768.webp`: decorative musician/listening scene for the closing section. This is illustration, not a real customer or testimonial.

Photography is generated atmosphere, not product evidence, a physical gear requirement, or a claim of specific modeled hardware. The source screenshot hashes, closed release/signup state, noindex and domain remain unchanged.

Selected originals remain in the generation output directory under `C:/Users/Daniel Kinsner/.codex/generated_images/01a06f95-38e0-74f1-b423-5af86b44f7c1/`:

- Laptop: `exec-4e81ef0d-371f-4a50-9cf6-e2eda4ff4bc1.png`
- Listening: `exec-83e96f9e-c710-429f-a9ae-99845853463c.png`
- Tape: `exec-23450e6e-2e1c-4ded-9581-1f428a7e0187.png`

Browser WebP encodes use quality 0.88, retaining 1536 × 1024 originals and 768 × 512 responsive photo variants. All five shipped assets total 334,292 bytes, load lazily, and are bound by SHA-256 in the landing manifest. The earlier angled Advanced composite remains on disk for history but is no longer imported or shipped.

## Prompts

### advanced-laptop

Use case: product-mockup. Create a photorealistic premium graphite laptop cutout for a music mastering website. Landscape canvas 1536 x 1024 with a genuinely transparent alpha background. Camera is DEAD CENTER, squarely facing the screen: horizontal top/bottom screen edges, vertical sides, zero rotation, no sideways skew or perspective foreshortening of the display. The open lid faces us head-on; keyboard base recedes symmetrically below. Whole laptop visible, framed generously but filling about 92% of image width. Display opening is a flat, solid black rectangular blank at approximately a 1.81:1 aspect ratio, with thin dark rounded bezels. Subtle cold blue rim lighting along left edge and warm amber rim light on right; deep charcoal aluminum, precise physical edges, understated unbranded hardware. Keyboard subtly lit, realistic trackpad and slim base. Soft realistic contact shadow directly beneath. Dark, tactile, high-end recording studio mood. Absolutely NO screen content, UI, letters, branding, logos, watermark or background scenery. This is a blank chassis: the real product screenshot will be overlaid separately in HTML, so the unobstructed blank screen must be perfectly frontal and rectangular.

### studio-listening

Use case: photorealistic-natural. Generate a cinematic editorial photograph for YES Master, a local music mastering tool for independent musicians and producers. Landscape 3:2, 1536x1024. Intimate late-night recording studio listening moment: a musician seen only from behind in the right third, wearing a dark casual knit shirt, seated at a worn walnut production desk, one hand resting naturally near the console, head slightly turned as they listen. Warm amber practical light catches the forearm, wood grain, small analog faders and a pair of headphones. Large nearfield speaker and acoustic treatment softly recognizable. Left side falls into dark midnight navy negative space and subtle haze, while the subject and instrument detail stay right. Cool steel blue ambient light and restrained amber pools, 35mm film grain, subtle halation, shallow depth of field, real lived-in texture and grounded photographic beauty, no glossy sci-fi neon. Quiet pride at finishing a record. No readable text, logos, signage, UI, visible computer display or exaggerated effects. This is atmospheric illustration, not a testimonial or product screenshot.

### studio-tape

Use case: photorealistic-natural. Generate an atmospheric editorial detail photograph for a premium music mastering website in midnight blue and warm amber. Landscape 3:2, 1536x1024. Extreme tactile close-up of a vintage open reel tape recorder in a small working recording studio: large brushed metal reel occupying upper right, warm brown magnetic tape through real rollers across the midframe, delicate amber meter glow blurred far behind, left foreground shallow focus wood edge and one cable. Accurate plausible audio equipment, no invented text. A narrow beam of warm side light reveals brushed aluminum machining and tiny dust particles against richly textured dark blue shadows. Filmic, restrained, subtle analog grain, rich blacks with detailed shadows, sophisticated music magazine still life, 85mm macro feel. Composition remains quiet and spacious, rich on right side with darker left half for adjoining HTML editorial typography. No people, no words, no logo, no watermark, no UI, no collage, no excessive bright neon.

## Unselected iteration

An additional built-in edit requested transparent extraction of the laptop. It still returned an opaque surrounding scene, so it was not selected. Instead the original generated chassis uses a responsive CSS silhouette mask; no screenshot transformations are involved.

Edit target: the generated front-facing laptop image. Remove ALL scenery and colored background outside the laptop and make it genuinely transparent alpha. Preserve the laptop exactly: head-on geometry, black blank display, precise graphite chassis, keyboard, blue left rim light and amber right rim. Keep the full device visible. Remove the blue/amber fog and gray background surrounding the laptop, leaving real alpha transparency, not a checkerboard. No changes to the blank display; no text or UI. This cutout will be layered over an existing studio photo.

