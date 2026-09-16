"""Build an isolated arithmetic experiment; never alter the production DSP."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--output', required=True, type=Path)
    parser.add_argument('--baseline-ref', default='3fecbbe9',
                        help='local pre-correction DSP commit; no checkout or network operation')
    args = parser.parse_args()
    assert not args.output.exists(), 'preserve previous experiments'
    root = Path(__file__).resolve().parents[3]
    baseline = subprocess.check_output(['git', 'rev-parse', args.baseline_ref], cwd=root, text=True).strip()
    original = subprocess.check_output(['git', 'show', f'{baseline}:src-tauri/src/dsp.rs'], cwd=root).decode('utf-8').replace('\r\n', '\n')
    start = original.index('#[derive(Debug, Clone, Copy, Default)]\npub struct BiquadState')
    end = original.index('// ============================================================================', start)
    replacement = '''#[derive(Debug, Clone, Copy, Default)]
pub struct BiquadState {
    z1: f64,
    z2: f64,
}

impl BiquadState {
    pub fn process(&mut self, c: &BiquadCoeffs, x: f32) -> f32 {
        let x = f64::from(x);
        let y = f64::from(c.b0) * x + self.z1;
        let z1 = f64::from(c.b1) * x - f64::from(c.a1) * y + self.z2;
        let z2 = f64::from(c.b2) * x - f64::from(c.a2) * y;
        // Preserve the existing floor, including its exact f32 value.
        self.z1 = if z1.abs() < f64::from(1.0e-20_f32) { 0.0 } else { z1 };
        self.z2 = if z2.abs() < f64::from(1.0e-20_f32) { 0.0 } else { z2 };
        y as f32
    }
}

'''
    wide = original[:start] + replacement + original[end:]
    args.output.mkdir(parents=True)
    source = args.output / 'src'
    source.mkdir()
    (source / 'dsp_narrow.rs').write_text(original, encoding='utf-8')
    (source / 'dsp_wide.rs').write_text(wide, encoding='utf-8')
    (source / 'sample_rate.rs').write_bytes((root / 'src-tauri/src/sample_rate.rs').read_bytes())
    (source / 'main.rs').write_bytes(Path(__file__).with_name('wide_state_probe.rs').read_bytes())
    # Use equal optimization for both DSP implementations: production is a
    # dependency here, so package-wide opt-level 3 must also apply to this root.
    manifest = f'''[package]
name = "mastering-wide-state-probe"
version = "0.1.0"
edition = "2021"
rust-version = "1.85"
[workspace]
[dependencies]
yes-master = {{path = {json.dumps((root / 'src-tauri').as_posix())}}}
serde_json = "1"
sha2 = "0.10"
hound = "3"
ebur128 = "0.1"
rubato = "1.0.1"
audioadapter-buffers = "2.0.0"
[profile.dev]
opt-level = 3
[profile.dev.package."*"]
opt-level = 3
'''
    (args.output / 'Cargo.toml').write_text(manifest, encoding='utf-8')
    # Retain all registry versions from the product. Cargo adds only this local
    # wrapper package; building without this lock would test different DSP deps.
    (args.output / 'Cargo.lock').write_bytes((root / 'src-tauri/Cargo.lock').read_bytes())
    (args.output / 'provenance.json').write_text(json.dumps(dict(
        baseline_commit=baseline,
        original_dsp_sha256=hashlib.sha256(original.encode()).hexdigest(),
        wide_dsp_sha256=hashlib.sha256(wide.encode()).hexdigest(),
        scope='Only BiquadState arithmetic/state widened; coefficients, f32 stage outputs, floor, presets and other DSP unchanged',
        files={p.name: hashlib.sha256(p.read_bytes()).hexdigest() for p in source.iterdir()},
    ), indent=2), encoding='utf-8')
    print(args.output / 'Cargo.toml')


if __name__ == '__main__':
    main()
