//! Portability pins for real-world path shapes (blind-spot review,
//! 2026-07-04): users keep music on NAS shares (UNC), inside deeply nested
//! folders (beyond the legacy 260-char MAX_PATH), and sessions can carry
//! canonicalized verbatim paths (`\\?\C:\...`). None of these shapes had a
//! single test before this file.
//!
//! Also pins the verbatim-`..` subtleties this battery surfaced on its
//! first run: `std` parses `..` inside a VERBATIM path as a *normal*
//! component (invisible to a ParentDir-only guard), `PathBuf::join("..")`
//! onto a verbatim base lexically POPS the component inside std itself,
//! and the OS refuses to open a raw verbatim-`..` path (error 123). The
//! shared guard now rejects the raw spelling upfront (typed error instead
//! of OS noise; holds if OS behavior ever changes).
//!
//! True cloud placeholders (OneDrive Files-On-Demand hydration stalls)
//! cannot be constructed in a unit test without a cloud provider; that
//! remains an owner exploratory test (docs/OWNER_SMOKE_TEST.html #13) and
//! an open gap recorded in APP_BEHAVIOR.md.

#[cfg(windows)]
use hound::{SampleFormat, WavSpec, WavWriter};
use std::path::Path;
#[cfg(windows)]
use std::path::PathBuf;
#[cfg(windows)]
use tempfile::TempDir;
use yes_master_lib::decode::decode_full;
use yes_master_lib::files::has_parent_dir_component;
use yes_master_lib::types::CommandError;

#[cfg(windows)]
fn write_stereo_sine_wav(path: &PathBuf, sample_rate: u32, duration_sec: f32, amplitude: f32) {
    let spec = WavSpec {
        channels: 2,
        sample_rate,
        bits_per_sample: 16,
        sample_format: SampleFormat::Int,
    };
    let mut writer = WavWriter::create(path, spec).expect("create wav");
    let n_frames = (sample_rate as f32 * duration_sec) as u32;
    let omega = 2.0 * std::f32::consts::PI * 1_000.0 / sample_rate as f32;
    for i in 0..n_frames {
        let v = amplitude * (omega * i as f32).sin();
        let s = (v.clamp(-1.0, 1.0) * 32_767.0).round() as i16;
        writer.write_sample(s).expect("write L");
        writer.write_sample(s).expect("write R");
    }
    writer.finalize().expect("finalize source wav");
}

#[cfg(windows)]
fn render_settings() -> yes_master_lib::types::MasteringSettings {
    use yes_master_lib::types::{AdvancedSettings, DeliveryProfile, MasteringSettings, Preset};
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
        delivery_profile: DeliveryProfile::StreamingUniversal,
        album: None,
        advanced: AdvancedSettings::default(),
    }
}

/// UNC share paths (`\\server\share\track.wav`) have no `..` components and
/// must NOT be rejected by the traversal guard — NAS users are legitimate.
/// An unreachable share then fails as a typed, file-naming error at decode,
/// never a panic or a hang.
#[test]
fn unc_share_paths_pass_the_guard_and_fail_typed_when_unreachable() {
    let unc = Path::new(r"\\yes-master-no-such-server\share\track.wav");
    assert!(
        !has_parent_dir_component(unc),
        "a plain UNC path must not be mistaken for traversal"
    );

    let err = decode_full(unc).expect_err("unreachable share cannot decode");
    assert!(
        matches!(err, CommandError::Io(_) | CommandError::Decode(_)),
        "unreachable UNC source must be a typed Io/Decode error, got {err:?}"
    );
}

/// `canonicalize()` on Windows returns VERBATIM paths (`\\?\C:\...`), and a
/// stored session or dialog result can carry one. The whole pipeline —
/// decode and render — must treat them exactly like their plain spelling.
#[cfg(windows)]
#[test]
fn verbatim_local_paths_decode_and_render() {
    use yes_master_lib::engine::mastering_render;
    use yes_master_lib::types::{RenderKind, TrackId};

    let tmp = TempDir::new().expect("tempdir");
    let plain = tmp.path().join("source.wav");
    write_stereo_sine_wav(&plain, 44_100, 1.0, 0.4);

    let verbatim = plain.canonicalize().expect("canonicalize");
    assert!(
        verbatim.to_string_lossy().starts_with(r"\\?\"),
        "premise: canonicalize yields a verbatim path, got {}",
        verbatim.display()
    );

    let decoded = decode_full(&verbatim).expect("verbatim path must decode");
    assert!(!decoded.samples.is_empty());

    let out_dir = tmp.path().join("rendered");
    std::fs::create_dir_all(&out_dir).expect("out dir");
    let job = mastering_render(
        TrackId::new(),
        &verbatim,
        &render_settings(),
        &out_dir,
        RenderKind::Master,
    )
    .expect("verbatim path must render");
    let output = PathBuf::from(&job.output_paths[0]);
    assert!(output.exists(), "rendered WAV missing");
    hound::WavReader::open(&output).expect("rendered WAV must reopen");
}

/// Sources nested beyond the legacy 260-char MAX_PATH must import, decode,
/// and render — deep artist/album/version folder trees hit this in real
/// libraries. (Rust std converts absolute paths to verbatim internally, so
/// this must work regardless of the registry LongPathsEnabled flag.)
#[cfg(windows)]
#[test]
fn paths_beyond_the_legacy_max_path_decode_and_render() {
    use yes_master_lib::engine::mastering_render;
    use yes_master_lib::types::{RenderKind, TrackId};

    let tmp = TempDir::new().expect("tempdir");
    let mut deep = tmp.path().to_path_buf();
    while deep.as_os_str().len() < 280 {
        deep.push("very-long-nested-album-folder-segment-0123456789");
    }
    std::fs::create_dir_all(&deep).expect("create long dir tree");

    let src = deep.join("track with spaces and a long name.wav");
    assert!(src.as_os_str().len() > 260, "premise: exceeds MAX_PATH");
    write_stereo_sine_wav(&src, 44_100, 1.0, 0.4);

    let decoded = decode_full(&src).expect("long path must decode");
    assert!(!decoded.samples.is_empty());

    let out_dir = deep.join("rendered");
    std::fs::create_dir_all(&out_dir).expect("long out dir");
    let job = mastering_render(
        TrackId::new(),
        &src,
        &render_settings(),
        &out_dir,
        RenderKind::Master,
    )
    .expect("long path must render");
    assert!(PathBuf::from(&job.output_paths[0]).exists());
}

/// What this battery caught on its first run (2026-07-04): `std` parses
/// `..` inside a VERBATIM path (`\\?\C:\...`) as a Normal component —
/// never `ParentDir` — so the ParentDir-only guard passed a raw string
/// like `\\?\C:\x\..\y` through every `..`-guarded command (§15 class:
/// import, render output_dir, waveform, prewarm, project save/load). The
/// OS then refuses to open it (error 123), so the exposure was a confusing
/// OS error rather than a real escape — but the guard now rejects the raw
/// spelling upfront: typed, honest, and robust to OS-behavior drift.
#[cfg(windows)]
#[test]
fn dot_dot_inside_verbatim_paths_is_rejected_by_the_guard() {
    let tmp = TempDir::new().expect("tempdir");
    let escape_target = tmp.path().join("outside.wav");
    write_stereo_sine_wav(&escape_target, 44_100, 0.3, 0.3);
    let sub = tmp.path().join("sub");
    std::fs::create_dir_all(&sub).expect("subdir");

    // Non-verbatim spelling: ParentDir component, guard fires.
    let plain_dotdot = sub.join("..").join("outside.wav");
    assert!(has_parent_dir_component(&plain_dotdot));

    // A RAW verbatim string carrying `..` — the IPC attack shape. (A
    // fixture built with `join("..")` would test nothing: std lexically
    // POPS a `..` pushed onto a verbatim base, so the traversal resolves
    // inside PathBuf before any guard or syscall could see it.)
    let canonical_sub = sub.canonicalize().expect("canonicalize sub");
    let raw = format!("{}\\..\\outside.wav", canonical_sub.display());
    let verbatim_dotdot = PathBuf::from(&raw);
    assert!(raw.starts_with(r"\\?\"), "premise: verbatim spelling");
    // Primary defense: the shared guard rejects the raw spelling.
    assert!(
        has_parent_dir_component(&verbatim_dotdot),
        "guard must reject the raw verbatim spelling of a `..` traversal"
    );
    // Backstop pin: the OS refuses to open it (error 123 today). If this
    // ever starts succeeding, Windows began normalizing raw verbatim `..`
    // — the guard above still protects, but we want to know.
    assert!(
        escape_target.exists(),
        "premise: the would-be escape target exists"
    );
    decode_full(&verbatim_dotdot).expect_err("the OS must not resolve a raw verbatim `..` path");
}
