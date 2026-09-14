//! Inert stand-ins for the desktop modules the included sources *name* but
//! the browser build never *calls*. `analysis.rs` reaches `decode`/`files`
//! only from its path-based entry point (`analyze_one_with_progress`), which
//! this crate does not use — the worker hands the already-decoded PCM to the
//! per-stage `analyze_*` functions instead. `mp3::delivery_rate` is reachable
//! only from `ExportEncoding::delivery_rate`, which the browser never calls.
pub mod decode {
    use crate::types::{CommandError, CommandResult};
    use std::path::Path;
    pub struct DecodedPcm {
        pub samples: Vec<f32>,
        pub sample_rate: u32,
        pub channels: u16,
    }
    pub fn decode_full(path: &Path) -> CommandResult<DecodedPcm> {
        Err(CommandError::Decode(format!(
            "file decoding is not available in the browser build: {}",
            path.display()
        )))
    }
}
pub mod files {
    use std::path::Path;
    pub fn has_parent_dir_component(_path: &Path) -> bool {
        false
    }
}
pub mod mp3 {
    /// Mirror of `mp3::delivery_rate` (MP3 legal rates).
    pub fn delivery_rate(rate: u32) -> u32 {
        if rate % 44_100 == 0 {
            44_100
        } else if rate == 32_000 {
            32_000
        } else {
            48_000
        }
    }
}
