"""Generate a pinned, isolated whole-chain E experiment without production edits."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess


def digest(data):
    return hashlib.sha256(data).hexdigest()


def main():
    parser=argparse.ArgumentParser()
    parser.add_argument('--output',type=Path,required=True)
    parser.add_argument('--baseline-ref',default='7e4a7327')
    args=parser.parse_args()
    assert not args.output.exists()
    root=Path(__file__).resolve().parents[3]
    commit=subprocess.check_output(['git','rev-parse',args.baseline_ref],cwd=root,text=True).strip()
    read=lambda path:subprocess.check_output(['git','show',f'{commit}:{path}'],cwd=root).decode('utf-8').replace('\r\n','\n')
    original=read('src-tauri/src/dsp.rs')
    assert (root/'src-tauri/src/dsp.rs').read_text(encoding='utf-8')==original, 'production control must still match the pinned DSP'
    begin=original.index('        if self.coeffs.saturation_amount > 0.0 {',original.index('// Pass 2: per-channel saturation.'))
    end=original.index('        self.limiter.process_frame_inplace(frame);',begin)
    changed=original[:begin]+'        saturation_experiment::process(frame, channels, self.coeffs.saturation_amount);\n'+original[end:]
    target='        self.limiter.lookahead_frames\n    }'
    assert changed.count(target)==1
    changed=changed.replace(target,'        self.limiter.lookahead_frames + saturation_experiment::delay()\n    }')
    changed='use crate::saturation_experiment;\n'+changed
    native=read('src-tauri/examples/mastering_quality_saturation_native.rs')
    native='use std::sync::Arc;\n'+native[native.index('struct Kernel {'):native.index('fn main() {')]
    native=native.replace('struct Kernel {','pub struct Kernel {').replace('struct Processor {','pub struct Processor {')
    native=native.replace('    fn new(','    pub fn new(').replace('    fn frame(','    pub fn frame(').replace('    delay: usize,','    pub delay: usize,')
    args.output.mkdir(parents=True)
    source=args.output/'src';source.mkdir()
    (source/'dsp.rs').write_text(changed,encoding='utf-8')
    (source/'native.rs').write_text(native,encoding='utf-8')
    (source/'sample_rate.rs').write_text(read('src-tauri/src/sample_rate.rs'),encoding='utf-8')
    (source/'main.rs').write_bytes(Path(__file__).with_name('saturation_whole_probe.rs').read_bytes())
    manifest=f'''[package]
name = "mastering-saturation-whole-probe"
version = "0.1.0"
edition = "2021"
rust-version = "1.85"
[workspace]
[dependencies]
yes-master = {{path = {json.dumps((root/'src-tauri').as_posix())}}}
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
    (args.output/'Cargo.toml').write_text(manifest,encoding='utf-8')
    (args.output/'Cargo.lock').write_bytes((root/'src-tauri/Cargo.lock').read_bytes())
    (args.output/'provenance.json').write_text(json.dumps(dict(baseline_commit=commit,
        original_dsp_sha256=digest(original.encode()),files={f.name:digest(f.read_bytes()) for f in source.iterdir()},
        scope='Only saturation dispatch and its added latency in an isolated DSP copy; current-control exact comparison is mandatory'),indent=2)+'\n',encoding='utf-8')
    print(args.output/'Cargo.toml')


if __name__=='__main__':main()
