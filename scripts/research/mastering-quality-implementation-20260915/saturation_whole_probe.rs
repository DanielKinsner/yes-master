//! Generated offline whole-chain E experiment. No production switch is added.
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{path::Path, sync::Arc, time::Instant};
use yes_master_lib::{decode, engine, guardrails, output_protection, types};
#[allow(dead_code)]
mod dsp;
mod native;
#[allow(dead_code)]
mod sample_rate;

mod saturation_experiment {
    use super::*;
    use std::cell::RefCell;
    enum Variant { Current, Wide, Filter(native::Processor) }
    struct State { variant: Variant, delay: usize }
    thread_local! { static STATE: RefCell<Option<State>> = const { RefCell::new(None) }; }
    pub fn initialize(mode: &str, kernel: Option<Arc<native::Kernel>>, amount: f32) -> usize {
        let delay = kernel.as_ref().map_or(0, |k| k.delay);
        let variant = match mode {
            "current" => Variant::Current,
            "wide" => Variant::Wide,
            _ => Variant::Filter(native::Processor::new(kernel.unwrap(), f64::from(amount))),
        };
        STATE.with(|s| *s.borrow_mut()=Some(State { variant, delay }));
        delay
    }
    pub fn delay() -> usize { STATE.with(|s| s.borrow().as_ref().unwrap().delay) }
    pub fn process(frame: &mut [f32], channels: usize, amount: f32) {
        assert_eq!(channels,2);
        STATE.with(|state| match &mut state.borrow_mut().as_mut().unwrap().variant {
            Variant::Current => {
                if amount>0. {
                    let drive=1.+amount*2.;
                    let denominator=drive.tanh().max(1e-3);
                    for x in frame.iter_mut().take(channels) { *x=(*x*drive).tanh()/denominator; }
                }
            }
            Variant::Wide => {
                if amount>0. {
                    let drive=1.+f64::from(amount)*2.;
                    let denominator=drive.tanh();
                    for x in frame.iter_mut().take(channels) { *x=((f64::from(*x)*drive).tanh()/denominator) as f32; }
                }
            }
            Variant::Filter(processor) => {
                let result=processor.frame(&[f64::from(frame[0]),f64::from(frame[1])]);
                frame[0]=result[0] as f32; frame[1]=result[1] as f32;
            }
        });
    }
}

fn sha(path: &Path) -> String { format!("{:x}",Sha256::digest(std::fs::read(path).unwrap())) }
fn read_kernel(path: &Path) -> Arc<native::Kernel> {
    let bytes=std::fs::read(path).unwrap();
    assert_eq!(bytes.len()%8,0);
    let values: Vec<_>=bytes.chunks_exact(8).map(|x| f64::from_le_bytes(x.try_into().unwrap())).collect();
    Arc::new(native::Kernel::new(&values))
}
fn maximum_difference(x: &[f32],y: &[f32]) -> f64 {
    assert_eq!(x.len(),y.len());
    x.iter().zip(y).map(|(&a,&b)| (f64::from(a)-f64::from(b)).abs()).fold(0.,f64::max)
}
fn render(input: &[f32],rate: u32,settings: &types::MasteringSettings,drive: f32,
          mode: &str,kernel: Option<Arc<native::Kernel>>,coefficients: &str) -> (Vec<f32>,Value) {
    let mut chain=dsp::MasteringChain::new(rate,2,settings);
    chain.coeffs.input_gain_lin*=10_f32.powf(drive/20.);
    assert_eq!(format!("{:?}",chain.coeffs),coefficients);
    assert!(chain.coeffs.saturation_amount>0.);
    let start=Instant::now();
    let filter_delay=saturation_experiment::initialize(mode,kernel,chain.coeffs.saturation_amount);
    let state_prepare_s=start.elapsed().as_secs_f64();
    let delay=chain.latency_frames();
    let mut samples=input.to_vec();
    samples.resize(samples.len()+delay*2,0.);
    let mut reductions=Vec::with_capacity(samples.len()/2);
    let start=Instant::now();
    for frame in samples.chunks_exact_mut(2) {
        chain.process_frame_inplace(frame);
        reductions.push(chain.limiter.gain_reduction_db());
    }
    let chain_s=start.elapsed().as_secs_f64();
    samples.copy_within(delay*2..,0); samples.truncate(input.len());
    let reductions=&mut reductions[delay..];
    let active=reductions.iter().filter(|&&x| x>0.01).count();
    reductions.sort_unstable_by(f32::total_cmp);
    let percentile=|q:f64| reductions[((reductions.len()-1) as f64*q).round() as usize];
    let metrics=json!({"chain_and_observation_s":chain_s,"state_prepare_s":state_prepare_s,
        "filter_delay_frames":filter_delay,"total_delay_frames":delay,
        "limiter":{"p50":percentile(0.5),"p95":percentile(0.95),"p99":percentile(0.99),
            "max":percentile(1.),"active_fraction_above_0_01db":active as f64/reductions.len() as f64}});
    (samples,metrics)
}
fn write(path: &Path,samples: &[f32]) {
    assert!(!path.exists());
    let mut writer=hound::WavWriter::create(path,hound::WavSpec {channels:2,sample_rate:48000,
        bits_per_sample:32,sample_format:hound::SampleFormat::Float}).unwrap();
    for &x in samples {assert!(x.is_finite()); writer.write_sample(x).unwrap();}
    writer.finalize().unwrap();
}
fn main() {
    let args:Vec<_>=std::env::args().collect();
    assert_eq!(args.len(),3,"job.json fresh-output-directory");
    let job_path=Path::new(&args[1]); let output=Path::new(&args[2]);
    assert!(!output.exists()); std::fs::create_dir_all(output).unwrap();
    let job:Value=serde_json::from_slice(&std::fs::read(job_path).unwrap()).unwrap();
    let start=Instant::now();
    let mut kernels=Vec::new();
    for info in job["kernels"].as_array().unwrap() {
        let path=Path::new(info["path"].as_str().unwrap());
        assert_eq!(sha(path),info["sha256"].as_str().unwrap());
        kernels.push(read_kernel(path));
    }
    assert_eq!(kernels.len(),2);
    let mut result=json!({"status":"running","job_sha256":sha(job_path),
        "kernel_read_and_prepare_s":start.elapsed().as_secs_f64(),"rows":[],
        "scope":"Isolated matched whole-chain E experiment; exact current-control proof, f64 arithmetic control, finite FIR latency and qualified full-file output; no production adoption"});
    let mut numeric_pass=true;
    for case in job["cases"].as_array().unwrap() {
        let report_path=Path::new(case["report"].as_str().unwrap());
        assert_eq!(sha(report_path),case["report_sha256"].as_str().unwrap());
        let prior:Value=serde_json::from_slice(&std::fs::read(report_path).unwrap()).unwrap();
        assert_eq!(prior["status"],"complete");
        let row=prior["rows"].as_array().unwrap().iter().find(|r| r["id"]==case["candidate"]).unwrap();
        assert_eq!(row["source_gain_db"],0.);
        let source=Path::new(prior["source"].as_str().unwrap());
        assert_eq!(sha(source),prior["source_sha256"].as_str().unwrap());
        let pcm=decode::decode_full(source).unwrap(); assert_eq!(pcm.channels,2);
        let mut input=pcm.samples;
        if row["policy"]!="control" {
            assert_eq!(row["policy"],"single");
            let peak=input.iter().map(|x| f64::from(x.abs())).fold(0.,f64::max);
            for x in &mut input {*x=(f64::from(*x)/peak) as f32;}
        }
        let settings:types::MasteringSettings=serde_json::from_value(row["requested_settings"].clone()).unwrap();
        let drive=row["operating_drive_db"].as_f64().unwrap() as f32;
        let coefficients=row["render"]["coefficients"].as_str().unwrap();
        let mut reference=input.clone();
        let mut chain=yes_master_lib::dsp::MasteringChain::new(pcm.sample_rate,2,&settings);
        chain.coeffs.input_gain_lin*=10_f32.powf(drive/20.);
        assert_eq!(format!("{:?}",chain.coeffs),coefficients);
        chain.process_interleaved(&mut reference,2);chain.flush_render_tail(&mut reference,2);
        for (mode,kernel) in [("current",None),("wide",None),("short",Some(kernels[0].clone())),("long",Some(kernels[1].clone()))] {
            let (raw,metrics)=render(&input,pcm.sample_rate,&settings,drive,mode,kernel,coefficients);
            let difference=maximum_difference(&raw,&reference);
            if mode=="current" {assert!(raw.iter().zip(&reference).all(|(a,b)| a.to_bits()==b.to_bits()),"instrumented control must match production exactly");}
            if mode=="wide" {numeric_pass &= difference<=1e-5;}
            let started=Instant::now();
            let mut delivered=sample_rate::convert_interleaved(&raw,pcm.sample_rate,48000,2).unwrap();
            let src_s=started.elapsed().as_secs_f64();
            let raw_lufs=engine::measure_integrated_lufs(&delivered,48000,2).unwrap();
            let started=Instant::now();
            let protection=output_protection::finalize(&mut delivered,48000,2,32,settings.effective_target_lufs(),settings.effective_ceiling_dbtp(),None).unwrap();
            let finalize_s=started.elapsed().as_secs_f64();
            assert!(protection.true_peak_dbtp<=settings.effective_ceiling_dbtp()+1e-5);
            let path=output.join(format!("{}-{mode}.wav",case["id"].as_str().unwrap()));write(&path,&delivered);
            result["rows"].as_array_mut().unwrap().push(json!({"case":case["id"],"variant":mode,"path":path,"sha256":sha(&path),
                "source":source,"source_sha256":prior["source_sha256"],"requested_settings":settings,"drive":drive,
                "raw_difference_from_current":difference,"render":metrics,"src_s":src_s,"finalize_s":finalize_s,
                "frames":delivered.len()/2,"rate":48000,"channels":2,"raw_lufs":raw_lufs,"lufs":protection.lufs,
                "peak":protection.true_peak_dbtp,"ceiling":settings.effective_ceiling_dbtp(),"gain":protection.gain_lin}));
            std::fs::write(output.join("report.json"),serde_json::to_vec_pretty(&result).unwrap()).unwrap();
            println!("Completed {} {mode}",case["id"]);
        }
    }
    result["status"]=json!("complete");result["f64_isolation_pass"]=json!(numeric_pass);
    std::fs::write(output.join("report.json"),serde_json::to_vec_pretty(&result).unwrap()).unwrap();
    assert!(numeric_pass,"f64 arithmetic isolation exceeded its frozen limit");
}
