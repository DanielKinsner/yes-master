# Standard View Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Ship "Standard" — the default desktop view: a stripped, opinionated UI (4 reference-tuned style tiles + Intensity + Loudness) riding the validated adaptive engine, with an asymmetric door into Advanced, plus iPhone parity.

**Architecture:** One settings truth — Standard binds to the same per-track `selectedSettings` and setters that Advanced uses (no new global). A `useViewMode` hook owns `standard | advanced` with a versioned, migration-aware default persisted in `localStorage`. Standard reuses the engine, transport, waveform, and export plumbing wholesale; it adds only friendly mappings, a managed-reset, a fixed-format export wrap, and the Option-B layout. The live "Mastered" audition is made WYSIWYG by forcing the existing `exportLufsPreview` flag on while Standard is active (the DSP audit confirmed the live and export chains are identical when that flag is true).

**Tech Stack:** React 18 + TypeScript (Vite), Vitest (`createRoot` + `act` component tests, pure-helper unit tests), Tauri + Rust (`yes_master_lib`), SwiftUI iPhone app + a Rust FFI bridge crate.

---

## Conventions for this plan

- **Branch:** do this work on a feature branch (yes-master house rule), e.g. `feat/standard-view`. Do NOT commit to `main` directly.
- **Frontend test command:** `npm test` (Vitest, single run). Watch: `npm run test:watch`. Umbrella before pushing: `npm run verify:fast`.
- **Rust bridge test command:** `cd apps/iphone-native/rust; cargo test`. **Gotcha:** close the running desktop app first (it locks `target\debug\yes-master.exe`); if it must stay open, append `--target-dir target\codex-rc`.
- **Commit trailer:** every commit message ends with
  `Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>`
- **Preset kinds** (TS `Preset` union, `src/bindings.ts:22-31`): `"universal" | "clarity" | "tape" | "spatial" | "oomph" | "warmth" | "punch" | "loud" | "custom"`.
- **The reference-4 mapping (single source of truth):** Balanced→`universal`, Bright→`clarity`, Warm→`tape`, Heavy→`oomph`.
- **Loudness mapping:** Low→`-14`, Medium→`-11`, High→`-9` (LUFS).

---

## File Structure

**New pure modules (FE, fully unit-tested):**
- `src/lib/standard-mapping.ts` — the reference-4 + loudness mappings and tile metadata. Owns: `STANDARD_STYLES`, `STANDARD_LOUDNESS`, `styleToPreset`, `presetToStyle`, `loudnessToTarget`, `targetToLoudness`.
- `src/lib/standard-managed.ts` — the §2b managed/non-managed model. Owns: `hasNonManagedEdits`, `resetToStandardManaged`.
- `src/lib/standard-export.ts` — the §6 export behavior. Owns: `standardExportSettings` (pins 44.1k/24-bit/−1 dBTP), `standardExportNotes` (suppress cosmetics, surface integrity note, block on hard-stops).
- `src/lib/view-mode.ts` — the §7 mode + migration. Owns: `ViewMode`, `PersistedViewState`, `resolveInitialViewMode`, `readPersistedViewMode`, `writePersistedViewMode`, `browserViewModeStore`.

**New hook + components:**
- `src/hooks/useViewMode.ts` — wraps `view-mode.ts` into a React hook (persist + migration default + setter).
- `src/components/StandardView.tsx` — Option-B container + `StyleTiles` + `LoudnessSegmented` + `StandardExportButton`. Binds to `tm`.

**New test files:**
- `src/lib/standard-mapping.test.ts`, `src/lib/standard-managed.test.ts`, `src/lib/standard-export.test.ts`, `src/lib/view-mode.test.ts`
- `src/hooks/useViewMode.test.tsx`
- `src/components/StandardView.test.tsx`

**Modified:**
- `src/hooks/useTrackMaster.ts` — add `hadPriorSession`, `resetToStandardManaged`, `exportStandardMaster` (extract a shared `runExport(settings)`); export them.
- `src/hooks/useTrackMaster.integration.test.tsx` — coverage for the three new hook members.
- `src/App.tsx` — `useViewMode` wiring, `TopHeader` Advanced/Back-to-Standard affordance, render Standard vs the existing desk, the return-confirm modal, Album→Advanced forcing, WYSIWYG `exportLufsPreview` drive.
- `src/App.css` — Standard layout (Option B) + tile/segment styles.
- `apps/iphone-native/YESMasterNative/ContentView.swift` — rename `NativeStylePreset` to the reference-4.
- `apps/iphone-native/rust/src/lib.rs` — re-point `native_preset` + update its tests.
- `docs/PRODUCT.md` — canon update for the Standard review ceremony.

---

## Task 1: Reference-4 + loudness mapping module

**Files:**
- Create: `src/lib/standard-mapping.ts`
- Test: `src/lib/standard-mapping.test.ts`

- [ ] **Step 1: Write the failing test**

```typescript
// src/lib/standard-mapping.test.ts
import { describe, expect, it } from "vitest";

import type { Preset } from "../bindings";
import {
  STANDARD_STYLES,
  STANDARD_LOUDNESS,
  styleToPreset,
  presetToStyle,
  loudnessToTarget,
  targetToLoudness,
} from "./standard-mapping";

describe("standard style mapping (the reference-4)", () => {
  it("maps the four styles to their internal presets", () => {
    expect(styleToPreset("balanced")).toEqual<Preset>({ kind: "universal" });
    expect(styleToPreset("bright")).toEqual<Preset>({ kind: "clarity" });
    expect(styleToPreset("warm")).toEqual<Preset>({ kind: "tape" });
    expect(styleToPreset("heavy")).toEqual<Preset>({ kind: "oomph" });
  });

  it("round-trips preset back to style for the reference-4", () => {
    expect(presetToStyle({ kind: "universal" })).toBe("balanced");
    expect(presetToStyle({ kind: "clarity" })).toBe("bright");
    expect(presetToStyle({ kind: "tape" })).toBe("warm");
    expect(presetToStyle({ kind: "oomph" })).toBe("heavy");
  });

  it("returns null for presets outside the Standard set", () => {
    expect(presetToStyle({ kind: "spatial" })).toBeNull();
    expect(presetToStyle({ kind: "punch" })).toBeNull();
    expect(presetToStyle({ kind: "custom", id: "x" })).toBeNull();
  });

  it("exposes exactly four ordered tiles with metadata", () => {
    expect(STANDARD_STYLES.map((s) => s.id)).toEqual([
      "balanced",
      "bright",
      "warm",
      "heavy",
    ]);
    for (const s of STANDARD_STYLES) {
      expect(s.label.length).toBeGreaterThan(0);
      expect(s.subtitle.length).toBeGreaterThan(0);
      expect(s.tone.length).toBeGreaterThan(0);
    }
  });
});

describe("standard loudness mapping", () => {
  it("maps the three loudness steps to LUFS targets", () => {
    expect(loudnessToTarget("low")).toBe(-14);
    expect(loudnessToTarget("medium")).toBe(-11);
    expect(loudnessToTarget("high")).toBe(-9);
  });

  it("matches an effective LUFS back to its step within tolerance", () => {
    expect(targetToLoudness(-14)).toBe("low");
    expect(targetToLoudness(-11)).toBe("medium");
    expect(targetToLoudness(-9)).toBe("high");
    expect(targetToLoudness(-13.999)).toBe("low");
  });

  it("returns null when the target is off-grid or absent", () => {
    expect(targetToLoudness(null)).toBeNull();
    expect(targetToLoudness(-7)).toBeNull();
  });

  it("exposes exactly three ordered loudness steps", () => {
    expect(STANDARD_LOUDNESS.map((l) => l.id)).toEqual([
      "low",
      "medium",
      "high",
    ]);
  });
});
```

- [ ] **Step 2: Run test to verify it fails**

Run: `npm test -- standard-mapping`
Expected: FAIL — `Cannot find module './standard-mapping'`.

- [ ] **Step 3: Write the implementation**

```typescript
// src/lib/standard-mapping.ts
//
// The Standard view's friendly vocabulary <-> the engine's real values.
// This is the ONE frontend source of truth for "what a style means",
// mirroring the iPhone bridge's native_preset() mapping
// (apps/iphone-native/rust/src/lib.rs). Keep the two in lockstep.

import type { Preset } from "../bindings";

export type StandardStyleId = "balanced" | "bright" | "warm" | "heavy";
export type StandardLoudnessId = "low" | "medium" | "high";

/// The four tiles, in display order, with the metadata the UI renders.
/// `tone` reuses the Knob/accent tone vocabulary already in the app.
export const STANDARD_STYLES: ReadonlyArray<{
  id: StandardStyleId;
  label: string;
  subtitle: string;
  preset: Preset;
  tone: string; // accent tone, drives the tile's accent color
}> = [
  { id: "balanced", label: "Balanced", subtitle: "Clean balance", preset: { kind: "universal" }, tone: "blue" },
  { id: "bright", label: "Bright", subtitle: "Air & detail", preset: { kind: "clarity" }, tone: "cyan" },
  { id: "warm", label: "Warm", subtitle: "Glue & body", preset: { kind: "tape" }, tone: "gold" },
  { id: "heavy", label: "Heavy", subtitle: "Sub & weight", preset: { kind: "oomph" }, tone: "purple" },
];

/// The three loudness steps, in display order (matches the iPhone's
/// NativeLoudness: Low/Medium/High -> -14/-11/-9 LUFS).
export const STANDARD_LOUDNESS: ReadonlyArray<{
  id: StandardLoudnessId;
  label: string;
  lufs: number;
}> = [
  { id: "low", label: "Low", lufs: -14 },
  { id: "medium", label: "Medium", lufs: -11 },
  { id: "high", label: "High", lufs: -9 },
];

export function styleToPreset(style: StandardStyleId): Preset {
  const found = STANDARD_STYLES.find((s) => s.id === style);
  // The union is exhaustive over StandardStyleId, so this never throws in
  // practice; the fallback keeps the function total for defensive callers.
  return found ? found.preset : { kind: "universal" };
}

/// Reverse: the engine preset -> the Standard tile that owns it, or null
/// when the current preset isn't one of the reference-4 (e.g. a track
/// returning from Advanced still carrying `spatial`). UI shows no active
/// tile in that case until the user picks one.
export function presetToStyle(preset: Preset): StandardStyleId | null {
  const found = STANDARD_STYLES.find((s) => s.preset.kind === preset.kind);
  return found ? found.id : null;
}

export function loudnessToTarget(loudness: StandardLoudnessId): number {
  const found = STANDARD_LOUDNESS.find((l) => l.id === loudness);
  return found ? found.lufs : -14;
}

/// Reverse: an effective LUFS target -> the matching step, or null when
/// the value is absent or off the three-step grid.
export function targetToLoudness(lufs: number | null): StandardLoudnessId | null {
  if (lufs === null) return null;
  for (const l of STANDARD_LOUDNESS) {
    if (Math.abs(l.lufs - lufs) < 1e-3) return l.id;
  }
  return null;
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `npm test -- standard-mapping`
Expected: PASS (all cases green).

- [ ] **Step 5: Commit**

```bash
git add src/lib/standard-mapping.ts src/lib/standard-mapping.test.ts
git commit -m "feat(standard): reference-4 style + loudness mapping module"
```

---

## Task 2: Managed / non-managed edit model (§2b)

**Files:**
- Create: `src/lib/standard-managed.ts`
- Test: `src/lib/standard-managed.test.ts`

Background (verified shapes): `MasteringSettings` (`src/bindings.ts:205-234`) has 7 EQ bands `eq_sub_db … eq_sparkle_db`, `input_gain_db`, `output_gain_db`, and `advanced` (`src/bindings.ts:35-71`) with `width`, `warmth`, `presence_air` (default `null`), `compression_mode?` (default `"preset"`), `compression_density` + 12 per-band `compression_*` (default `null`), `compression_link_stereo` (default `null`), and `adaptive_strength?` (default `0.5`). `DEFAULT_SETTINGS` lives at `src/hooks/useTrackMaster.ts:43-84`.

**Scope decision (documented deviation from spec §2b table):** the spec's table enumerates EQ / width-warmth-presence / compressor / gains. We additionally treat `advanced.adaptive_strength !== 0.5` as a non-managed edit and normalize it back to `0.5` on reset. Rationale: Standard's stated invariant is "always the validated default engine"; `adaptive_strength` is sound-affecting and Advanced-exposed, so leaving a power-user's `0` would silently change Standard's sound. This is a deliberate superset of the table — flagged for the owner in the handoff.

- [ ] **Step 1: Write the failing test**

```typescript
// src/lib/standard-managed.test.ts
import { describe, expect, it } from "vitest";

import type { MasteringSettings } from "../bindings";
import { hasNonManagedEdits, resetToStandardManaged } from "./standard-managed";

function base(overrides: Partial<MasteringSettings> = {}): MasteringSettings {
  return {
    preset: { kind: "tape" },
    intensity: 0.72,
    eq_sub_db: 0,
    eq_low_db: 0,
    eq_low_mid_db: 0,
    eq_mid_db: 0,
    eq_high_mid_db: 0,
    eq_high_db: 0,
    eq_sparkle_db: 0,
    volume_match: false,
    source_lufs_integrated: null,
    input_gain_db: 0,
    output_gain_db: 0,
    delivery_profile: "custom",
    album: null,
    advanced: {
      lufs_offset_db: -11,
      ceiling_dbtp: -1,
      width: null,
      warmth: null,
      presence_air: null,
      compression_mode: "preset",
      compression_density: null,
      compression_low_threshold_db: null,
      compression_low_ratio: null,
      compression_low_attack_ms: null,
      compression_low_release_ms: null,
      compression_mid_threshold_db: null,
      compression_mid_ratio: null,
      compression_mid_attack_ms: null,
      compression_mid_release_ms: null,
      compression_high_threshold_db: null,
      compression_high_ratio: null,
      compression_high_attack_ms: null,
      compression_high_release_ms: null,
      compression_link_stereo: null,
      bit_depth: 24,
      target_sample_rate: 44_100,
      adaptive_strength: 0.5,
    },
    ...overrides,
  };
}

describe("hasNonManagedEdits", () => {
  it("is false for a clean Standard-shaped settings object", () => {
    expect(hasNonManagedEdits(base())).toBe(false);
  });

  it("ignores managed fields (preset, intensity, loudness, delivery format)", () => {
    expect(
      hasNonManagedEdits(
        base({
          preset: { kind: "oomph" },
          intensity: 0.95,
          advanced: { ...base().advanced, lufs_offset_db: -9, bit_depth: 16, target_sample_rate: 48_000, ceiling_dbtp: -2 },
        }),
      ),
    ).toBe(false);
  });

  it("flags any non-zero EQ band", () => {
    expect(hasNonManagedEdits(base({ eq_low_db: 1.5 }))).toBe(true);
    expect(hasNonManagedEdits(base({ eq_sparkle_db: -0.5 }))).toBe(true);
  });

  it("flags width / warmth / presence_air", () => {
    expect(hasNonManagedEdits(base({ advanced: { ...base().advanced, width: 0.2 } }))).toBe(true);
    expect(hasNonManagedEdits(base({ advanced: { ...base().advanced, warmth: 0.1 } }))).toBe(true);
    expect(hasNonManagedEdits(base({ advanced: { ...base().advanced, presence_air: 0.3 } }))).toBe(true);
  });

  it("flags compressor mode != preset and any density / per-band override", () => {
    expect(hasNonManagedEdits(base({ advanced: { ...base().advanced, compression_mode: "manual" } }))).toBe(true);
    expect(hasNonManagedEdits(base({ advanced: { ...base().advanced, compression_mode: "off" } }))).toBe(true);
    expect(hasNonManagedEdits(base({ advanced: { ...base().advanced, compression_density: 0.4 } }))).toBe(true);
    expect(hasNonManagedEdits(base({ advanced: { ...base().advanced, compression_low_ratio: 3 } }))).toBe(true);
  });

  it("flags input/output gain and a non-default adaptive_strength", () => {
    expect(hasNonManagedEdits(base({ input_gain_db: -1 }))).toBe(true);
    expect(hasNonManagedEdits(base({ output_gain_db: 0.5 }))).toBe(true);
    expect(hasNonManagedEdits(base({ advanced: { ...base().advanced, adaptive_strength: 0 } }))).toBe(true);
  });
});

describe("resetToStandardManaged", () => {
  it("clears all non-managed fields but preserves preset/intensity/loudness/format", () => {
    const dirty = base({
      preset: { kind: "oomph" },
      intensity: 0.83,
      eq_low_db: 3,
      eq_high_db: -2,
      input_gain_db: -1.5,
      output_gain_db: 0.7,
      advanced: {
        ...base().advanced,
        lufs_offset_db: -9,
        bit_depth: 24,
        target_sample_rate: 44_100,
        ceiling_dbtp: -1,
        width: 0.3,
        warmth: 0.2,
        presence_air: 0.1,
        compression_mode: "manual",
        compression_density: 0.6,
        compression_low_ratio: 4,
        compression_link_stereo: false,
        adaptive_strength: 0,
      },
    });
    const clean = resetToStandardManaged(dirty);

    // Managed values preserved:
    expect(clean.preset).toEqual({ kind: "oomph" });
    expect(clean.intensity).toBe(0.83);
    expect(clean.delivery_profile).toBe("custom");
    expect(clean.advanced.lufs_offset_db).toBe(-9);
    expect(clean.advanced.bit_depth).toBe(24);
    expect(clean.advanced.target_sample_rate).toBe(44_100);
    expect(clean.advanced.ceiling_dbtp).toBe(-1);

    // Non-managed cleared:
    expect(clean.eq_low_db).toBe(0);
    expect(clean.eq_high_db).toBe(0);
    expect(clean.input_gain_db).toBe(0);
    expect(clean.output_gain_db).toBe(0);
    expect(clean.advanced.width).toBeNull();
    expect(clean.advanced.warmth).toBeNull();
    expect(clean.advanced.presence_air).toBeNull();
    expect(clean.advanced.compression_mode).toBe("preset");
    expect(clean.advanced.compression_density).toBeNull();
    expect(clean.advanced.compression_low_ratio).toBeNull();
    expect(clean.advanced.compression_link_stereo).toBeNull();
    expect(clean.advanced.adaptive_strength).toBe(0.5);

    // And the result is, by definition, clean:
    expect(hasNonManagedEdits(clean)).toBe(false);
  });

  it("does not mutate its input", () => {
    const input = base({ eq_low_db: 5 });
    resetToStandardManaged(input);
    expect(input.eq_low_db).toBe(5);
  });
});
```

- [ ] **Step 2: Run test to verify it fails**

Run: `npm test -- standard-managed`
Expected: FAIL — module not found.

- [ ] **Step 3: Write the implementation**

```typescript
// src/lib/standard-managed.ts
//
// Standard "owns" {preset, intensity, loudness target, delivery format}.
// Everything else that affects the sound is a "non-managed edit": it can
// only exist because the user went into Advanced. Standard holds a hard
// invariant — its non-managed fields are always at their defaults — which
// these two pure functions detect and enforce (see design spec §2b).

import type { MasteringSettings } from "../bindings";

const STANDARD_ADAPTIVE_STRENGTH = 0.5;

export function hasNonManagedEdits(s: MasteringSettings): boolean {
  if (
    s.eq_sub_db !== 0 ||
    s.eq_low_db !== 0 ||
    s.eq_low_mid_db !== 0 ||
    s.eq_mid_db !== 0 ||
    s.eq_high_mid_db !== 0 ||
    s.eq_high_db !== 0 ||
    s.eq_sparkle_db !== 0
  ) {
    return true;
  }
  if (s.input_gain_db !== 0 || s.output_gain_db !== 0) return true;

  const a = s.advanced;
  if (a.width !== null || a.warmth !== null || a.presence_air !== null) return true;
  if (a.compression_mode !== undefined && a.compression_mode !== "preset") return true;
  if (a.compression_density !== null) return true;
  if (
    a.compression_low_threshold_db !== null ||
    a.compression_low_ratio !== null ||
    a.compression_low_attack_ms !== null ||
    a.compression_low_release_ms !== null ||
    a.compression_mid_threshold_db !== null ||
    a.compression_mid_ratio !== null ||
    a.compression_mid_attack_ms !== null ||
    a.compression_mid_release_ms !== null ||
    a.compression_high_threshold_db !== null ||
    a.compression_high_ratio !== null ||
    a.compression_high_attack_ms !== null ||
    a.compression_high_release_ms !== null
  ) {
    return true;
  }
  if (a.compression_link_stereo !== null) return true;
  // Deliberate superset of the spec table — keep Standard's adaptive
  // behavior at the validated default. See plan Task 2 scope note.
  if ((a.adaptive_strength ?? STANDARD_ADAPTIVE_STRENGTH) !== STANDARD_ADAPTIVE_STRENGTH) {
    return true;
  }
  return false;
}

/// Returns a NEW settings object with every non-managed field reset to its
/// default, preserving preset / intensity / loudness target / delivery
/// format. Broader than `resetToneSettings` (which also resets intensity
/// and only touches EQ) — used by the Advanced->Standard return.
export function resetToStandardManaged(s: MasteringSettings): MasteringSettings {
  return {
    ...s,
    eq_sub_db: 0,
    eq_low_db: 0,
    eq_low_mid_db: 0,
    eq_mid_db: 0,
    eq_high_mid_db: 0,
    eq_high_db: 0,
    eq_sparkle_db: 0,
    input_gain_db: 0,
    output_gain_db: 0,
    advanced: {
      ...s.advanced,
      width: null,
      warmth: null,
      presence_air: null,
      compression_mode: "preset",
      compression_density: null,
      compression_low_threshold_db: null,
      compression_low_ratio: null,
      compression_low_attack_ms: null,
      compression_low_release_ms: null,
      compression_mid_threshold_db: null,
      compression_mid_ratio: null,
      compression_mid_attack_ms: null,
      compression_mid_release_ms: null,
      compression_high_threshold_db: null,
      compression_high_ratio: null,
      compression_high_attack_ms: null,
      compression_high_release_ms: null,
      compression_link_stereo: null,
      adaptive_strength: STANDARD_ADAPTIVE_STRENGTH,
    },
  };
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `npm test -- standard-managed`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add src/lib/standard-managed.ts src/lib/standard-managed.test.ts
git commit -m "feat(standard): managed/non-managed edit model + resetToStandardManaged"
```

---

## Task 3: Fixed-format export wrap + review-note classification (§6)

**Files:**
- Create: `src/lib/standard-export.ts`
- Test: `src/lib/standard-export.test.ts`

Background: `effectiveLoudnessTarget` (`src/lib/effective-settings.ts:44-52`) returns the LUFS the chain will actually apply (profile target when non-Custom, else `advanced.lufs_offset_db`). The iPhone fixed export (`apps/iphone-native/rust/src/lib.rs:176-247`) pins `Custom` / `lufs_offset_db = target` / `ceiling_dbtp = -1` / `bit_depth = 24` / `target_sample_rate = 44_100`. Export checks (`src-tauri/src/exports.rs`) classify: `Critical` = hard-stop (`bit_depth_low`, `sample_rate_mismatch`, `non_finite_metering`); `true_peak_high` = integrity (Warning level); all other Warnings + `export_ok` Info = cosmetic. `QualityCheck` shape on the wire: `{ level: "info" | "warning" | "critical"; code: string; message: string }` (serde rename_all = lowercase of `QualityLevel`).

- [ ] **Step 1: Write the failing test**

```typescript
// src/lib/standard-export.test.ts
import { describe, expect, it } from "vitest";

import type { MasteringSettings, QualityCheck } from "../bindings";
import { standardExportSettings, standardExportNotes } from "./standard-export";

function settings(overrides: Partial<MasteringSettings["advanced"]> = {}): MasteringSettings {
  return {
    preset: { kind: "tape" },
    intensity: 0.6,
    eq_sub_db: 0, eq_low_db: 0, eq_low_mid_db: 0, eq_mid_db: 0,
    eq_high_mid_db: 0, eq_high_db: 0, eq_sparkle_db: 0,
    volume_match: false,
    source_lufs_integrated: null,
    input_gain_db: 0,
    output_gain_db: 0,
    delivery_profile: "streaming-universal",
    album: null,
    advanced: {
      lufs_offset_db: null, ceiling_dbtp: null, width: null, warmth: null,
      presence_air: null, compression_mode: "preset", compression_density: null,
      compression_low_threshold_db: null, compression_low_ratio: null,
      compression_low_attack_ms: null, compression_low_release_ms: null,
      compression_mid_threshold_db: null, compression_mid_ratio: null,
      compression_mid_attack_ms: null, compression_mid_release_ms: null,
      compression_high_threshold_db: null, compression_high_ratio: null,
      compression_high_attack_ms: null, compression_high_release_ms: null,
      compression_link_stereo: null, bit_depth: null, target_sample_rate: null,
      adaptive_strength: 0.5, ...overrides,
    },
  };
}

describe("standardExportSettings", () => {
  it("pins Custom / 44.1k / 24-bit / -1 dBTP and captures the effective loudness", () => {
    // Fresh streaming-universal track: effective target is -14 via the profile.
    const out = standardExportSettings(settings());
    expect(out.delivery_profile).toBe("custom");
    expect(out.advanced.lufs_offset_db).toBe(-14);
    expect(out.advanced.target_sample_rate).toBe(44_100);
    expect(out.advanced.bit_depth).toBe(24);
    expect(out.advanced.ceiling_dbtp).toBe(-1);
  });

  it("keeps an explicit custom loudness target", () => {
    const out = standardExportSettings(
      settings({ }) // start streaming-universal
    );
    expect(out.advanced.lufs_offset_db).toBe(-14);

    const custom = settings();
    custom.delivery_profile = "custom";
    custom.advanced.lufs_offset_db = -9;
    expect(standardExportSettings(custom).advanced.lufs_offset_db).toBe(-9);
  });

  it("does not mutate its input", () => {
    const input = settings();
    standardExportSettings(input);
    expect(input.delivery_profile).toBe("streaming-universal");
  });
});

describe("standardExportNotes", () => {
  const crit = (code: string): QualityCheck => ({ level: "critical", code, message: `${code} msg` });
  const warn = (code: string): QualityCheck => ({ level: "warning", code, message: `${code} msg` });
  const info = (code: string): QualityCheck => ({ level: "info", code, message: `${code} msg` });

  it("is clean when only cosmetic warnings + info are present", () => {
    const notes = standardExportNotes([info("export_ok"), warn("lufs_very_loud"), warn("dynamic_range_low")]);
    expect(notes.blocked).toBe(false);
    expect(notes.blockMessage).toBeUndefined();
    expect(notes.integrityNote).toBeUndefined();
  });

  it("surfaces a tiny integrity note for true-peak, without blocking", () => {
    const notes = standardExportNotes([warn("true_peak_high")]);
    expect(notes.blocked).toBe(false);
    expect(notes.integrityNote).toContain("true_peak_high msg");
  });

  it("blocks on any critical hard-stop", () => {
    const notes = standardExportNotes([warn("lufs_very_loud"), crit("non_finite_metering")]);
    expect(notes.blocked).toBe(true);
    expect(notes.blockMessage).toContain("non_finite_metering msg");
  });
});
```

- [ ] **Step 2: Run test to verify it fails**

Run: `npm test -- standard-export`
Expected: FAIL — module not found.

- [ ] **Step 3: Write the implementation**

```typescript
// src/lib/standard-export.ts
//
// Standard's export is the iPhone's fixed recipe (44.1k / 24-bit / -1 dBTP
// WAV) plus the Standard-chosen loudness, and its review ceremony is
// stripped: no blocking gate, cosmetic warnings suppressed, one tiny
// non-blocking integrity note, hard-stops still surfaced (design spec §6).

import type { MasteringSettings, QualityCheck } from "../bindings";
import { effectiveLoudnessTarget } from "./effective-settings";

/// Wrap the live Standard settings into the known-safe delivery format,
/// preserving the loudness the user (or the default profile) is targeting.
/// Mirrors apps/iphone-native/rust/src/lib.rs::export_settings_for_options.
export function standardExportSettings(s: MasteringSettings): MasteringSettings {
  const target = effectiveLoudnessTarget(s);
  return {
    ...s,
    delivery_profile: "custom",
    advanced: {
      ...s.advanced,
      lufs_offset_db: target,
      ceiling_dbtp: -1,
      bit_depth: 24,
      target_sample_rate: 44_100,
    },
  };
}

export interface StandardExportNotes {
  /// A technical hard-stop was found — present this prominently and do not
  /// celebrate the export.
  blocked: boolean;
  blockMessage?: string;
  /// A genuine integrity issue (true-peak slipped over) — a tiny inline
  /// note alongside success, never a modal.
  integrityNote?: string;
}

const INTEGRITY_CODES = new Set(["true_peak_high"]);

export function standardExportNotes(checks: QualityCheck[]): StandardExportNotes {
  const critical = checks.find((c) => c.level === "critical");
  if (critical) {
    return { blocked: true, blockMessage: critical.message };
  }
  const integrity = checks.find((c) => INTEGRITY_CODES.has(c.code));
  if (integrity) {
    return { blocked: false, integrityNote: integrity.message };
  }
  // Everything else (cosmetic warnings + info) is suppressed in Standard.
  return { blocked: false };
}
```

- [ ] **Step 4: Verify the `QualityCheck` binding shape**

Run: `npm test -- standard-export`
If TypeScript errors on `QualityCheck` not being exported from `../bindings`, open `src/bindings.ts`, confirm the interface name and `level` literal casing (it is generated to match the Rust serde rename — lowercase `"info" | "warning" | "critical"`). Adjust the test's literals to match the actual binding, not the implementation.
Expected after correction: PASS.

- [ ] **Step 5: Commit**

```bash
git add src/lib/standard-export.ts src/lib/standard-export.test.ts
git commit -m "feat(standard): fixed-format export wrap + review-note classification"
```

---

## Task 4: View-mode resolution + persistence (§7)

**Files:**
- Create: `src/lib/view-mode.ts`
- Test: `src/lib/view-mode.test.ts`

Background: persistence house pattern is `src/lib/export-location.ts` — a tiny `getItem/setItem` store interface with a `browser*Store()` that returns `window.localStorage` or `null`. We mirror it. The migration rule (§7): flag absent + prior session → Advanced; flag absent + no prior session → Standard; set the flag either way; thereafter reopen the last-used view.

- [ ] **Step 1: Write the failing test**

```typescript
// src/lib/view-mode.test.ts
import { describe, expect, it } from "vitest";

import {
  resolveInitialViewMode,
  readPersistedViewMode,
  writePersistedViewMode,
  type ViewModeStore,
} from "./view-mode";

function memStore(initial: Record<string, string> = {}): ViewModeStore {
  const map = new Map(Object.entries(initial));
  return {
    getItem: (k) => map.get(k) ?? null,
    setItem: (k, v) => { map.set(k, v); },
  };
}

describe("resolveInitialViewMode (migration-aware default)", () => {
  it("new user (not migrated, no prior session) -> standard, and marks migrated", () => {
    const { view, persist } = resolveInitialViewMode(null, false);
    expect(view).toBe("standard");
    expect(persist).toEqual({ migrated: true, lastView: "standard" });
  });

  it("returning user (not migrated, prior session) -> advanced, and marks migrated", () => {
    const { view, persist } = resolveInitialViewMode(null, true);
    expect(view).toBe("advanced");
    expect(persist).toEqual({ migrated: true, lastView: "advanced" });
  });

  it("once migrated, reopens the last-used view regardless of prior session", () => {
    expect(resolveInitialViewMode({ migrated: true, lastView: "standard" }, true).view).toBe("standard");
    expect(resolveInitialViewMode({ migrated: true, lastView: "advanced" }, false).view).toBe("advanced");
  });
});

describe("view-mode persistence", () => {
  it("round-trips through the store", () => {
    const store = memStore();
    writePersistedViewMode(store, { migrated: true, lastView: "advanced" });
    expect(readPersistedViewMode(store)).toEqual({ migrated: true, lastView: "advanced" });
  });

  it("returns null for absent or malformed data", () => {
    expect(readPersistedViewMode(memStore())).toBeNull();
    expect(readPersistedViewMode(memStore({ "yes-master:view-mode": "not json" }))).toBeNull();
    expect(readPersistedViewMode(null)).toBeNull();
  });
});
```

- [ ] **Step 2: Run test to verify it fails**

Run: `npm test -- view-mode`
Expected: FAIL — module not found.

- [ ] **Step 3: Write the implementation**

```typescript
// src/lib/view-mode.ts
//
// Standard vs Advanced, with a versioned, migration-aware default
// persisted in localStorage (design spec §7). Mirrors the tiny
// store-interface pattern in export-location.ts so it stays testable
// without a real DOM.

export type ViewMode = "standard" | "advanced";

export interface PersistedViewState {
  migrated: boolean;
  lastView: ViewMode;
}

export interface ViewModeStore {
  getItem(key: string): string | null;
  setItem(key: string, value: string): void;
}

const KEY = "yes-master:view-mode";

export function browserViewModeStore(): ViewModeStore | null {
  try {
    if (typeof window !== "undefined" && window.localStorage) {
      return window.localStorage;
    }
  } catch {
    /* localStorage can throw in locked-down webviews; treat as absent */
  }
  return null;
}

export function readPersistedViewMode(store: ViewModeStore | null): PersistedViewState | null {
  if (!store) return null;
  const raw = store.getItem(KEY);
  if (!raw) return null;
  try {
    const parsed = JSON.parse(raw) as Partial<PersistedViewState>;
    if (
      typeof parsed.migrated === "boolean" &&
      (parsed.lastView === "standard" || parsed.lastView === "advanced")
    ) {
      return { migrated: parsed.migrated, lastView: parsed.lastView };
    }
  } catch {
    /* malformed — treat as absent */
  }
  return null;
}

export function writePersistedViewMode(store: ViewModeStore | null, state: PersistedViewState): void {
  if (!store) return;
  try {
    store.setItem(KEY, JSON.stringify(state));
  } catch {
    /* best-effort; a failed write just means we re-resolve next launch */
  }
}

/// Decide the initial view. When already migrated, reopen the last-used
/// view. Otherwise this is the first launch after the update: returning
/// users (a prior session exists) stay in Advanced; brand-new users land
/// in Standard. Either way we return the `persist` state to write back.
export function resolveInitialViewMode(
  persisted: PersistedViewState | null,
  hadPriorSession: boolean,
): { view: ViewMode; persist: PersistedViewState } {
  if (persisted && persisted.migrated) {
    return { view: persisted.lastView, persist: persisted };
  }
  const view: ViewMode = hadPriorSession ? "advanced" : "standard";
  return { view, persist: { migrated: true, lastView: view } };
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `npm test -- view-mode`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add src/lib/view-mode.ts src/lib/view-mode.test.ts
git commit -m "feat(standard): view-mode resolution + versioned migration persistence"
```

---

## Task 5: `useTrackMaster` — expose `hadPriorSession`, `resetToStandardManaged`, `exportStandardMaster`

**Files:**
- Modify: `src/hooks/useTrackMaster.ts` (session-load effect ~509-571; refs ~309-316; `exportMaster` 1326-1392; return object 1965-2049)
- Test: `src/hooks/useTrackMaster.integration.test.tsx` (append)

- [ ] **Step 1: Write the failing tests (append to the integration suite)**

Add inside the existing `describe("useTrackMaster integration dispatches", ...)` block in `src/hooks/useTrackMaster.integration.test.tsx`, following the established `renderHookHarness()` / `waitFor()` / `act()` style:

```typescript
  it("exposes hadPriorSession=false when there is no restorable session", async () => {
    mocks.api.loadRecentSession.mockResolvedValue(null);
    const harness = await renderHookHarness();
    await waitFor(() => {
      expect(harness.current().hadPriorSession).toBe(false);
    });
    await act(async () => harness.root.unmount());
  });

  it("resetToStandardManaged clears manual EQ but keeps preset/intensity", async () => {
    const track = makeTrack("reset-managed-1", "C:/audio/r.wav");
    mocks.api.importTracks.mockResolvedValue([track]);
    const harness = await renderHookHarness();
    await act(async () => { await harness.current().importFiles([track.path]); });
    await waitFor(() => { expect(harness.current().selectedTrackId).toBe(track.id); });

    await act(async () => {
      harness.current().setPreset({ kind: "oomph" });
      harness.current().setIntensity(0.8);
      harness.current().setEqBand("low", 4);
    });
    await waitFor(() => { expect(harness.current().selectedSettings.eq_low_db).toBe(4); });

    await act(async () => { harness.current().resetToStandardManaged(); });
    await waitFor(() => {
      const s = harness.current().selectedSettings;
      expect(s.eq_low_db).toBe(0);
      expect(s.preset).toEqual({ kind: "oomph" });
      expect(s.intensity).toBe(0.8);
    });
    await act(async () => harness.root.unmount());
  });

  it("exportStandardMaster renders with the fixed 44.1k/24-bit/-1 dBTP wrap", async () => {
    const track = makeTrack("std-export-1", "C:/audio/e.wav");
    mocks.api.importTracks.mockResolvedValue([track]);
    mocks.save.mockResolvedValue("C:/out/e.wav");
    mocks.api.renderTrackMaster.mockResolvedValue({
      output_paths: ["C:/out/e.wav"],
      measurements: null,
    });
    mocks.api.runExportChecks.mockResolvedValue([]);
    const harness = await renderHookHarness();
    await act(async () => { await harness.current().importFiles([track.path]); });
    await waitFor(() => { expect(harness.current().selectedTrackId).toBe(track.id); });

    await act(async () => { await harness.current().exportStandardMaster(); });
    await waitFor(() => { expect(mocks.api.renderTrackMaster).toHaveBeenCalled(); });

    const sent = mocks.api.renderTrackMaster.mock.calls[0][2] as { delivery_profile: string; advanced: { target_sample_rate: number; bit_depth: number; ceiling_dbtp: number } };
    expect(sent.delivery_profile).toBe("custom");
    expect(sent.advanced.target_sample_rate).toBe(44_100);
    expect(sent.advanced.bit_depth).toBe(24);
    expect(sent.advanced.ceiling_dbtp).toBe(-1);
    await act(async () => harness.root.unmount());
  });
```

If `mocks.save` / `mocks.api.runExportChecks` / `makeTrack` aren't already present in the file, reuse the existing helpers/mocks (the suite already mocks `renderTrackMaster` and `save` per the recon; confirm names and match them).

- [ ] **Step 2: Run tests to verify they fail**

Run: `npm test -- useTrackMaster.integration`
Expected: FAIL — `hadPriorSession`, `resetToStandardManaged`, `exportStandardMaster` are `undefined`.

- [ ] **Step 3a: Add the imports**

At the top of `src/hooks/useTrackMaster.ts`, alongside the existing `resetToneSettings` import (line ~23), add:

```typescript
import { resetToStandardManaged as resetToStandardManagedSettings } from "../lib/standard-managed";
import { standardExportSettings } from "../lib/standard-export";
```

- [ ] **Step 3b: Add the `hadPriorSession` state**

Next to `const [sessionLoaded, setSessionLoaded] = useState(false);` (line ~246), add:

```typescript
  const [hadPriorSession, setHadPriorSession] = useState<boolean | null>(null);
```

- [ ] **Step 3c: Set `hadPriorSession` in every branch of the session-load effect**

In the effect at `src/hooks/useTrackMaster.ts:509-571`:

In the early-return branch (currently lines 515-518):
```typescript
        if (cancelled || !session || session.schema_version !== 1) {
          setHadPriorSession(false);
          setSessionLoaded(true);
          return;
        }
```

After `const restoredTracks = session.tracks ?? [];` (line ~519), add:
```typescript
        setHadPriorSession(restoredTracks.length > 0);
```

In the `.catch(...)` branch (currently lines 564-567):
```typescript
      .catch((err) => {
        console.warn("Session load failed", err);
        setHadPriorSession(false);
        setSessionLoaded(true);
      });
```

- [ ] **Step 3d: Add `resetToStandardManaged`**

Next to `resetToneControls` (`src/hooks/useTrackMaster.ts:1104-1107`), add:

```typescript
  const resetToStandardManaged = useCallback(() => {
    if (!selectedTrackId) return;
    updateSettings(selectedTrackId, (prev) => resetToStandardManagedSettings(prev));
  }, [selectedTrackId, updateSettings]);
```

- [ ] **Step 3e: Extract `runExport(settings)` and add `exportStandardMaster`**

Refactor `exportMaster` (`src/hooks/useTrackMaster.ts:1326-1392`) so the body becomes a shared `runExport` that takes the settings to render, and `exportMaster` / `exportStandardMaster` are thin callers. Replace the whole `const exportMaster = useCallback(...)` block with:

```typescript
  const runExport = useCallback(
    async (exportSettings: MasteringSettings) => {
      if (!selectedTrackId || !selectedAnalysis) return;
      setError(null);
      try {
        if (!selectedTrack) return;
        const store = browserExportLocationStore();
        const chosenPath = await save({
          defaultPath: defaultExportPath(
            store,
            "track",
            suggestedMasterFilename(selectedTrack),
          ),
          filters: [{ name: "WAV audio", extensions: ["wav"] }],
        });
        if (!chosenPath) return;
        const chosenOutputPath = ensureWavExtension(chosenPath);
        rememberExportDirectory(store, "track", chosenOutputPath);
        setIsExporting(true);
        const job = await api.renderTrackMaster(
          selectedTrackId,
          selectedTrack.path,
          exportSettings,
          chosenOutputPath,
        );
        const outputPath = job.output_paths[0] ?? "";
        const m = job.measurements ?? null;
        const report: ExportReport = {
          track_id: selectedTrackId,
          output_path: outputPath,
          measured_lufs: m?.lufs_integrated ?? selectedAnalysis.lufs_integrated,
          measured_true_peak_dbtp:
            m?.true_peak_dbtp ?? selectedAnalysis.true_peak_dbtp,
          measured_dynamic_range_lu:
            m?.dynamic_range_lu ?? selectedAnalysis.dynamic_range_lu,
          source_format: selectedTrack?.source_format ?? "unknown",
          destination_format: "wav",
          sample_rate: m?.sample_rate ?? 44_100,
          bit_depth: m?.bit_depth ?? exportSettings.advanced.bit_depth ?? 24,
          effective_adaptive_strength: m?.effective_adaptive_strength ?? 0,
          source_profile_digest: m?.source_profile_digest ?? null,
          confidence_digest: m?.confidence_digest ?? null,
          checks: [],
        };
        const checks = await api.runExportChecks(report, selectedAnalysis, exportSettings);
        setLastExportReceipt({
          trackId: selectedTrackId,
          outputPath,
          checks,
          job,
          kind: "track",
        });
      } catch (err) {
        setError(messageOf(err));
      } finally {
        setIsExporting(false);
      }
    },
    [selectedTrackId, selectedAnalysis, selectedTrack],
  );

  const exportMaster = useCallback(
    () => runExport(selectedSettings),
    [runExport, selectedSettings],
  );

  const exportStandardMaster = useCallback(
    () => runExport(standardExportSettings(selectedSettings)),
    [runExport, selectedSettings],
  );
```

(The render/checks logic is byte-for-byte the prior `exportMaster`, with `selectedSettings` parameterized as `exportSettings`.)

- [ ] **Step 3f: Export the three new members**

In the return object (`src/hooks/useTrackMaster.ts:1965-2049`), add `hadPriorSession,` near the other status fields, `resetToStandardManaged,` next to `resetToneControls,`, and `exportStandardMaster,` next to `exportMaster,`.

- [ ] **Step 4: Run tests to verify they pass**

Run: `npm test -- useTrackMaster.integration`
Expected: PASS. Then run the full suite to catch regressions: `npm test`.
Expected: all green (the existing `exportMaster` tests still pass — behavior is unchanged).

- [ ] **Step 5: Commit**

```bash
git add src/hooks/useTrackMaster.ts src/hooks/useTrackMaster.integration.test.tsx
git commit -m "feat(standard): hook exposes hadPriorSession, resetToStandardManaged, exportStandardMaster"
```

---

## Task 6: `useViewMode` hook

**Files:**
- Create: `src/hooks/useViewMode.ts`
- Test: `src/hooks/useViewMode.test.tsx`

The hook resolves the initial view from `localStorage` + the `hadPriorSession` signal, then lets the caller switch. When already migrated it resolves synchronously on mount; otherwise it waits for `hadPriorSession` to become non-null (the one-time migration). `view` is `null` until resolved so `App` can avoid flashing the wrong mode.

- [ ] **Step 1: Write the failing test**

```typescript
// src/hooks/useViewMode.test.tsx
import { act } from "react";
import { createRoot, type Root } from "react-dom/client";
import { afterEach, beforeEach, describe, expect, it } from "vitest";

import { useViewMode } from "./useViewMode";

(globalThis as unknown as { IS_REACT_ACT_ENVIRONMENT: boolean }).IS_REACT_ACT_ENVIRONMENT = true;

function Harness({ hadPriorSession, onRender }: { hadPriorSession: boolean | null; onRender: (v: ReturnType<typeof useViewMode>) => void }) {
  onRender(useViewMode(hadPriorSession));
  return null;
}

async function renderWith(hadPriorSession: boolean | null): Promise<{ current: () => ReturnType<typeof useViewMode>; root: Root; rerender: (next: boolean | null) => Promise<void> }> {
  const container = document.createElement("div");
  document.body.appendChild(container);
  let current: ReturnType<typeof useViewMode> | null = null;
  const root = createRoot(container);
  const onRender = (v: ReturnType<typeof useViewMode>) => { current = v; };
  await act(async () => { root.render(<Harness hadPriorSession={hadPriorSession} onRender={onRender} />); });
  return {
    current: () => { if (!current) throw new Error("not rendered"); return current; },
    root,
    rerender: async (next) => { await act(async () => { root.render(<Harness hadPriorSession={next} onRender={onRender} />); }); },
  };
}

beforeEach(() => { localStorage.clear(); });
afterEach(() => { document.body.innerHTML = ""; });

describe("useViewMode", () => {
  it("new user (no flag, no prior session) resolves to standard and persists migration", async () => {
    const h = await renderWith(false);
    expect(h.current().view).toBe("standard");
    expect(JSON.parse(localStorage.getItem("yes-master:view-mode")!)).toEqual({ migrated: true, lastView: "standard" });
    await act(async () => h.root.unmount());
  });

  it("returning user (no flag, prior session) resolves to advanced", async () => {
    const h = await renderWith(true);
    expect(h.current().view).toBe("advanced");
    await act(async () => h.root.unmount());
  });

  it("holds view=null until hadPriorSession is known, then resolves", async () => {
    const h = await renderWith(null);
    expect(h.current().view).toBeNull();
    await h.rerender(true);
    expect(h.current().view).toBe("advanced");
    await act(async () => h.root.unmount());
  });

  it("once migrated, reopens last-used view immediately and ignores prior session", async () => {
    localStorage.setItem("yes-master:view-mode", JSON.stringify({ migrated: true, lastView: "standard" }));
    const h = await renderWith(true);
    expect(h.current().view).toBe("standard");
    await act(async () => h.root.unmount());
  });

  it("setView persists the new view", async () => {
    const h = await renderWith(false);
    await act(async () => { h.current().setView("advanced"); });
    expect(h.current().view).toBe("advanced");
    expect(JSON.parse(localStorage.getItem("yes-master:view-mode")!)).toEqual({ migrated: true, lastView: "advanced" });
    await act(async () => h.root.unmount());
  });
});
```

- [ ] **Step 2: Run test to verify it fails**

Run: `npm test -- useViewMode`
Expected: FAIL — module not found.

- [ ] **Step 3: Write the implementation**

```typescript
// src/hooks/useViewMode.ts
import { useCallback, useEffect, useRef, useState } from "react";

import {
  browserViewModeStore,
  readPersistedViewMode,
  resolveInitialViewMode,
  writePersistedViewMode,
  type ViewMode,
} from "../lib/view-mode";

/// Owns the Standard/Advanced view with a migration-aware default.
/// `hadPriorSession` comes from useTrackMaster's session load:
///   null  = not yet known (still loading)
///   true  = a prior session with tracks was restored (returning user)
///   false = no restorable session (new user)
/// `view` stays null until resolvable, so the caller can avoid flashing.
export function useViewMode(hadPriorSession: boolean | null): {
  view: ViewMode | null;
  setView: (next: ViewMode) => void;
} {
  const storeRef = useRef(browserViewModeStore());
  const resolvedRef = useRef(false);
  const [view, setViewState] = useState<ViewMode | null>(() => {
    // If we've already migrated, resolve synchronously on first render so
    // the steady-state case never flashes.
    const persisted = readPersistedViewMode(storeRef.current);
    if (persisted && persisted.migrated) {
      resolvedRef.current = true;
      return persisted.lastView;
    }
    return null;
  });

  useEffect(() => {
    if (resolvedRef.current) return;
    if (hadPriorSession === null) return; // wait for the signal
    const persisted = readPersistedViewMode(storeRef.current);
    const { view: resolved, persist } = resolveInitialViewMode(persisted, hadPriorSession);
    resolvedRef.current = true;
    writePersistedViewMode(storeRef.current, persist);
    setViewState(resolved);
  }, [hadPriorSession]);

  const setView = useCallback((next: ViewMode) => {
    resolvedRef.current = true;
    writePersistedViewMode(storeRef.current, { migrated: true, lastView: next });
    setViewState(next);
  }, []);

  return { view, setView };
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `npm test -- useViewMode`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add src/hooks/useViewMode.ts src/hooks/useViewMode.test.tsx
git commit -m "feat(standard): useViewMode hook with migration-aware default"
```

---

## Task 7: `StyleTiles` component

**Files:**
- Create part of: `src/components/StandardView.tsx` (start the file with `StyleTiles`)
- Test: `src/components/StandardView.test.tsx` (start the file)

- [ ] **Step 1: Write the failing test**

```typescript
// src/components/StandardView.test.tsx
import { act } from "react";
import { createRoot } from "react-dom/client";
import { afterEach, describe, expect, it, vi } from "vitest";

import { StyleTiles } from "./StandardView";

(globalThis as unknown as { IS_REACT_ACT_ENVIRONMENT: boolean }).IS_REACT_ACT_ENVIRONMENT = true;

afterEach(() => { document.body.innerHTML = ""; });

async function render(node: React.ReactElement) {
  const container = document.createElement("div");
  document.body.appendChild(container);
  const root = createRoot(container);
  await act(async () => { root.render(node); });
  return { container, root };
}

describe("StyleTiles", () => {
  it("renders the four reference-4 tiles with labels", async () => {
    const { container, root } = await render(
      <StyleTiles preset={{ kind: "universal" }} onSelect={() => {}} />,
    );
    const text = container.textContent ?? "";
    for (const label of ["Balanced", "Bright", "Warm", "Heavy"]) {
      expect(text).toContain(label);
    }
    await act(async () => root.unmount());
  });

  it("marks the active tile from the current preset", async () => {
    const { container, root } = await render(
      <StyleTiles preset={{ kind: "tape" }} onSelect={() => {}} />,
    );
    const active = container.querySelector(".std-tile.is-active");
    expect(active?.textContent).toContain("Warm");
    await act(async () => root.unmount());
  });

  it("calls onSelect with the mapped preset when a tile is clicked", async () => {
    const onSelect = vi.fn();
    const { container, root } = await render(
      <StyleTiles preset={{ kind: "universal" }} onSelect={onSelect} />,
    );
    const tiles = Array.from(container.querySelectorAll<HTMLButtonElement>(".std-tile"));
    const heavy = tiles.find((t) => t.textContent?.includes("Heavy"))!;
    await act(async () => { heavy.click(); });
    expect(onSelect).toHaveBeenCalledWith({ kind: "oomph" });
    await act(async () => root.unmount());
  });
});
```

- [ ] **Step 2: Run test to verify it fails**

Run: `npm test -- StandardView`
Expected: FAIL — module not found.

- [ ] **Step 3: Start `src/components/StandardView.tsx` with `StyleTiles`**

```typescript
// src/components/StandardView.tsx
import type { Preset } from "../bindings";
import {
  STANDARD_STYLES,
  presetToStyle,
  styleToPreset,
} from "../lib/standard-mapping";

export function StyleTiles({
  preset,
  onSelect,
}: {
  preset: Preset;
  onSelect: (preset: Preset) => void;
}) {
  const activeStyle = presetToStyle(preset);
  return (
    <div className="std-tiles" role="group" aria-label="Style">
      {STANDARD_STYLES.map((s) => (
        <button
          key={s.id}
          type="button"
          className={"std-tile" + (s.id === activeStyle ? " is-active" : "")}
          data-tone={s.tone}
          aria-pressed={s.id === activeStyle}
          onClick={() => onSelect(styleToPreset(s.id))}
        >
          <span className="std-tile-label">{s.label}</span>
          <span className="std-tile-subtitle">{s.subtitle}</span>
        </button>
      ))}
    </div>
  );
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `npm test -- StandardView`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add src/components/StandardView.tsx src/components/StandardView.test.tsx
git commit -m "feat(standard): StyleTiles component"
```

---

## Task 8: `LoudnessSegmented` component

**Files:**
- Modify: `src/components/StandardView.tsx` (add `LoudnessSegmented`)
- Test: `src/components/StandardView.test.tsx` (add a `LoudnessSegmented` describe block)

- [ ] **Step 1: Write the failing test (append)**

```typescript
import { LoudnessSegmented } from "./StandardView";

describe("LoudnessSegmented", () => {
  it("renders Low/Medium/High and marks the active step from the LUFS target", async () => {
    const { container, root } = await render(
      <LoudnessSegmented targetLufs={-11} onSelect={() => {}} />,
    );
    const text = container.textContent ?? "";
    for (const label of ["Low", "Medium", "High"]) expect(text).toContain(label);
    const active = container.querySelector(".std-seg-option.is-active");
    expect(active?.textContent).toContain("Medium");
    await act(async () => root.unmount());
  });

  it("calls onSelect with the mapped LUFS target", async () => {
    const onSelect = vi.fn();
    const { container, root } = await render(
      <LoudnessSegmented targetLufs={-14} onSelect={onSelect} />,
    );
    const opts = Array.from(container.querySelectorAll<HTMLButtonElement>(".std-seg-option"));
    const high = opts.find((o) => o.textContent?.includes("High"))!;
    await act(async () => { high.click(); });
    expect(onSelect).toHaveBeenCalledWith(-9);
    await act(async () => root.unmount());
  });
});
```

- [ ] **Step 2: Run test to verify it fails**

Run: `npm test -- StandardView`
Expected: FAIL — `LoudnessSegmented` is not exported.

- [ ] **Step 3: Add `LoudnessSegmented` to `src/components/StandardView.tsx`**

Add the import for the loudness helpers and the component:

```typescript
import {
  STANDARD_LOUDNESS,
  STANDARD_STYLES,
  loudnessToTarget,
  presetToStyle,
  styleToPreset,
  targetToLoudness,
} from "../lib/standard-mapping";
```

(replace the earlier `standard-mapping` import line with this expanded one), then:

```typescript
export function LoudnessSegmented({
  targetLufs,
  onSelect,
}: {
  targetLufs: number | null;
  onSelect: (targetLufs: number) => void;
}) {
  const active = targetToLoudness(targetLufs);
  return (
    <div className="std-seg" role="group" aria-label="Loudness">
      {STANDARD_LOUDNESS.map((l) => (
        <button
          key={l.id}
          type="button"
          className={"std-seg-option" + (l.id === active ? " is-active" : "")}
          aria-pressed={l.id === active}
          onClick={() => onSelect(loudnessToTarget(l.id))}
        >
          <span className="std-seg-label">{l.label}</span>
          <span className="std-seg-lufs">{l.lufs} LUFS</span>
        </button>
      ))}
    </div>
  );
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `npm test -- StandardView`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add src/components/StandardView.tsx src/components/StandardView.test.tsx
git commit -m "feat(standard): LoudnessSegmented component"
```

---

## Task 9: `StandardView` container + `StandardExportButton`

**Files:**
- Modify: `src/components/StandardView.tsx` (add `StandardView` + `StandardExportButton`)
- Test: `src/components/StandardView.test.tsx` (add a `StandardView` describe block)

`StandardView` takes the whole `tm` (the `useTrackMaster` return) — the same pattern as `TrackMaster` (`<TrackMaster tm={tm} />`, `App.tsx:115`). It composes the Option-B layout: a left hero (track chip + import, Play, the existing `WaveformView`/`WaveformLoading` surface, Original/Mastered + Volume Match) and a right control column (`StyleTiles`, the Intensity `Knob`, `LoudnessSegmented`, and the `StandardExportButton`). Reused exports from `App.tsx`: `WaveformView` (`App.tsx:1240`), `WaveformLoading` (`App.tsx:1178`). Reused: `Knob` (`src/components/Knob.tsx:80`), `intensityLabel` (`src/components/Knob.tsx:367`).

- [ ] **Step 1: Write the failing test (append)**

```typescript
import { StandardView } from "./StandardView";
import type { useTrackMaster } from "../hooks/useTrackMaster";

type TM = ReturnType<typeof useTrackMaster>;

function fakeSettings() {
  return {
    preset: { kind: "tape" }, intensity: 0.5,
    eq_sub_db: 0, eq_low_db: 0, eq_low_mid_db: 0, eq_mid_db: 0,
    eq_high_mid_db: 0, eq_high_db: 0, eq_sparkle_db: 0,
    volume_match: false, source_lufs_integrated: null,
    input_gain_db: 0, output_gain_db: 0, delivery_profile: "custom",
    album: null,
    advanced: {
      lufs_offset_db: -11, ceiling_dbtp: -1, width: null, warmth: null,
      presence_air: null, compression_mode: "preset", compression_density: null,
      compression_low_threshold_db: null, compression_low_ratio: null,
      compression_low_attack_ms: null, compression_low_release_ms: null,
      compression_mid_threshold_db: null, compression_mid_ratio: null,
      compression_mid_attack_ms: null, compression_mid_release_ms: null,
      compression_high_threshold_db: null, compression_high_ratio: null,
      compression_high_attack_ms: null, compression_high_release_ms: null,
      compression_link_stereo: null, bit_depth: 24, target_sample_rate: 44_100,
      adaptive_strength: 0.5,
    },
  };
}

function fakeTm(overrides: Partial<TM> = {}): TM {
  const noop = () => {};
  return {
    tracks: [{ id: "t1", path: "C:/a.wav", display_name: "Song.wav", source_format: "wav", duration_sec: 100 }],
    selectedTrackId: "t1",
    selectedTrack: { id: "t1", path: "C:/a.wav", display_name: "Song.wav", source_format: "wav", duration_sec: 100 },
    selectedAnalysis: { track_id: "t1", lufs_integrated: -16, true_peak_dbtp: -1.2, dynamic_range_lu: 9 },
    selectedWaveform: undefined,
    selectedSettings: fakeSettings(),
    isAnalyzing: false, isLoadingWaveform: false, analysisProgress: null,
    isExporting: false, isRendering: false, lastExportReceipt: null,
    transport: { isPlaying: false, currentTimeSec: 0, playbackKind: "master", volumeMatch: false },
    selectedRegion: null,
    setPreset: noop, setIntensity: noop, setLoudnessTarget: noop,
    setPlaybackKind: noop, setVolumeMatch: noop, togglePlay: noop, seek: noop,
    setRegion: noop, clearRegion: noop, openImportDialog: noop, selectTrack: noop,
    exportStandardMaster: noop, clearExportReceipt: noop,
    ...overrides,
  } as unknown as TM;
}

describe("StandardView", () => {
  it("renders style tiles, an intensity control, loudness steps, and the Create Master CTA", async () => {
    const { container, root } = await render(<StandardView tm={fakeTm()} />);
    const text = container.textContent ?? "";
    expect(text).toContain("Balanced");
    expect(text).toContain("Low");
    expect(text).toContain("Create Master");
    await act(async () => root.unmount());
  });

  it("routes a style click to setPreset", async () => {
    const setPreset = vi.fn();
    const { container, root } = await render(<StandardView tm={fakeTm({ setPreset })} />);
    const tiles = Array.from(container.querySelectorAll<HTMLButtonElement>(".std-tile"));
    tiles.find((t) => t.textContent?.includes("Bright"))!;
    await act(async () => { tiles.find((t) => t.textContent?.includes("Bright"))!.click(); });
    expect(setPreset).toHaveBeenCalledWith({ kind: "clarity" });
    await act(async () => root.unmount());
  });

  it("Create Master triggers exportStandardMaster", async () => {
    const exportStandardMaster = vi.fn();
    const { container, root } = await render(<StandardView tm={fakeTm({ exportStandardMaster })} />);
    const cta = Array.from(container.querySelectorAll<HTMLButtonElement>("button")).find((b) => b.textContent?.includes("Create Master"))!;
    await act(async () => { cta.click(); });
    expect(exportStandardMaster).toHaveBeenCalled();
    await act(async () => root.unmount());
  });
});
```

- [ ] **Step 2: Run test to verify it fails**

Run: `npm test -- StandardView`
Expected: FAIL — `StandardView` not exported.

- [ ] **Step 3: Add `StandardView` + `StandardExportButton`**

Add imports at the top of `src/components/StandardView.tsx`:

```typescript
import type { useTrackMaster } from "../hooks/useTrackMaster";
import { Knob, intensityLabel } from "./Knob";
import { WaveformLoading, WaveformView } from "../App";
import { effectiveLoudnessTarget } from "../lib/effective-settings";
import { standardExportNotes } from "../lib/standard-export";
```

> Note on imports: `StandardView` imports `WaveformView`/`WaveformLoading` from `../App`, while `App.tsx` will import `StandardView` from `./components/StandardView`. This is a benign one-way usage at runtime (App renders StandardView; StandardView only references App's already-defined function components), but to avoid a static circular-import smell, if the bundler warns, hoist `WaveformView`/`WaveformLoading` into a small `src/components/Waveform.tsx` and re-export from `App.tsx`. Prefer the direct import first; only split if a warning appears.

Then add the components:

```typescript
type TM = ReturnType<typeof useTrackMaster>;

function StandardExportButton({ tm }: { tm: TM }) {
  const notes = tm.lastExportReceipt
    ? standardExportNotes(tm.lastExportReceipt.checks)
    : null;
  return (
    <div className="std-export">
      <button
        type="button"
        className="primary std-create-master"
        disabled={!tm.selectedAnalysis || tm.isExporting || tm.isRendering}
        onClick={() => { void tm.exportStandardMaster(); }}
      >
        {tm.isExporting ? "Creating Master…" : "Create Master"}
      </button>
      {notes?.blocked && (
        <p className="std-export-block" role="alert">{notes.blockMessage}</p>
      )}
      {notes?.integrityNote && (
        <p className="std-export-note">{notes.integrityNote}</p>
      )}
    </div>
  );
}

export function StandardView({ tm }: { tm: TM }) {
  const s = tm.selectedSettings;
  return (
    <div className="standard-view">
      <section className="std-hero">
        <div className="std-hero-head">
          {tm.tracks.length > 1 ? (
            <select
              className="std-track-select"
              aria-label="Track"
              value={tm.selectedTrackId ?? ""}
              onChange={(e) => tm.selectTrack(e.target.value)}
            >
              {tm.tracks.map((t) => (
                <option key={t.id} value={t.id}>{t.display_name}</option>
              ))}
            </select>
          ) : (
            <span className="std-track-chip">{tm.selectedTrack?.display_name ?? "No track"}</span>
          )}
          <button type="button" className="ghost-btn" onClick={tm.openImportDialog}>Import</button>
        </div>

        <button
          type="button"
          className="std-play"
          aria-label={tm.transport.isPlaying ? "Pause" : "Play"}
          onClick={tm.togglePlay}
        >
          {tm.transport.isPlaying ? "❚❚" : "►"}
        </button>

        <div className="std-wave">
          {tm.selectedWaveform ? (
            <WaveformView
              peaks={tm.selectedWaveform}
              isLoading={tm.isLoadingWaveform}
              isAnalyzing={tm.isAnalyzing}
              analysisProgress={tm.analysisProgress}
              currentTimeSec={tm.transport.currentTimeSec}
              durationSec={tm.selectedTrack?.duration_sec ?? 0}
              region={tm.selectedRegion}
              onSeek={tm.seek}
              onSetRegion={tm.setRegion}
              onClearRegion={tm.clearRegion}
            />
          ) : (
            <WaveformLoading
              isAnalyzing={tm.isAnalyzing}
              isLoadingWaveform={tm.isLoadingWaveform}
              analysisProgress={tm.analysisProgress}
            />
          )}
        </div>

        <div className="std-ab">
          <div className="ab-toggle">
            <button className={tm.transport.playbackKind === "source" ? "on" : ""} onClick={() => tm.setPlaybackKind("source")}>Original</button>
            <button className={tm.transport.playbackKind === "master" ? "on" : ""} onClick={() => tm.setPlaybackKind("master")}>Mastered</button>
          </div>
          <button
            type="button"
            className={`toolbar-toggle ${tm.transport.volumeMatch ? "is-on" : ""}`}
            aria-pressed={tm.transport.volumeMatch}
            onClick={() => tm.setVolumeMatch(!tm.transport.volumeMatch)}
          >
            <span className="toolbar-toggle-box" aria-hidden />
            <span>Volume Match</span>
          </button>
        </div>
      </section>

      <section className="std-controls">
        <div className="std-step">
          <span className="std-step-label">1 · Style</span>
          <StyleTiles preset={s.preset} onSelect={tm.setPreset} />
        </div>

        <div className="std-step">
          <span className="std-step-label">2 · Intensity</span>
          <Knob
            label=""
            size="lg"
            value={s.intensity}
            min={0}
            max={1}
            step={0.01}
            defaultValue={0.5}
            format={(v) => `${Math.round(v * 100)}%`}
            caption={intensityLabel(s.intensity)}
            onChange={tm.setIntensity}
            centerValue
          />
        </div>

        <div className="std-step">
          <span className="std-step-label">3 · Loudness</span>
          <LoudnessSegmented targetLufs={effectiveLoudnessTarget(s)} onSelect={tm.setLoudnessTarget} />
        </div>

        <StandardExportButton tm={tm} />
      </section>
    </div>
  );
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `npm test -- StandardView`
Expected: PASS. If a circular-import warning appears, apply the `Waveform.tsx` hoist noted above and re-run.

- [ ] **Step 5: Commit**

```bash
git add src/components/StandardView.tsx src/components/StandardView.test.tsx
git commit -m "feat(standard): StandardView container + Create Master CTA"
```

---

## Task 10: Wire Standard into `App.tsx` (chrome, transition, WYSIWYG, Album guard)

**Files:**
- Modify: `src/App.tsx` (`App` 67-184; `TopHeader` 266-363)
- Test: `src/App.standard-view.test.tsx` (new)

This is the integration task: pick Standard vs the existing desk, add the chrome affordance, run the asymmetric transition (seed-forward / warn-reset-on-return), force the WYSIWYG flag, and keep Album in Advanced.

- [ ] **Step 1: Write the failing test**

```typescript
// src/App.standard-view.test.tsx
import { act } from "react";
import { createRoot } from "react-dom/client";
import { afterEach, describe, expect, it } from "vitest";

import { TopHeader } from "./App";

(globalThis as unknown as { IS_REACT_ACT_ENVIRONMENT: boolean }).IS_REACT_ACT_ENVIRONMENT = true;

afterEach(() => { document.body.innerHTML = ""; });

async function render(node: React.ReactElement) {
  const container = document.createElement("div");
  document.body.appendChild(container);
  const root = createRoot(container);
  await act(async () => { root.render(node); });
  return { container, root };
}

describe("TopHeader Standard/Advanced affordance", () => {
  it("shows an Advanced door when in Standard", async () => {
    const { container, root } = await render(
      <TopHeader
        mode="track" onModeChange={() => {}}
        onSaveProject={() => {}} onOpenProject={() => {}}
        onOpenSettings={() => {}} onOpenHelp={() => {}}
        viewMode="standard" onEnterAdvanced={() => {}} onBackToStandard={() => {}}
      />,
    );
    const btn = Array.from(container.querySelectorAll("button")).find((b) => b.textContent === "Advanced");
    expect(btn).toBeTruthy();
    await act(async () => root.unmount());
  });

  it("shows Back to Standard when in Advanced", async () => {
    const { container, root } = await render(
      <TopHeader
        mode="track" onModeChange={() => {}}
        onSaveProject={() => {}} onOpenProject={() => {}}
        onOpenSettings={() => {}} onOpenHelp={() => {}}
        viewMode="advanced" onEnterAdvanced={() => {}} onBackToStandard={() => {}}
      />,
    );
    const btn = Array.from(container.querySelectorAll("button")).find((b) => b.textContent?.includes("Back to Standard"));
    expect(btn).toBeTruthy();
    await act(async () => root.unmount());
  });
});
```

- [ ] **Step 2: Run test to verify it fails**

Run: `npm test -- App.standard-view`
Expected: FAIL — `TopHeader` doesn't accept `viewMode` / the buttons don't exist.

- [ ] **Step 3a: Extend `TopHeader`**

Replace the `TopHeader` prop type and add the affordance. Update the signature (`src/App.tsx:266-280`) to add the three props:

```typescript
export function TopHeader({
  mode,
  onModeChange,
  onSaveProject,
  onOpenProject,
  onOpenSettings,
  onOpenHelp,
  viewMode,
  onEnterAdvanced,
  onBackToStandard,
}: {
  mode: "track" | "album";
  onModeChange: (mode: "track" | "album") => void;
  onSaveProject: () => void;
  onOpenProject: () => void;
  onOpenSettings: () => void;
  onOpenHelp: () => void;
  viewMode: "standard" | "advanced";
  onEnterAdvanced: () => void;
  onBackToStandard: () => void;
}) {
```

In the `top-header-right` div (after the tabs / before the icon-tiles, `src/App.tsx:310`), add:

```typescript
        {viewMode === "standard" ? (
          <button type="button" className="ghost-btn top-advanced" onClick={onEnterAdvanced}>
            Advanced
          </button>
        ) : (
          <button type="button" className="ghost-btn top-advanced" onClick={onBackToStandard}>
            ‹ Back to Standard
          </button>
        )}
```

- [ ] **Step 3b: Wire `useViewMode` + the transition into `App`**

In `App` (`src/App.tsx:67`), after `const tm = useTrackMaster();` add:

```typescript
  const { view, setView } = useViewMode(tm.hadPriorSession);
  const [returnConfirm, setReturnConfirm] = useState(false);

  // WYSIWYG: the live Mastered audition equals the export only when the
  // loudness landing + limiter are applied in real time. Force that on in
  // Standard (and back to Advanced's responsive default otherwise).
  useEffect(() => {
    if (view === null) return;
    tm.setExportLufsPreview(view === "standard");
  }, [view, tm.setExportLufsPreview]);

  // Album Master is Advanced-only in v1.
  useEffect(() => {
    if (view === "standard" && tm.mode === "album") setView("advanced");
  }, [view, tm.mode, setView]);

  const requestBackToStandard = () => {
    if (hasNonManagedEdits(tm.selectedSettings)) {
      setReturnConfirm(true);
    } else {
      setView("standard");
    }
  };
```

Add the imports at the top of `src/App.tsx`:

```typescript
import { useEffect, useState } from "react"; // extend the existing react import
import { useViewMode } from "./hooks/useViewMode";
import { StandardView } from "./components/StandardView";
import { hasNonManagedEdits } from "./lib/standard-managed";
```

(Merge `useEffect`/`useState` into whatever React import already exists — do not duplicate the import.)

- [ ] **Step 3c: Pass the new props to `TopHeader` and branch the body**

Update the `<TopHeader ... />` render (`src/App.tsx:74-81`) to pass:

```typescript
      <TopHeader
        mode={tm.mode}
        onModeChange={tm.setMode}
        onSaveProject={tm.saveProjectAs}
        onOpenProject={tm.openProjectFromDisk}
        onOpenSettings={() => setChromePanel("settings")}
        onOpenHelp={() => setChromePanel("help")}
        viewMode={view === "advanced" ? "advanced" : "standard"}
        onEnterAdvanced={() => setView("advanced")}
        onBackToStandard={requestBackToStandard}
      />
```

Then branch the `.app` body. Replace the `<div className="app"> … </div>` block (`src/App.tsx:82-180`) so that Standard renders `StandardView` instead of the Sidebar/workspace/RightRail desk, while keeping the shared overlays. Wrap the existing desk (`Sidebar` + `main.workspace` + `RightRail`) in `{view !== "standard" && ( … )}` and add, before the drop-overlay:

```typescript
      {view === "standard" && tm.selectedTrack && <StandardView tm={tm} />}
      {view === "standard" && !tm.selectedTrack && <EmptyState onAdd={tm.openImportDialog} />}
```

Keep `Toast`, `drop-overlay`, `SettingsPanel`, `HelpPanel`, and `BottomStatusBar` rendering in both views. **Suppress the big `ExportReceiptCard` in Standard** (Standard shows its own compact note via `StandardExportButton`): change its guard to `{tm.lastExportReceipt && view !== "standard" && ( … )}`.

- [ ] **Step 3d: Add the return-confirm modal**

Before the closing `</div>` of `.app`, add:

```typescript
      {returnConfirm && (
        <BackToStandardConfirm
          onCancel={() => setReturnConfirm(false)}
          onReset={() => { tm.resetToStandardManaged(); setReturnConfirm(false); setView("standard"); }}
          onSaveAsPreset={(name) => { tm.saveUserPreset(name); tm.resetToStandardManaged(); setReturnConfirm(false); setView("standard"); }}
        />
      )}
```

And add the small component near the other panels in `src/App.tsx`:

```typescript
function BackToStandardConfirm({
  onCancel,
  onReset,
  onSaveAsPreset,
}: {
  onCancel: () => void;
  onReset: () => void;
  onSaveAsPreset: (name: string) => void;
}) {
  const [name, setName] = useState("");
  return (
    <div className="modal-scrim" role="dialog" aria-modal="true" aria-label="Back to Standard">
      <div className="modal-card">
        <h2 className="modal-title">Back to Standard</h2>
        <p className="modal-body">
          Back to Standard resets your manual edits to the preset's clean sound.
          Save them as a preset first?
        </p>
        <div className="modal-save-row">
          <input
            className="modal-input"
            placeholder="Preset name"
            value={name}
            onChange={(e) => setName(e.target.value)}
          />
          <button
            type="button"
            className="primary"
            disabled={name.trim().length === 0}
            onClick={() => onSaveAsPreset(name.trim())}
          >
            Save as preset
          </button>
        </div>
        <div className="modal-actions">
          <button type="button" className="ghost-btn" onClick={onCancel}>Cancel</button>
          <button type="button" className="danger-btn" onClick={onReset}>Reset & continue</button>
        </div>
      </div>
    </div>
  );
}
```

- [ ] **Step 4: Run the test + full suite**

Run: `npm test -- App.standard-view` then `npm test`.
Expected: PASS. Watch for: the existing `App.progress-and-reset.test.tsx` still passes (it imports `Macros`/`WaveformView`/`WaveformLoading` from `./App`, which are unchanged).

- [ ] **Step 5: Commit**

```bash
git add src/App.tsx src/App.standard-view.test.tsx
git commit -m "feat(standard): wire Standard view, asymmetric transition, WYSIWYG + Album guard into App"
```

---

## Task 11: Standard layout styles (Option B)

**Files:**
- Modify: `src/App.css`

CSS is not unit-tested; this step delivers the functional Option-B layout using the existing design tokens (`--bg-*`, `--text-*`, `--accent*` defined at `src/App.css:1-20`). The flagship visual polish pass (motion, hero treatment, micro-interactions per spec §1) is a follow-up using the design skills and `docs/simple-mode-mockup.html` as the reference of record — see the final "Design polish" note.

- [ ] **Step 1: Append the Standard styles to `src/App.css`**

```css
/* ---- Standard view (Option B: desktop reflow) ---- */
.standard-view {
  display: grid;
  grid-template-columns: minmax(0, 1.4fr) minmax(320px, 0.9fr);
  gap: 1.5rem;
  padding: 1.5rem;
  height: 100%;
  background: radial-gradient(120% 100% at 0% 0%, #0c1322 0%, var(--bg-0) 70%);
}
.std-hero { display: flex; flex-direction: column; gap: 1rem; min-width: 0; }
.std-hero-head { display: flex; align-items: center; justify-content: space-between; gap: 0.75rem; }
.std-track-chip, .std-track-select {
  font-size: 0.95rem; color: var(--text-0);
  background: var(--bg-2); border: 1px solid var(--border);
  border-radius: 8px; padding: 0.4rem 0.7rem;
}
.std-play {
  align-self: flex-start; width: 56px; height: 56px; border-radius: 50%;
  border: none; cursor: pointer; font-size: 1.2rem;
  color: #fff; background: linear-gradient(135deg, var(--accent-deep), var(--accent-bright));
}
.std-wave { flex: 1; min-height: 180px; background: var(--bg-1); border: 1px solid var(--border); border-radius: 12px; overflow: hidden; }
.std-ab { display: flex; align-items: center; justify-content: space-between; gap: 1rem; }

.std-controls { display: flex; flex-direction: column; gap: 1.4rem; }
.std-step { display: flex; flex-direction: column; gap: 0.6rem; }
.std-step-label { font-size: 0.8rem; letter-spacing: 0.08em; text-transform: uppercase; color: var(--text-2); }

.std-tiles { display: grid; grid-template-columns: 1fr 1fr; gap: 0.7rem; }
.std-tile {
  display: flex; flex-direction: column; gap: 0.2rem; text-align: left;
  padding: 0.85rem 0.95rem; border-radius: 12px; cursor: pointer;
  background: var(--bg-2); border: 1px solid var(--border); color: var(--text-1);
  transition: border-color 120ms, background 120ms, transform 120ms;
}
.std-tile:hover { transform: translateY(-1px); }
.std-tile.is-active { border-color: var(--accent-bright); background: rgba(111,163,255,0.12); color: var(--text-0); }
.std-tile-label { font-size: 1rem; font-weight: 600; }
.std-tile-subtitle { font-size: 0.8rem; color: var(--text-2); }
.std-tile[data-tone="cyan"].is-active { border-color: #45e0f5; background: rgba(69,224,245,0.12); }
.std-tile[data-tone="gold"].is-active { border-color: var(--accent-warm); background: rgba(255,184,107,0.12); }
.std-tile[data-tone="purple"].is-active { border-color: #a875f5; background: rgba(168,117,245,0.14); }

.std-seg { display: grid; grid-template-columns: repeat(3, 1fr); gap: 0.5rem; }
.std-seg-option {
  display: flex; flex-direction: column; gap: 0.15rem; align-items: center;
  padding: 0.6rem 0.4rem; border-radius: 10px; cursor: pointer;
  background: var(--bg-2); border: 1px solid var(--border); color: var(--text-1);
}
.std-seg-option.is-active { border-color: var(--accent-bright); background: rgba(111,163,255,0.12); color: var(--text-0); }
.std-seg-label { font-weight: 600; }
.std-seg-lufs { font-size: 0.72rem; color: var(--text-2); font-variant-numeric: tabular-nums; }

.std-export { display: flex; flex-direction: column; gap: 0.5rem; margin-top: 0.5rem; }
.std-create-master { padding: 0.9rem 1rem; font-size: 1rem; }
.std-export-block { color: var(--accent-danger); font-size: 0.85rem; margin: 0; }
.std-export-note { color: var(--text-2); font-size: 0.8rem; margin: 0; }

/* Back-to-Standard confirm modal */
.modal-scrim { position: fixed; inset: 0; display: flex; align-items: center; justify-content: center; background: rgba(4,6,12,0.6); z-index: 50; }
.modal-card { width: min(420px, 92vw); background: var(--bg-1); border: 1px solid var(--border-strong); border-radius: 14px; padding: 1.3rem; display: flex; flex-direction: column; gap: 0.9rem; }
.modal-title { margin: 0; font-size: 1.05rem; color: var(--text-0); }
.modal-body { margin: 0; color: var(--text-1); font-size: 0.9rem; }
.modal-save-row { display: flex; gap: 0.5rem; }
.modal-input { flex: 1; background: var(--bg-2); border: 1px solid var(--border); border-radius: 8px; padding: 0.5rem 0.7rem; color: var(--text-0); }
.modal-actions { display: flex; justify-content: flex-end; gap: 0.6rem; }
.danger-btn { background: rgba(255,118,118,0.14); border: 1px solid var(--accent-danger); color: var(--accent-danger); border-radius: 8px; padding: 0.5rem 0.8rem; cursor: pointer; }

@media (max-width: 980px) {
  .standard-view { grid-template-columns: 1fr; }
}
```

- [ ] **Step 2: Manual smoke (no automated test for CSS)**

Run the app (`npm run tauri dev`), import a track, confirm: Standard renders two columns; tiles/loudness highlight the active selection; Play, Original/Mastered, Volume Match, and Create Master are present and clickable. Close the app before any later `cargo` step.

- [ ] **Step 3: Commit**

```bash
git add src/App.css
git commit -m "feat(standard): Option-B layout styles for the Standard view"
```

---

## Task 12: iPhone parity — rename `NativeStylePreset` + re-point the Rust mapping (§8)

**Files:**
- Modify: `apps/iphone-native/YESMasterNative/ContentView.swift:3-62`
- Modify: `apps/iphone-native/rust/src/lib.rs:249-262` (mapping) and `:332-345` (test)

- [ ] **Step 1: Update the Rust mapping test first (TDD)**

In `apps/iphone-native/rust/src/lib.rs`, replace the body of `native_options_map_to_shared_preset_and_intensity` (`:332-345`) with the reference-4 expectations:

```rust
    #[test]
    fn native_options_map_to_shared_preset_and_intensity() {
        let balanced = export_settings_for_options(Some("balanced"), 0.5, -11.0);
        assert_eq!(balanced.preset, Preset::Universal);

        let bright = export_settings_for_options(Some("bright"), 0.5, -11.0);
        assert_eq!(bright.preset, Preset::Clarity);

        let warm = export_settings_for_options(Some("warm"), 0.8, -11.0);
        assert_eq!(warm.preset, Preset::Tape);
        assert_eq!(warm.intensity, 0.8);

        let heavy = export_settings_for_options(Some("heavy"), 1.0, -11.0);
        assert_eq!(heavy.preset, Preset::Oomph);

        // Back-compat aliases still resolve.
        assert_eq!(export_settings_for_options(Some("open"), 0.5, -11.0).preset, Preset::Clarity);
        assert_eq!(export_settings_for_options(Some("warmth"), 0.5, -11.0).preset, Preset::Warmth);
        assert_eq!(export_settings_for_options(Some("punch"), 0.5, -11.0).preset, Preset::Punch);

        let fallback = export_settings_for_options(Some("unknown"), -1.0, -11.0);
        assert_eq!(fallback.preset, Preset::Universal);
        assert_eq!(fallback.intensity, 0.0);
    }
```

- [ ] **Step 2: Run the bridge tests to verify failure**

Run: `cd apps/iphone-native/rust; cargo test native_options_map_to_shared_preset_and_intensity`
Expected: FAIL — `bright`→`Universal` (fallback) and `warm`→`Warmth`, not yet `Clarity`/`Tape`.

- [ ] **Step 3: Re-point `native_preset`**

Replace `native_preset` (`apps/iphone-native/rust/src/lib.rs:249-262`):

```rust
fn native_preset(preset: Option<&str>) -> Preset {
    match preset
        .unwrap_or("balanced")
        .trim()
        .to_ascii_lowercase()
        .as_str()
    {
        "balanced" | "universal" => Preset::Universal,
        "bright" | "clarity" | "open" => Preset::Clarity,
        "warm" | "tape" => Preset::Tape,
        "heavy" | "oomph" => Preset::Oomph,
        // Back-compat aliases for older builds / saved payloads.
        "warmth" => Preset::Warmth,
        "punch" => Preset::Punch,
        _ => Preset::Universal,
    }
}
```

- [ ] **Step 4: Re-run the bridge tests**

Run: `cd apps/iphone-native/rust; cargo test`
Expected: PASS — including the unchanged `fixed_export_settings_match_simple_iphone_target` and `native_options_map_loudness_target` (they use `"balanced"`, still Universal). If other tests reference the old `"warm"→Warmth` expectation, update them to `Tape`.

- [ ] **Step 5: Rename the Swift `NativeStylePreset`**

Replace `NativeStylePreset` (`apps/iphone-native/YESMasterNative/ContentView.swift:3-62`) with the reference-4 (matching desktop's labels/subtitles/tones; `bridgeIdentifier` strings drive the Rust mapping above):

```swift
enum NativeStylePreset: String, CaseIterable, Identifiable {
    case balanced = "Balanced"
    case bright = "Bright"
    case warm = "Warm"
    case heavy = "Heavy"

    var id: String { rawValue }

    var subtitle: String {
        switch self {
        case .balanced: "Clean balance"
        case .bright: "Air & detail"
        case .warm: "Glue & body"
        case .heavy: "Sub & weight"
        }
    }

    var symbol: String {
        switch self {
        case .balanced: "circle.hexagongrid.fill"
        case .bright: "snowflake"
        case .warm: "flame.fill"
        case .heavy: "bolt.fill"
        }
    }

    var accent: Color {
        switch self {
        case .balanced: Color(red: 0.25, green: 0.57, blue: 1.0)   // blue
        case .bright: Color(red: 0.27, green: 0.88, blue: 0.96)    // cyan
        case .warm: Color(red: 1.0, green: 0.72, blue: 0.30)       // amber
        case .heavy: Color(red: 0.66, green: 0.45, blue: 0.96)     // violet
        }
    }

    var bridgeIdentifier: String {
        switch self {
        case .balanced: "balanced"
        case .bright: "bright"
        case .warm: "warm"
        case .heavy: "heavy"
        }
    }
}
```

- [ ] **Step 6: Find and fix any other references to the removed cases**

Run a search for the removed cases across the Swift sources:
Use Grep for `\.open\b|\.punch\b|NativeStylePreset` under `apps/iphone-native/YESMasterNative/`. Any `switch` over `NativeStylePreset` or default-selection referencing `.open`/`.punch` must be updated to `.bright`/`.heavy`. (The enum is `CaseIterable`, so the picker grid auto-includes the new cases.) Confirm the default-selected style still compiles (e.g. a `@State` initialized to `.balanced` is unaffected).

- [ ] **Step 7: Build/verify the Swift side compiles**

If Xcode/`xcodebuild` is available, build the iPhone target; otherwise rely on the Rust bridge tests (Step 4) for the mapping contract and review the Swift diff for exhaustive `switch` coverage. Note in the commit if the Swift build wasn't run locally.

- [ ] **Step 8: Commit**

```bash
git add apps/iphone-native/YESMasterNative/ContentView.swift apps/iphone-native/rust/src/lib.rs
git commit -m "feat(standard): rename iPhone styles to the reference-4 + re-point bridge mapping"
```

---

## Task 13: Canon update — Standard's export ceremony (§6)

**Files:**
- Modify: `docs/PRODUCT.md` (the "Export Philosophy" section, lines ~61-86)

- [ ] **Step 1: Read the current passage**

Open `docs/PRODUCT.md:61-86` and confirm the "Export Philosophy" block (technical failures block; quality warnings move through `Export With Review` / `Adjust Settings` / `Export Anyway`).

- [ ] **Step 2: Append the Standard subsection**

Immediately after the existing "Export Philosophy" block, add:

```markdown
### Standard view — export ceremony

The review ceremony above describes **Advanced**. In **Standard** (the default
view), the deliberate behavior is:

- **No blocking review gate.** `Create Master` renders directly. Standard
  trusts the validated engine + the user's ears.
- **Cosmetic / advisory warnings are suppressed** (e.g. loudness-vs-reference,
  low dynamic range, codec headroom, already-compressed source). A
  non-technical user shouldn't have to weigh them.
- **One tiny, non-blocking integrity note** is kept for a genuine integrity
  issue (true-peak slipping over) — inline text, never a modal. The
  fixed-ceiling limiter makes this rare by construction.
- **Technical hard-stops still block in both modes** (invalid path,
  non-finite / corrupt render state, requested-vs-rendered sample-rate
  mismatch, sub-16-bit). Standard surfaces these prominently instead of
  celebrating the export.
- **Standard exports a fixed, known-safe default: 44.1 kHz / 24-bit WAV at a
  −1 dBTP ceiling**, with the Standard-chosen loudness (−14 / −11 / −9 LUFS).
  Sample rate / bit depth / ceiling are configurable only in Advanced. This
  mirrors the iPhone app's fixed export.
```

- [ ] **Step 3: Commit**

```bash
git add docs/PRODUCT.md
git commit -m "docs(standard): canon — Standard export ceremony (no blocking gate, fixed format)"
```

---

## Task 14: Full verification pass

**Files:** none (verification only)

- [ ] **Step 1: Frontend suite**

Run: `npm test`
Expected: all green (new Standard suites + unchanged existing suites).

- [ ] **Step 2: Type + build + Rust + bridge (umbrella)**

Run: `npm run verify:fast`
Expected: green (frontend test+build+build:windows; rust fmt/clippy/lib/integration; iPhone bridge check+test). If the desktop app is running, re-run cargo lanes with `--target-dir target\codex-rc` or close the app first.

- [ ] **Step 3: Manual WYSIWYG check (the §4 hard requirement)**

Run the app, import a track, switch to Standard, pick a Loudness step, press Play → Mastered. Confirm the audition is loudness-landed + limited (it should sound at the chosen LUFS, not the raw preset). Then Create Master to the same loudness and confirm the exported file's character/level matches the audition. (This validates that forcing `exportLufsPreview=true` in Standard closed the preview-vs-export gap.)

- [ ] **Step 4: Manual transition check**

- New-user path: clear `localStorage` key `yes-master:view-mode` and delete `session.json` → relaunch → lands in **Standard**.
- Returning-user path: with a saved session present and the flag cleared → relaunch → lands in **Advanced**.
- Standard → Advanced ("Advanced") seeds the same preset/intensity/loudness.
- Advanced (make a manual EQ edit) → "Back to Standard" → confirm modal appears; "Reset & continue" returns to Standard clean; "Save as preset" saves then returns clean; "Cancel" stays in Advanced.
- Switch to Album Master from Standard → forced into Advanced.

- [ ] **Step 5: Push the branch**

```bash
git push -u origin feat/standard-view
```

---

## Self-review (author's check against the spec)

**Spec coverage:**
- §1 goal / flagship polish → Tasks 7-11 (functional) + a flagged design-polish follow-up (below). ✓ (polish pass is explicitly deferred, not silently dropped)
- §2 one settings truth / per-track binding → StandardView binds `tm.selectedSettings` + setters (Task 9); no new global. ✓
- §2a asymmetric transition (seed-forward / warn-reset-on-return + save-as-preset) → Task 10 (`requestBackToStandard`, `BackToStandardConfirm`). ✓
- §2b managed/non-managed model + `resetToStandardManaged` → Task 2 + Task 5. ✓
- §3 reference-4 (Oomph 1:1) → Task 1. ✓
- §4 Style/Intensity/Loudness mapping + WYSIWYG → Tasks 1, 8, 9, 10 (force `exportLufsPreview`). ✓
- §5 Option-B layout + chrome → Tasks 9, 10, 11. ✓
- §6 export fixed format + ceremony + canon → Tasks 3, 9, 13. ✓
- §7 versioned migration default + persistence → Tasks 4, 6, and `hadPriorSession` (Task 5). ✓
- §8 desktop↔mobile parity (mapping + iPhone rename/remap) → Tasks 1, 12. ✓
- §9 component breakdown → Tasks 1-9 map 1:1 to the listed units. ✓
- §10 testing → every task is TDD; render-parity is exercised by the fixed-format wrap test (Task 5) + bridge parity tests (existing). ✓
- §11 out of scope (Tier-2 corrective, all-8 renaming, Album-in-Standard) → not built; Album→Advanced guard added (Task 10). ✓

**Known scoping decisions (flag to owner):**
1. `adaptive_strength` is treated as a non-managed edit (reset to 0.5) — a deliberate superset of the §2b table, to keep Standard's adaptive at the validated default (Task 2 note).
2. A preset outside the reference-4 (e.g. `spatial` carried back from Advanced) shows **no** active Standard tile until the user picks one (Task 1 `presetToStyle` → null). Acceptable v1 behavior.
3. Standard's hero includes a minimal track switcher (`<select>` when >1 track) — pragmatic, not in the mockup; revisit during the polish pass.

**Type consistency:** `ViewMode`, `StandardStyleId`, `StandardLoudnessId`, `PersistedViewState`, `StandardExportNotes`, the `tm` member names (`hadPriorSession`, `resetToStandardManaged`, `exportStandardMaster`, `setExportLufsPreview`, `setLoudnessTarget`, `setPreset`, `setIntensity`), and the `Preset` `kind` literals are used identically across tasks. ✓

**Design-polish follow-up (deferred, not dropped):** §1 demands flagship motion/hero/micro-interaction polish. Tasks 7-11 ship a correct, accessible, functional Option-B layout; the visual polish pass should run *after* this plan lands, using the design skills against `docs/simple-mode-mockup.html`. Surfaced here so it isn't mistaken for complete.
