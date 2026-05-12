# ADR 0001 — Tauri 2.x Shell + Rust Audio Engine

Status: Accepted
Date: 2026-05-11
Phase: 0 — Workspace scaffold and architecture decision

## Context

The Claude-build repo for Album Mastering Studio starts from a zero-state. `docs/PRODUCT.md` locks the installed Tauri desktop app as the primary product surface and requires real-time or near-real-time audition, Windows packaging, offline rendering quality, file safety, and testability. `docs/CLAUDE_BUILD_BRIEF.md` requires that the framework not be locked before an evidence-based architecture decision.

This ADR records the architecture choice for Phase 0 and beyond, the alternatives considered, and what remains reversible.

## Decision

Build the app as a Tauri 2.x desktop shell with a React + TypeScript frontend and a Rust audio engine in `src-tauri/`.

- Frontend: React 19, TypeScript 5, Vite 6, served via Tauri webview in dev and bundled static in release.
- Backend: Rust 2021 edition, Tauri 2, with typed `#[tauri::command]` handlers exposing product concepts (see Phase 1 in `docs/IMPLEMENTATION_PLAN.md`).
- Real-time audio: Rust thread using `cpal` for device I/O, `symphonia` for decode, `hound` for WAV write. Hand-rolled DSP for the first slice; consider `fundsp` or `nih-plug`-derived primitives when DSP audit (Phase 11) demands.
- No Python sidecar on the realtime path. A Python lane is permitted only for offline DSP R&D and offline parity testing, never on the audition path.
- Packaging: Windows installer (NSIS or MSI) via `tauri build`. Code signing deferred to Phase 14.

## Alternatives Considered

### A. JUCE / native C++ application

- **Pros:** Mature audio framework. Best-in-class real-time audio history. Strong DSP ecosystem.
- **Cons:** UI development is heavier than HTML/CSS/React for the product surface this app needs (left rail, waveform, transport, preset tiles, EQ). Windows packaging fine but iteration speed and developer ergonomics are slower for non-audio UI work. Steeper barrier for non-C++ contributors and AI assistants.
- **Verdict:** Rejected for primary surface. Kept as a reversible fallback if Phase 5 real-time targets fail under Tauri+Rust.

### B. Rust-native UI (egui / iced / dioxus) + Rust audio

- **Pros:** Single language. Strongest real-time discipline. No webview overhead.
- **Cons:** Native Rust UI ecosystems still trail HTML/CSS for visual polish, animation, and accessibility. Waveform interaction, drag/drop, complex transport UI, and design iteration are slower. Smaller pool of UI components.
- **Verdict:** Rejected as primary surface. Reconsider if Tauri webview rendering or IPC becomes a real bottleneck.

### C. Tauri UI + Python audio engine (sidecar)

- **Pros:** Rich NumPy/SciPy ecosystem for DSP iteration. Matches the sibling Codex repo's approach.
- **Cons:** Process boundary between UI and audio thread complicates real-time audition (parameter updates, A/B toggle, region loop all crossing a process barrier). `docs/CLAUDE_BUILD_BRIEF.md` explicitly allows Python only if paired with a credible realtime path; the realtime path must be in-process Rust. Adding both adds complexity without removing the Rust audio dependency.
- **Verdict:** Rejected for primary path. Python allowed as opt-in offline R&D lane only.

### D. Hybrid: Tauri UI + Rust audio + optional C++ DSP via FFI

- **Pros:** Best of Rust dev velocity with access to mature C++ DSP libraries when needed.
- **Cons:** FFI surface adds maintenance and packaging cost. Not needed until DSP audit (Phase 11) demonstrates a specific gap that only C++ fills.
- **Verdict:** Deferred. Treated as a Phase 11 escape hatch if benchmarking finds Rust crates insufficient.

## Evidence Gathered

This decision is based on:

- `docs/PRODUCT.md` locks Tauri as primary product surface (Locked Decisions #1, Architecture Direction section).
- `docs/CLAUDE_BUILD_BRIEF.md` lists Tauri+Rust audio as an explicitly allowed direction.
- The sibling Codex repo proved Tauri can ship a working desktop mastering app with native audio behavior (proof of feasibility; no code imported).
- Rust audio crates (`cpal`, `symphonia`, `hound`, `fundsp`) are mature enough for a release-candidate engine without requiring C++ FFI on day one.

What this ADR does **not** prove:

- That Tauri+Rust will meet Phase 5's ~150 ms real-time latency targets. That is a spike, not an assumption.
- That hand-rolled DSP will reach release-candidate quality without help from established DSP libraries. Phase 11 may pull in additional crates or, if needed, C++ FFI.

## Risks

1. **Real-time latency.** Tauri's webview-to-Rust IPC is not on the audio thread, but parameter smoothing and control updates must remain responsive. Mitigation: Phase 5 spike with explicit measurements; ADR 0002 will record the result.
2. **Webview render performance for waveforms at zoom levels.** Canvas/WebGL is fine, but very large peak caches may stress repaint. Mitigation: stream peak data via Tauri commands; render at viewport resolution.
3. **DSP quality without C++ DSP libraries.** Rust ecosystem is younger. Mitigation: Phase 11 DSP audit with research docs; FFI escape hatch documented above.
4. **Windows packaging quirks.** Tauri 2's NSIS/MSI flow is mature but icon requirements and signing add Phase 14 work. Mitigation: defer bundle config until Phase 14.

## What Stays Reversible

- The frontend framework (React) can be swapped for Svelte, Solid, or vanilla TS if the UI surface ever outgrows React's strengths. The Tauri IPC layer is framework-agnostic.
- The audio engine can absorb additional crates or move parts to C++ via FFI when Phase 11 evidence demands.
- If Phase 5 real-time targets fail under Tauri+Rust, JUCE/native (alternative A) is the documented fallback. The Phase 0 scaffold does not foreclose this; the typed command surface (Phase 1) defines product concepts independent of the implementation language.

## Consequences

- Phase 0 scaffold uses React + TypeScript + Vite + Tauri 2 + Rust.
- Phase 1 defines typed commands in Rust; the frontend never speaks raw shell.
- Phase 3 wires `cpal` and `symphonia` into a real Rust audio thread.
- Phase 5 verifies real-time targets and records the verdict in ADR 0002.
- Python is not on the realtime path; if used at all, it lives in a separate offline R&D directory.
