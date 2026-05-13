use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
#[serde(transparent)]
pub struct TrackId(pub String);

impl TrackId {
    pub fn new() -> Self {
        Self(uuid::Uuid::new_v4().to_string())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Default for TrackId {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ImportedTrack {
    pub id: TrackId,
    pub path: String,
    pub display_name: String,
    pub source_format: String,
    pub duration_seconds: Option<f64>,
    pub sample_rate: Option<u32>,
    pub channels: Option<u16>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SpectralBalance {
    pub low: f32,
    pub mid: f32,
    pub high: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AnalysisResult {
    pub track_id: TrackId,
    pub lufs_integrated: f32,
    pub lufs_short_term_max: f32,
    pub true_peak_dbtp: f32,
    pub dynamic_range_lu: f32,
    pub spectral_balance: SpectralBalance,
    pub transient_density: f32,
    pub stereo_width: f32,
    pub recommended_universal: MasteringSettings,
    pub measured_at_iso: String,
    // Phase 9: heuristic role + character inference. Optional so older callers
    // / serialized analyses still parse cleanly.
    #[serde(default)]
    pub inferred_role: Option<TrackRole>,
    #[serde(default)]
    pub role_confidence: Option<InferenceConfidence>,
    #[serde(default)]
    pub inferred_character: Option<TrackCharacter>,
    #[serde(default)]
    pub character_confidence: Option<InferenceConfidence>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TrackRole {
    Opener,
    Closer,
    Single,
    Ballad,
    Interlude,
    AlbumTrack,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TrackCharacter {
    Bright,
    Dark,
    Dense,
    Sparse,
    Balanced,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum InferenceConfidence {
    Strong,
    Moderate,
    Unsure,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Preset {
    Universal,
    Clarity,
    Tape,
    Spatial,
    Oomph,
    Warmth,
    Punch,
    Loud,
    Custom { id: String },
}

/// Phase A3: Delivery profile presets ported from
/// `../album-mastering-studio/src/album_mastering_studio/standards.py`.
///
/// Each non-`Custom` variant carries a complete (target LUFS, ceiling,
/// sample-rate hint, bit-depth) bundle for a specific delivery target.
/// At render time, when `delivery_profile != Custom`, the profile's
/// values shadow the corresponding fields in `AdvancedSettings`. `Custom`
/// means "use the user's explicit `lufs_offset_db` / `ceiling_dbtp` /
/// `bit_depth` / `target_sample_rate` exactly as set."
///
/// Sample rate is captured per profile but resampling is deferred to a
/// later phase — A3 honors `bit_depth`, `target_lufs`, and `ceiling_dbtp`
/// but writes WAVs at the source sample rate regardless. The captured
/// rate is exposed via `output_sample_rate()` for future use.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum DeliveryProfile {
    /// -14 LUFS, -1 dBTP, 48 kHz, 24-bit. Spotify / YouTube / Tidal / Amazon.
    StreamingUniversal,
    /// -16 LUFS, -1 dBTP, 48 kHz, 24-bit. Apple Music's tighter target.
    AppleMusic,
    /// -14 LUFS, -1 dBTP, 44.1 kHz, 16-bit. Red Book CD.
    Cd,
    /// -18 LUFS, -3 dBTP, 48 kHz, 24-bit. Generous headroom for the
    /// cutting engineer / RIAA pre-emphasis.
    VinylPremaster,
    /// -10.5 LUFS, -1 dBTP, 48 kHz, 24-bit. Rock / metal masters that
    /// don't translate well at the streaming -14 target.
    LoudRock,
    /// -23 LUFS, -1 dBTP, 48 kHz, 24-bit. EBU R128 broadcast.
    BroadcastEu,
    /// -24 LUFS, -2 dBTP, 48 kHz, 24-bit. ATSC A/85 broadcast.
    BroadcastUs,
    /// No shadowing — render uses the user's explicit `lufs_offset_db`,
    /// `ceiling_dbtp`, `bit_depth`, and `target_sample_rate` fields
    /// from `AdvancedSettings` verbatim.
    Custom,
}

impl Default for DeliveryProfile {
    fn default() -> Self {
        Self::StreamingUniversal
    }
}

impl DeliveryProfile {
    /// Target integrated LUFS for non-Custom profiles. `None` when
    /// `Custom` (engine falls back to `AdvancedSettings::lufs_offset_db`).
    pub fn target_lufs(&self) -> Option<f32> {
        match self {
            Self::StreamingUniversal => Some(-14.0),
            Self::AppleMusic => Some(-16.0),
            Self::Cd => Some(-14.0),
            Self::VinylPremaster => Some(-18.0),
            Self::LoudRock => Some(-10.5),
            Self::BroadcastEu => Some(-23.0),
            Self::BroadcastUs => Some(-24.0),
            Self::Custom => None,
        }
    }

    /// True-peak ceiling in dBTP for non-Custom profiles. `None` for
    /// `Custom`.
    pub fn ceiling_dbtp(&self) -> Option<f32> {
        match self {
            Self::StreamingUniversal
            | Self::AppleMusic
            | Self::Cd
            | Self::LoudRock
            | Self::BroadcastEu => Some(-1.0),
            Self::VinylPremaster => Some(-3.0),
            Self::BroadcastUs => Some(-2.0),
            Self::Custom => None,
        }
    }

    /// Recommended output sample rate. Captured for future resampling
    /// support — A3 does NOT resample; the renderer writes at the
    /// source's sample rate regardless of this value.
    pub fn output_sample_rate(&self) -> Option<u32> {
        match self {
            Self::Cd => Some(44_100),
            Self::StreamingUniversal
            | Self::AppleMusic
            | Self::VinylPremaster
            | Self::LoudRock
            | Self::BroadcastEu
            | Self::BroadcastUs => Some(48_000),
            Self::Custom => None,
        }
    }

    /// Output bit depth for non-Custom profiles. Honored by the WAV writer.
    pub fn output_bit_depth(&self) -> Option<u16> {
        match self {
            Self::Cd => Some(16),
            Self::StreamingUniversal
            | Self::AppleMusic
            | Self::VinylPremaster
            | Self::LoudRock
            | Self::BroadcastEu
            | Self::BroadcastUs => Some(24),
            Self::Custom => None,
        }
    }

    pub fn is_custom(&self) -> bool {
        matches!(self, Self::Custom)
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Self::StreamingUniversal => "Streaming (Spotify / YouTube / Tidal / Amazon)",
            Self::AppleMusic => "Apple Music",
            Self::Cd => "CD (16-bit)",
            Self::VinylPremaster => "Vinyl Premaster",
            Self::LoudRock => "Loud Rock / Aggressive",
            Self::BroadcastEu => "Broadcast EU (EBU R128)",
            Self::BroadcastUs => "Broadcast US (ATSC A/85)",
            Self::Custom => "Custom",
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MasteringSettings {
    pub preset: Preset,
    pub intensity: f32,
    pub eq_low_db: f32,
    /// Phase A2: low-mid peaking EQ (400 Hz, Q=0.9). User offset on top of
    /// the preset's baseline `low_mid_db`. `#[serde(default)]` so projects
    /// saved before this field existed load with a 0.0 value (matching the
    /// chain's identity behavior at neutral gain).
    #[serde(default)]
    pub eq_low_mid_db: f32,
    pub eq_mid_db: f32,
    pub eq_high_db: f32,
    pub volume_match: bool,
    /// Pre-chain gain. Negative values back off the source before the preset
    /// EQ/saturation/limiter sees it — useful for already-mastered material
    /// that would otherwise clip when the preset adds its baseline gain push.
    /// Default 0 dB. Phase 12.1 Dan feedback.
    #[serde(default)]
    pub input_gain_db: f32,
    /// Post-limiter trim. Applied after the chain's limiter and volume-match
    /// stages. Default 0 dB. Boosting above 0 here can re-introduce peaks
    /// above the ceiling, which is intentionally allowed so a user can match
    /// reference loudness — but the export receipt's true-peak check will
    /// catch the result.
    #[serde(default)]
    pub output_gain_db: f32,
    /// Phase A3 — delivery profile preset. When non-`Custom`, shadows
    /// `lufs_offset_db`, `ceiling_dbtp`, and `bit_depth` at render time
    /// with the profile's values. `Custom` means "use the explicit
    /// advanced fields as-is." `#[serde(default)]` so older `.ams.json`
    /// projects load with the streaming-universal default.
    #[serde(default)]
    pub delivery_profile: DeliveryProfile,
    pub advanced: AdvancedSettings,
}

impl MasteringSettings {
    /// Phase A3 — effective target LUFS for the post-chain landing stage.
    /// When `delivery_profile` is non-`Custom`, the profile's target wins;
    /// `Custom` falls back to `advanced.lufs_offset_db`. Used by the render
    /// pipeline in `engine.rs`.
    pub fn effective_target_lufs(&self) -> Option<f32> {
        self.delivery_profile
            .target_lufs()
            .or(self.advanced.lufs_offset_db)
    }

    /// Phase A3 — effective true-peak ceiling in dBTP. Profile shadows the
    /// user-set value when non-`Custom`. Falls back to `-1.0` when no
    /// value is set anywhere (matches the prior default).
    pub fn effective_ceiling_dbtp(&self) -> f32 {
        self.delivery_profile
            .ceiling_dbtp()
            .or(self.advanced.ceiling_dbtp)
            .unwrap_or(-1.0)
    }

    /// Phase A3 — effective output bit depth. Profile shadows the user-
    /// set value when non-`Custom`. Falls back to `24` when no value is
    /// set anywhere (matches the prior default).
    pub fn effective_bit_depth(&self) -> u16 {
        self.delivery_profile
            .output_bit_depth()
            .or(self.advanced.bit_depth)
            .unwrap_or(24)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct AdvancedSettings {
    pub lufs_offset_db: Option<f32>,
    pub ceiling_dbtp: Option<f32>,
    pub width: Option<f32>,
    pub warmth: Option<f32>,
    pub presence_air: Option<f32>,
    pub compression_density: Option<f32>,
    // Phase 12.2 — per-band compressor overrides. `None` => the macro slider
    // (compression_density) drives that band's threshold; per-band ratio /
    // attack / release fall back to fixed musical defaults (see
    // `ChainCoeffs::from_settings`). `Some(v)` => override the macro for this
    // band/parameter only. All `#[serde(default)]` so older sessions and
    // older frontends parse cleanly.
    #[serde(default)]
    pub compression_low_threshold_db: Option<f32>,
    #[serde(default)]
    pub compression_low_ratio: Option<f32>,
    #[serde(default)]
    pub compression_low_attack_ms: Option<f32>,
    #[serde(default)]
    pub compression_low_release_ms: Option<f32>,
    #[serde(default)]
    pub compression_mid_threshold_db: Option<f32>,
    #[serde(default)]
    pub compression_mid_ratio: Option<f32>,
    #[serde(default)]
    pub compression_mid_attack_ms: Option<f32>,
    #[serde(default)]
    pub compression_mid_release_ms: Option<f32>,
    #[serde(default)]
    pub compression_high_threshold_db: Option<f32>,
    #[serde(default)]
    pub compression_high_ratio: Option<f32>,
    #[serde(default)]
    pub compression_high_attack_ms: Option<f32>,
    #[serde(default)]
    pub compression_high_release_ms: Option<f32>,
    /// Phase 12.2 — when `Some(false)`, the multiband compressor runs
    /// independent L/R envelope followers per band. Default (`None` or
    /// `Some(true)`) links stereo: a single max-of-|L|,|R| envelope drives the
    /// same gain reduction on both channels, the standard mastering choice.
    #[serde(default)]
    pub compression_link_stereo: Option<bool>,
    pub bit_depth: Option<u16>,
    pub target_sample_rate: Option<u32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WaveformPeaks {
    pub track_id: TrackId,
    pub channels: Vec<Vec<f32>>,
    pub samples_per_pixel: u32,
    pub total_samples: u64,
    pub sample_rate: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PlaybackKind {
    Source,
    Master,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PlaybackHandle {
    pub id: String,
    pub track_id: TrackId,
    pub kind: PlaybackKind,
    pub duration_seconds: f64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AbPreview {
    pub track_id: TrackId,
    pub source_handle: PlaybackHandle,
    pub master_handle: PlaybackHandle,
    pub volume_match_offset_db: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RenderKind {
    Preview,
    Master,
    Album,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum JobStatus {
    Pending,
    Running,
    Done,
    Failed { reason: String },
    Cancelled,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RenderJob {
    pub id: String,
    pub kind: RenderKind,
    pub target_tracks: Vec<TrackId>,
    pub status: JobStatus,
    pub progress: f32,
    pub started_at_iso: String,
    pub output_paths: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "snake_case")]
pub enum QualityLevel {
    Info,
    Warning,
    Critical,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct QualityCheck {
    pub level: QualityLevel,
    pub code: String,
    pub message: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ExportReport {
    pub track_id: TrackId,
    pub output_path: String,
    pub measured_lufs: f32,
    pub measured_true_peak_dbtp: f32,
    pub measured_dynamic_range_lu: f32,
    pub source_format: String,
    pub destination_format: String,
    pub sample_rate: u32,
    pub bit_depth: u16,
    pub checks: Vec<QualityCheck>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "snake_case")]
pub enum ProjectMode {
    Track,
    Album,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProjectState {
    pub schema_version: u32,
    pub mode: ProjectMode,
    pub tracks: Vec<ImportedTrack>,
    pub track_order: Vec<TrackId>,
    pub track_settings: HashMap<String, MasteringSettings>,
    pub album_intent: Option<MasteringSettings>,
    /// Set of track IDs whose per-track `track_settings` should override the
    /// shared `album_intent` during album rendering. Defaulted so older
    /// sessions (without this field) deserialize cleanly as "no overrides."
    #[serde(default)]
    pub track_override_album: Vec<TrackId>,
    pub last_saved_iso: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "snake_case")]
pub enum PresetKind {
    Track,
    Album,
    Shared,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UserPreset {
    pub id: String,
    pub name: String,
    pub kind: PresetKind,
    pub settings: MasteringSettings,
    pub created_at_iso: String,
}

#[derive(Debug, Error, Clone)]
pub enum CommandError {
    #[error("not implemented: {0}")]
    NotImplemented(String),
    #[error("invalid path: {0}")]
    InvalidPath(String),
    #[error("decode error: {0}")]
    Decode(String),
    #[error("render error: {0}")]
    Render(String),
    #[error("io error: {0}")]
    Io(String),
    #[error("{0}")]
    Other(String),
}

impl Serialize for CommandError {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}

pub type CommandResult<T> = Result<T, CommandError>;

pub const ISO_PLACEHOLDER: &str = "2026-05-11T12:00:00Z";

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PlaybackTick {
    pub track_id: Option<TrackId>,
    pub position_sec: f64,
    pub is_playing: bool,
    pub is_loaded: bool,
    /// Post-output-gain peak across all channels since the last tick, in dBFS.
    /// `-120.0` is the silence sentinel (no signal in the window). Values
    /// above `-0.1` indicate clipping risk; values above `0.0` are clipping.
    /// Defaulted so older sessions/frontends parse cleanly as "no info."
    #[serde(default = "default_silence_dbfs")]
    pub peak_dbfs: f32,
    /// Phase 12.2 — gain reduction (in dB, negative) from the low band of the
    /// multiband compressor, captured as the maximum reduction seen in the
    /// last snapshot window. `-120.0` is the silence sentinel (no signal or
    /// compressor inactive in the window). Defaulted so older sessions and
    /// older frontends parse cleanly.
    #[serde(default = "default_silence_dbfs")]
    pub gr_low_db: f32,
    #[serde(default = "default_silence_dbfs")]
    pub gr_mid_db: f32,
    #[serde(default = "default_silence_dbfs")]
    pub gr_high_db: f32,
    /// Phase 12.2 P3 — live BS.1770 momentary LUFS (400 ms K-weighted
    /// sliding window). `-120.0` is the silence sentinel.
    #[serde(default = "default_silence_dbfs")]
    pub lufs_momentary: f32,
    /// Phase 12.2 P3+ — live BS.1770-4 integrated LUFS over the current
    /// playback session. Updates every 100 ms as new 400 ms blocks complete
    /// and pass the absolute (-70 LUFS) and relative (-10 LU) gates. Resets
    /// when a new playback starts. `-120.0` is the silence sentinel.
    /// Defaulted so older sessions/frontends parse cleanly as "no info."
    #[serde(default = "default_silence_dbfs")]
    pub lufs_integrated: f32,
}

fn default_silence_dbfs() -> f32 {
    -120.0
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub struct LoopRegion {
    pub start_sec: f64,
    pub end_sec: f64,
}
