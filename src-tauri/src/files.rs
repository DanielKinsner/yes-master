use crate::types::*;
use std::path::{Component, Path};

use symphonia::core::codecs::CODEC_TYPE_NULL;
use symphonia::core::formats::FormatOptions;
use symphonia::core::formats::Track;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;

#[tauri::command]
pub async fn import_tracks(paths: Vec<String>) -> CommandResult<Vec<ImportedTrack>> {
    paths.into_iter().map(|p| import_one(&p)).collect()
}

/// Reject `..` traversal components. Shared guard: the desktop commands and the
/// mobile native facade (`apps/iphone-native/rust`) both call it. `pub` so the
/// facade crate can guard `output_dir` before it `create_dir_all`s it (§15).
///
/// NOTE: this only blocks literal `..`. Absolute paths are intentionally allowed
/// — this is a local-first desktop app where the user picks arbitrary files via
/// native dialogs, so confining to a base dir would break import/export. A
/// sandboxed/base-confined mode is an owner product decision (see
/// docs/OPEN_THREADS_AND_DECISIONS.md), not a guard bug.
pub fn has_parent_dir_component(path: &Path) -> bool {
    path.components().any(|c| matches!(c, Component::ParentDir))
}

fn import_one(path_str: &str) -> CommandResult<ImportedTrack> {
    if path_str.is_empty() {
        return Err(CommandError::InvalidPath("empty path".to_string()));
    }
    let path = Path::new(path_str);
    if has_parent_dir_component(path) {
        return Err(CommandError::InvalidPath(format!(
            "path traversal not allowed: {path_str}"
        )));
    }
    let display_name = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Untitled")
        .to_string();
    let source_format = path
        .extension()
        .and_then(|s| s.to_str())
        .map(|s| s.to_lowercase())
        .unwrap_or_else(|| "unknown".to_string());

    let metadata = probe_metadata(path)?;

    Ok(ImportedTrack {
        id: TrackId::new(),
        path: path_str.to_string(),
        display_name,
        source_format,
        duration_seconds: metadata.duration_seconds,
        sample_rate: metadata.sample_rate,
        channels: metadata.channels,
    })
}

#[derive(Default)]
struct TrackMetadata {
    duration_seconds: Option<f64>,
    sample_rate: Option<u32>,
    channels: Option<u16>,
}

fn first_decodable_track(tracks: &[Track]) -> Option<&Track> {
    tracks
        .iter()
        .find(|track| track.codec_params.codec != CODEC_TYPE_NULL)
}

fn probe_metadata(path: &Path) -> CommandResult<TrackMetadata> {
    let file = std::fs::File::open(path).map_err(|e| CommandError::Io(e.to_string()))?;
    let mss = MediaSourceStream::new(Box::new(file), Default::default());
    let mut hint = Hint::new();
    if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
        hint.with_extension(ext);
    }
    let probed = symphonia::default::get_probe()
        .format(
            &hint,
            mss,
            &FormatOptions::default(),
            &MetadataOptions::default(),
        )
        .map_err(|e| CommandError::Decode(e.to_string()))?;
    let format = probed.format;
    let track = first_decodable_track(format.tracks())
        .ok_or_else(|| CommandError::Decode("no decodable track".to_string()))?;
    let params = &track.codec_params;
    let sample_rate = params.sample_rate;
    let channels = params.channels.map(|c| c.count() as u16);
    let duration = match (params.n_frames, params.sample_rate) {
        (Some(frames), Some(sr)) if sr > 0 => Some(frames as f64 / sr as f64),
        _ => None,
    };
    Ok(TrackMetadata {
        duration_seconds: duration,
        sample_rate,
        channels,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use symphonia::core::audio::Channels;
    use symphonia::core::codecs::{CodecParameters, CODEC_TYPE_PCM_S16LE};

    fn audio_track(id: u32, sample_rate: u32, channels: Channels) -> Track {
        let mut params = CodecParameters::new();
        params
            .for_codec(CODEC_TYPE_PCM_S16LE)
            .with_sample_rate(sample_rate)
            .with_channels(channels)
            .with_n_frames(u64::from(sample_rate));
        Track::new(id, params)
    }

    #[test]
    fn parent_dir_guard_rejects_traversal_but_allows_absolute_paths() {
        // §14 — pin the guard's contract. `..` anywhere is rejected...
        assert!(has_parent_dir_component(Path::new("../secret.wav")));
        assert!(has_parent_dir_component(Path::new(
            "music/../../etc/passwd"
        )));
        assert!(has_parent_dir_component(Path::new("a/b/../c")));
        // ...including when a symlink-looking component is followed by `..`.
        assert!(has_parent_dir_component(Path::new("link/../escape")));

        // Absolute paths WITHOUT `..` are intentionally ALLOWED: this is a
        // local-first desktop app where the user picks arbitrary files via
        // native dialogs, so import/export legitimately span the whole
        // filesystem. Base-dir confinement / symlink-target rejection would
        // break that and is an owner sandbox decision (see OPEN_THREADS), not a
        // guard bug — this assertion documents that intent.
        assert!(!has_parent_dir_component(Path::new("/home/me/song.wav")));
        assert!(!has_parent_dir_component(Path::new("song.wav")));
        #[cfg(windows)]
        {
            assert!(!has_parent_dir_component(Path::new(
                r"C:\Users\me\song.wav"
            )));
            assert!(has_parent_dir_component(Path::new(
                r"C:\Users\me\..\..\Windows"
            )));
        }
    }

    #[test]
    fn import_one_rejects_a_traversal_path() {
        let err = import_one("../escape.wav").expect_err("traversal must be rejected");
        assert!(
            matches!(err, CommandError::InvalidPath(ref m) if m.contains("path traversal")),
            "unexpected error: {err:?}"
        );
    }

    #[test]
    fn metadata_track_selection_skips_null_tracks_before_audio() {
        let tracks = vec![
            Track::new(1, CodecParameters::new()),
            audio_track(7, 48_000, Channels::FRONT_LEFT | Channels::FRONT_RIGHT),
        ];

        let selected = first_decodable_track(&tracks).expect("audio track");
        assert_eq!(selected.id, 7);
        assert_eq!(selected.codec_params.sample_rate, Some(48_000));
        assert_eq!(selected.codec_params.channels.map(|c| c.count()), Some(2));
    }
}
