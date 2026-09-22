#[path = "../../../src-tauri/src/types.rs"] pub mod types;
pub mod dsp;
#[path = "../../../src-tauri/src/analysis.rs"] pub mod analysis;
#[path = "../../../src-tauri/src/deep_analysis.rs"] pub mod deep_analysis;
#[path = "../../../src-tauri/src/export_format.rs"] pub mod export_format;
mod guardrails; mod confidence;
#[path = "../../../web/tryit/wasm/src/stubs.rs"] mod stubs;
pub use stubs::{decode,files,mp3};
#[path = "../../../src-tauri/src/sample_rate.rs"] mod sample_rate;
#[path = "../../../src-tauri/src/wav_writer.rs"] mod wav_writer;
use types::*;
use dsp::MasteringChain;
use serde_json::{json,Value};
use std::path::PathBuf;
use sha2::{Digest,Sha256};
fn measure(x:&[f32],ch:u32,sr:u32)->Value {
    let mut e=ebur128::EbuR128::new(ch,sr,ebur128::Mode::I|ebur128::Mode::LRA|ebur128::Mode::TRUE_PEAK).unwrap();
    e.add_frames_f32(x).unwrap();
    let tp=(0..ch).map(|c|e.true_peak(c).unwrap()).fold(0.0_f64,f64::max);
    let pk=x.iter().map(|s|s.abs() as f64).fold(0.0,f64::max);
    let rms=(x.iter().map(|s|(*s as f64).powi(2)).sum::<f64>()/x.len() as f64).sqrt();
    json!({"lufs":e.loudness_global().unwrap(),"lra":e.loudness_range().unwrap(),"true_peak_dbtp":20.0*tp.log10(),"sample_peak_dbfs":20.0*pk.log10(),"rms_dbfs":20.0*rms.log10(),"crest_db":20.0*(pk/rms).log10(),"frames":x.len()/ch as usize})
}
fn n(v:&Value,key:&str,default:f32)->f32 {v[key].as_f64().map(|x|x as f32).unwrap_or(default)}
fn main(){
 let args:Vec<String>=std::env::args().collect();
 let job:Value=serde_json::from_slice(&std::fs::read(&args[1]).unwrap()).unwrap();
 let src=PathBuf::from(job["source"].as_str().unwrap());let out=PathBuf::from(job["output"].as_str().unwrap());std::fs::create_dir_all(&out).unwrap();
 let mut r=hound::WavReader::open(&src).unwrap();let spec=r.spec();let ch=spec.channels as u32;
 let mut input:Vec<f32>=if spec.sample_format==hound::SampleFormat::Float {r.samples::<f32>().map(Result::unwrap).collect()} else {let scale=(1i64<<(spec.bits_per_sample-1)) as f32;r.samples::<i32>().map(|s|s.unwrap() as f32/scale).collect()};
 let ing=n(&job,"source_gain_db",0.0);if ing!=0.0 {for x in &mut input{*x*=10f32.powf(ing/20.0);}}
 assert!(input.iter().all(|x|x.is_finite()));
 let loud=measure(&input,ch,spec.sample_rate);let dr=analysis::compute_dynamic_range_p95_p10(&input,spec.sample_rate,ch as usize);
 let profile=SourceProfile::from_measurements(analysis::compute_spectral_balance_6band(&input,spec.sample_rate,ch as usize),dr,loud["lra"].as_f64().unwrap_or(0.0) as f32,analysis::compute_stereo_correlation(&input,ch as usize),analysis::compute_stereo_width(&input,ch as usize)).unwrap();
 let settings=MasteringSettings{preset:Preset::Universal,intensity:0.75,eq_sub_db:0.0,eq_low_db:0.0,eq_low_mid_db:0.0,eq_mid_db:0.0,eq_high_mid_db:0.0,eq_high_db:0.0,eq_sparkle_db:0.0,eq_bands:EqBandFrequencies::default(),volume_match:false,source_lufs_integrated:loud["lufs"].as_f64().map(|x|x as f32),input_gain_db:0.0,output_gain_db:0.0,delivery_profile:DeliveryProfile::Custom,album:None,advanced:AdvancedSettings{lufs_offset_db:Some(-9.0),adaptive_strength:Some(0.5),source_profile:Some(profile.clone()),target_sample_rate:Some(48000),bit_depth:Some(24),..AdvancedSettings::default()}};
 std::fs::write(out.join("source.json"),serde_json::to_vec_pretty(&json!({"measurements":loud,"native_dr100":dr,"profile":profile,"source_gain_db":ing,"source":src,"base_settings":settings})).unwrap()).unwrap();
 for v in job["variants"].as_array().unwrap(){
  let name=v["name"].as_str().unwrap();let path=out.join(format!("{name}.json"));assert!(!path.exists(),"Refusing mixed evidence: {}",path.display());
  let time=std::time::Instant::now();let mut s=settings.clone();s.input_gain_db=n(v,"trim",0.0);s.advanced.compression_density=Some(n(v,"density",0.5));s.advanced.adaptive_strength=Some(n(v,"adapt",0.5));s.intensity=n(v,"intensity",0.75);
  if v["comp"].as_str()==Some("off"){s.advanced.compression_mode=CompressionMode::Off;}
  if let Some(p)=v["preset"].as_str(){s.preset=serde_json::from_value(json!({"kind":p})).unwrap();}
  let mut chain=MasteringChain::new(spec.sample_rate,ch as usize,&s);
  let mut target_s=s.clone();target_s.advanced.lufs_offset_db=Some(-14.0);
  assert_eq!(format!("{:?}",chain.coeffs),format!("{:?}",dsp::ChainCoeffs::from_settings(spec.sample_rate,&target_s)));
  match v["sat"].as_str().unwrap_or("current") {"off"=>chain.coeffs.saturation_amount=0.0,"half"=>chain.coeffs.saturation_amount*=0.5,"continuous"=>chain.coeffs.saturation_amount*=-1.0,_=>{}}
  if v["limit"].as_str()==Some("off"){chain.limiter=dsp::Limiter::new(spec.sample_rate,ch as usize,60.0,3.0,50.0);}
  let coeff=format!("{:#?}",chain.coeffs);let coeff_hash=format!("{:x}",Sha256::digest(coeff.as_bytes()));
  let mut pcm=input.clone();let chunk=n(v,"chunk",4096.0) as usize;
  for block in pcm.chunks_mut(chunk*ch as usize){chain.process_interleaved(block,ch as usize);}
  chain.flush_render_tail(&mut pcm,ch as usize);
  let pcm=sample_rate::convert_interleaved(&pcm,spec.sample_rate,48000,spec.channels).unwrap();assert!(pcm.iter().all(|x|x.is_finite()&&x.abs()<1000.0));
  let before=measure(&pcm,ch,48000);let lufs=before["lufs"].as_f64().unwrap_or(-200.0) as f32;let tp=before["true_peak_dbtp"].as_f64().unwrap_or(-200.0) as f32;
  let mut deliveries=vec![];
  for target in [-9.0_f32,-14.0]{let delta=(target-lufs).min(s.effective_ceiling_dbtp()-tp);deliveries.push(json!({"target":target,"landing_db":delta,"predicted_lufs":lufs+delta,"predicted_tp":tp+delta,"target_error":target-lufs-delta}));}
  let mut row=json!({"name":name,"variant":v,"pre_landing":before,"deliveries":deliveries,"stats":chain.diag,"coeff_hash":coeff_hash,"settings":s,"elapsed_seconds":time.elapsed().as_secs_f64()});
  if v["write"].as_bool().unwrap_or(false){
    let target=n(v,"target",-9.0);let delta=(target-lufs).min(s.effective_ceiling_dbtp()-tp);let mut pcm=pcm;
    if delta.abs()>1.0e-4{let g=10f32.powf(delta/20.0);for x in &mut pcm{*x*=g;}}
    let wav=wav_writer::write_wav(&out.join(format!("{name}.wav")),&pcm,48000,spec.channels,24).unwrap();
    row["file"]=json!(wav);row["delivered_native"]=json!(wav_writer::measure_delivery(&pcm,48000,spec.channels,24).unwrap());
    std::fs::write(out.join(format!("{name}-coeffs.txt")),coeff).unwrap();
  }
  std::fs::write(path,serde_json::to_vec_pretty(&row).unwrap()).unwrap();println!("{} {:.2} LUFS {:.2} crest {:.2}s",name,lufs,before["crest_db"].as_f64().unwrap_or(0.0),time.elapsed().as_secs_f64());
 }
}
