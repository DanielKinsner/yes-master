//! U12 — private fixture and reference-runner output path correctness.
//!
//! These runners are the documented way to produce deep DSP evidence, and
//! before this file they had **no integration test at all** — every existing
//! test covered pure helpers, so nothing ever executed the runner end to end
//! with the output path the docs tell you to use.
//!
//! Path handling only. Nothing here asserts a DSP value, and nothing here may
//! change one.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard, OnceLock};

use yes_master_lib::fixture_matrix::run_manifest_path;
use yes_master_lib::reference_tuning::run_reference_tuning_dir;

/// `set_current_dir` is process-global, so the relative-path tests must not
/// interleave. Cargo runs tests in a binary on parallel threads.
fn cwd_lock() -> MutexGuard<'static, ()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

struct CwdGuard {
    previous: PathBuf,
    _lock: MutexGuard<'static, ()>,
}

impl CwdGuard {
    fn enter(dir: &Path) -> Self {
        let lock = cwd_lock();
        let previous = std::env::current_dir().expect("current dir");
        std::env::set_current_dir(dir).expect("set current dir");
        Self {
            previous,
            _lock: lock,
        }
    }
}

impl Drop for CwdGuard {
    fn drop(&mut self) {
        let _ = std::env::set_current_dir(&self.previous);
    }
}

fn temp_root(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "yes-master-u12-{tag}-{}-{:?}",
        std::process::id(),
        std::thread::current().id()
    ));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("create temp root");
    dir
}

fn write_sine_wav(path: &Path, sample_rate: u32, duration_sec: f32, freq: f32) {
    let spec = hound::WavSpec {
        channels: 2,
        sample_rate,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let mut writer = hound::WavWriter::create(path, spec).expect("wav create");
    let n = (sample_rate as f32 * duration_sec) as u32;
    let amplitude = (0.5 * i16::MAX as f32) as i16;
    for i in 0..n {
        let t = i as f32 / sample_rate as f32;
        let s = (t * 2.0 * std::f32::consts::PI * freq).sin();
        let sample = (s * amplitude as f32) as i16;
        writer.write_sample(sample).expect("write sample");
        writer.write_sample(sample).expect("write sample");
    }
    writer.finalize().expect("wav finalize");
}

/// Build `<root>/fixtures/{source.wav, manifest.json}` and return the manifest.
fn write_fixture_manifest(root: &Path) -> PathBuf {
    let fixtures = root.join("fixtures");
    fs::create_dir_all(&fixtures).expect("create fixtures dir");
    write_sine_wav(&fixtures.join("source.wav"), 44_100, 2.0, 220.0);

    let manifest = fixtures.join("manifest.json");
    fs::write(
        &manifest,
        r#"{
  "version": 1,
  "notes": "U12 path test fixture",
  "fixtures": [
    { "id": "u12-source", "path": "source.wav", "mode": ["track"] }
  ]
}
"#,
    )
    .expect("write manifest");
    manifest
}

/// Build `<root>/references/` with the `*-original-test.wav` source plus the
/// four preset reference masters `discover_reference_suite` requires.
fn write_reference_suite(root: &Path) -> PathBuf {
    let references = root.join("references");
    fs::create_dir_all(&references).expect("create references dir");
    write_sine_wav(
        &references.join("u12track-original-test.wav"),
        44_100,
        2.0,
        220.0,
    );
    for preset in ["universal", "clarity", "oomph", "tape"] {
        write_sine_wav(
            &references.join(format!("u12track-{preset}-test.wav")),
            44_100,
            2.0,
            220.0,
        );
    }
    references
}

// ---------------------------------------------------------------------------
// The documented invocations must work without path rewriting.
// ---------------------------------------------------------------------------

#[test]
fn fixture_matrix_accepts_the_documented_relative_output_dir() {
    // docs/TESTING.md and docs/PRIVATE_AUDIO_FIXTURES.md both tell the owner to
    // run this from `src-tauri` with `--output ..\test-output\...`. The runner
    // must therefore accept a relative output that climbs OUT of the crate.
    let root = temp_root("matrix-relative");
    let manifest = write_fixture_manifest(&root);
    let workdir = root.join("workdir");
    fs::create_dir_all(&workdir).expect("create workdir");

    let _cwd = CwdGuard::enter(&workdir);
    let result = run_manifest_path(&manifest, Path::new("../evidence/matrix"))
        .expect("documented relative output must be accepted");

    assert!(
        result.report_path.is_absolute(),
        "report path must be absolute so evidence is unambiguous, got {}",
        result.report_path.display()
    );
    assert!(result.report_path.is_file(), "JSON report was not written");
    assert!(result.csv_path.is_file(), "CSV report was not written");
    assert!(result.render_dir.is_dir(), "render dir was not created");
    assert!(!result.rows.is_empty(), "no ledger rows produced");

    // The relative output must land where the caller meant: a sibling of the
    // working directory, not inside it.
    let expected = root.join("evidence").join("matrix");
    assert_eq!(
        fs::canonicalize(&expected).expect("canonicalize expected"),
        fs::canonicalize(result.report_path.parent().expect("report parent"))
            .expect("canonicalize actual"),
    );

    // Every rendered output must sit under the requested output dir, not beside
    // the private source.
    for row in &result.rows {
        let output = PathBuf::from(&row.output_path);
        assert!(
            output.starts_with(&result.render_dir),
            "render escaped the output dir: {}",
            output.display()
        );
    }
}

#[test]
fn reference_tuning_accepts_the_documented_relative_output_dir() {
    let root = temp_root("reference-relative");
    let references = write_reference_suite(&root);
    let workdir = root.join("workdir");
    fs::create_dir_all(&workdir).expect("create workdir");

    let _cwd = CwdGuard::enter(&workdir);
    let result = run_reference_tuning_dir(&references, Path::new("../evidence/reference"))
        .expect("documented relative output must be accepted");

    assert!(result.report_path.is_absolute());
    assert!(result.report_path.is_file(), "JSON report was not written");
    assert!(result.csv_path.is_file(), "CSV report was not written");
    assert!(!result.rows.is_empty(), "no comparison rows produced");
}

// ---------------------------------------------------------------------------
// Unsafe destinations stay rejected — and are rejected BEFORE anything is
// created on disk.
// ---------------------------------------------------------------------------

#[test]
fn fixture_matrix_rejects_an_output_dir_inside_the_private_source_dir() {
    // Writing renders inside the private fixture directory is how private audio
    // ends up somewhere it was never meant to be. The manifest's own directory
    // is off limits as an output destination.
    let root = temp_root("matrix-inside-source");
    let manifest = write_fixture_manifest(&root);
    let inside = manifest.parent().expect("manifest parent").join("renders");

    let error = run_manifest_path(&manifest, &inside)
        .expect_err("an output dir inside the private source dir must be rejected");
    let message = format!("{error}");
    assert!(
        message.contains("source"),
        "rejection must explain itself, got: {message}"
    );
    assert!(
        !inside.exists(),
        "the rejected directory must NOT have been created: {}",
        inside.display()
    );
}

#[test]
fn fixture_matrix_rejects_an_output_dir_equal_to_the_private_source_dir() {
    let root = temp_root("matrix-equals-source");
    let manifest = write_fixture_manifest(&root);
    let source_dir = manifest.parent().expect("manifest parent").to_path_buf();

    let error = run_manifest_path(&manifest, &source_dir)
        .expect_err("output dir equal to the source dir must be rejected");
    assert!(format!("{error}").contains("source"));
}

#[test]
fn reference_tuning_rejects_an_output_dir_inside_the_reference_dir() {
    let root = temp_root("reference-inside-source");
    let references = write_reference_suite(&root);
    let inside = references.join("out");

    let error = run_reference_tuning_dir(&references, &inside)
        .expect_err("an output dir inside the reference dir must be rejected");
    assert!(format!("{error}").contains("source"));
    assert!(
        !inside.exists(),
        "the rejected directory must NOT have been created"
    );
}

#[test]
fn fixture_matrix_rejects_traversal_past_the_filesystem_root() {
    let root = temp_root("matrix-traversal");
    let manifest = write_fixture_manifest(&root);
    let workdir = root.join("workdir");
    fs::create_dir_all(&workdir).expect("create workdir");

    let _cwd = CwdGuard::enter(&workdir);
    // Enough `..` to climb out of any real path. After normalization the
    // leftover parent components prove the request escaped the root.
    let escaping = Path::new("../../../../../../../../../../../../../../evidence");
    let error = run_manifest_path(manifest.as_path(), escaping)
        .expect_err("a path that climbs past the filesystem root must be rejected");
    let message = format!("{error}");
    assert!(
        message.contains("escapes") || message.contains("traversal"),
        "rejection must name the traversal, got: {message}"
    );
}

#[test]
fn fixture_matrix_rejects_an_empty_output_dir() {
    let root = temp_root("matrix-empty-output");
    let manifest = write_fixture_manifest(&root);

    let error = run_manifest_path(&manifest, Path::new(""))
        .expect_err("an empty output dir must be rejected");
    assert!(format!("{error}").contains("empty"));
}
