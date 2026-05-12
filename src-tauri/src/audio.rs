use crate::types::*;

#[tauri::command]
pub async fn prepare_source_playback(track_id: TrackId) -> CommandResult<PlaybackHandle> {
    Ok(handle(track_id, PlaybackKind::Source))
}

#[tauri::command]
pub async fn prepare_master_playback(
    track_id: TrackId,
    settings: MasteringSettings,
) -> CommandResult<PlaybackHandle> {
    let _ = settings;
    Ok(handle(track_id, PlaybackKind::Master))
}

#[tauri::command]
pub async fn prepare_ab_preview(
    track_id: TrackId,
    settings: MasteringSettings,
    volume_match: bool,
) -> CommandResult<AbPreview> {
    let _ = settings;
    let source_handle = handle(track_id.clone(), PlaybackKind::Source);
    let master_handle = handle(track_id.clone(), PlaybackKind::Master);
    Ok(AbPreview {
        track_id,
        source_handle,
        master_handle,
        volume_match_offset_db: if volume_match { -2.4 } else { 0.0 },
    })
}

#[tauri::command]
pub async fn prepare_waveform(
    track_id: TrackId,
    samples_per_pixel: u32,
) -> CommandResult<WaveformPeaks> {
    let total_pixels: u32 = 1000;
    let peaks: Vec<f32> = (0..total_pixels).map(|i| mock_peak(i, total_pixels)).collect();
    let sample_rate: u32 = 44_100;
    Ok(WaveformPeaks {
        track_id,
        channels: vec![peaks.clone(), peaks],
        samples_per_pixel,
        total_samples: u64::from(total_pixels) * u64::from(samples_per_pixel),
        sample_rate,
    })
}

fn handle(track_id: TrackId, kind: PlaybackKind) -> PlaybackHandle {
    PlaybackHandle {
        id: uuid::Uuid::new_v4().to_string(),
        track_id,
        kind,
        duration_seconds: 180.0,
    }
}

fn mock_peak(i: u32, total: u32) -> f32 {
    let t = i as f32 / total as f32;
    let envelope = (t * std::f32::consts::PI).sin();
    let detail = (t * 60.0).sin().abs() * 0.3;
    (envelope * 0.7 + detail).abs().min(1.0)
}
