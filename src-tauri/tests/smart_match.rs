//! Smart Match plan, Slice 3: the decision function's links to the measured
//! library. The per-rule tests live next to the function in
//! `src/smart_match.rs`; these need the committed evidence and the runner.

use std::path::PathBuf;

use yes_master_lib::smart_balance_reference::{
    median, read_stats, typical_shape_rust_source, TYPICAL_SHAPE_RUST_FILE,
};
use yes_master_lib::smart_match::{
    median6, TYPICAL_SHAPE_BUILD_SONGS, TYPICAL_SHAPE_DB, TYPICAL_SHAPE_PROVISIONAL,
};

fn committed_stats_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "../docs/reviews/evidence/2026-11-24-smart-balance-reference-range/smart-balance-reference-stats.json",
    )
}

#[test]
fn smart_match_typical_shape_matches_the_committed_statistics() {
    let stats = read_stats(&committed_stats_path()).expect("committed statistics");
    let range = stats.range.as_ref().expect("a range");
    assert_eq!(TYPICAL_SHAPE_DB, range.typical_shape_db);
    assert_eq!(TYPICAL_SHAPE_BUILD_SONGS, stats.counts.build);
    assert_eq!(TYPICAL_SHAPE_PROVISIONAL, stats.provisional);

    // The committed table is exactly what the tool emits: nobody typed it.
    let emitted = typical_shape_rust_source(&stats).expect("emit");
    let committed = std::fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(TYPICAL_SHAPE_RUST_FILE),
    )
    .expect("read the generated table");
    assert_eq!(
        committed.replace('\r', ""),
        emitted,
        "re-run the tool with --emit-typical-shape"
    );
}

#[test]
fn the_product_median_agrees_with_the_runner_median() {
    let mut seed = 0xC0FFEE_u64;
    let mut next = || {
        seed = seed
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        ((seed >> 11) as f64 / (1u64 << 53) as f64) * 60.0 - 30.0
    };
    for _ in 0..500 {
        let values = [next(), next(), next(), next(), next(), next()];
        assert_eq!(median6(&values), median(&values));
    }
}
