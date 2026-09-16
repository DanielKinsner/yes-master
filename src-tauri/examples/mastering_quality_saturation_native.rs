//! Isolated native finite-filter/cost probe. Production DSP is unchanged.
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{path::Path, sync::Arc, time::Instant};

fn sha(path: &Path) -> String {
    format!("{:x}", Sha256::digest(std::fs::read(path).unwrap()))
}
fn read_f64(path: &Path) -> Vec<f64> {
    let bytes = std::fs::read(path).unwrap();
    assert_eq!(bytes.len() % 8, 0);
    let values: Vec<_> = bytes
        .chunks_exact(8)
        .map(|x| f64::from_le_bytes(x.try_into().unwrap()))
        .collect();
    assert!(values.iter().all(|x| x.is_finite()));
    values
}
fn write_f64(path: &Path, values: &[f64]) {
    assert!(!path.exists());
    let bytes: Vec<_> = values.iter().flat_map(|x| x.to_le_bytes()).collect();
    std::fs::write(path, bytes).unwrap();
}

struct Kernel {
    phases: Vec<Vec<f64>>,
    down: Vec<f64>,
    delay: usize,
}
impl Kernel {
    fn new(coefficients: &[f64]) -> Self {
        assert!(coefficients.len() > 8 && (coefficients.len() - 1) % 8 == 0);
        let width = coefficients.len().div_ceil(8);
        let phases = (0..8)
            .map(|phase| {
                (0..width)
                    .rev()
                    .map(|j| coefficients.get(j * 8 + phase).copied().unwrap_or(0.) * 8.)
                    .collect()
            })
            .collect();
        Self {
            phases,
            down: coefficients.iter().rev().copied().collect(),
            delay: (coefficients.len() - 1) / 8,
        }
    }
}

struct Ring {
    data: Vec<f64>,
    next: usize,
}
impl Ring {
    fn new(length: usize) -> Self {
        Self {
            data: vec![0.; length * 2],
            next: 0,
        }
    }
    fn push(&mut self, value: f64) {
        let length = self.data.len() / 2;
        self.data[self.next] = value;
        self.data[self.next + length] = value;
        self.next = (self.next + 1) % length;
    }
    fn dot(&self, coefficients: &[f64]) -> f64 {
        let length = self.data.len() / 2;
        let mut samples = self.data[self.next..self.next + length].chunks_exact(4);
        let mut weights = coefficients.chunks_exact(4);
        // Independent dependency chains allow instruction-level parallelism.
        // The changed f64 addition order has a separate finite-output check.
        let (mut a, mut b, mut c, mut d) = (0., 0., 0., 0.);
        for (x, h) in samples.by_ref().zip(weights.by_ref()) {
            a += x[0] * h[0];
            b += x[1] * h[1];
            c += x[2] * h[2];
            d += x[3] * h[3];
        }
        samples
            .remainder()
            .iter()
            .zip(weights.remainder())
            .fold((a + b) + (c + d), |sum, (x, h)| sum + x * h)
    }
}
struct Processor {
    kernel: Arc<Kernel>,
    up: [Ring; 2],
    down: [Ring; 2],
    drive: f64,
    denominator: f64,
}
impl Processor {
    fn new(kernel: Arc<Kernel>, amount: f64) -> Self {
        let up = std::array::from_fn(|_| Ring::new(kernel.phases[0].len()));
        let down = std::array::from_fn(|_| Ring::new(kernel.down.len()));
        let drive = 1. + 2. * amount;
        Self {
            kernel,
            up,
            down,
            drive,
            denominator: drive.tanh(),
        }
    }
    fn frame(&mut self, input: &[f64]) -> [f64; 2] {
        let mut result = [0.; 2];
        for ch in 0..2 {
            self.up[ch].push(input[ch]);
            for phase in 0..8 {
                let interpolated = self.up[ch].dot(&self.kernel.phases[phase]);
                self.down[ch].push((interpolated * self.drive).tanh() / self.denominator);
                if phase == 0 {
                    result[ch] = self.down[ch].dot(&self.kernel.down);
                }
            }
        }
        result
    }
}

fn main() {
    let args: Vec<_> = std::env::args().collect();
    assert_eq!(args.len(), 3, "job.json fresh-output-directory");
    let job_path = Path::new(&args[1]);
    let output = Path::new(&args[2]);
    assert!(!output.exists());
    let job: Value = serde_json::from_slice(&std::fs::read(job_path).unwrap()).unwrap();
    std::fs::create_dir_all(output).unwrap();
    let mut report = json!({"status":"running","job_sha256":sha(job_path),"rows":[],
        "scope":"Native f64 finite positive-curve/filter mechanism; no production f32 chain or device callback claim"});
    for filter in job["filters"].as_array().unwrap() {
        let path = Path::new(filter["path"].as_str().unwrap());
        assert_eq!(sha(path), filter["sha256"].as_str().unwrap());
        let coefficients = read_f64(path);
        let start = Instant::now();
        let kernel = Arc::new(Kernel::new(&coefficients));
        let preparation_s = start.elapsed().as_secs_f64();
        for case in job["signals"].as_array().unwrap() {
            let input_path = Path::new(case["path"].as_str().unwrap());
            assert_eq!(sha(input_path), case["sha256"].as_str().unwrap());
            let mut input = read_f64(input_path);
            assert_eq!(input.len() % 2, 0);
            let frames = input.len() / 2;
            input.resize(input.len() + kernel.delay * 4, 0.);
            let mut reference: Option<Vec<f64>> = None;
            let mut trials = Vec::new();
            for partition in [1, 257, 1024] {
                let start = Instant::now();
                let mut processor = Processor::new(kernel.clone(), 0.0715);
                let construction_s = start.elapsed().as_secs_f64();
                let mut actual = Vec::with_capacity(input.len());
                let start = Instant::now();
                let mut block_start = Instant::now();
                let mut block_times_ns = Vec::with_capacity(input.len() / 512 + 1);
                let mut count = 0;
                for chunk in input.chunks(partition * 2) {
                    for frame in chunk.chunks_exact(2) {
                        actual.extend_from_slice(&processor.frame(frame));
                        count += 1;
                        if count % 256 == 0 {
                            block_times_ns.push(block_start.elapsed().as_nanos() as u64);
                            block_start = Instant::now();
                        }
                    }
                }
                let processing_s = start.elapsed().as_secs_f64();
                assert!(actual.iter().all(|x| x.is_finite()));
                if let Some(reference) = &reference {
                    assert_eq!(&actual, reference);
                } else {
                    reference = Some(actual);
                }
                trials.push(
                    json!({"partition":partition,"construction_s":construction_s,
                    "processing_s":processing_s,"block_256_ns":block_times_ns}),
                );
            }
            let path = output.join(format!(
                "{}-{}.f64le",
                filter["id"].as_str().unwrap(),
                case["id"].as_str().unwrap()
            ));
            write_f64(&path, reference.as_ref().unwrap());
            report["rows"].as_array_mut().unwrap().push(json!({"filter":filter["id"],"signal":case["id"],
                "path":path,"sha256":sha(&path),"frames_in":frames,"frames_out":input.len()/2,
                "rate":44100,"channels":2,"delay_frames":kernel.delay,"preparation_s":preparation_s,
                "kernel_bytes":8*(kernel.down.len()+kernel.phases.iter().map(Vec::len).sum::<usize>()),
                "state_bytes":2*2*8*(kernel.phases[0].len()+kernel.down.len()),"trials":trials}));
            std::fs::write(
                output.join("report.json"),
                serde_json::to_vec_pretty(&report).unwrap(),
            )
            .unwrap();
        }
    }
    report["status"] = json!("complete");
    std::fs::write(
        output.join("report.json"),
        serde_json::to_vec_pretty(&report).unwrap(),
    )
    .unwrap();
}
