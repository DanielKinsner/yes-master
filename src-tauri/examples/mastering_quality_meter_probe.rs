//! Qualified provider-based library meter against retained standalone evidence.
//! Args: input-list.json fresh-output.json. Does not overwrite evidence.
use serde_json::json;
use sha2::{Digest, Sha256};
use std::path::Path;
use std::time::Instant;
use yes_master_lib::peak_meter;

fn main() {
    let args: Vec<_> = std::env::args().collect();
    assert_eq!(args.len(), 3);
    let entries: Vec<serde_json::Value> =
        serde_json::from_slice(&std::fs::read(&args[1]).unwrap()).unwrap();
    let out = Path::new(&args[2]);
    assert!(!out.exists(), "preserve completed evidence");
    let mut rows = Vec::new();
    for entry in entries {
        let path = Path::new(entry["path"].as_str().unwrap());
        let pcm = yes_master_lib::decode::decode_full(path).unwrap();
        let start = Instant::now();
        let measured =
            peak_meter::measure(&pcm.samples, usize::from(pcm.channels), || false).unwrap();
        let seconds = start.elapsed().as_secs_f64();
        rows.push(json!({"id":entry["id"], "source_sha256":format!("{:x}",Sha256::digest(std::fs::read(path).unwrap())),
            "rate":pcm.sample_rate,"frames":pcm.samples.len()/usize::from(pcm.channels),"channels":pcm.channels,
            "candidate":measured.channels.iter().map(|c| &c.finite).collect::<Vec<_>>(),
            "candidate_seconds":seconds,"fir_grid":measured.channels.iter().map(|c|c.lowpass_grid).collect::<Vec<_>>(),
            "combined_upper":measured.upper(),"interval_db":measured.widest_interval_db()}));
        std::fs::write(
            out,
            serde_json::to_vec_pretty(&json!({"version":peak_meter::VERSION,"rows":rows})).unwrap(),
        )
        .unwrap();
        println!("{}: {seconds:.3} s", entry["id"]);
    }
}
