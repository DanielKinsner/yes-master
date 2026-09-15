//! Local A/B witness: real export plus a zero-padded continuous-chain reference.
//! Args: source.wav historical-receipt.json fresh-output-directory
use serde_json::json;
use sha2::{Digest, Sha256};
use std::path::Path;
use yes_master_lib::types;
use yes_master_lib::{decode, dsp::MasteringChain, engine, types::*};
#[path = "../src/sample_rate.rs"]
mod sample_rate;

fn main() {
    let args: Vec<_> = std::env::args().collect();
    assert_eq!(args.len(), 4);
    let source = Path::new(&args[1]);
    let record: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&args[2]).unwrap()).unwrap();
    let settings: MasteringSettings = serde_json::from_value(record["settings"].clone()).unwrap();
    let out = Path::new(&args[3]);
    assert!(!out.exists(), "preserve completed evidence");
    std::fs::create_dir_all(out).unwrap();
    let job = engine::mastering_render_to_path(
        TrackId("witness".into()),
        source,
        &settings,
        out,
        RenderKind::Master,
        &out.join("delivered.wav"),
    )
    .unwrap();
    let pcm = decode::decode_full(source).unwrap();
    let ch = usize::from(pcm.channels);
    let mut chain = MasteringChain::new(pcm.sample_rate, ch, &settings);
    let mut raw = pcm.samples.clone();
    for block in raw.chunks_mut(8192 * ch) {
        chain.process_interleaved(block, ch);
    }
    chain.flush_render_tail(&mut raw, ch);
    let rate = settings.effective_sample_rate(pcm.sample_rate);
    let expected = ((raw.len() / ch) as u128 * u128::from(rate))
        .div_ceil(u128::from(pcm.sample_rate)) as usize;
    raw.resize(raw.len() + 8192 * ch, 0.0);
    let padded =
        sample_rate::convert_interleaved(&raw, pcm.sample_rate, rate, pcm.channels).unwrap();
    let mut writer = hound::WavWriter::create(
        out.join("padded-reference.wav"),
        hound::WavSpec {
            channels: pcm.channels,
            sample_rate: rate,
            bits_per_sample: 32,
            sample_format: hound::SampleFormat::Float,
        },
    )
    .unwrap();
    for &x in &padded[..expected * ch] {
        writer.write_sample(x).unwrap();
    }
    writer.finalize().unwrap();
    let report = json!({"source_sha256":format!("{:x}",Sha256::digest(std::fs::read(source).unwrap())),
        "settings":settings,"job":job,"expected_frames":expected,"rate":rate,
        "delivered_sha256":format!("{:x}",Sha256::digest(std::fs::read(out.join("delivered.wav")).unwrap()))});
    std::fs::write(
        out.join("report.json"),
        serde_json::to_vec_pretty(&report).unwrap(),
    )
    .unwrap();
}
