# iPhone Shippability Plan (2026-06-12)

Device-scoped execution plan: everything between the current iPhone app
(TestFlight-grade, engine excellent) and an App Store-submittable product.
Desktop ships first (owner decision 2026-06-12); this plan is authored now
so a non-Claude implementer can run it without re-deriving context.

Parent docs: `docs/reviews/2026-06-12-master-shippability-audit.md`
(findings IP-01..IP-05, MB-01, XP-04/05, UF-B1..B5),
`docs/plans/2026-06-12-shippability-roadmap.md` (slice specs + global
rules — rule 7 applies: interrogate the owner on every product call),
`docs/plans/2026-06-12-adaptive-compressor-mvp-spec.md` (AC-4 inheritance
proof touches this app's bridge tests).

Current state, one paragraph: Swift app + Rust facade over the shared
`yes_master_lib`. Bit-exact engine parity with desktop is test-pinned;
security-scoped bookmarks, interruption/route handling, and FFI string
ownership are verified-correct. The gaps are crash-class bridge issues,
main-thread freezes, memory on long sources, store packaging, and a
hardening list from the 2026-06-01 handoff that never ran.

## Definition of "iPhone shippable"

App Store submission passes validation; no panic can cross FFI; import of
a 60-minute file neither freezes the UI nor OOM-kills the app; live
audition survives a 5-minute Instruments-profiled session near ceiling;
the Standard recipe/vocabulary is fixture-pinned against desktop; owner
has done one on-device listening pass.

## Phase 1 — Crash class (do first, in this order)

1. **IP-1.1 Panic guards on every extern (audit IP-01; roadmap S4.1).**
   `catch_unwind` on all `#[no_mangle]` fns in `rust/src/lib.rs` and
   `live_stream.rs` (only `live_process` is guarded today), returning the
   `{"error":...}` JSON contract. Clone Android's `catch_panic` shape.
   Forced-panic injection test. Lane: iPhone rust + Android rust (links
   the facade).
2. **IP-1.2 Async import + single decode (IP-02 + MB-01; roadmap S4.4).**
   Swift `.preparing` state, copy+load off the main actor (mirror
   Android's Dispatchers.IO); Rust: one decode shared between stream
   create and adaptive context (today it decodes the full track TWICE at
   create); 30-minute import duration cap with a clear error message,
   threshold in one constant, unit-tested. Decode-count proxy test.
3. **IP-1.3 Stale-landing guard (IP-05 item).** Mirror Android's
   `handle != h` check in `AuditionController.refreshLanding`
   (`AuditionController.swift:359-372`); pin with the fake-stream rig.

## Phase 2 — Audio-thread hardening (UF-B; the 2026-06-01 list)

4. **IP-2.1 FTZ + no-alloc callback (UF-B2, UF-B3; roadmap S9.3).** Set
   ARM FPCR flush-to-zero on the render thread; replace the in-callback
   `Vec` clones (`live_stream.rs:181`) with a preallocated coeff
   double-buffer. One test rig for both.
5. **IP-2.2 Landing rate reconciliation (UF-B4; roadmap S9.4).** Live
   landing currently measures at source rate; export lands post-resample
   at 44.1 kHz. Reconcile so WYSIWYG holds on 48/96 kHz sources; pin with
   a 48 kHz fixture asserting live and export landings agree within the
   documented window error.
6. **IP-2.3 Instruments RT profiling pass (UF-B1).** Owner-Mac session:
   Mastered playback near ceiling, ≥5 min, thermal observation. Called
   "the real spike gate" in the handoff; it has never run. Record results
   in a dated evidence doc. (Owner time; schedule with Session 2.)

## Phase 3 — Contract pins (shared with desktop Gate C)

7. **IP-3.1** Export-recipe + Swift vocabulary parity fixture (XP-04;
   roadmap S3.3 — the Swift side: bundle `standard-mapping-parity.json`
   into the test target; assert `bridgeIdentifier` map and the
   −14/−11/−9 trio from it; debug-flag unknown style ids).
8. **IP-3.2** FFI header pin (XP-05; roadmap S3.5 — cbindgen diff test).
9. **IP-3.3** AC-4 inheritance proof (adaptive-compressor spec §4):
   bridge test rendering a dense fixture gate-ON, bit-equal to desktop.

## Phase 4 — Product completeness (interrogate the owner first — rule 7)

10. **IP-4.1 Honest progress (IP-05; roadmap S8.2).** Replace the staged
    wall-clock progress theater (sleeps, parks at 94%) with indeterminate
    + honest labels minimum; real progress events if the facade grows
    them.
11. **IP-4.2 Error-state model (IP-05; S8.3).** Replace the
    substring-sniffing error display (`ContentView.swift:722-732`) with an
    explicit error enum + view test. Also surface (not `.ok()`-swallow)
    render-time adaptive-context failures (audit IP-05 tail).
12. **IP-4.3 Background audio decision (IP-04; owner question).** Lock
    screen currently kills audition. One plist line if wanted
    (`UIBackgroundModes: audio`) + device QA item. Ask, then do.
13. **IP-4.4 Files-app honesty (IP-05 tail).** Either move masters to
    Documents (visible in Files) or drop
    `UIFileSharingEnabled`/`LSSupportsOpeningDocumentsInPlace`. Ask the
    owner which way; ShareLink remains either way.
14. **IP-4.5 Dead code deletion.** `TrackPlaybackController.swift`,
    `AudioSessionController.swift` + tests (superseded by
    LiveAudioEngine); grep-verify zero references first.

## Phase 5 — Store packaging

15. **IP-5.1 PrivacyInfo.xcprivacy (IP-03; S8.1).** Required-reason
    file-timestamp APIs (C617.1) — hard ASC rejection without it. Add
    `ITSAppUsesNonExemptEncryption=false`. Wire into `project.yml`;
    validate via archive validation on the mac CI lane.
16. **IP-5.2 Versioning single-source.** `1.0 (1)` is hardcoded in a
    committed plist that xcodegen also generates — pick one source; align
    with the repo's 0.1.0 scheme (roadmap S0.2 test covers it).
17. **IP-5.3 Listing assets** (screenshots, description, privacy labels)
    + TestFlight round. Owner-involved by nature.
18. **IP-5.4 Signing.** `DEVELOPMENT_TEAM` is hardcoded in `project.yml`
    (fine for owner-built); revisit only if distribution changes.

## Lanes

Every phase: `cd apps/iphone-native/rust; cargo check --all-targets;
cargo test`. Swift changes additionally: mac CI job (`xcodegen` +
`xcodebuild test`) from roadmap S0.1. Shared-crate touches: full desktop
Rust lane + Android lane too (CLAUDE.md rule).

## Owner gates (the only human-time items)

Instruments pass (IP-2.3), background-audio + Files-app decisions
(IP-4.3/4.4), the AC-5 iPhone spot-check (compressor spec §5), listing
assets (IP-5.3), and one end-to-end on-device listening pass before
submission.
