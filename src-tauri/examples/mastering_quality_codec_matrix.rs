//! B4 actual encoders/decoded meters over a frozen list of prepared stimuli.
//! Args: inputs.json fresh-output-dir. Audio and reports stay in ignored storage.
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{path::Path, time::Instant};
use yes_master_lib::{
    decode, export_encoding,
    export_format::ExportEncoding,
    mp3, output_protection,
    peak_meter::{self, pcm::PcmSource},
    wav_writer::DeliveryPcm,
};

fn sha(path: &Path) -> String {
    format!("{:x}", Sha256::digest(std::fs::read(path).unwrap()))
}

fn write_pcm(path: &Path, samples: &[f32], rate: u32, channels: u16, bits: u16) {
    let source = DeliveryPcm::new(samples, channels, bits, || false).unwrap();
    let mut writer = hound::WavWriter::create(
        path,
        hound::WavSpec {
            channels,
            sample_rate: rate,
            bits_per_sample: bits,
            sample_format: if bits == 32 {
                hound::SampleFormat::Float
            } else {
                hound::SampleFormat::Int
            },
        },
    )
    .unwrap();
    let mut buffers = vec![vec![0.; 4096]; usize::from(channels)];
    for start in (0..source.frames()).step_by(4096) {
        let length = 4096.min(source.frames() - start);
        for (channel, buffer) in buffers.iter_mut().enumerate() {
            source
                .read_channel(channel, start, &mut buffer[..length])
                .unwrap();
        }
        for frame in 0..length {
            for buffer in &buffers {
                if bits == 32 {
                    writer.write_sample(buffer[frame] as f32).unwrap();
                } else {
                    writer
                        .write_sample((buffer[frame] * (1_u32 << (bits - 1)) as f64) as i32)
                        .unwrap();
                }
            }
        }
    }
    writer.finalize().unwrap();
}

fn main() {
    let args: Vec<_> = std::env::args().collect();
    assert_eq!(args.len(), 3);
    let inputs: Vec<Value> = serde_json::from_slice(&std::fs::read(&args[1]).unwrap()).unwrap();
    let out = Path::new(&args[2]);
    assert!(!out.exists(), "preserve evidence");
    std::fs::create_dir(out).unwrap();
    let encoder = export_encoding::Encoder::packaged().unwrap();
    let mut rows = Vec::new();
    for input in inputs {
        let path = Path::new(input["path"].as_str().unwrap());
        assert_eq!(sha(path), input["sha256"].as_str().unwrap());
        let pcm = decode::decode_full(path).unwrap();
        let id = input["id"].as_str().unwrap();
        let encodings: Vec<ExportEncoding> =
            serde_json::from_value(input["encodings"].clone()).unwrap();
        for bits in [16, 24, 32] {
            let choices: Vec<_> = encodings
                .iter()
                .copied()
                .filter(|e| e.is_lossy() == (bits == 32))
                .collect();
            if choices.is_empty() {
                continue;
            }
            let mut samples = pcm.samples.clone();
            let start = Instant::now();
            // Push the prepared stimulus to the requested peak constraint; this
            // matrix characterizes codec growth, not a new mastering voicing.
            let protected = output_protection::finalize(
                &mut samples,
                pcm.sample_rate,
                pcm.channels,
                bits,
                if input["preserve_level"].as_bool().unwrap_or(false) {
                    None
                } else {
                    Some(-3.)
                },
                -1.,
                None,
            )
            .unwrap();
            let preparation_s = start.elapsed().as_secs_f64();
            assert!(protected.true_peak_dbtp <= -1. + 1e-5);
            let prepared_path = out.join(format!("{id}-{bits}-pcm.wav"));
            write_pcm(
                &prepared_path,
                &samples,
                pcm.sample_rate,
                pcm.channels,
                bits,
            );
            for encoding in choices {
                assert_eq!(encoding.delivery_rate(pcm.sample_rate), pcm.sample_rate);
                let suffix = match encoding {
                    ExportEncoding::Mp3 { bitrate_kbps }
                    | ExportEncoding::M4a { bitrate_kbps }
                    | ExportEncoding::Aac { bitrate_kbps } => bitrate_kbps.to_string(),
                    ExportEncoding::Ogg { quality } => quality.to_string(),
                    _ => bits.to_string(),
                };
                let destination = out.join(format!("{id}-{suffix}.{}", encoding.extension()));
                let start = Instant::now();
                let result = if let ExportEncoding::Mp3 { bitrate_kbps } = encoding {
                    mp3::write(
                        &destination,
                        samples.iter().copied().map(Ok),
                        pcm.sample_rate,
                        pcm.channels,
                        bitrate_kbps,
                        None,
                    )
                    .and_then(|path| mp3::measure(&path, None).map(|m| (path, m)))
                } else {
                    export_encoding::deliver_staged(
                        &encoder,
                        &prepared_path,
                        &destination,
                        encoding,
                        None,
                    )
                    .map(|file| (file.path, file.measurements))
                };
                let elapsed = start.elapsed().as_secs_f64();
                let row = match result {
                    Ok((path, measured)) => {
                        json!({"id":id,"encoding":encoding,"bits":bits,"rate":pcm.sample_rate,"channels":pcm.channels,
                        "input":prepared_path,"path":path,"sha256":sha(&path),"input_sha256":sha(&prepared_path),
                        "source_sha256":input["sha256"],"experiment":input["experiment"],"frames":samples.len()/usize::from(pcm.channels),
                        "input_lufs":protected.lufs,"input_peak":protected.true_peak_dbtp,"decoded_lufs":measured.0,"decoded_peak":measured.1,
                        "decoded_lra":measured.2,"growth_db":measured.1-protected.true_peak_dbtp,
                        "ceiling_miss":measured.1 > -1. + 1e-5,"target_miss_lu":measured.0+3.,
                        "preparation_s":preparation_s,"encode_readback_s":elapsed})
                    }
                    Err(error) => {
                        json!({"id":id,"encoding":encoding,"bits":bits,"rate":pcm.sample_rate,"channels":pcm.channels,"error":error.to_string(),"encode_readback_s":elapsed})
                    }
                };
                println!("{id} {encoding:?} {bits}: {:.3} s", elapsed);
                rows.push(row);
                std::fs::write(out.join("report.json"),serde_json::to_vec_pretty(&json!({"status":"in_progress","estimator":peak_meter::VERSION,"inputs_sha256":sha(Path::new(&args[1])),"rows":rows})).unwrap()).unwrap();
            }
        }
    }
    std::fs::write(out.join("report.json"),serde_json::to_vec_pretty(&json!({"status":"complete","estimator":peak_meter::VERSION,"inputs_sha256":sha(Path::new(&args[1])),"rows":rows})).unwrap()).unwrap();
    assert!(
        rows.iter().all(|r| r.get("error").is_none()),
        "retain errors; matrix did not pass"
    );
}
