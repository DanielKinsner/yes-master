//! Smart Balance reference measurement (execution plan Slice 1).
//!
//! A local-only evidence runner. It measures a folder of released songs the
//! owner trusts and turns them into the **healthy range** Smart Balance will
//! correct toward. It changes no product behavior.
//!
//! Two outputs, with different privacy rules:
//!
//! - a **private ledger** (per-file paths, numbers and skip reasons) that stays
//!   under the git-ignored `test-output/`;
//! - an **aggregate statistics file** that holds numbers and anonymous hash
//!   prefixes only, so it is safe to commit.
//!
//! **Same meter as the app.** Each song's six band shares come from
//! [`crate::analysis::compute_spectral_balance_6band`], the exact function the
//! app's analysis calls (pinned by `measures_through_the_same_meter_as_analyze_one`).
//! A second implementation would make the range describe a different meter.
//!
//! **Deviation space.** The range is expressed where the decision function
//! will compare: each band in dB, minus the library's typical shape (the
//! per-band median), minus that song's own median across the six bands. The
//! last step removes the overall offset robustly, so one loud band does not
//! make the other five look quiet.
//!
//! **Determinism.** The same folder gives byte-identical statistics: files are
//! visited in sorted order, the split and bootstrap use a recorded seed, and the
//! statistics carry no timestamps or machine paths.
//!
//! **Held-out material stays out.** `exclude_matching` names folders (the test
//! corpus) whose files must not shape the range. A library file with the same
//! bytes as any file there is skipped with a logged reason.
//!
//! **Placement (Slice 2).** [`run_placement`] measures other songs through the
//! same meter and reports where they sit against an existing range. It never
//! changes the range.

use crate::analysis::compute_spectral_balance_6band;
use crate::decode::decode_full;
use crate::evidence_lanes::{csv_escape, normalized_absolute_path, prepare_evidence_output_dir};
use crate::types::{CommandError, CommandResult, SpectralBalance6};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;

/// Machine names of the six bands, in meter order.
pub const BAND_KEYS: [&str; 6] = ["sub", "bass", "low_mids", "mids", "presence", "air"];
/// Band edges of the meter, in Hz. The top edge is `min(Nyquist, 16 kHz)`.
pub const BAND_EDGES_HZ: [f64; 7] = [20.0, 80.0, 250.0, 800.0, 2500.0, 6500.0, 16_000.0];

/// Identifies the statistics file format.
pub const STATS_SCHEMA: &str = "yes-master.smart-balance.reference-stats.v1";
/// Identifies the meter the statistics describe.
pub const METER_ID: &str = "analysis::compute_spectral_balance_6band";

/// Songs shorter than this are skipped (plan Slice 1).
pub const MIN_DURATION_SEC: f64 = 30.0;
/// Below this rate the air band (6.5–16 kHz) is truncated by Nyquist, so the
/// song would bias the air statistic. Released masters are 44.1 kHz or more.
pub const MIN_SAMPLE_RATE_HZ: u32 = 32_000;
/// A file whose sample peak never reaches this level is treated as silent.
pub const SILENCE_PEAK_DBFS: f64 = -60.0;
/// The seed recorded with every run unless the caller supplies another.
pub const DEFAULT_SEED: u64 = 20_261_124;
/// Share of accepted songs held back from the range (plan Slice 2: 75/25).
pub const HOLDOUT_FRACTION: f64 = 0.25;
/// Range half-width multiplier on the robust spread (plan D16, parameter table).
pub const RANGE_SPREAD_MULTIPLIER: f64 = 2.0;
/// Minimum range half-width in dB (plan D16, parameter table).
pub const RANGE_MIN_HALF_WIDTH_DB: f64 = 1.5;
/// Scale that makes the median absolute deviation estimate a normal sigma.
pub const MAD_TO_SIGMA: f64 = 1.4826;
/// Bootstrap resamples for edge stability.
pub const BOOTSTRAP_RESAMPLES: usize = 1000;
/// A bootstrap 90% interval wider than this marks an edge as unstable.
pub const UNSTABLE_EDGE_INTERVAL_DB: f64 = 1.0;
/// Shares are floored here before conversion to dB (−90 dB).
pub const SHARE_FLOOR: f64 = 1.0e-9;
/// The plan's minimum library size. Fewer accepted songs is a stop trigger.
pub const MIN_LIBRARY_SONGS: usize = 40;

/// File extensions the runner tries to decode. Anything else is "not audio".
const AUDIO_EXTENSIONS: [&str; 12] = [
    "wav", "wave", "flac", "mp3", "m4a", "mp4", "aac", "aif", "aiff", "aifc", "ogg", "oga",
];
/// Length of the anonymous ID written to the committable statistics.
const ANON_ID_HEX_LEN: usize = 10;
/// Upper bound on decode workers (each holds one decoded song in memory).
const MAX_WORKERS: usize = 8;

/// Tunables for one run. `Default` is the plan's configuration; tests lower
/// `min_duration_sec` so synthetic fixtures stay small.
#[derive(Debug, Clone, PartialEq)]
pub struct ReferenceRunConfig {
    pub seed: u64,
    pub min_duration_sec: f64,
    pub bootstrap_resamples: usize,
    /// Recorded in the statistics: the range is a stand-in, to be re-measured
    /// (plan decision D8 as changed on 2026-11-24).
    pub provisional: bool,
    /// Folders whose files must not shape the range. A library file with the
    /// same bytes as any file in them is skipped as
    /// [`SkipReason::MatchesExcluded`].
    pub exclude_matching: Vec<PathBuf>,
}

impl Default for ReferenceRunConfig {
    fn default() -> Self {
        Self {
            seed: DEFAULT_SEED,
            min_duration_sec: MIN_DURATION_SEC,
            bootstrap_resamples: BOOTSTRAP_RESAMPLES,
            provisional: false,
            exclude_matching: Vec::new(),
        }
    }
}

/// Which side of the seeded split a song landed on.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SplitSide {
    Build,
    Holdout,
}

/// Why a file was not measured. Every skip is logged; none fails the run.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum SkipReason {
    NotAudio,
    Unreadable,
    Undecodable,
    TooShort,
    LowSampleRate,
    Silent,
    Duplicate,
    /// Same bytes as a file in an `exclude_matching` folder.
    MatchesExcluded,
}

/// One accepted song. Private: it carries the relative path.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ReferenceSongRecord {
    /// Full 64-bit content hash, hex. Its prefix is the anonymous ID.
    pub content_hash: String,
    pub anon_id: String,
    pub relative_path: String,
    pub duration_sec: f64,
    pub sample_rate: u32,
    pub channels: u16,
    pub peak_dbfs: f64,
    /// Band shares exactly as the app's meter returns them (sum ~1).
    pub shares: [f64; 6],
    /// `10·log10(max(share, SHARE_FLOOR))`.
    pub shares_db: [f64; 6],
    pub split: SplitSide,
    /// Position in deviation space against the build-split statistics.
    /// `None` when the build split is empty.
    pub deviation_db: Option<[f64; 6]>,
    /// Per band: true when inside the healthy range.
    pub inside_range: Option<[bool; 6]>,
}

/// One skipped file. Private: it carries the relative path.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SkippedReferenceFile {
    pub relative_path: String,
    pub reason: SkipReason,
    pub detail: String,
}

/// The private per-file ledger.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ReferenceLedger {
    pub schema: String,
    pub seed: u64,
    pub accepted: Vec<ReferenceSongRecord>,
    pub skipped: Vec<SkippedReferenceFile>,
}

/// Run parameters recorded in the statistics so a reader knows what produced
/// them (and so a test-mode run can never pass for a real one).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ReferenceStatsParameters {
    pub min_duration_sec: f64,
    pub min_sample_rate_hz: u32,
    pub silence_peak_dbfs: f64,
    pub holdout_fraction: f64,
    pub range_spread_multiplier: f64,
    pub range_min_half_width_db: f64,
    pub spread_estimator: String,
    pub offset_removal: String,
    pub share_floor: f64,
    pub bootstrap_resamples: usize,
    pub unstable_edge_interval_db: f64,
    pub minimum_library_songs: usize,
}

/// How many files took each path through the runner.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct ReferenceCounts {
    pub files_seen: usize,
    pub accepted: usize,
    pub build: usize,
    pub holdout: usize,
    pub skipped_not_audio: usize,
    pub skipped_unreadable: usize,
    pub skipped_undecodable: usize,
    pub skipped_too_short: usize,
    pub skipped_low_sample_rate: usize,
    pub skipped_silent: usize,
    pub skipped_duplicate: usize,
    #[serde(default)]
    pub skipped_matches_excluded: usize,
}

/// Bootstrap stability of one band's two range edges.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EdgeStability {
    pub low_edge_sd_db: f64,
    pub low_edge_p05_db: f64,
    pub low_edge_p95_db: f64,
    pub high_edge_sd_db: f64,
    pub high_edge_p05_db: f64,
    pub high_edge_p95_db: f64,
    /// True when either edge's 90% interval is wider than
    /// [`UNSTABLE_EDGE_INTERVAL_DB`].
    pub unstable: bool,
}

/// The healthy range of one band, in deviation space (dB).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BandRange {
    pub band: String,
    pub center_db: f64,
    pub robust_spread_db: f64,
    pub half_width_db: f64,
    pub low_edge_db: f64,
    pub high_edge_db: f64,
    /// Build-split spread for context (5th/95th percentile of deviation).
    pub build_p05_db: f64,
    pub build_p95_db: f64,
    /// Share of build songs inside this band's range.
    pub build_inside_share: f64,
    pub stability: Option<EdgeStability>,
}

/// The pure range computation's result: the library's typical shape plus
/// the per-band range.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RangeStats {
    /// Per-band median of `shares_db` across the build split.
    pub typical_shape_db: [f64; 6],
    pub bands: Vec<BandRange>,
    /// Share of build songs inside the range in all six bands at once.
    pub build_all_bands_inside_share: f64,
}

/// Seeded split membership by anonymous ID.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SplitMembership {
    pub method: String,
    pub build_ids: Vec<String>,
    pub holdout_ids: Vec<String>,
}

/// The committable aggregate. Numbers and anonymous IDs only.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ReferenceStats {
    pub schema: String,
    pub meter: String,
    pub bands: Vec<String>,
    pub band_edges_hz: Vec<f64>,
    pub seed: u64,
    pub parameters: ReferenceStatsParameters,
    pub counts: ReferenceCounts,
    pub meets_minimum_library_size: bool,
    /// True when the run was labelled provisional: the range is a stand-in
    /// that is expected to be re-measured.
    #[serde(default)]
    pub provisional: bool,
    /// `None` when the build split is empty.
    pub range: Option<RangeStats>,
    pub split: SplitMembership,
}

/// Where a run wrote its outputs.
#[derive(Debug, Clone)]
pub struct ReferenceRunResult {
    pub output_dir: PathBuf,
    pub stats_path: PathBuf,
    pub ledger_json_path: PathBuf,
    pub ledger_csv_path: PathBuf,
    pub stats: ReferenceStats,
    pub ledger: ReferenceLedger,
}

/// File names inside the output directory.
pub const STATS_FILE_NAME: &str = "smart-balance-reference-stats.json";
pub const LEDGER_JSON_FILE_NAME: &str = "smart-balance-reference-ledger.json";
pub const LEDGER_CSV_FILE_NAME: &str = "smart-balance-reference-ledger.csv";

/// Measure every song under `reference_dir` (recursively) and write the
/// private ledger plus the committable statistics into `output_dir`.
///
/// The output may not be the reference folder or inside it; that is checked
/// before anything is created (see `evidence_lanes::prepare_evidence_output_dir`).
pub fn run_reference_measurement(
    reference_dir: &Path,
    output_dir: &Path,
    config: &ReferenceRunConfig,
) -> CommandResult<ReferenceRunResult> {
    let cwd = std::env::current_dir().map_err(|e| CommandError::Io(format!("current dir: {e}")))?;
    let reference_root = normalized_absolute_path(&cwd, reference_dir);
    if !reference_root.is_dir() {
        return Err(CommandError::InvalidPath(format!(
            "reference folder does not exist: {}",
            reference_dir.display()
        )));
    }
    let mut exclude_roots = Vec::with_capacity(config.exclude_matching.len());
    for dir in &config.exclude_matching {
        let root = normalized_absolute_path(&cwd, dir);
        if !root.is_dir() {
            return Err(CommandError::InvalidPath(format!(
                "exclude folder does not exist: {}",
                dir.display()
            )));
        }
        exclude_roots.push(root);
    }
    // Neither the library nor an excluded folder may receive the output.
    let mut source_roots: Vec<&Path> = vec![reference_root.as_path()];
    source_roots.extend(exclude_roots.iter().map(PathBuf::as_path));
    let output_dir = prepare_evidence_output_dir(&cwd, output_dir, &source_roots)?;

    let excluded = excluded_hashes(&exclude_roots)?;
    let files = collect_files(&reference_root)?;
    let (ledger, stats) = measure_files(&reference_root, &files, config, &excluded);

    let stats_path = output_dir.join(STATS_FILE_NAME);
    let ledger_json_path = output_dir.join(LEDGER_JSON_FILE_NAME);
    let ledger_csv_path = output_dir.join(LEDGER_CSV_FILE_NAME);
    write_text(&stats_path, &stats_json(&stats)?)?;
    let ledger_json = serde_json::to_string_pretty(&ledger)
        .map_err(|e| CommandError::Other(format!("serialize reference ledger: {e}")))?;
    write_text(&ledger_json_path, &format!("{ledger_json}\n"))?;
    write_text(&ledger_csv_path, &ledger_csv(&ledger))?;

    Ok(ReferenceRunResult {
        output_dir,
        stats_path,
        ledger_json_path,
        ledger_csv_path,
        stats,
        ledger,
    })
}

/// Serialize statistics exactly as the runner writes them (pretty JSON plus
/// a trailing newline), so a committed copy can be compared byte for byte.
pub fn stats_json(stats: &ReferenceStats) -> CommandResult<String> {
    let json = serde_json::to_string_pretty(stats)
        .map_err(|e| CommandError::Other(format!("serialize reference stats: {e}")))?;
    Ok(format!("{json}\n"))
}

fn write_text(path: &Path, text: &str) -> CommandResult<()> {
    fs::write(path, text).map_err(|e| CommandError::Io(format!("write {}: {e}", path.display())))
}

/// Content hash of every file under the excluded folders, mapped to a label
/// for the private ledger (`<folder name>/<path inside it>`).
fn excluded_hashes(roots: &[PathBuf]) -> CommandResult<BTreeMap<u64, String>> {
    let mut out = BTreeMap::new();
    for root in roots {
        for path in collect_files(root)? {
            let bytes = fs::read(&path)
                .map_err(|e| CommandError::Io(format!("read {}: {e}", path.display())))?;
            out.entry(fnv1a64(&bytes, FNV_OFFSET))
                .or_insert_with(|| root_label(root, &path));
        }
    }
    Ok(out)
}

/// `<root folder name>/<path inside it>`, for runs that read several roots.
fn root_label(root: &Path, path: &Path) -> String {
    let name = root
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    format!("{name}/{}", relative_label(root, path))
}

/// Every regular file under `root`, sorted by its `/`-joined relative path.
/// Symbolic links are not followed, so a link loop cannot hang the run.
fn collect_files(root: &Path) -> CommandResult<Vec<PathBuf>> {
    let mut out = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let entries = fs::read_dir(&dir)
            .map_err(|e| CommandError::Io(format!("read dir {}: {e}", dir.display())))?;
        for entry in entries {
            let entry = entry.map_err(|e| CommandError::Io(format!("read dir entry: {e}")))?;
            let file_type = entry
                .file_type()
                .map_err(|e| CommandError::Io(format!("file type: {e}")))?;
            if file_type.is_dir() {
                stack.push(entry.path());
            } else if file_type.is_file() {
                out.push(entry.path());
            }
        }
    }
    out.sort_by_key(|path| relative_label(root, path));
    Ok(out)
}

fn relative_label(root: &Path, path: &Path) -> String {
    let relative = path.strip_prefix(root).unwrap_or(path);
    relative
        .components()
        .map(|c| c.as_os_str().to_string_lossy().into_owned())
        .collect::<Vec<_>>()
        .join("/")
}

fn is_audio_extension(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| {
            let ext = ext.to_ascii_lowercase();
            AUDIO_EXTENSIONS.contains(&ext.as_str())
        })
        .unwrap_or(false)
}

/// What measuring one file produced, before the split and duplicate pass.
enum Measured {
    Song(MeasuredSong),
    Skipped(SkipReason, String),
}

struct MeasuredSong {
    content_hash: u64,
    duration_sec: f64,
    sample_rate: u32,
    channels: u16,
    peak_dbfs: f64,
    shares: [f64; 6],
}

/// Measure one file. Never panics on bad input; every failure is a skip.
fn measure_one(
    path: &Path,
    config: &ReferenceRunConfig,
    excluded: &BTreeMap<u64, String>,
) -> Measured {
    if !is_audio_extension(path) {
        return Measured::Skipped(SkipReason::NotAudio, "not an audio file extension".into());
    }
    let bytes = match fs::read(path) {
        Ok(bytes) => bytes,
        Err(e) => return Measured::Skipped(SkipReason::Unreadable, e.to_string()),
    };
    let content_hash = fnv1a64(&bytes, FNV_OFFSET);
    drop(bytes);
    if let Some(label) = excluded.get(&content_hash) {
        return Measured::Skipped(
            SkipReason::MatchesExcluded,
            format!("same content as excluded {label}"),
        );
    }

    let pcm = match decode_full(path) {
        Ok(pcm) => pcm,
        Err(e) => return Measured::Skipped(SkipReason::Undecodable, e.to_string()),
    };
    let channels = usize::from(pcm.channels.max(1));
    if pcm.sample_rate == 0 || pcm.samples.is_empty() {
        return Measured::Skipped(SkipReason::Undecodable, "no samples decoded".into());
    }
    let frames = pcm.samples.len() / channels;
    let duration_sec = frames as f64 / f64::from(pcm.sample_rate);
    if duration_sec < config.min_duration_sec {
        return Measured::Skipped(
            SkipReason::TooShort,
            format!(
                "{duration_sec:.1} s is under the {:.0} s minimum",
                config.min_duration_sec
            ),
        );
    }
    if pcm.sample_rate < MIN_SAMPLE_RATE_HZ {
        return Measured::Skipped(
            SkipReason::LowSampleRate,
            format!(
                "{} Hz is under {MIN_SAMPLE_RATE_HZ} Hz, so the air band is incomplete",
                pcm.sample_rate
            ),
        );
    }
    let peak = pcm.samples.iter().fold(
        0.0_f32,
        |acc, s| if s.is_finite() { acc.max(s.abs()) } else { acc },
    );
    let peak_dbfs = if peak > 0.0 {
        20.0 * f64::from(peak).log10()
    } else {
        f64::NEG_INFINITY
    };
    if peak_dbfs < SILENCE_PEAK_DBFS {
        return Measured::Skipped(
            SkipReason::Silent,
            format!("peak never reaches {SILENCE_PEAK_DBFS:.0} dBFS"),
        );
    }
    // The app's exact meter, called with exactly the arguments `analyze_one`
    // passes it.
    let Some(balance) =
        compute_spectral_balance_6band(&pcm.samples, pcm.sample_rate, usize::from(pcm.channels))
    else {
        return Measured::Skipped(SkipReason::Silent, "meter found no energy".into());
    };
    let shares = shares_array(&balance);
    if shares.iter().any(|s| !s.is_finite() || *s < 0.0) {
        return Measured::Skipped(SkipReason::Silent, "meter returned invalid shares".into());
    }
    Measured::Song(MeasuredSong {
        content_hash,
        duration_sec,
        sample_rate: pcm.sample_rate,
        channels: pcm.channels,
        peak_dbfs,
        shares,
    })
}

/// The meter's six shares in band order, widened losslessly to f64.
pub fn shares_array(balance: &SpectralBalance6) -> [f64; 6] {
    [
        f64::from(balance.sub),
        f64::from(balance.low),
        f64::from(balance.low_mid),
        f64::from(balance.mid),
        f64::from(balance.presence),
        f64::from(balance.air),
    ]
}

/// Shares to dB, floored at [`SHARE_FLOOR`].
pub fn shares_to_db(shares: &[f64; 6]) -> [f64; 6] {
    let mut out = [0.0; 6];
    for (o, s) in out.iter_mut().zip(shares) {
        *o = 10.0 * s.max(SHARE_FLOOR).log10();
    }
    out
}

/// Measure in parallel (bounded), then do every order-dependent step
/// sequentially in sorted path order so the result is deterministic.
/// Measure `files` on a bounded worker pool. The result is in `files` order.
fn measure_parallel(
    files: &[PathBuf],
    config: &ReferenceRunConfig,
    excluded: &BTreeMap<u64, String>,
) -> Vec<Measured> {
    let workers = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1)
        .clamp(1, MAX_WORKERS)
        .min(files.len().max(1));
    let next = AtomicUsize::new(0);
    let slots: Vec<Mutex<Option<Measured>>> = files.iter().map(|_| Mutex::new(None)).collect();
    std::thread::scope(|scope| {
        for _ in 0..workers {
            scope.spawn(|| loop {
                let index = next.fetch_add(1, Ordering::Relaxed);
                let Some(path) = files.get(index) else {
                    break;
                };
                let measured = measure_one(path, config, excluded);
                if let Ok(mut slot) = slots[index].lock() {
                    *slot = Some(measured);
                }
            });
        }
    });
    slots
        .into_iter()
        .map(|slot| {
            slot.into_inner().ok().flatten().unwrap_or_else(|| {
                Measured::Skipped(
                    SkipReason::Undecodable,
                    "measurement worker did not finish".into(),
                )
            })
        })
        .collect()
}

fn measure_files(
    root: &Path,
    files: &[PathBuf],
    config: &ReferenceRunConfig,
    excluded: &BTreeMap<u64, String>,
) -> (ReferenceLedger, ReferenceStats) {
    let measured_all = measure_parallel(files, config, excluded);

    let mut counts = ReferenceCounts {
        files_seen: files.len(),
        ..Default::default()
    };
    let mut skipped = Vec::new();
    let mut songs: Vec<(String, MeasuredSong)> = Vec::new();
    let mut seen_hashes = std::collections::BTreeSet::new();
    for (path, measured) in files.iter().zip(measured_all) {
        let label = relative_label(root, path);
        match measured {
            Measured::Song(song) => {
                if seen_hashes.insert(song.content_hash) {
                    songs.push((label, song));
                } else {
                    skipped.push(SkippedReferenceFile {
                        relative_path: label,
                        reason: SkipReason::Duplicate,
                        detail: "same content as an earlier file".into(),
                    });
                }
            }
            Measured::Skipped(reason, detail) => skipped.push(SkippedReferenceFile {
                relative_path: label,
                reason,
                detail,
            }),
        }
    }
    for skip in &skipped {
        match skip.reason {
            SkipReason::NotAudio => counts.skipped_not_audio += 1,
            SkipReason::Unreadable => counts.skipped_unreadable += 1,
            SkipReason::Undecodable => counts.skipped_undecodable += 1,
            SkipReason::TooShort => counts.skipped_too_short += 1,
            SkipReason::LowSampleRate => counts.skipped_low_sample_rate += 1,
            SkipReason::Silent => counts.skipped_silent += 1,
            SkipReason::Duplicate => counts.skipped_duplicate += 1,
            SkipReason::MatchesExcluded => counts.skipped_matches_excluded += 1,
        }
    }

    let hashes: Vec<u64> = songs.iter().map(|(_, song)| song.content_hash).collect();
    let sides = seeded_split(&hashes, config.seed, HOLDOUT_FRACTION);
    let build_db: Vec<[f64; 6]> = songs
        .iter()
        .zip(&sides)
        .filter(|(_, side)| **side == SplitSide::Build)
        .map(|((_, song), _)| shares_to_db(&song.shares))
        .collect();
    // Round once, up front: the ledger's positions are then computed against
    // exactly the numbers the committed statistics will hold.
    let range =
        compute_range_stats(&build_db, config.bootstrap_resamples, config.seed).map(round_range);

    let mut accepted = Vec::with_capacity(songs.len());
    let mut build_ids = Vec::new();
    let mut holdout_ids = Vec::new();
    for ((label, song), side) in songs.into_iter().zip(sides) {
        let hash_hex = format!("{:016x}", song.content_hash);
        let anon_id = hash_hex[..ANON_ID_HEX_LEN].to_string();
        match side {
            SplitSide::Build => build_ids.push(anon_id.clone()),
            SplitSide::Holdout => holdout_ids.push(anon_id.clone()),
        }
        let shares_db = shares_to_db(&song.shares);
        let deviation_db = range
            .as_ref()
            .map(|r| deviation_from_typical(&shares_db, &r.typical_shape_db));
        let inside_range = range
            .as_ref()
            .zip(deviation_db.as_ref())
            .map(|(r, dev)| inside_flags(r, dev));
        accepted.push(ReferenceSongRecord {
            content_hash: hash_hex,
            anon_id,
            relative_path: label,
            duration_sec: round4(song.duration_sec),
            sample_rate: song.sample_rate,
            channels: song.channels,
            peak_dbfs: round4(song.peak_dbfs),
            shares: song.shares,
            shares_db: shares_db.map(round4),
            split: side,
            deviation_db: deviation_db.map(|d| d.map(round4)),
            inside_range,
        });
    }
    build_ids.sort();
    holdout_ids.sort();
    counts.accepted = accepted.len();
    counts.build = build_ids.len();
    counts.holdout = holdout_ids.len();

    let stats = ReferenceStats {
        schema: STATS_SCHEMA.to_string(),
        meter: METER_ID.to_string(),
        bands: BAND_KEYS.iter().map(|b| (*b).to_string()).collect(),
        band_edges_hz: BAND_EDGES_HZ.to_vec(),
        seed: config.seed,
        parameters: ReferenceStatsParameters {
            min_duration_sec: config.min_duration_sec,
            min_sample_rate_hz: MIN_SAMPLE_RATE_HZ,
            silence_peak_dbfs: SILENCE_PEAK_DBFS,
            holdout_fraction: HOLDOUT_FRACTION,
            range_spread_multiplier: RANGE_SPREAD_MULTIPLIER,
            range_min_half_width_db: RANGE_MIN_HALF_WIDTH_DB,
            spread_estimator: format!("{MAD_TO_SIGMA} x median absolute deviation"),
            offset_removal: "subtract per-band library median, then the song's median of the six"
                .to_string(),
            share_floor: SHARE_FLOOR,
            bootstrap_resamples: config.bootstrap_resamples,
            unstable_edge_interval_db: UNSTABLE_EDGE_INTERVAL_DB,
            minimum_library_songs: MIN_LIBRARY_SONGS,
        },
        meets_minimum_library_size: counts.accepted >= MIN_LIBRARY_SONGS,
        provisional: config.provisional,
        counts,
        range,
        split: SplitMembership {
            method: format!(
                "rank by splitmix64(seed ^ content hash); lowest round({HOLDOUT_FRACTION} x n) are held out"
            ),
            build_ids,
            holdout_ids,
        },
    };
    let ledger = ReferenceLedger {
        schema: "yes-master.smart-balance.reference-ledger.v1".to_string(),
        seed: config.seed,
        accepted,
        skipped,
    };
    (ledger, stats)
}

/// Assign each song (by content hash) to build or holdout. Ranking by a seeded
/// hash gives an exact `round(fraction × n)` holdout that does not depend on
/// file names or folder order.
pub fn seeded_split(hashes: &[u64], seed: u64, holdout_fraction: f64) -> Vec<SplitSide> {
    let mut order: Vec<(u64, u64, usize)> = hashes
        .iter()
        .enumerate()
        .map(|(i, h)| (splitmix64(seed ^ h.rotate_left(17)), *h, i))
        .collect();
    order.sort_unstable();
    let holdout = ((hashes.len() as f64) * holdout_fraction).round() as usize;
    let mut sides = vec![SplitSide::Build; hashes.len()];
    for (_, _, i) in order.into_iter().take(holdout) {
        sides[i] = SplitSide::Holdout;
    }
    sides
}

/// A song's position in deviation space: band dB minus the typical shape,
/// minus the median of those six differences (robust offset removal).
pub fn deviation_from_typical(shares_db: &[f64; 6], typical_shape_db: &[f64; 6]) -> [f64; 6] {
    let mut relative = [0.0; 6];
    for (r, (s, t)) in relative
        .iter_mut()
        .zip(shares_db.iter().zip(typical_shape_db))
    {
        *r = s - t;
    }
    let offset = median(&relative);
    relative.map(|r| r - offset)
}

fn inside_flags(range: &RangeStats, deviation: &[f64; 6]) -> [bool; 6] {
    let mut flags = [false; 6];
    for (flag, (band, d)) in flags.iter_mut().zip(range.bands.iter().zip(deviation)) {
        *flag = *d >= band.low_edge_db && *d <= band.high_edge_db;
    }
    flags
}

/// The healthy range from the build split's band levels (dB). `None` for an
/// empty build split. Bootstrap stability is added when `bootstrap_resamples`
/// is above zero and there are at least two songs.
pub fn compute_range_stats(
    build_db: &[[f64; 6]],
    bootstrap_resamples: usize,
    seed: u64,
) -> Option<RangeStats> {
    let mut stats = range_core(build_db)?;
    if bootstrap_resamples > 0 && build_db.len() >= 2 {
        let mut rng = SplitMix64(seed ^ 0xB007_5742_u64);
        let mut lows: Vec<Vec<f64>> = (0..6)
            .map(|_| Vec::with_capacity(bootstrap_resamples))
            .collect();
        let mut highs: Vec<Vec<f64>> = (0..6)
            .map(|_| Vec::with_capacity(bootstrap_resamples))
            .collect();
        let mut sample = Vec::with_capacity(build_db.len());
        for _ in 0..bootstrap_resamples {
            sample.clear();
            for _ in 0..build_db.len() {
                sample.push(build_db[rng.below(build_db.len())]);
            }
            if let Some(resampled) = range_core(&sample) {
                for (b, band) in resampled.bands.iter().enumerate() {
                    lows[b].push(band.low_edge_db);
                    highs[b].push(band.high_edge_db);
                }
            }
        }
        for (b, band) in stats.bands.iter_mut().enumerate() {
            let low_p05 = percentile(&lows[b], 0.05);
            let low_p95 = percentile(&lows[b], 0.95);
            let high_p05 = percentile(&highs[b], 0.05);
            let high_p95 = percentile(&highs[b], 0.95);
            band.stability = Some(EdgeStability {
                low_edge_sd_db: std_dev(&lows[b]),
                low_edge_p05_db: low_p05,
                low_edge_p95_db: low_p95,
                high_edge_sd_db: std_dev(&highs[b]),
                high_edge_p05_db: high_p05,
                high_edge_p95_db: high_p95,
                unstable: (low_p95 - low_p05) > UNSTABLE_EDGE_INTERVAL_DB
                    || (high_p95 - high_p05) > UNSTABLE_EDGE_INTERVAL_DB,
            });
        }
    }
    Some(stats)
}

/// Typical shape, deviations, and the D16 range, without bootstrap.
fn range_core(build_db: &[[f64; 6]]) -> Option<RangeStats> {
    if build_db.is_empty() {
        return None;
    }
    let mut typical = [0.0; 6];
    for (b, t) in typical.iter_mut().enumerate() {
        let column: Vec<f64> = build_db.iter().map(|song| song[b]).collect();
        *t = median(&column);
    }
    let deviations: Vec<[f64; 6]> = build_db
        .iter()
        .map(|song| deviation_from_typical(song, &typical))
        .collect();

    let mut bands = Vec::with_capacity(6);
    for (b, key) in BAND_KEYS.iter().enumerate() {
        let column: Vec<f64> = deviations.iter().map(|d| d[b]).collect();
        let center = median(&column);
        let abs_dev: Vec<f64> = column.iter().map(|d| (d - center).abs()).collect();
        let spread = MAD_TO_SIGMA * median(&abs_dev);
        let half_width = (RANGE_SPREAD_MULTIPLIER * spread).max(RANGE_MIN_HALF_WIDTH_DB);
        let low = center - half_width;
        let high = center + half_width;
        let inside = column.iter().filter(|d| **d >= low && **d <= high).count();
        bands.push(BandRange {
            band: (*key).to_string(),
            center_db: center,
            robust_spread_db: spread,
            half_width_db: half_width,
            low_edge_db: low,
            high_edge_db: high,
            build_p05_db: percentile(&column, 0.05),
            build_p95_db: percentile(&column, 0.95),
            build_inside_share: inside as f64 / column.len() as f64,
            stability: None,
        });
    }
    let all_inside = deviations
        .iter()
        .filter(|d| {
            d.iter()
                .zip(&bands)
                .all(|(v, band)| *v >= band.low_edge_db && *v <= band.high_edge_db)
        })
        .count();
    Some(RangeStats {
        typical_shape_db: typical,
        bands,
        build_all_bands_inside_share: all_inside as f64 / deviations.len() as f64,
    })
}

fn round_range(mut range: RangeStats) -> RangeStats {
    range.typical_shape_db = range.typical_shape_db.map(round4);
    range.build_all_bands_inside_share = round4(range.build_all_bands_inside_share);
    for band in &mut range.bands {
        band.center_db = round4(band.center_db);
        band.robust_spread_db = round4(band.robust_spread_db);
        band.half_width_db = round4(band.half_width_db);
        band.low_edge_db = round4(band.low_edge_db);
        band.high_edge_db = round4(band.high_edge_db);
        band.build_p05_db = round4(band.build_p05_db);
        band.build_p95_db = round4(band.build_p95_db);
        band.build_inside_share = round4(band.build_inside_share);
        if let Some(s) = band.stability.as_mut() {
            s.low_edge_sd_db = round4(s.low_edge_sd_db);
            s.low_edge_p05_db = round4(s.low_edge_p05_db);
            s.low_edge_p95_db = round4(s.low_edge_p95_db);
            s.high_edge_sd_db = round4(s.high_edge_sd_db);
            s.high_edge_p05_db = round4(s.high_edge_p05_db);
            s.high_edge_p95_db = round4(s.high_edge_p95_db);
        }
    }
    range
}

/// Four decimals, and never a negative zero, so the committed JSON is stable.
fn round4(value: f64) -> f64 {
    let rounded = (value * 10_000.0).round() / 10_000.0;
    if rounded == 0.0 {
        0.0
    } else {
        rounded
    }
}

/// Median; the mean of the two middle values for an even count. 0 if empty.
pub fn median(values: &[f64]) -> f64 {
    percentile(values, 0.5)
}

/// Linear-interpolated percentile (`q` in 0..=1). 0 if empty.
pub fn percentile(values: &[f64], q: f64) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    let mut sorted = values.to_vec();
    sorted.sort_by(f64::total_cmp);
    let pos = q.clamp(0.0, 1.0) * (sorted.len() - 1) as f64;
    let lo = pos.floor() as usize;
    let hi = pos.ceil() as usize;
    let frac = pos - lo as f64;
    sorted[lo] + (sorted[hi] - sorted[lo]) * frac
}

fn std_dev(values: &[f64]) -> f64 {
    if values.len() < 2 {
        return 0.0;
    }
    let mean = values.iter().sum::<f64>() / values.len() as f64;
    let var = values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / (values.len() - 1) as f64;
    var.sqrt()
}

const FNV_OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;

/// FNV-1a 64-bit. Stable across machines and Rust versions (unlike the
/// standard library's hasher), which the recorded split depends on.
fn fnv1a64(bytes: &[u8], mut hash: u64) -> u64 {
    for &b in bytes {
        hash ^= u64::from(b);
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash
}

/// The splitmix64 finalizer: a fixed, well-mixed 64-bit permutation.
fn splitmix64(mut z: u64) -> u64 {
    z = z.wrapping_add(0x9E37_79B9_7F4A_7C15);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

/// Small seeded generator for the bootstrap. Deterministic everywhere.
struct SplitMix64(u64);

impl SplitMix64 {
    fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    fn below(&mut self, n: usize) -> usize {
        (self.next_u64() % n as u64) as usize
    }
}

// ---------------------------------------------------------------------------
// Placement: where other songs sit against an existing range.
// ---------------------------------------------------------------------------

/// One song placed against an existing healthy range. Private: it carries
/// the file's label.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PlacedSong {
    /// `<placed folder name>/<path inside it>`.
    pub label: String,
    pub content_hash: String,
    pub duration_sec: f64,
    pub sample_rate: u32,
    pub channels: u16,
    pub peak_dbfs: f64,
    pub shares: [f64; 6],
    pub shares_db: [f64; 6],
    /// Position in the range's deviation space.
    pub deviation_db: [f64; 6],
    pub inside_range: [bool; 6],
    /// Signed distance beyond the nearest edge: negative below the range,
    /// positive above it, exactly zero inside.
    pub outside_by_db: [f64; 6],
}

/// The private placement report.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PlacementReport {
    pub schema: String,
    /// What the range came from, copied from the statistics.
    pub stats_schema: String,
    pub library_build_songs: usize,
    pub provisional: bool,
    pub placed: Vec<PlacedSong>,
    pub skipped: Vec<SkippedReferenceFile>,
}

/// Where a placement run wrote its outputs.
#[derive(Debug, Clone)]
pub struct PlacementRunResult {
    pub output_dir: PathBuf,
    pub report_json_path: PathBuf,
    pub report_csv_path: PathBuf,
    pub report: PlacementReport,
}

pub const PLACEMENT_JSON_FILE_NAME: &str = "smart-balance-placement.json";
pub const PLACEMENT_CSV_FILE_NAME: &str = "smart-balance-placement.csv";

/// Read statistics written by [`run_reference_measurement`] (or a committed
/// copy of them).
pub fn read_stats(path: &Path) -> CommandResult<ReferenceStats> {
    let text = fs::read_to_string(path)
        .map_err(|e| CommandError::Io(format!("read {}: {e}", path.display())))?;
    let stats: ReferenceStats = serde_json::from_str(&text)
        .map_err(|e| CommandError::Other(format!("parse {}: {e}", path.display())))?;
    if stats.schema != STATS_SCHEMA {
        return Err(CommandError::Other(format!(
            "{} has schema {:?}, expected {STATS_SCHEMA:?}",
            path.display(),
            stats.schema
        )));
    }
    Ok(stats)
}

/// Measure every file under each of `place_dirs` (recursively) and report
/// where it sits against `stats`' range. Duplicates are all reported; the
/// skip rules otherwise match the library run's.
pub fn run_placement(
    place_dirs: &[PathBuf],
    stats: &ReferenceStats,
    output_dir: &Path,
    config: &ReferenceRunConfig,
) -> CommandResult<PlacementRunResult> {
    let range = stats.range.as_ref().ok_or_else(|| {
        CommandError::Other("the statistics hold no range (empty build split)".into())
    })?;
    let cwd = std::env::current_dir().map_err(|e| CommandError::Io(format!("current dir: {e}")))?;
    let mut roots = Vec::with_capacity(place_dirs.len());
    for dir in place_dirs {
        let root = normalized_absolute_path(&cwd, dir);
        if !root.is_dir() {
            return Err(CommandError::InvalidPath(format!(
                "folder to place does not exist: {}",
                dir.display()
            )));
        }
        roots.push(root);
    }
    let root_refs: Vec<&Path> = roots.iter().map(PathBuf::as_path).collect();
    let output_dir = prepare_evidence_output_dir(&cwd, output_dir, &root_refs)?;

    let mut files = Vec::new();
    let mut labels = Vec::new();
    for root in &roots {
        for path in collect_files(root)? {
            labels.push(root_label(root, &path));
            files.push(path);
        }
    }
    let measured_all = measure_parallel(&files, config, &BTreeMap::new());

    let mut placed = Vec::new();
    let mut skipped = Vec::new();
    for (label, measured) in labels.into_iter().zip(measured_all) {
        match measured {
            Measured::Song(song) => {
                let shares_db = shares_to_db(&song.shares);
                let deviation = deviation_from_typical(&shares_db, &range.typical_shape_db);
                placed.push(PlacedSong {
                    label,
                    content_hash: format!("{:016x}", song.content_hash),
                    duration_sec: round4(song.duration_sec),
                    sample_rate: song.sample_rate,
                    channels: song.channels,
                    peak_dbfs: round4(song.peak_dbfs),
                    shares: song.shares,
                    shares_db: shares_db.map(round4),
                    deviation_db: deviation.map(round4),
                    inside_range: inside_flags(range, &deviation),
                    outside_by_db: outside_by(range, &deviation).map(round4),
                });
            }
            Measured::Skipped(reason, detail) => skipped.push(SkippedReferenceFile {
                relative_path: label,
                reason,
                detail,
            }),
        }
    }

    let report = PlacementReport {
        schema: "yes-master.smart-balance.placement.v1".to_string(),
        stats_schema: stats.schema.clone(),
        library_build_songs: stats.counts.build,
        provisional: stats.provisional,
        placed,
        skipped,
    };
    let report_json_path = output_dir.join(PLACEMENT_JSON_FILE_NAME);
    let report_csv_path = output_dir.join(PLACEMENT_CSV_FILE_NAME);
    let json = serde_json::to_string_pretty(&report)
        .map_err(|e| CommandError::Other(format!("serialize placement report: {e}")))?;
    write_text(&report_json_path, &format!("{json}\n"))?;
    write_text(&report_csv_path, &placement_csv(&report))?;
    Ok(PlacementRunResult {
        output_dir,
        report_json_path,
        report_csv_path,
        report,
    })
}

/// Signed distance beyond the nearest range edge, per band (0 inside).
fn outside_by(range: &RangeStats, deviation: &[f64; 6]) -> [f64; 6] {
    let mut out = [0.0; 6];
    for (o, (band, d)) in out.iter_mut().zip(range.bands.iter().zip(deviation)) {
        *o = if *d < band.low_edge_db {
            d - band.low_edge_db
        } else if *d > band.high_edge_db {
            d - band.high_edge_db
        } else {
            0.0
        };
    }
    out
}

fn placement_csv(report: &PlacementReport) -> String {
    let mut out = String::from(
        "status,label,duration_sec,sample_rate,peak_dbfs,\
         sub_dev,bass_dev,low_mids_dev,mids_dev,presence_dev,air_dev,\
         sub_out,bass_out,low_mids_out,mids_out,presence_out,air_out,detail\n",
    );
    for song in &report.placed {
        let dev = song.deviation_db.map(|v| format!("{v:.4}")).join(",");
        let outside = song.outside_by_db.map(|v| format!("{v:.4}")).join(",");
        out.push_str(&format!(
            "placed,{},{:.3},{},{:.2},{dev},{outside},\n",
            csv_escape(&song.label),
            song.duration_sec,
            song.sample_rate,
            song.peak_dbfs,
        ));
    }
    for skip in &report.skipped {
        let reason = serde_json::to_value(skip.reason)
            .ok()
            .and_then(|v| v.as_str().map(str::to_string))
            .unwrap_or_default();
        // Status, label, 15 empty measurement columns, detail: 18 columns.
        out.push_str(&format!(
            "skipped:{reason},{},{}{}\n",
            csv_escape(&skip.relative_path),
            ",".repeat(15),
            csv_escape(&skip.detail),
        ));
    }
    out
}

fn ledger_csv(ledger: &ReferenceLedger) -> String {
    let mut out = String::from(
        "status,relative_path,anon_id,split,duration_sec,sample_rate,channels,peak_dbfs,\
         sub_db,bass_db,low_mids_db,mids_db,presence_db,air_db,\
         sub_dev,bass_dev,low_mids_dev,mids_dev,presence_dev,air_dev,detail\n",
    );
    for song in &ledger.accepted {
        let split = match song.split {
            SplitSide::Build => "build",
            SplitSide::Holdout => "holdout",
        };
        let db = song.shares_db.map(|v| format!("{v:.4}")).join(",");
        let dev = song
            .deviation_db
            .map(|d| d.map(|v| format!("{v:.4}")).join(","))
            .unwrap_or_else(|| ",,,,,".to_string());
        out.push_str(&format!(
            "accepted,{},{},{split},{:.3},{},{},{:.2},{db},{dev},\n",
            csv_escape(&song.relative_path),
            song.anon_id,
            song.duration_sec,
            song.sample_rate,
            song.channels,
            song.peak_dbfs,
        ));
    }
    for skip in &ledger.skipped {
        let reason = serde_json::to_value(skip.reason)
            .ok()
            .and_then(|v| v.as_str().map(str::to_string))
            .unwrap_or_default();
        // Status, path, 18 empty measurement columns, detail: 21 columns.
        out.push_str(&format!(
            "skipped:{reason},{},{}{}\n",
            csv_escape(&skip.relative_path),
            ",".repeat(18),
            csv_escape(&skip.detail),
        ));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir(tag: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "yes-master-sb-ref-{tag}-{}-{:?}",
            std::process::id(),
            std::thread::current().id()
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).expect("create temp dir");
        dir
    }

    fn write_wav(path: &Path, sample_rate: u32, seconds: f32, tones: &[(f32, f32)]) {
        let spec = hound::WavSpec {
            channels: 2,
            sample_rate,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };
        let mut writer = hound::WavWriter::create(path, spec).expect("wav create");
        let n = (sample_rate as f32 * seconds) as u32;
        for i in 0..n {
            let t = i as f32 / sample_rate as f32;
            let s: f32 = tones
                .iter()
                .map(|(f, a)| a * (2.0 * std::f32::consts::PI * f * t).sin())
                .sum();
            let v = (s.clamp(-1.0, 1.0) * f32::from(i16::MAX)) as i16;
            writer.write_sample(v).expect("write");
            writer.write_sample(v).expect("write");
        }
        writer.finalize().expect("finalize");
    }

    #[test]
    fn measures_through_the_same_meter_as_analyze_one() {
        // The range is only valid for the app's meter. Pin that the runner's
        // shares are bit-identical to what the app's analysis stores.
        let dir = temp_dir("same-meter");
        let path = dir.join("song.wav");
        write_wav(
            &path,
            44_100,
            2.0,
            &[(60.0, 0.2), (440.0, 0.2), (3000.0, 0.1), (9000.0, 0.05)],
        );
        let config = ReferenceRunConfig {
            min_duration_sec: 1.0,
            ..Default::default()
        };
        let Measured::Song(song) = measure_one(&path, &config, &BTreeMap::new()) else {
            panic!("synthetic song should be accepted");
        };
        let analysis =
            crate::analysis::analyze_one(crate::types::TrackId("same-meter".into()), &path, false)
                .expect("analyze");
        let app = shares_array(&analysis.spectral_balance_6band.expect("6-band"));
        assert_eq!(song.shares.map(f64::to_bits), app.map(f64::to_bits));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn median_and_percentile_match_hand_values() {
        assert_eq!(median(&[3.0, 1.0, 2.0]), 2.0);
        assert_eq!(median(&[4.0, 1.0, 3.0, 2.0]), 2.5);
        assert_eq!(percentile(&[0.0, 10.0], 0.25), 2.5);
        assert_eq!(median(&[]), 0.0);
    }

    #[test]
    fn deviation_removes_the_overall_offset_robustly() {
        // One band 6 dB hot. With a mean offset the other five would all read
        // about -1 dB; the median removal keeps them at exactly zero.
        let typical = [-20.0, -8.0, -6.0, -7.0, -12.0, -16.0];
        let mut song = typical;
        song[2] += 6.0;
        let dev = deviation_from_typical(&song, &typical);
        assert_eq!(dev, [0.0, 0.0, 6.0, 0.0, 0.0, 0.0]);

        // A uniform shift in every band is no deviation at all.
        let shifted = typical.map(|v| v + 3.0);
        assert_eq!(deviation_from_typical(&shifted, &typical), [0.0; 6]);
    }

    fn synthetic_library(n: usize, spread_db: f64, seed: u64) -> Vec<[f64; 6]> {
        let base = [-20.0, -8.0, -6.0, -7.0, -12.0, -16.0];
        let mut rng = SplitMix64(seed);
        (0..n)
            .map(|_| {
                let mut song = base;
                for v in &mut song {
                    // Uniform in [-spread, spread].
                    let u = (rng.next_u64() >> 11) as f64 / (1u64 << 53) as f64;
                    *v += (u * 2.0 - 1.0) * spread_db;
                }
                song
            })
            .collect()
    }

    #[test]
    fn identical_songs_get_the_minimum_half_width() {
        let library = vec![[-20.0, -8.0, -6.0, -7.0, -12.0, -16.0]; 12];
        let stats = compute_range_stats(&library, 50, 7).expect("stats");
        assert_eq!(stats.typical_shape_db, library[0]);
        for band in &stats.bands {
            assert_eq!(band.center_db, 0.0);
            assert_eq!(band.robust_spread_db, 0.0);
            assert_eq!(band.half_width_db, RANGE_MIN_HALF_WIDTH_DB);
            assert_eq!(band.low_edge_db, -RANGE_MIN_HALF_WIDTH_DB);
            assert_eq!(band.high_edge_db, RANGE_MIN_HALF_WIDTH_DB);
            assert_eq!(band.build_inside_share, 1.0);
            let s = band.stability.as_ref().expect("bootstrap");
            assert!(!s.unstable);
        }
        assert_eq!(stats.build_all_bands_inside_share, 1.0);
    }

    #[test]
    fn wide_libraries_use_twice_the_robust_spread() {
        let library = synthetic_library(400, 6.0, 11);
        let stats = compute_range_stats(&library, 0, 7).expect("stats");
        for band in &stats.bands {
            assert!(
                band.robust_spread_db > RANGE_MIN_HALF_WIDTH_DB,
                "{} spread {}",
                band.band,
                band.robust_spread_db
            );
            let expected = RANGE_SPREAD_MULTIPLIER * band.robust_spread_db;
            assert!((band.half_width_db - expected).abs() < 1e-12);
            assert!((band.low_edge_db - (band.center_db - expected)).abs() < 1e-12);
            assert!((band.high_edge_db - (band.center_db + expected)).abs() < 1e-12);
            assert!(band.stability.is_none(), "no bootstrap requested");
        }
    }

    #[test]
    fn spread_is_the_scaled_median_absolute_deviation() {
        // Deviations in one band of -4, -2, 0, 2, 4 (others flat): MAD = 2.
        let base = [-20.0, -8.0, -6.0, -7.0, -12.0, -16.0];
        let library: Vec<[f64; 6]> = [-4.0, -2.0, 0.0, 2.0, 4.0]
            .iter()
            .map(|d| {
                let mut song = base;
                song[3] += d;
                song
            })
            .collect();
        let stats = compute_range_stats(&library, 0, 1).expect("stats");
        let mids = &stats.bands[3];
        assert_eq!(mids.center_db, 0.0);
        assert!((mids.robust_spread_db - MAD_TO_SIGMA * 2.0).abs() < 1e-12);
        assert!((mids.half_width_db - 2.0 * MAD_TO_SIGMA * 2.0).abs() < 1e-12);
    }

    #[test]
    fn bootstrap_is_seeded() {
        let library = synthetic_library(40, 3.0, 5);
        let a = compute_range_stats(&library, 200, 99).expect("a");
        let b = compute_range_stats(&library, 200, 99).expect("b");
        let c = compute_range_stats(&library, 200, 100).expect("c");
        assert_eq!(a, b, "same seed must reproduce the bootstrap exactly");
        assert_ne!(
            a.bands[0].stability, c.bands[0].stability,
            "a different seed should resample differently"
        );
        // The range itself does not depend on the bootstrap seed.
        assert_eq!(a.bands[0].low_edge_db, c.bands[0].low_edge_db);
    }

    #[test]
    fn empty_build_split_has_no_range() {
        assert!(compute_range_stats(&[], 100, 1).is_none());
    }

    #[test]
    fn seeded_split_is_exact_and_name_independent() {
        let hashes: Vec<u64> = (0..40u64).map(|i| splitmix64(i * 7919)).collect();
        let sides = seeded_split(&hashes, DEFAULT_SEED, HOLDOUT_FRACTION);
        let holdout = sides.iter().filter(|s| **s == SplitSide::Holdout).count();
        assert_eq!(holdout, 10);
        // Reordering the input does not change any song's side.
        let mut reversed = hashes.clone();
        reversed.reverse();
        let sides_rev = seeded_split(&reversed, DEFAULT_SEED, HOLDOUT_FRACTION);
        for (i, side) in sides.iter().enumerate() {
            assert_eq!(*side, sides_rev[hashes.len() - 1 - i]);
        }
        // A different seed gives a different split.
        assert_ne!(
            sides,
            seeded_split(&hashes, DEFAULT_SEED + 1, HOLDOUT_FRACTION)
        );
    }

    #[test]
    fn fnv_matches_the_published_test_vectors() {
        assert_eq!(fnv1a64(b"", FNV_OFFSET), 0xcbf2_9ce4_8422_2325);
        assert_eq!(fnv1a64(b"a", FNV_OFFSET), 0xaf63_dc4c_8601_ec8c);
        assert_eq!(fnv1a64(b"foobar", FNV_OFFSET), 0x8594_4171_f739_67e8);
    }

    #[test]
    fn shares_to_db_floors_empty_bands() {
        let db = shares_to_db(&[0.0, 0.1, 0.2, 0.3, 0.4, 1.0]);
        assert_eq!(db[0], -90.0);
        assert!((db[1] - -10.0).abs() < 1e-12);
        assert_eq!(db[5], 0.0);
    }

    #[test]
    fn round4_never_writes_negative_zero() {
        assert_eq!(round4(-0.000_01).to_bits(), 0.0_f64.to_bits());
        assert_eq!(round4(1.234_56), 1.2346);
    }
}
