//! Diagnostic of cached, compensating gain edits through the actual live chain.
use super::*;
use crate::{
    dsp::{ChainCoeffs, MasteringChain},
    quality_source::QualitySource,
    sources::FadeEnvelope,
};
use serde_json::json;

fn make_source(
    samples: &[f32],
    rate: u32,
    settings: &MasteringSettings,
    coeffs: ChainCoeffs,
) -> (MasteringSource, Sender<LiveCoeffUpdate>) {
    let (tx, rx) = mpsc::channel();
    let mut chain = MasteringChain::new(rate, 2, settings);
    chain.coeffs = coeffs;
    let source = MasteringSource::new(
        samples.to_vec(),
        2,
        rate,
        chain,
        rx,
        Arc::new(AtomicU32::new(0)),
        Arc::new(AtomicU32::new(0)),
        Arc::new(AtomicU32::new(0)),
        Arc::new(AtomicI32::new(i32::MIN)),
        Arc::new(AtomicI32::new(i32::MIN)),
        Arc::new(SpectrumRing::new()),
    )
    .with_initial_revision(1);
    (source, tx)
}

#[test]
fn mastering_quality_compensating_gain_transitions() {
    let path = std::env::var("YES_MASTER_GAIN_TRANSITION_REPORT")
        .ok()
        .map(PathBuf::from);
    assert!(path.as_ref().is_none_or(|path| !path.exists()));
    let mut rows = Vec::new();
    for (rate, file, device) in [
        (44100, 44100, 44100),
        (44100, 96000, 48000),
        (48000, 48000, 44100),
        (96000, 44100, 48000),
    ] {
        let samples: Vec<f32> = (0..rate * 4)
            .flat_map(|i| {
                let x = 0.05 * (i as f32 * std::f32::consts::TAU * 997. / rate as f32).sin();
                [x, x]
            })
            .collect();
        let settings = tests::settings_with_intensity(0.75);
        let mut initial = ChainCoeffs::from_settings(rate, &settings);
        initial.export_landing_gain_lin = 1.;
        initial.volume_match_gain_lin = 1.;
        for output_db in [-12., 12.] {
            let mut edited = settings.clone();
            edited.output_gain_db = output_db;
            let mut next = ChainCoeffs::from_settings(rate, &edited);
            next.export_landing_gain_lin = 1.;
            next.volume_match_gain_lin = 1.;
            let landing = 0.5 * initial.user_output_gain_lin / next.user_output_gain_lin;
            let mut compensated = initial;
            compensated.export_landing_gain_lin = 0.5;
            let (source, tx) = make_source(&samples, rate, &settings, compensated);
            let mailbox = gain_stage::GainMailbox::new(gain_stage::GainPlan {
                revision: 1,
                raw_revision: 1,
                landing: 1.,
                volume_match: 1.,
            });
            let (mut current, failure) = output_route::mastered_source(
                source,
                file,
                device,
                FadeEnvelope::inactive(),
                mailbox.clone(),
            )
            .unwrap();
            for _ in 0..device * 2 {
                current.next().unwrap();
            }
            mailbox.publish(gain_stage::GainPlan {
                revision: 2,
                raw_revision: 2,
                landing: 1.,
                volume_match: 1.,
            });
            let mut compensated = next;
            compensated.export_landing_gain_lin = landing;
            tx.send(LiveCoeffUpdate {
                generation: 2,
                coeffs: compensated,
            })
            .unwrap();
            let actual: Vec<f32> = current.take(device as usize * 2).collect();
            assert_eq!(failure.load(Ordering::Acquire), 0);
            // Prior route mixed the already compensated masters before SRC.
            // Keep this reference local to the experiment, with the same DSP,
            // converters, finite alignment and exact update point.
            let mut before = initial;
            before.export_landing_gain_lin = 0.5;
            let (source, tx) = make_source(&samples, rate, &settings, before);
            let mut control = QualitySource::new(
                QualitySource::new(source.with_render_alignment(), file).unwrap(),
                device,
            )
            .unwrap();
            for _ in 0..device * 2 {
                control.next().unwrap();
            }
            let mut after = next;
            after.export_landing_gain_lin = landing;
            tx.send(LiveCoeffUpdate {
                generation: 2,
                coeffs: after,
            })
            .unwrap();
            let reference: Vec<f32> = control.take(device as usize * 2).collect();
            assert_eq!(actual.len(), reference.len());
            let window = device as usize / 200 * 2;
            let stride = device as usize / 1000 * 2;
            let mut windows = Vec::new();
            for start in (0..device as usize / 5 * 2).step_by(stride) {
                let rms = |x: &[f32]| x.iter().map(|v| f64::from(*v).powi(2)).sum::<f64>();
                let delta = 10.
                    * (rms(&actual[start..start + window])
                        / rms(&reference[start..start + window]))
                    .log10();
                windows
                    .push(json!({"start_ms":start as f64/2./device as f64*1000.,"delta_db":delta}));
            }
            rows.push(json!({"source_rate":rate,"file_rate":file,"device_rate":device,"output_gain_db":output_db,
                "initial_landing":0.5,"new_landing":landing,
                "min_window_delta_db":windows.iter().map(|x|x["delta_db"].as_f64().unwrap()).fold(f64::INFINITY,f64::min),
                "max_window_delta_db":windows.iter().map(|x|x["delta_db"].as_f64().unwrap()).fold(f64::NEG_INFINITY,f64::max),
                "windows":windows}));
        }
    }
    let report = json!({"status":"complete","rows":rows,"maximum_allowed_window_delta_db":0.01,
        "scope":"5ms windows of actual production gain/SRC transitions versus prior compensated-master crossfade; steady endpoints have equal intended gain; no listening verdict"});
    if let Some(path) = path {
        std::fs::write(path, serde_json::to_vec_pretty(&report).unwrap()).unwrap();
    }
    for row in report["rows"].as_array().unwrap() {
        assert!(
            row["min_window_delta_db"].as_f64().unwrap() >= -0.01
                && row["max_window_delta_db"].as_f64().unwrap() <= 0.01,
            "compensating edit changed intended level: {row}"
        );
    }
}
