//! Export I/O failure battery (hardening plan D4).
//!
//! The filesystem is user territory: unicode names, pre-existing files,
//! read-only collisions, absurd path lengths, files locked by other
//! apps. The contract for every case: a typed `CommandError` or a
//! correct render — never a panic, never a clobbered prior file, never
//! stray `.tmp` litter after a completed call.
//!
//! (The mid-render never-overwrite race itself is pinned at the unit
//! layer in `wav_writer.rs` — `write_wav_diverts_to_a_sibling...`; this
//! battery drives the public render surface end-to-end.)

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use tempfile::TempDir;
use yes_master_lib::engine::{
    mastering_render, mastering_render_to_path, mastering_render_with_cancel, RenderJobOptions,
    RenderJobRegistry,
};
use yes_master_lib::types::{
    AdvancedSettings, DeliveryProfile, JobStatus, MasteringSettings, Preset, RenderKind, TrackId,
};

fn write_sine_wav(path: &Path, sample_rate: u32, seconds: f32, freq: f32, channels: u16) {
    let spec = hound::WavSpec {
        channels,
        sample_rate,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let mut w = hound::WavWriter::create(path, spec).expect("create test wav");
    let frames = (sample_rate as f32 * seconds) as usize;
    for n in 0..frames {
        let s = (0.4
            * (2.0 * std::f32::consts::PI * freq * n as f32 / sample_rate as f32).sin()
            * 32_767.0) as i16;
        for _ in 0..channels {
            w.write_sample(s).expect("write sample");
        }
    }
    w.finalize().expect("finalize test wav");
}

fn settings() -> MasteringSettings {
    MasteringSettings {
        preset: Preset::Universal,
        intensity: 0.5,
        eq_sub_db: 0.0,
        eq_low_db: 0.0,
        eq_low_mid_db: 0.0,
        eq_mid_db: 0.0,
        eq_high_mid_db: 0.0,
        eq_high_db: 0.0,
        eq_sparkle_db: 0.0,
        eq_bands: yes_master_lib::EqBandFrequencies::default(),
        volume_match: false,
        source_lufs_integrated: None,
        input_gain_db: 0.0,
        output_gain_db: 0.0,
        delivery_profile: DeliveryProfile::Custom,
        album: None,
        advanced: AdvancedSettings::default(),
    }
}

fn no_tmp_litter(dir: &Path) {
    let leftovers: Vec<PathBuf> = std::fs::read_dir(dir)
        .expect("read dir")
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("tmp"))
        .collect();
    assert!(
        leftovers.is_empty(),
        "stray .tmp files left behind: {leftovers:?}",
    );
}

#[test]
fn unicode_source_and_output_names_render_fine() {
    let tmp = TempDir::new().expect("tempdir");
    let src = tmp.path().join("τρίτο-曲-🎵-mix.wav");
    write_sine_wav(&src, 44_100, 0.6, 440.0, 2);
    let chosen = tmp
        .path()
        .join("Ünïcode déstination")
        .join("最終-master-🎧.wav");

    let job = mastering_render_to_path(
        TrackId("unicode".to_string()),
        &src,
        &settings(),
        tmp.path(),
        RenderKind::Master,
        &chosen,
    )
    .expect("unicode paths must render");

    assert!(matches!(job.status, JobStatus::Done));
    let out = Path::new(&job.output_paths[0]);
    assert!(out.exists(), "output missing at {}", out.display());
    hound::WavReader::open(out).expect("unicode output decodes");
    no_tmp_litter(out.parent().unwrap());
}

#[test]
fn destination_whose_parent_is_a_file_errors_cleanly() {
    let tmp = TempDir::new().expect("tempdir");
    let src = tmp.path().join("input.wav");
    write_sine_wav(&src, 44_100, 0.4, 330.0, 2);
    // `blocker` is a FILE; using it as a directory must produce a typed
    // error from create_dir_all, not a panic.
    let blocker = tmp.path().join("blocker");
    std::fs::write(&blocker, b"not a directory").expect("write blocker");
    let chosen = blocker.join("master.wav");

    let err = mastering_render_to_path(
        TrackId("bad-parent".to_string()),
        &src,
        &settings(),
        tmp.path(),
        RenderKind::Master,
        &chosen,
    )
    .expect_err("file-as-directory must error");
    let msg = format!("{err:?}");
    assert!(
        msg.contains("Io") || msg.contains("InvalidPath"),
        "expected a typed Io/InvalidPath error, got: {msg}",
    );
    no_tmp_litter(tmp.path());
}

#[test]
fn absurdly_long_output_name_never_panics_and_leaves_no_litter() {
    let tmp = TempDir::new().expect("tempdir");
    let src = tmp.path().join("input.wav");
    write_sine_wav(&src, 44_100, 0.4, 330.0, 2);
    // ~300-char file name: past the classic Windows MAX_PATH once joined.
    // Contract is honest behavior either way — a real file or a typed
    // error — with no panic and no tmp litter.
    let long_name = format!("{}.wav", "m".repeat(300));
    let chosen = tmp.path().join(long_name);

    match mastering_render_to_path(
        TrackId("long-path".to_string()),
        &src,
        &settings(),
        tmp.path(),
        RenderKind::Master,
        &chosen,
    ) {
        Ok(job) => {
            let out = Path::new(&job.output_paths[0]);
            assert!(out.exists(), "reported output must exist");
        }
        Err(err) => {
            let msg = format!("{err:?}");
            assert!(
                msg.contains("Io") || msg.contains("InvalidPath"),
                "long-path failure must be typed, got: {msg}",
            );
        }
    }
    no_tmp_litter(tmp.path());
}

#[test]
fn existing_file_at_chosen_path_is_never_clobbered_even_when_read_only() {
    let tmp = TempDir::new().expect("tempdir");
    let src = tmp.path().join("input.wav");
    write_sine_wav(&src, 44_100, 0.4, 330.0, 2);

    // A pre-existing file at the chosen name — marked read-only, so any
    // clobber attempt would also be an I/O error. The render must divert
    // to the __{n} sibling without touching it.
    let chosen = tmp.path().join("master.wav");
    std::fs::write(&chosen, b"precious prior render").expect("write existing");
    let mut perms = std::fs::metadata(&chosen).expect("meta").permissions();
    perms.set_readonly(true);
    std::fs::set_permissions(&chosen, perms).expect("set read-only");

    let job = mastering_render_to_path(
        TrackId("collision".to_string()),
        &src,
        &settings(),
        tmp.path(),
        RenderKind::Master,
        &chosen,
    )
    .expect("collision must divert, not fail");

    let out = PathBuf::from(&job.output_paths[0]);
    assert_ne!(out, chosen, "render must not land on the existing file");
    assert_eq!(
        out,
        tmp.path().join("master__1.wav"),
        "divert follows the __{{n}} convention",
    );
    assert_eq!(
        std::fs::read(&chosen).expect("read prior file"),
        b"precious prior render",
        "prior file must survive untouched",
    );

    // Render again with the same chosen name: master.wav and master__1.wav
    // both exist now, so the next render must land at master__2.wav.
    let job2 = mastering_render_to_path(
        TrackId("collision-2".to_string()),
        &src,
        &settings(),
        tmp.path(),
        RenderKind::Master,
        &chosen,
    )
    .expect("second collision must also divert");
    assert_eq!(
        PathBuf::from(&job2.output_paths[0]),
        tmp.path().join("master__2.wav"),
        "suffix numbering must keep advancing",
    );

    // Restore writability so TempDir cleanup works on Windows.
    let mut perms = std::fs::metadata(&chosen).expect("meta").permissions();
    #[allow(clippy::permissions_set_readonly_false)]
    perms.set_readonly(false);
    std::fs::set_permissions(&chosen, perms).expect("restore perms");
    no_tmp_litter(tmp.path());
}

#[test]
fn default_output_dir_renders_also_avoid_prior_renders() {
    // Same promise on the non-explicit path (default out_dir naming).
    let tmp = TempDir::new().expect("tempdir");
    let src = tmp.path().join("song.wav");
    write_sine_wav(&src, 44_100, 0.4, 330.0, 2);

    let first = mastering_render(
        TrackId("dup-1".to_string()),
        &src,
        &settings(),
        tmp.path(),
        RenderKind::Master,
    )
    .expect("first render");
    let second = mastering_render(
        TrackId("dup-2".to_string()),
        &src,
        &settings(),
        tmp.path(),
        RenderKind::Master,
    )
    .expect("second render");

    let first_path = PathBuf::from(&first.output_paths[0]);
    let second_path = PathBuf::from(&second.output_paths[0]);
    assert_ne!(
        first_path, second_path,
        "back-to-back renders of the same source must not share a path",
    );
    assert!(first_path.exists() && second_path.exists());
    no_tmp_litter(tmp.path());
}

#[test]
fn cancelling_mid_render_returns_cancelled_and_writes_no_output() {
    let tmp = TempDir::new().expect("tempdir");
    let src = tmp.path().join("cancel-source.wav");
    let chosen = tmp.path().join("cancelled-output.wav");
    write_sine_wav(&src, 44_100, 0.8, 440.0, 2);

    let cancel_flag = AtomicBool::new(false);
    let on_progress = |fraction: f32| {
        if fraction > 0.0 {
            cancel_flag.store(true, Ordering::SeqCst);
        }
    };
    let job = mastering_render_with_cancel(
        TrackId("cancel-mid-render".to_string()),
        &src,
        &settings(),
        tmp.path(),
        RenderKind::Master,
        RenderJobOptions {
            on_progress: Some(&on_progress),
            output_path: Some(&chosen),
            job_id: Some("cancel-job"),
            cancel_flag: Some(&cancel_flag),
        },
    )
    .expect("cancelled render should return a job");

    assert!(matches!(job.status, JobStatus::Cancelled));
    assert_eq!(job.job_id, "cancel-job");
    assert!(job.output_paths.is_empty());
    assert!(
        !chosen.exists(),
        "cancelled render must not leave a final output"
    );
    no_tmp_litter(tmp.path());
}

#[test]
fn render_job_registry_cancel_is_safe_for_unknown_and_finished_jobs() {
    let registry = RenderJobRegistry::default();

    registry.cancel("missing-job");
    let flag = registry.register("known-job".to_string());
    assert!(!flag.load(Ordering::SeqCst));

    registry.cancel("known-job");
    assert!(flag.load(Ordering::SeqCst));

    registry.remove("known-job");
    registry.cancel("known-job");
    registry.cancel("missing-job");
}

#[test]
fn render_job_registry_keeps_overlapping_job_ids_independent() {
    let registry = RenderJobRegistry::default();
    let first = registry.register("render-a".to_string());
    let second = registry.register("render-b".to_string());

    registry.cancel("render-a");

    assert!(first.load(Ordering::SeqCst));
    assert!(
        !second.load(Ordering::SeqCst),
        "cancelling one overlapping render must not cancel another"
    );
    registry.remove("render-a");
    registry.remove("render-b");
}

#[cfg(windows)]
#[test]
fn source_file_locked_by_another_process_errors_cleanly() {
    use std::os::windows::fs::OpenOptionsExt;
    let tmp = TempDir::new().expect("tempdir");
    let src = tmp.path().join("locked.wav");
    write_sine_wav(&src, 44_100, 0.4, 330.0, 2);

    // Exclusive open (share_mode 0): any other open of this file fails —
    // the classic "file is open in another program" situation.
    let _lock = std::fs::OpenOptions::new()
        .read(true)
        .share_mode(0)
        .open(&src)
        .expect("exclusive open");

    let err = mastering_render(
        TrackId("locked-src".to_string()),
        &src,
        &settings(),
        tmp.path(),
        RenderKind::Master,
    )
    .expect_err("locked source must produce a typed error");
    let msg = format!("{err:?}");
    assert!(
        msg.contains("Io") || msg.contains("Decode"),
        "locked-source failure must be typed, got: {msg}",
    );
    no_tmp_litter(tmp.path());
}
