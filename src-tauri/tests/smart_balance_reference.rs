//! Smart Balance reference measurement runner (execution plan Slices 1-2):
//! skip rules, determinism, the privacy of the committable statistics, corpus
//! exclusion, the provisional label, and placement.
//!
//! Output-path safety for this runner lives with the other private runners in
//! `tests/evidence_runner_paths.rs`.
//!
//! Synthetic fixtures only. Nothing here reads private audio.

use std::fs;
use std::path::{Path, PathBuf};

use yes_master_lib::smart_balance_reference::{
    read_stats, run_placement, run_reference_measurement, ReferenceRunConfig, SkipReason,
    SplitSide, LEDGER_CSV_FILE_NAME, LEDGER_JSON_FILE_NAME, MIN_DURATION_SEC, STATS_FILE_NAME,
    STATS_SCHEMA,
};

fn temp_root(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "yes-master-sb-reference-{tag}-{}-{:?}",
        std::process::id(),
        std::thread::current().id()
    ));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("create temp root");
    dir
}

/// Stereo 16-bit WAV: a sum of sines at the given (frequency, amplitude).
fn write_wav(path: &Path, sample_rate: u32, seconds: f32, tones: &[(f32, f32)]) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("create parent");
    }
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
        let v = (s.clamp(-1.0, 1.0) * f32::from(i16::MAX)).round() as i16;
        writer.write_sample(v).expect("write");
        writer.write_sample(v).expect("write");
    }
    writer.finalize().expect("finalize");
}

const BALANCED: [(f32, f32); 5] = [
    (50.0, 0.15),
    (150.0, 0.2),
    (500.0, 0.15),
    (1500.0, 0.1),
    (9000.0, 0.03),
];

fn short_config() -> ReferenceRunConfig {
    ReferenceRunConfig {
        min_duration_sec: 2.0,
        bootstrap_resamples: 64,
        ..Default::default()
    }
}

/// A folder with three good songs (one nested) and one of every skip case.
fn write_mixed_library(root: &Path) -> PathBuf {
    let lib = root.join("refsongs");
    write_wav(&lib.join("good-one.wav"), 44_100, 3.0, &BALANCED);
    write_wav(
        &lib.join("good-two.wav"),
        48_000,
        3.0,
        &[(60.0, 0.1), (200.0, 0.3), (700.0, 0.2), (3000.0, 0.08)],
    );
    write_wav(
        &lib.join("nested").join("deeper").join("good-three.wav"),
        44_100,
        3.0,
        &[(45.0, 0.2), (120.0, 0.15), (2000.0, 0.15), (7000.0, 0.05)],
    );
    // Same bytes as good-one: counted once.
    fs::copy(
        lib.join("good-one.wav"),
        lib.join("nested").join("copy.wav"),
    )
    .expect("copy");
    write_wav(&lib.join("too-short.wav"), 44_100, 1.0, &BALANCED);
    write_wav(&lib.join("low-rate.wav"), 22_050, 3.0, &BALANCED);
    write_wav(&lib.join("silent.wav"), 44_100, 3.0, &[]);
    // Peak about -70 dBFS: present but far below anything released.
    write_wav(
        &lib.join("near-silent.wav"),
        44_100,
        3.0,
        &[(440.0, 0.0003)],
    );
    fs::write(
        lib.join("broken.wav"),
        b"RIFF\x00\x00\x00\x00not really audio",
    )
    .expect("write");
    fs::write(lib.join("notes.txt"), b"not audio").expect("write");
    fs::write(lib.join("cover.jpg"), [0xFF_u8, 0xD8, 0xFF, 0xE0]).expect("write");
    lib
}

#[test]
fn skip_rules_are_logged_and_never_fail_the_run() {
    let root = temp_root("skips");
    let lib = write_mixed_library(&root);
    let result = run_reference_measurement(&lib, &root.join("out"), &short_config())
        .expect("a folder with bad files must still measure the good ones");

    let mut accepted: Vec<&str> = result
        .ledger
        .accepted
        .iter()
        .map(|s| s.relative_path.as_str())
        .collect();
    accepted.sort_unstable();
    assert_eq!(
        accepted,
        [
            "good-one.wav",
            "good-two.wav",
            "nested/deeper/good-three.wav"
        ]
    );

    let reason_of = |name: &str| {
        result
            .ledger
            .skipped
            .iter()
            .find(|s| s.relative_path == name)
            .unwrap_or_else(|| panic!("{name} should be skipped"))
            .reason
    };
    assert_eq!(reason_of("too-short.wav"), SkipReason::TooShort);
    assert_eq!(reason_of("low-rate.wav"), SkipReason::LowSampleRate);
    assert_eq!(reason_of("silent.wav"), SkipReason::Silent);
    assert_eq!(reason_of("near-silent.wav"), SkipReason::Silent);
    assert_eq!(reason_of("broken.wav"), SkipReason::Undecodable);
    assert_eq!(reason_of("notes.txt"), SkipReason::NotAudio);
    assert_eq!(reason_of("cover.jpg"), SkipReason::NotAudio);
    assert_eq!(reason_of("nested/copy.wav"), SkipReason::Duplicate);
    assert!(
        result.ledger.skipped.iter().all(|s| !s.detail.is_empty()),
        "every skip must carry a reason a person can read"
    );

    let counts = &result.stats.counts;
    assert_eq!(counts.files_seen, 11);
    assert_eq!(counts.accepted, 3);
    assert_eq!(counts.build + counts.holdout, 3);
    assert_eq!(counts.skipped_not_audio, 2);
    assert_eq!(counts.skipped_undecodable, 1);
    assert_eq!(counts.skipped_too_short, 1);
    assert_eq!(counts.skipped_low_sample_rate, 1);
    assert_eq!(counts.skipped_silent, 2);
    assert_eq!(counts.skipped_duplicate, 1);
    assert!(!result.stats.meets_minimum_library_size);

    // Shares are the meter's level-invariant fractions: they sum to ~1.
    for song in &result.ledger.accepted {
        let sum: f64 = song.shares.iter().sum();
        assert!((sum - 1.0).abs() < 1e-4, "shares sum {sum}");
    }
}

#[test]
fn the_default_minimum_is_thirty_seconds() {
    assert_eq!(MIN_DURATION_SEC, 30.0);
    let root = temp_root("thirty");
    let lib = root.join("library");
    write_wav(&lib.join("under.wav"), 32_000, 29.5, &BALANCED);
    write_wav(&lib.join("over.wav"), 32_000, 30.5, &BALANCED);
    let config = ReferenceRunConfig {
        bootstrap_resamples: 0,
        ..Default::default()
    };
    let result = run_reference_measurement(&lib, &root.join("out"), &config).expect("run");
    assert_eq!(result.ledger.accepted.len(), 1);
    assert_eq!(result.ledger.accepted[0].relative_path, "over.wav");
    assert_eq!(result.ledger.skipped.len(), 1);
    assert_eq!(result.ledger.skipped[0].relative_path, "under.wav");
    assert_eq!(result.ledger.skipped[0].reason, SkipReason::TooShort);
    assert_eq!(result.stats.parameters.min_duration_sec, 30.0);
}

#[test]
fn the_same_folder_gives_byte_identical_statistics() {
    let root = temp_root("determinism");
    let lib = write_mixed_library(&root);
    let first = run_reference_measurement(&lib, &root.join("out-a"), &short_config()).expect("a");
    let second = run_reference_measurement(&lib, &root.join("out-b"), &short_config()).expect("b");

    let stats_a = fs::read(first.output_dir.join(STATS_FILE_NAME)).expect("stats a");
    let stats_b = fs::read(second.output_dir.join(STATS_FILE_NAME)).expect("stats b");
    assert_eq!(stats_a, stats_b, "statistics must be byte-identical");
    let ledger_a = fs::read(first.output_dir.join(LEDGER_JSON_FILE_NAME)).expect("ledger a");
    let ledger_b = fs::read(second.output_dir.join(LEDGER_JSON_FILE_NAME)).expect("ledger b");
    assert_eq!(
        ledger_a, ledger_b,
        "the private ledger is deterministic too"
    );
    assert!(first.output_dir.join(LEDGER_CSV_FILE_NAME).is_file());

    // A different seed changes the split record but not what was measured.
    let reseeded = ReferenceRunConfig {
        seed: 7,
        ..short_config()
    };
    let third = run_reference_measurement(&lib, &root.join("out-c"), &reseeded).expect("c");
    assert_eq!(third.stats.seed, 7);
    assert_eq!(third.stats.counts.accepted, first.stats.counts.accepted);
}

#[test]
fn committable_statistics_carry_no_names_or_paths() {
    let root = temp_root("privacy");
    let lib = write_mixed_library(&root);
    let result = run_reference_measurement(&lib, &root.join("out"), &short_config()).expect("run");
    let stats = fs::read_to_string(&result.stats_path).expect("stats");

    for needle in [
        "good-one",
        "good-two",
        "good-three",
        "nested",
        "deeper",
        "copy",
        "broken",
        "notes",
        ".wav",
        "refsongs",
    ] {
        assert!(
            !stats.contains(needle),
            "committable statistics leaked `{needle}`"
        );
    }
    assert!(!stats.contains(&root.to_string_lossy().to_string()));
    assert!(stats.contains(STATS_SCHEMA));

    // Split membership is recorded by anonymous ID only, one entry per song.
    let ids: Vec<&String> = result
        .stats
        .split
        .build_ids
        .iter()
        .chain(&result.stats.split.holdout_ids)
        .collect();
    assert_eq!(ids.len(), result.stats.counts.accepted);
    for id in ids {
        assert_eq!(id.len(), 10);
        assert!(id.chars().all(|c| c.is_ascii_hexdigit()));
    }

    // The private ledger keeps the names, and agrees with the split.
    for song in &result.ledger.accepted {
        let list = match song.split {
            SplitSide::Build => &result.stats.split.build_ids,
            SplitSide::Holdout => &result.stats.split.holdout_ids,
        };
        assert!(list.contains(&song.anon_id));
    }
}

#[test]
fn the_range_comes_from_the_build_split_only() {
    let root = temp_root("build-only");
    let lib = root.join("library");
    // Eight songs with distinct shapes so the split matters.
    for i in 0..8 {
        let tilt = 0.02 * i as f32;
        write_wav(
            &lib.join(format!("song-{i}.wav")),
            44_100,
            3.0,
            &[
                (50.0, 0.1 + tilt),
                (150.0, 0.2),
                (500.0, 0.15),
                (1500.0, 0.1),
                (4000.0, 0.05 + tilt * 0.5),
                (9000.0, 0.03),
            ],
        );
    }
    let result = run_reference_measurement(&lib, &root.join("out"), &short_config()).expect("run");
    assert_eq!(result.stats.counts.holdout, 2, "round(8 x 0.25)");
    assert_eq!(result.stats.counts.build, 6);

    let build_db: Vec<[f64; 6]> = result
        .ledger
        .accepted
        .iter()
        .filter(|s| s.split == SplitSide::Build)
        .map(|s| s.shares.map(|v| 10.0 * v.max(1.0e-9).log10()))
        .collect();
    let expected = yes_master_lib::smart_balance_reference::compute_range_stats(
        &build_db,
        0,
        result.stats.seed,
    )
    .expect("range");
    let range = result.stats.range.as_ref().expect("range");
    for (b, band) in range.bands.iter().enumerate() {
        assert!(
            (band.low_edge_db - expected.bands[b].low_edge_db).abs() < 1e-4,
            "{} low edge",
            band.band
        );
        assert!(
            (band.high_edge_db - expected.bands[b].high_edge_db).abs() < 1e-4,
            "{} high edge",
            band.band
        );
    }
}

#[test]
fn a_missing_reference_folder_is_an_error() {
    let root = temp_root("missing");
    let error = run_reference_measurement(
        &root.join("does-not-exist"),
        &root.join("out"),
        &short_config(),
    )
    .expect_err("a missing folder must fail loudly");
    assert!(format!("{error}").contains("does not exist"));
    assert!(!root.join("out").exists(), "nothing created on failure");
}

// ---------------------------------------------------------------------------
// Slice 2 additions: exclusion, the provisional label, placement, CSV shape.
// ---------------------------------------------------------------------------

#[test]
fn files_matching_an_excluded_folder_are_skipped_and_counted() {
    // The test corpus must not shape the range it is later checked against.
    let root = temp_root("exclude");
    let lib = write_mixed_library(&root);
    let corpus = root.join("corpus");
    fs::create_dir_all(&corpus).expect("corpus dir");
    fs::copy(lib.join("good-two.wav"), corpus.join("piece.wav")).expect("copy");

    let config = ReferenceRunConfig {
        exclude_matching: vec![corpus.clone()],
        ..short_config()
    };
    let result = run_reference_measurement(&lib, &root.join("out"), &config).expect("run");
    let skip = result
        .ledger
        .skipped
        .iter()
        .find(|s| s.relative_path == "good-two.wav")
        .expect("good-two must be skipped");
    assert_eq!(skip.reason, SkipReason::MatchesExcluded);
    assert!(skip.detail.contains("corpus/piece.wav"), "{}", skip.detail);
    assert_eq!(result.stats.counts.skipped_matches_excluded, 1);
    assert_eq!(result.stats.counts.accepted, 2);
    assert!(
        result
            .ledger
            .accepted
            .iter()
            .all(|s| s.relative_path != "good-two.wav"),
        "an excluded song must not be measured into the range"
    );

    let missing = ReferenceRunConfig {
        exclude_matching: vec![root.join("no-such-corpus")],
        ..short_config()
    };
    let error = run_reference_measurement(&lib, &root.join("out-missing"), &missing)
        .expect_err("a missing exclude folder must fail loudly");
    assert!(format!("{error}").contains("exclude folder does not exist"));
}

#[test]
fn the_provisional_label_is_recorded_only_when_asked() {
    let root = temp_root("provisional");
    let lib = write_mixed_library(&root);
    let plain = run_reference_measurement(&lib, &root.join("out-a"), &short_config()).expect("a");
    assert!(!plain.stats.provisional);
    let text = fs::read_to_string(&plain.stats_path).expect("stats");
    assert!(text.contains("\"provisional\": false"));

    let labelled = ReferenceRunConfig {
        provisional: true,
        ..short_config()
    };
    let marked = run_reference_measurement(&lib, &root.join("out-b"), &labelled).expect("b");
    assert!(marked.stats.provisional);
    let text = fs::read_to_string(&marked.stats_path).expect("stats");
    assert!(text.contains("\"provisional\": true"));
    // The label changes nothing that was measured.
    assert_eq!(plain.stats.range, marked.stats.range);
    assert_eq!(plain.stats.split, marked.stats.split);
}

#[test]
fn placement_agrees_with_the_ledger_positions() {
    let root = temp_root("placement");
    let lib = write_mixed_library(&root);
    let result = run_reference_measurement(&lib, &root.join("out"), &short_config()).expect("run");

    // Placement reads the statistics back the way a committed copy is read.
    let stats = read_stats(&result.stats_path).expect("read stats");
    assert_eq!(stats, result.stats);

    let placed = run_placement(
        std::slice::from_ref(&lib),
        &stats,
        &root.join("placed"),
        &short_config(),
    )
    .expect("place");
    let report = &placed.report;
    assert_eq!(report.library_build_songs, stats.counts.build);
    assert!(placed.report_json_path.is_file());
    assert!(placed.report_csv_path.is_file());
    // Duplicates are reported, not skipped: good-one and its copy both appear.
    assert_eq!(report.placed.len(), 4);
    assert!(report
        .placed
        .iter()
        .any(|p| p.label == "refsongs/nested/copy.wav"));
    assert!(report
        .skipped
        .iter()
        .any(|s| s.relative_path == "refsongs/notes.txt" && s.reason == SkipReason::NotAudio));

    let range = stats.range.as_ref().expect("range");
    for song in &result.ledger.accepted {
        let p = report
            .placed
            .iter()
            .find(|p| p.content_hash == song.content_hash)
            .expect("every accepted song is placed");
        assert_eq!(
            Some(p.deviation_db),
            song.deviation_db,
            "{}",
            song.relative_path
        );
        assert_eq!(
            Some(p.inside_range),
            song.inside_range,
            "{}",
            song.relative_path
        );
    }
    for p in &report.placed {
        for (b, band) in range.bands.iter().enumerate() {
            let out = p.outside_by_db[b];
            if p.inside_range[b] {
                assert_eq!(
                    out, 0.0,
                    "{} {} inside but outside_by {out}",
                    p.label, band.band
                );
            } else if p.deviation_db[b] < band.low_edge_db {
                assert!(out < 0.0, "{} {} below the range", p.label, band.band);
            } else {
                assert!(out > 0.0, "{} {} above the range", p.label, band.band);
            }
        }
    }

    let mut empty = stats.clone();
    empty.range = None;
    let error = run_placement(
        std::slice::from_ref(&lib),
        &empty,
        &root.join("placed-empty"),
        &short_config(),
    )
    .expect_err("placing against no range must fail");
    assert!(format!("{error}").contains("no range"));
}

/// Number of fields in one CSV line, honoring double-quoted fields.
fn csv_field_count(line: &str) -> usize {
    let mut in_quotes = false;
    let mut fields = 1;
    for c in line.chars() {
        match c {
            '"' => in_quotes = !in_quotes,
            ',' if !in_quotes => fields += 1,
            _ => {}
        }
    }
    fields
}

#[test]
fn every_csv_row_has_the_header_column_count() {
    let root = temp_root("csv-shape");
    let lib = write_mixed_library(&root);
    let result = run_reference_measurement(&lib, &root.join("out"), &short_config()).expect("run");
    let placed = run_placement(
        std::slice::from_ref(&lib),
        &result.stats,
        &root.join("placed"),
        &short_config(),
    )
    .expect("place");

    for path in [&result.ledger_csv_path, &placed.report_csv_path] {
        let text = fs::read_to_string(path).expect("csv");
        let mut lines = text.lines();
        let header = csv_field_count(lines.next().expect("header"));
        let mut rows = 0;
        for line in lines {
            rows += 1;
            assert_eq!(
                csv_field_count(line),
                header,
                "{}: row has the wrong column count: {line}",
                path.display()
            );
        }
        assert!(
            rows >= 4,
            "{} should list accepted and skipped files",
            path.display()
        );
    }
}
