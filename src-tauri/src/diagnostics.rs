//! Local-first diagnostics: a small rotating on-disk log plus a one-file
//! plain-text report the user can save and send with a bug report.
//!
//! Privacy contract: nothing here ever leaves the machine on its own. There
//! is no telemetry, no network, no background upload — the ONLY way log
//! content moves anywhere is the user explicitly saving a report file and
//! choosing to share it. The report says so in its header, and it may contain
//! file paths from this machine (that's what makes it useful for debugging).
//!
//! Logging discipline: error paths and lifecycle milestones only — never
//! per-sample/per-tick work, so the audio thread and DSP hot paths stay
//! untouched. Every I/O failure inside the logger is swallowed; diagnostics
//! must never take the app down or change behavior.

use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};

use crate::types::{now_iso, CommandError, CommandResult};

const LOG_FILENAME: &str = "yes-master.log";
const ROTATED_LOG_FILENAME: &str = "yes-master.1.log";
/// Rotate when the active log passes ~1 MiB; with one archive kept the
/// on-disk cap is ~2 MiB worst case.
const ROTATE_AT_BYTES: u64 = 1_048_576;
/// Per-file cap quoted into a report, from the END of the file (newest
/// lines win). Keeps reports comfortably attachable.
const REPORT_TAIL_BYTES: u64 = 200_000;
/// session.json cap inside a report.
const REPORT_SESSION_BYTES: u64 = 100_000;

/// The directory the logger appends into, set once at app startup.
/// `None` until `init` — logging before init is a silent no-op (the
/// engine crate is also compiled by the mobile bridges, which never init).
static LOG_DIR: OnceLock<Mutex<PathBuf>> = OnceLock::new();

/// Point the logger at `dir` (created if missing). Second and later calls
/// are ignored — the first init wins for the process lifetime.
pub fn init(dir: PathBuf) {
    let _ = fs::create_dir_all(&dir);
    let _ = LOG_DIR.set(Mutex::new(dir));
}

pub fn log_dir() -> Option<PathBuf> {
    LOG_DIR.get().and_then(|m| m.lock().ok().map(|d| d.clone()))
}

pub fn info(message: impl AsRef<str>) {
    write_line("INFO", message.as_ref());
}

pub fn warn(message: impl AsRef<str>) {
    write_line("WARN", message.as_ref());
}

pub fn error(message: impl AsRef<str>) {
    write_line("ERROR", message.as_ref());
}

/// Log panics (message + location) before the default hook prints to a
/// stderr nobody can see in a packaged build. Chains to the previous hook
/// so debug behavior is unchanged.
pub fn install_panic_hook() {
    let previous = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        error(format!("PANIC: {panic_info}"));
        previous(panic_info);
    }));
}

fn write_line(level: &str, message: &str) {
    #[cfg(debug_assertions)]
    eprintln!("[{level}] {message}");
    let Some(dir) = log_dir() else { return };
    let Some(guard) = LOG_DIR.get().and_then(|m| m.lock().ok()) else {
        return;
    };
    rotate_if_needed(&dir, ROTATE_AT_BYTES);
    let line = format!("{} [{level}] {message}\n", now_iso());
    if let Ok(mut file) = OpenOptions::new()
        .create(true)
        .append(true)
        .open(dir.join(LOG_FILENAME))
    {
        let _ = file.write_all(line.as_bytes());
    }
    drop(guard);
}

/// If the active log has grown past `limit`, shift it to the single archive
/// slot (replacing any prior archive). Failures are ignored — worst case the
/// log keeps growing until a later rotation succeeds.
fn rotate_if_needed(dir: &Path, limit: u64) {
    let active = dir.join(LOG_FILENAME);
    let size = fs::metadata(&active).map(|m| m.len()).unwrap_or(0);
    if size <= limit {
        return;
    }
    let archive = dir.join(ROTATED_LOG_FILENAME);
    let _ = fs::remove_file(&archive);
    let _ = fs::rename(&active, &archive);
}

/// Last `max_bytes` of `path` as lossy UTF-8, or a marker line when the file
/// is missing/unreadable. Reading the tail (not the head) keeps the newest
/// lines when a file exceeds the cap.
fn tail_of_file(path: &Path, max_bytes: u64) -> String {
    match fs::read(path) {
        Ok(bytes) => {
            let start = bytes.len().saturating_sub(max_bytes as usize);
            let mut text = String::from_utf8_lossy(&bytes[start..]).into_owned();
            if start > 0 {
                text = format!("[... truncated to the last {max_bytes} bytes ...]\n{text}");
            }
            text
        }
        Err(_) => "(not present)".to_string(),
    }
}

/// Assemble the plain-text diagnostics report. Pure over the directories so
/// it's testable without a Tauri runtime; the command layer supplies the real
/// app-data dir.
pub fn build_report(app_data_dir: &Path) -> String {
    let logs_dir = app_data_dir.join("logs");
    let mut report = String::new();
    report.push_str("YES Master — diagnostics report\n");
    report.push_str("================================\n");
    report.push_str(&format!("generated: {}\n", now_iso()));
    report.push_str(&format!("app version: {}\n", env!("CARGO_PKG_VERSION")));
    report.push_str(&format!(
        "os: {} ({})\n",
        std::env::consts::OS,
        std::env::consts::ARCH
    ));
    report.push_str(&format!("app data dir: {}\n", app_data_dir.display()));
    report.push_str(
        "\nNOTE: this file contains log lines and file paths from this machine.\n\
         Nothing was sent anywhere — share it only if you choose to.\n",
    );

    report.push_str("\n---- log (current) ----\n");
    report.push_str(&tail_of_file(
        &logs_dir.join(LOG_FILENAME),
        REPORT_TAIL_BYTES,
    ));
    report.push_str("\n---- log (previous rotation) ----\n");
    report.push_str(&tail_of_file(
        &logs_dir.join(ROTATED_LOG_FILENAME),
        REPORT_TAIL_BYTES,
    ));
    report.push_str("\n---- session.json ----\n");
    report.push_str(&tail_of_file(
        &app_data_dir.join("session.json"),
        REPORT_SESSION_BYTES,
    ));
    report.push('\n');
    report
}

/// Build the report and write it to `target_path`. Split from the command so
/// the write path (traversal guard included) is testable without a Tauri app.
pub fn write_report_to(app_data_dir: &Path, target_path: &str) -> CommandResult<String> {
    if target_path.trim().is_empty() {
        return Err(CommandError::InvalidPath("empty target path".to_string()));
    }
    let target = Path::new(target_path);
    if crate::files::has_parent_dir_component(target) {
        return Err(CommandError::InvalidPath(format!(
            "path traversal not allowed: {target_path}"
        )));
    }
    let report = build_report(app_data_dir);
    fs::write(target, report).map_err(|e| CommandError::Io(e.to_string()))?;
    info(format!("diagnostics report saved to {target_path}"));
    Ok(target_path.to_string())
}

/// Save a diagnostics report to a user-chosen path (the frontend supplies it
/// from a save dialog, which already confirmed any overwrite).
#[tauri::command]
pub async fn save_diagnostics_report(
    target_path: String,
    app: tauri::AppHandle,
) -> CommandResult<String> {
    use tauri::Manager;
    let app_data = app
        .path()
        .app_data_dir()
        .map_err(|e| CommandError::Other(format!("app data dir unavailable: {e}")))?;
    write_report_to(&app_data, &target_path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rotation_shifts_the_active_log_into_the_single_archive_slot() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path();
        fs::write(dir.join(LOG_FILENAME), b"old old old").unwrap();
        fs::write(dir.join(ROTATED_LOG_FILENAME), b"ancient").unwrap();

        // Under the limit: nothing moves.
        rotate_if_needed(dir, 1024);
        assert_eq!(fs::read(dir.join(LOG_FILENAME)).unwrap(), b"old old old");

        // Over the limit: active becomes the archive (replacing the prior
        // archive), and the next write recreates the active file.
        rotate_if_needed(dir, 4);
        assert!(!dir.join(LOG_FILENAME).exists());
        assert_eq!(
            fs::read(dir.join(ROTATED_LOG_FILENAME)).unwrap(),
            b"old old old"
        );
    }

    #[test]
    fn tail_of_file_keeps_the_newest_bytes_and_marks_truncation() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("t.log");
        fs::write(&path, b"AAAABBBB").unwrap();

        assert_eq!(tail_of_file(&path, 100), "AAAABBBB");
        let tail = tail_of_file(&path, 4);
        assert!(tail.ends_with("BBBB"), "got {tail}");
        assert!(tail.contains("truncated"), "got {tail}");
        assert_eq!(
            tail_of_file(&tmp.path().join("missing.log"), 4),
            "(not present)"
        );
    }

    #[test]
    fn report_carries_header_logs_and_session_with_missing_files_marked() {
        let tmp = tempfile::tempdir().unwrap();
        let app_data = tmp.path();
        fs::create_dir_all(app_data.join("logs")).unwrap();
        fs::write(
            app_data.join("logs").join(LOG_FILENAME),
            b"2026-07-04T00:00:00Z [ERROR] decode exploded\n",
        )
        .unwrap();

        let report = build_report(app_data);

        assert!(report.contains(env!("CARGO_PKG_VERSION")));
        assert!(report.contains("decode exploded"));
        assert!(
            report.contains("Nothing was sent anywhere"),
            "the privacy note is part of the contract"
        );
        // Missing rotated log + session must be marked, not omitted or fatal.
        assert!(report.contains("(not present)"));
        assert!(report.contains("session.json"));
    }

    #[test]
    fn write_report_rejects_traversal_and_writes_real_targets() {
        let tmp = tempfile::tempdir().unwrap();
        let app_data = tmp.path().join("appdata");
        fs::create_dir_all(app_data.join("logs")).unwrap();

        let escaping = tmp.path().join("sub").join("..").join("out.txt");
        let outcome = write_report_to(&app_data, escaping.to_str().unwrap());
        assert!(matches!(outcome, Err(CommandError::InvalidPath(_))));

        let target = tmp.path().join("report.txt");
        let written = write_report_to(&app_data, target.to_str().unwrap()).unwrap();
        assert_eq!(written, target.to_str().unwrap());
        let content = fs::read_to_string(&target).unwrap();
        assert!(content.contains("diagnostics report"));
    }

    /// The single test that touches the process-global sink (parallel tests
    /// must not race init with a second directory).
    #[test]
    fn global_logger_appends_leveled_timestamped_lines_after_init() {
        let tmp = tempfile::tempdir().unwrap();
        init(tmp.path().to_path_buf());
        info("hello from the smoke test");
        error("and an error line");

        let dir = log_dir().expect("initialized");
        let content = fs::read_to_string(dir.join(LOG_FILENAME)).unwrap_or_default();
        assert!(
            content.contains("[INFO] hello from the smoke test"),
            "got {content}"
        );
        assert!(content.contains("[ERROR] and an error line"));
        // Every line starts with an ISO timestamp (rough shape check).
        assert!(content.starts_with("20"), "got {content}");
    }
}
