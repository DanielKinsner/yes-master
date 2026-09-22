"""Create an instrumented private DSP copy. Control operations stay exact."""
from pathlib import Path
import hashlib,json
out=Path(__file__).resolve().parent
source=out.parents[1]/'src-tauri/src/dsp.rs'
s=source.read_text(encoding='utf8')
old='''        if self.coeffs.saturation_amount > 0.0 {
            let drive = 1.0 + self.coeffs.saturation_amount * 2.0;
            let denom = drive.tanh().max(1.0e-3);
            for sample in frame.iter_mut().take(channels) {
                *sample = (*sample * drive).tanh() / denom;
            }
        }'''
assert s.count(old)==1
new='''        for sample in frame.iter().take(channels) {
            self.diag.sat_in_power += (*sample as f64).powi(2);
            self.diag.sat_in_peak = self.diag.sat_in_peak.max(sample.abs());
        }
        if self.coeffs.saturation_amount > 0.0 {
            let drive = 1.0 + self.coeffs.saturation_amount * 2.0;
            let denom = drive.tanh().max(1.0e-3);
            for sample in frame.iter_mut().take(channels) {
                *sample = (*sample * drive).tanh() / denom;
            }
        } else if self.coeffs.saturation_amount < 0.0 {
            // Diagnostic negative coefficient denotes continuous unity-slope mapping.
            let k = -2.0 * self.coeffs.saturation_amount;
            for sample in frame.iter_mut().take(channels) {
                *sample = (*sample * k).tanh() / k;
            }
        }
        for sample in frame.iter().take(channels) {
            self.diag.sat_out_power += (*sample as f64).powi(2);
            self.diag.sat_out_peak = self.diag.sat_out_peak.max(sample.abs());
        }
        self.diag.frames += 1;'''
s=s.replace(old,new)
s=s.replace('pub struct MasteringChain {','''#[derive(Default, Clone, Debug, serde::Serialize)]
pub struct DiagnosticStats {
    pub frames: u64,
    pub sat_in_power: f64, pub sat_out_power: f64,
    pub sat_in_peak: f32, pub sat_out_peak: f32,
    pub limiter_gr_sum: f64, pub limiter_gr_max: f32, pub limiter_active: u64,
    pub comp_gr_sum: [f64;3], pub comp_gr_max: [f32;3], pub comp_active: [u64;3],
}
pub struct MasteringChain {
    pub diag: DiagnosticStats,''')
s=s.replace('            gr_snapshots: GrSnapshotSlots::default(),','            gr_snapshots: GrSnapshotSlots::default(),\n            diag: DiagnosticStats::default(),')
s=s.replace('            gr_snapshots: prior.gr_snapshots.clone(),','            gr_snapshots: prior.gr_snapshots.clone(),\n            diag: DiagnosticStats::default(),')
s=s.replace('        self.limiter.process_frame_inplace(frame);','''        self.limiter.process_frame_inplace(frame);
        let lgr = -20.0 * self.limiter.gain.max(1.0e-20).log10();
        self.diag.limiter_gr_sum += lgr as f64;
        self.diag.limiter_gr_max = self.diag.limiter_gr_max.max(lgr);
        self.diag.limiter_active += u64::from(lgr > 0.1);''')
needle='''        let to_u = |db: f32| (db.max(0.0) * 100.0) as u32;'''
assert s.count(needle)==1
s=s.replace(needle,'''        for (b, gr) in [max_gr_db_low,max_gr_db_mid,max_gr_db_high].iter().enumerate() {
            self.diag.comp_gr_sum[b] += *gr as f64;
            self.diag.comp_gr_max[b] = self.diag.comp_gr_max[b].max(*gr);
            self.diag.comp_active[b] += u64::from(*gr > 0.1);
        }
'''+needle)
(out/'src/dsp.rs').write_text(s,encoding='utf8')
main=(out/'src/main.rs').read_text()
main=main.replace('#[path = "../../../src-tauri/src/dsp.rs"]\npub mod dsp;','pub mod dsp;')
(out/'src/main.rs').write_text(main)
(out/'instrumentation-provenance.json').write_text(json.dumps(dict(source_sha256=hashlib.sha256(source.read_bytes()).hexdigest(),private_dsp_sha256=hashlib.sha256((out/'src/dsp.rs').read_bytes()).hexdigest(),changes='Added read-only stage statistics; retained exact positive saturation operations; negative coefficient selects private continuous mapping.'),indent=2))
