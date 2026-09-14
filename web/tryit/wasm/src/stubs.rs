//! Inert stand-ins for the desktop-only adaptive modules. In the browser
//! build `source_profile`, `source_confidence` and `compression_guards` are
//! always `None`, so the chain never calls into these; they exist only so
//! `dsp.rs` / `types.rs` compile unchanged. Every operation is the identity,
//! which is exactly the desktop chain's own "no analysis" path.
pub mod deep_analysis {
    #[derive(Debug, Clone)]
    pub struct DeepAnalysis;
}
pub mod confidence {
    use serde::{Deserialize, Serialize};
    #[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
    pub struct AxisConfidence { pub coverage: f32, pub consistency: f32, pub confidence: f32 }
    #[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
    pub struct Confidence { pub bright: AxisConfidence, pub low: AxisConfidence, pub density: AxisConfidence, pub width: AxisConfidence }
    impl Default for Confidence {
        fn default() -> Self {
            let a = AxisConfidence { coverage: 1.0, consistency: 1.0, confidence: 1.0 };
            Self { bright: a, low: a, density: a, width: a }
        }
    }
}
pub mod guardrails {
    use serde::{Deserialize, Serialize};
    use crate::types::SourceProfile;
    use crate::confidence::Confidence;
    pub const ADAPTIVE_STRENGTH_DEFAULT: f32 = 0.5;
    pub fn is_adaptive_compression_enabled() -> bool { false }
    #[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
    pub struct BandCompressionGuard { pub density_mult: f32, pub threshold_lift_db: f32, pub ratio_mult: f32 }
    #[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
    #[serde(rename_all = "snake_case")]
    pub enum GuardReason { LowBandDense, MidBandDense, HighBandDense, AlreadyMastered }
    #[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
    pub struct CompressionGuards { pub low: BandCompressionGuard, pub mid: BandCompressionGuard, pub high: BandCompressionGuard, pub stand_down: f32, pub reasons: Vec<GuardReason> }
    #[derive(Debug, Clone, Copy)]
    pub struct SourceGuardrails;
    impl SourceGuardrails {
        pub fn compute_with_confidence(_p: &SourceProfile, _strength: f32, _c: &Confidence) -> Self { Self }
        pub fn trim_bright_db(&self, db: f32) -> f32 { db }
        pub fn trim_low_db(&self, db: f32) -> f32 { db }
        pub fn trim_width(&self, w: f32) -> f32 { w }
        pub fn scale_density(&self, d: f32) -> f32 { d }
    }
}
pub mod mp3 {
    /// Mirror of `mp3::delivery_rate` (MP3 legal rates); only reachable from
    /// `ExportEncoding::delivery_rate`, which the browser build never calls.
    pub fn delivery_rate(rate: u32) -> u32 {
        if rate % 44_100 == 0 { 44_100 } else if rate == 32_000 { 32_000 } else { 48_000 }
    }
}
