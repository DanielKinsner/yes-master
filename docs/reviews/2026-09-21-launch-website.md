# September 22 website preparation

The existing production domain is already `yesdsp.com`, as named in the
September 5 owner analytics decision. Live verification on September 21 found:

- Vercel project `yes-master` has `yesdsp.com`, `www.yesdsp.com` and the existing
  Vercel aliases attached to production deployment
  `dpl_4gsheBu1MwmX6FJXuzzL6uHRHQix`, source `ab420654`.
- `https://yesdsp.com/` redirects to `https://www.yesdsp.com/`, which returns
  HTTP 200. The production share image also returns HTTP 200 as JPEG.
- The live document still declares the old Vercel address in its canonical and
  three social tags. Correct those four URLs and the existing same-origin
  regression to the actual final production origin.
- The production landing smoke suite passes. This verifies the existing live
  site, not deployment of the new launch copy or new metadata.

This website follow-up starts at installer candidate `2bc36054`. It changes no
processing, export, installer or application behavior. The candidate's exact
packaging and remote CI remain tracked in the
[launch execution record](2026-09-21-launch-preparation.md).

Keep `RELEASE_METADATA` null and retain noindex until a verified published
release supplies actual URLs, hashes and dates. At activation, the beta end date
is actual publication plus 56 days (November 17 if published September 22).
The website branch disables its automatic deployment. A release decision,
website deployment and a real chat-paste share-card check remain explicit.

Local verification after the metadata correction passes all 903 frontend tests
and the full headless suite: 40 application scenarios plus responsive and 200%
zoom landing checks with normal and fallback fonts. Evidence is retained in
`test-output/launch-website-20260921/` and
`test-output/headless/2026-09-22T06-20-36-622Z/` in this website worktree.
