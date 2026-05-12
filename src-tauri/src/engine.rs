use crate::types::*;

#[tauri::command]
pub async fn analyze_tracks(track_ids: Vec<TrackId>) -> CommandResult<Vec<AnalysisResult>> {
    Ok(track_ids.into_iter().map(mock_analysis).collect())
}

fn mock_analysis(track_id: TrackId) -> AnalysisResult {
    AnalysisResult {
        track_id,
        lufs_integrated: -14.2,
        lufs_short_term_max: -10.1,
        true_peak_dbtp: -1.3,
        dynamic_range_lu: 8.4,
        spectral_balance: SpectralBalance {
            low: 0.32,
            mid: 0.41,
            high: 0.27,
        },
        transient_density: 0.55,
        stereo_width: 0.62,
        recommended_universal: MasteringSettings {
            preset: Preset::Universal,
            intensity: 0.5,
            eq_low_db: 0.0,
            eq_mid_db: 0.0,
            eq_high_db: 0.0,
            volume_match: false,
            advanced: AdvancedSettings {
                lufs_offset_db: Some(-14.0),
                ceiling_dbtp: Some(-1.0),
                bit_depth: Some(24),
                target_sample_rate: Some(44_100),
                ..Default::default()
            },
        },
        measured_at_iso: ISO_PLACEHOLDER.to_string(),
    }
}

#[tauri::command]
pub async fn render_track_preview(
    track_id: TrackId,
    settings: MasteringSettings,
) -> CommandResult<RenderJob> {
    let _ = settings;
    Ok(mock_job(RenderKind::Preview, vec![track_id]))
}

#[tauri::command]
pub async fn render_track_master(
    track_id: TrackId,
    settings: MasteringSettings,
) -> CommandResult<RenderJob> {
    let _ = settings;
    Ok(mock_job(RenderKind::Master, vec![track_id]))
}

#[tauri::command]
pub async fn render_album_master(
    track_ids: Vec<TrackId>,
    album_intent: MasteringSettings,
    per_track_overrides: Option<std::collections::HashMap<String, MasteringSettings>>,
) -> CommandResult<RenderJob> {
    let _ = album_intent;
    let _ = per_track_overrides;
    Ok(mock_job(RenderKind::Album, track_ids))
}

fn mock_job(kind: RenderKind, target_tracks: Vec<TrackId>) -> RenderJob {
    RenderJob {
        id: uuid::Uuid::new_v4().to_string(),
        kind,
        target_tracks,
        status: JobStatus::Done,
        progress: 1.0,
        started_at_iso: ISO_PLACEHOLDER.to_string(),
        output_paths: vec!["renders/mock-output.wav".to_string()],
    }
}
