//! Compile-time canary for the iPhone native bridge's import surface.
//!
//! The bridge crate (`apps/iphone-native/rust`) re-uses `yes_master_lib`, but
//! no desktop fast-lane target builds it — a rename or visibility change on
//! anything below compiles clean here and breaks the bridge later (it
//! happened once already; see CLAUDE.md's bridge-lane note). This test makes
//! that failure happen in `cargo test --target-dir target\codex-rc` with a
//! message that names the bridge.
//!
//! Source of truth for the list: `apps/iphone-native/rust/src/lib.rs:6-11`
//! plus `live_stream.rs` (dsp, decode, deep_analysis, confidence imports).
//! If this test fails after a deliberate library change, update the bridge
//! crate in the same commit and run its lane:
//!   cd apps/iphone-native/rust && cargo check --all-targets && cargo test

use yes_master_lib::dsp::{ChainCoeffs, MasteringChain};
use yes_master_lib::engine::AnalyzeRequest;
use yes_master_lib::guardrails::{
    AlreadyMasteredStandDown, CompressionBandPlan, CompressionGuards, CompressionPlan,
    CompressionPlanReason, GuardReason,
};
use yes_master_lib::{
    AdvancedSettings, AnalysisResult, CompressionMode, DeliveryProfile, MasteringSettings, Preset,
    RenderKind, SourceProfile, TrackId,
};

/// Never called — exists so every TYPE the bridge imports must keep
/// resolving from the desktop crate.
#[allow(dead_code, clippy::too_many_arguments)]
fn bridge_type_surface(
    _: AnalyzeRequest,
    _: AdvancedSettings,
    _: AnalysisResult,
    _: CompressionMode,
    _: DeliveryProfile,
    _: MasteringSettings,
    _: Preset,
    _: RenderKind,
    _: SourceProfile,
    _: TrackId,
    _: ChainCoeffs,
    _: MasteringChain,
    _: yes_master_lib::deep_analysis::DeepAnalysis,
    _: AlreadyMasteredStandDown,
    _: CompressionBandPlan,
    _: CompressionGuards,
    _: CompressionPlan,
    _: CompressionPlanReason,
    _: GuardReason,
) {
}

#[test]
fn bridge_function_surface_still_resolves() {
    // Binding a fn item only requires the path to resolve — no call, no
    // engine work. Each line is one function the bridge crate imports.
    let _ = yes_master_lib::engine::analyze_tracks_core;
    let _ = yes_master_lib::engine::mastering_render;
    let _ = yes_master_lib::engine::preview_landing;
    let _ = yes_master_lib::profile_store::apply_resolved_compression_guards;
    let _ = yes_master_lib::profile_store::apply_resolved_confidence;
    let _ = yes_master_lib::profile_store::apply_resolved_profile;
    let _ = yes_master_lib::decode::decode_full;
    let _ = yes_master_lib::confidence::init_confidence_gating_from_env;
    let _ = yes_master_lib::confidence::is_confidence_gating_enabled;
    let _ = yes_master_lib::confidence::resolve_source_confidence;
    let _ = yes_master_lib::guardrails::init_adaptive_compression_from_env;
    let _ = yes_master_lib::guardrails::is_adaptive_compression_enabled;
    let _ = yes_master_lib::guardrails::set_adaptive_compression_enabled;
    let _ = yes_master_lib::guardrails::classify_already_mastered_stand_down;
    let _ = yes_master_lib::guardrails::compression_plan_for_resolved_settings;
    let _ = yes_master_lib::guardrails::resolve_compression_guards;
}
