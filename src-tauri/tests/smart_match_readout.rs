//! Smart Match plan, Slice 5: the readout and the receipt.
//!
//! The readout's state, bands, amount and partial flag, and the receipt's
//! locked wording, all come from the decision the chain uses (contract rules
//! 10 and 11, plan M17).

use yes_master_lib::guardrails::{readout_for, readout_for_surface};
use yes_master_lib::smart_match::{
    decide_for_settings, readout_for_settings, receipt_digest, SmartMatchState, TYPICAL_SHAPE_DB,
};
use yes_master_lib::{
    AdvancedSettings, DeliveryProfile, EqBandFrequencies, MasteringSettings, Preset,
    SmartMatchReference, SmartMatchSettings, SourceProfile, SpectralBalance6,
};

fn shares_from_db(levels: [f64; 6]) -> SpectralBalance6 {
    let lin = levels.map(|d| 10f64.powf(d / 10.0));
    let total: f64 = lin.iter().sum();
    let s = lin.map(|v| (v / total) as f32);
    SpectralBalance6 {
        sub: s[0],
        low: s[1],
        low_mid: s[2],
        mid: s[3],
        presence: s[4],
        air: s[5],
    }
}

fn with_offsets(offsets: [f64; 6]) -> SpectralBalance6 {
    let mut levels = TYPICAL_SHAPE_DB;
    for (l, o) in levels.iter_mut().zip(offsets) {
        *l += o;
    }
    shares_from_db(levels)
}

fn settings(
    track: Option<SpectralBalance6>,
    state: Option<SmartMatchSettings>,
) -> MasteringSettings {
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
        eq_bands: EqBandFrequencies::default(),
        volume_match: false,
        source_lufs_integrated: None,
        input_gain_db: 0.0,
        output_gain_db: 0.0,
        delivery_profile: DeliveryProfile::Custom,
        album: None,
        advanced: AdvancedSettings {
            source_profile: track.map(|spectral_6| SourceProfile {
                spectral_6,
                dynamic_range_p95_p10_db: 12.0,
                dynamic_range_lu: 6.0,
                stereo_correlation: Some(0.6),
                stereo_width: 0.3,
            }),
            smart_match: state,
            ..Default::default()
        },
    }
}

fn state(
    reference: Option<SpectralBalance6>,
    enabled: bool,
    amount: f32,
) -> Option<SmartMatchSettings> {
    Some(SmartMatchSettings {
        reference: reference.map(|spectral_6| SmartMatchReference {
            name: "LANDR-It’s a coat-original-test-Warm-High".into(),
            spectral_6,
        }),
        enabled,
        amount,
    })
}

#[test]
fn each_copy_table_state_comes_from_the_decision() {
    let track = with_offsets([0.0; 6]);
    let darker = with_offsets([0.0, 0.0, -2.4, 0.0, 0.0, 1.6]);
    let cases: Vec<(MasteringSettings, bool, Option<SmartMatchState>)> = vec![
        (settings(Some(track), None), false, None),
        (
            settings(Some(track), state(None, true, 0.5)),
            false,
            Some(SmartMatchState::NoReference),
        ),
        (
            settings(None, state(Some(darker), true, 0.5)),
            false,
            Some(SmartMatchState::TrackNotReady),
        ),
        (
            settings(Some(track), state(Some(darker), true, 0.5)),
            false,
            Some(SmartMatchState::Applied),
        ),
        (
            settings(Some(track), state(Some(track), true, 0.5)),
            false,
            Some(SmartMatchState::NothingToChange),
        ),
        (
            settings(Some(track), state(Some(darker), false, 0.5)),
            false,
            Some(SmartMatchState::SwitchedOff),
        ),
        (
            settings(Some(track), state(Some(darker), true, 0.0)),
            false,
            Some(SmartMatchState::ZeroAmount),
        ),
        (
            settings(Some(track), state(Some(darker), true, 0.5)),
            true,
            Some(SmartMatchState::AlbumDisabled),
        ),
    ];
    for (s, album, expected) in cases {
        let readout = readout_for_settings(&s, album);
        assert_eq!(readout.map(|r| r.state), expected, "album {album}");
        assert_eq!(readout_for_surface(&s, album).smart_match, readout);
    }
}

#[test]
fn readout_numbers_are_the_chains_numbers() {
    let track = with_offsets([0.0; 6]);
    let reference = with_offsets([3.0, -9.0, 0.0, 0.0, 0.0, 2.0]);
    let s = settings(Some(track), state(Some(reference), true, 0.5));
    let decision = decide_for_settings(&s);
    let readout = readout_for(&s).smart_match.expect("a readout");
    assert_eq!(readout.amount, decision.amount);
    assert_eq!(readout.partial, decision.partial);
    assert!(
        readout.partial,
        "a 9 dB bass gap at 50% hits the 4 dB cut cap"
    );
    for b in 0..6 {
        assert_eq!(readout.bands[b].correction_db, decision.correction_db[b]);
        assert_eq!(readout.bands[b].capped, decision.capped[b]);
        assert_eq!(readout.bands[b].gap_db, decision.gap_db.map(|g| g[b]));
    }
    // Switched off, the readout still shows how far apart the songs are.
    let off = settings(Some(track), state(Some(reference), false, 0.5));
    let r = readout_for(&off).smart_match.unwrap();
    assert!(r
        .bands
        .iter()
        .all(|b| b.gap_db.is_some() && b.correction_db == 0.0));
}

#[test]
fn the_receipt_line_uses_the_locked_wording() {
    let track = with_offsets([0.0; 6]);
    // No reference stored: no line at all.
    assert_eq!(receipt_digest(&settings(Some(track), None)), None);
    assert_eq!(
        receipt_digest(&settings(Some(track), state(None, true, 0.5))),
        None
    );
    // Stored but off, at zero, or the track not analysed: off.
    let r = with_offsets([0.0, 0.0, -2.4, 0.0, 0.0, 1.6]);
    for s in [
        settings(Some(track), state(Some(r), false, 0.5)),
        settings(Some(track), state(Some(r), true, 0.0)),
        settings(None, state(Some(r), true, 0.5)),
    ] {
        assert_eq!(receipt_digest(&s).as_deref(), Some("Smart Match: off"));
    }
    // Already close.
    assert_eq!(
        receipt_digest(&settings(Some(track), state(Some(track), true, 0.5))).as_deref(),
        Some("Smart Match: \"LANDR-It’s a coat-original-test-Warm-High\" at 50% · already close")
    );
    // The plan's example: cleared 1.2 dB mud, added 0.8 dB air.
    assert_eq!(
        receipt_digest(&settings(Some(track), state(Some(r), true, 0.5))).as_deref(),
        Some(
            "Smart Match: \"LANDR-It’s a coat-original-test-Warm-High\" at 50% · cleared 1.2 dB mud, added 0.8 dB air"
        )
    );
}

#[test]
fn the_receipt_names_the_three_largest_moves_in_both_directions() {
    let track = with_offsets([0.0; 6]);
    // Gaps (median 0): sub −5, bass +3, low mids 0, mids 0, presence −2, air +4.
    let r = with_offsets([-5.0, 3.0, 0.0, 0.0, -2.0, 4.0]);
    // At 70%: sub −3.5, air +2.8, bass +2.1, presence −1.4 (the fourth, unnamed).
    let line = receipt_digest(&settings(Some(track), state(Some(r), true, 0.7))).unwrap();
    assert_eq!(
        line,
        "Smart Match: \"LANDR-It’s a coat-original-test-Warm-High\" at 70% · tamed 3.5 dB sub rumble, added 2.8 dB air, added 2.1 dB bass"
    );
    // Every cut and boost phrase.
    let phrases = [
        ("tamed", "sub rumble", "added", "deep bass"),
        ("tamed", "boom", "added", "bass"),
        ("cleared", "mud", "added", "warmth"),
        ("eased", "midrange", "added", "midrange"),
        ("softened", "harshness", "added", "presence"),
        ("softened", "brightness", "added", "air"),
    ];
    for (b, (cut_verb, cut_noun, boost_verb, boost_noun)) in phrases.iter().enumerate() {
        for (sign, verb, noun) in [(-1.0, cut_verb, cut_noun), (1.0, boost_verb, boost_noun)] {
            let mut offsets = [0.0; 6];
            offsets[b] = 2.0 * sign;
            let line = receipt_digest(&settings(
                Some(track),
                state(Some(with_offsets(offsets)), true, 0.5),
            ))
            .unwrap();
            assert!(
                line.ends_with(&format!("· {verb} 1.0 dB {noun}")),
                "band {b} sign {sign}: {line}"
            );
        }
    }
}

#[test]
fn a_render_with_an_active_match_carries_the_receipt_line() {
    let dir = std::env::temp_dir().join(format!("ym-sm-receipt-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let source = yes_master_lib::demo::prepare_demo_track_in(&dir).unwrap();
    let pcm = yes_master_lib::decode::decode_full(&source).unwrap();
    let track =
        yes_master_lib::smart_match::measure_shares(&pcm.samples, pcm.sample_rate, 2).unwrap();
    let mut levels = yes_master_lib::smart_match::shares_to_db(
        &yes_master_lib::smart_match::shares_array(&track),
    );
    levels[2] -= 3.0;
    levels[5] += 2.0;
    let s = settings(Some(track), state(Some(shares_from_db(levels)), true, 0.5));
    let expected = receipt_digest(&s).unwrap();
    assert!(expected.contains("mud"), "{expected}");
    let out = dir.join("out");
    std::fs::create_dir_all(&out).unwrap();
    let job = yes_master_lib::engine::mastering_render(
        yes_master_lib::TrackId("receipt".into()),
        &source,
        &s,
        &out,
        yes_master_lib::RenderKind::Master,
    )
    .unwrap();
    let m = job.measurements.expect("rendered measurements");
    assert_eq!(m.smart_match_digest.as_deref(), Some(expected.as_str()));
    // A render that never used Smart Match writes no such key.
    let plain = settings(Some(track), None);
    let job = yes_master_lib::engine::mastering_render(
        yes_master_lib::TrackId("receipt-plain".into()),
        &source,
        &plain,
        &out,
        yes_master_lib::RenderKind::Master,
    )
    .unwrap();
    let json = serde_json::to_string(&job).unwrap();
    assert!(!json.contains("smart_match_digest"), "{json}");
    let _ = std::fs::remove_dir_all(&dir);
}
