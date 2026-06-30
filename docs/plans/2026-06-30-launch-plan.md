# YES Master — Launch Plan (2026-06-30)

> First-launch go-to-market + the engineering work it requires. Strategy decided
> with the owner on 2026-06-30 after market research (pricing, distribution, GTM,
> platform sequencing, tiered-pricing, and positioning). This is the working
> checklist; it is NOT product canon — see `docs/PRODUCT.md` for canon (which
> still needs the positioning reframe in §2, owner-gated).

## TL;DR

Launch **desktop-first (Mac + Windows)** as a **free, time-boxed public beta**
given away from the existing landing page, then flip to a **single paid product
at $29 founder → $49**, sold direct via **Lemon Squeezy** (merchant-of-record),
with a **permanent free export-locked demo** as the conversion engine. Lead
marketing with **performance + simplicity + honesty**, not privacy. Mobile and a
later Standard/Advanced tier split are deferred. The scarce resource is not money
or code — it is *other people vouching for you* — so the free beta exists to buy
that with reviews, testimonials, and a warm email list.

---

## 1. Decisions locked (2026-06-30)

| Area | Decision |
|---|---|
| **Model** | Free public beta → paid 1.0. Single SKU. Open to discounted paid v2 upgrades later. |
| **Price** | **$29 founder intro → $49 standard.** One-time perpetual license. Beta users lock in the $29 founder price. |
| **Free tier** | Permanent **export-locked demo** (play + full real-time chain + full receipt visible; cannot render/export until purchase). NOT a free "Standard" tier. |
| **Tier split** | Standard-vs-Advanced split **deferred to v2**, gated on album + proof/export depth once buyer data exists. Never gate receipt *visibility*. |
| **Distribution** | Sell **direct** from the landing page via **Lemon Squeezy** (MoR, handles global tax). Installers hosted on **GitHub Releases**. Microsoft Store later; **skip Mac App Store**. |
| **Licensing** | **One-time online activation** (cached, with an offline grace period). Keys issued by Lemon Squeezy. |
| **Signing** | macOS **Apple Developer ID** + notarization ($99/yr). Windows **Azure Trusted Signing** ($10/mo). Sign from the **first** beta build. |
| **Updates** | Tauri updater pulling from GitHub Releases. |
| **Platform** | Desktop Mac + Win first. iPhone/Android parked to an audience-gated later phase (shared engine = no cost to defer). |
| **Positioning** | Lead with performance / simplicity / honesty. Demote "local/private" to a speed-and-convenience footnote. |

**Brand — RESOLVED (2026-06-30):** the name is **YES Master**. The landing page
currently uses `Y.E.S. Master / Your Endgame Sound` and **must be changed** to
match. This also closes the long-parked branding decision (OPEN_THREADS B22).

---

## 2. Positioning (reframed — privacy demoted)

The local-first design is about **performance and simplicity**, not privacy as a
stance (owner correction, 2026-06-30). Marketing leads accordingly.

**Landing headline:** *"Master your track in real time — and see exactly what it did."*

**Supporting bullets:**
- **Real-time, every tweak.** The full chain runs as you listen — no upload, no
  reprocessing wait. Cloud tools re-upload and re-analyze on every change; you just hear it.
- **Simple by default, deep when you want it.** Standard masters in one move;
  Advanced opens full metering, compressor, and album tools when you're ready.
- **No black box.** A pass/fail receipt shows LUFS, true-peak, and dynamic range —
  push as hard as you like and still see the truth. (Every cloud/AI masterer buries this; we lead with it.)
- **Own it forever.** One-time purchase, no subscription.

**Where "local" goes:** one line beneath the bullets — *"Runs on your machine —
no account, no upload, works offline after a one-time activation."* Framed as *why
it's instant and zero-friction*, never as a privacy crusade. Privacy = a quiet FAQ
perk only.

**Hero conversion asset:** an interactive **Original/Mastered A/B** (the
playhead-preserving switch already built) embedded in the page — proof-by-audio
substitutes for the celebrity social proof we don't have.

> Canon impact: `docs/PRODUCT.md`, `README.md`, and `docs/landing-brief.md` still
> headline privacy. Reframing them is **owner-gated** (per CLAUDE.md working style) —
> get sign-off before editing canon.

---

## 3. Pricing & licensing

| Tier | Price | Includes | Withholds |
|---|---|---|---|
| **Free Demo** (permanent) | $0 | Full real-time master, A/B, **full receipt visible** | **Export/render only** — hard gate |
| **YES Master** (the SKU) | $29 founder → $49 | Everything built: Standard + Advanced, metering, compressor, album, all formats, unlimited export | Nothing |
| **Advanced/Studio** (v2, deferred) | +$20–30 "pay the difference" | Album/batch + deep proof/export depth | Held back from base only after telemetry confirms demand |

- **Beta → paid transition:** time-box the beta with a publicly stated flip date;
  honor a founder discount for beta users. Avoid an indefinite free tier.
- **Recurring revenue path:** future major versions are paid upgrades, discounted
  for existing owners (the no-subscription way to earn again).
- **Upgrade mechanic (v2):** "pay the difference" is the audio-industry norm;
  Lemon Squeezy's native upgrade flow is weak, so plan manual SKUs/discount codes.

---

## 4. Distribution & signing stack

- **Checkout:** Lemon Squeezy as merchant-of-record (remits VAT/GST/sales tax;
  no company required to start; issues license keys + activation API; ~5% + $0.50).
  Confirm payout-country support.
- **Hosting:** signed installers published to **GitHub Releases**; the Tauri
  updater reads its manifest from there.
- **macOS signing:** Apple Developer Program ($99/yr) → Developer ID Application
  cert + `notarytool` notarization + hardened runtime + entitlements. Currently
  `signingIdentity` is ad-hoc `"-"` — must become a real identity.
- **Windows signing:** Azure Trusted Signing (~$10/mo; open to individuals since
  2026) → Authenticode. Sign from the first public build so SmartScreen reputation
  starts accruing now (no cert buys instant trust; early users may still see a
  warning until reputation builds).
- **Stores:** Microsoft Store later (near-0% cut, extra reach). Mac App Store
  skipped (15–30% cut + sandbox fights local file access).

---

## 5. Go-to-market (solo dev, no network)

- **Hub:** the existing landing page + an email list via a privacy-respecting tool
  (Buttondown / MailerLite / Kit) — plain opt-in newsletter, not tied to the app.
- **Seed channels:** KVR Audio (Instruments & Effects announce; consider the
  Developer Challenge), Bedroom Producers Blog (actively features notable free
  releases), and 3–5 sub-10k producer micro-creators. **Participate genuinely for
  2–3 weeks before promoting** or get flagged as a spammer.
- **Skip:** paid ads, big influencers. Short-form video optional (depends on
  owner's on-camera comfort).
- **Hold for when polished:** Product Hunt + a BPB feature are one-shot spikes;
  only fire them once the landing page + download capture intent flawlessly.

**First-30-days checklist**
1. Put the beta→paid model, flip date, and founder-price promise in writing on the landing page.
2. Sign up for Lemon Squeezy; confirm payout country; decide sole-trader vs company.
3. Enroll Apple Developer ($99/yr) + set up Azure Trusted Signing ($10/mo) — start the clock.
4. Wire the release pipeline: signed installers → GitHub Releases + Tauri updater JSON; verify clean install + auto-update on real Mac and Windows.
5. Add email opt-in to the landing page.
6. Produce proof assets: 2–3 owned before/after clips + one 60–90s metering/receipt screen recording.
7. Create KVR + one subreddit account; start participating (no promo).
8. Announce the free beta on KVR once the page + download are flawless; collect emails.
9. Send BPB a clean press note; line up micro-creators.
10. Run a one-question price survey + watch Advanced/album usage to validate $39→$49 before the flip.

---

## 6. Engineering backlog (to be launch-ready)

Derived from the launch-readiness scan + the strategy above. Ordered by leverage.
Most of these become the detailed implementation plan (next step).

**P0 — without these there is nothing to launch**
- [ ] **Release pipeline:** GitHub Actions workflow that builds, **signs**,
  notarizes, and publishes installers to GitHub Releases on a tagged release.
- [ ] **macOS signing + notarization** wired into the release build.
- [ ] **Windows Authenticode signing** (Azure Trusted Signing) wired in.
- [ ] **Landing "Download" button** wired to real release artifacts (currently a `mailto:`).
- [ ] **Export gate (free/paid boundary):** *Feasibility confirmed CLEAN
  (2026-06-30).* All exports funnel through a single chokepoint — the
  `render_track_master` and `render_album_plan` commands in
  `src-tauri/src/engine.rs`. Gate there via a **runtime license check** (single
  binary, unlock-in-place to match online activation — NOT a separate demo
  binary). Audition + metering are unaffected; DSP snapshot tests are untouched.
  This is the SAME seam as the license check below — build them together at the
  1.0 flip (needs the Lemon Squeezy account).

**P1 — needed for the paid flip**
- [ ] **License activation** in the Tauri core: Lemon Squeezy key, one-time online
  check, cached with offline grace; fails gracefully offline.
- [ ] **Tauri updater** integrated (currently absent) + signing keypair for update artifacts.
- [ ] **Version bump** from 0.1.0 to a real beta version; set Windows publisher metadata.
- [ ] **Landing page rewrite** to the §2 positioning + embed the A/B hero demo + email capture.
- [ ] **Brand decision** applied across landing/app/docs.

**P2 — required for taking money / good practice**
- [ ] **End-user EULA** bundled with the app (distinct from the repo source LICENSE).
- [ ] **Privacy policy** (short — the app collects nothing) + terms + refund policy.
- [ ] Consider swapping the self-authored source LICENSE for PolyForm Noncommercial (author already flagged for review).
- [ ] Decide crash-reporting/feedback posture (opt-in only, if any).

**Owner-gated (not engineering, but launch-blocking by CLAUDE.md non-negotiable)**
- [ ] **Manual Listening Gate** sign-off (normal / already-mastered / long-source
  sweeps + clean-vs-warning export). Adaptive compressor (AC-5) and Phase-B stay
  gated OFF for v1 unless a listening sitting flips them.
- [ ] Confirm **macOS build/install** works on a real machine.
- [ ] Confirm **real-time is genuinely snappy** on a mid-tier Mac AND Windows laptop
  (the headline depends on it; fall back to leading with "simple" if not).

---

## 7. Phased roadmap

1. **Phase 1 — Free beta** (~8–12 wks): signed desktop build (Win+Mac), GitHub
   Releases, email-gated from the landing page. Full app, no watermark. GTM = KVR +
   BPB + micro-creators + list. Goal: reviews, testimonials, warm list, hardened claims — not revenue.
2. **Phase 2 — Paid 1.0** (at the announced flip date): $29 founder → $49 via
   Lemon Squeezy. Beta users keep the $29 founder price. Free build becomes the
   export-locked demo. Launch to the warm list; consider a one-shot Product Hunt spike.
3. **Phase 3 — Expand:** Microsoft Store; introduce the Advanced/Studio tier (~$69–79
   all-in, pay-the-difference) if beta data supports it; re-price against real conversion.
4. **Phase 4 — Mobile** (audience-gated trigger): revive the shared-engine iPhone/
   Android bridges, leading with a *free* metering/receipt-viewer companion to the paid desktop app.

---

## 8. Risks

- **Price is directional**, validated against comps not your funnel — confirm with
  real beta→paid numbers; lean cheaper/more generous early to buy word-of-mouth.
- **"Real-time" is a literal headline promise** — verify snappiness on mid-tier laptops.
- **Export-lock leakage** would sink the funnel — must be a hard gate.
- **Windows SmartScreen** friction until download reputation accrues — sign early, add an install note.
- **No celebrity proof** — deliberately harvest named beta quotes + before/after clips, or the A/B angle has nothing to show.
- **Beta discipline** — a hard flip date prevents an indefinite unpaid support burden.

---

## 9. What only the owner can provide

- Apple Developer account ($99/yr) + Azure Trusted Signing setup ($10/mo) — identity verification is yours.
- Lemon Squeezy account; payout country; sole-trader vs company decision.
- 2–3 royalty-clear before/after song snippets you own, for the A/B hero demo + clips.
- The final brand-name call.
- The owner-gated listening sign-off (Manual Listening Gate) and a real-machine macOS build confirmation.
- Time commitment: 2–3 weeks of genuine community participation before promoting; on-camera comfort decides whether short-form video is in scope.
