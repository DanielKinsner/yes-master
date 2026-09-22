"""Independent final-pass checks. Research only; never changes sealed evidence.

Run from the repo: python scripts/research/final-review-20260915/verify.py
Requires restored independent workspace at the conventional sibling path.
"""
from pathlib import Path
import collections
import hashlib
import json
import subprocess
import re
import sys
import os

import numpy as np
from scipy.signal import resample
import soundfile as sf

ROOT = Path(__file__).resolve().parents[3]
AUDIT = ROOT.parent / 'yes-master-independent-audit-20260915'
RECON = AUDIT / 'audit-output/reconciliation-20260915'
OUT = ROOT / 'test-output/mastering-quality-final-review-20260915'


def sha(path):
    with path.open('rb') as f:
        return hashlib.file_digest(f, 'sha256').hexdigest()


def db(v):
    return float(20 * np.log10(max(float(v), 1e-15)))


def interp(seg, factor, corrected=True):
    spectrum = np.fft.rfft(seg)
    if corrected and len(seg) % 2 == 0:
        # An even-length input Nyquist bin becomes a positive/negative pair.
        spectrum[-1] *= 0.5
    return np.fft.irfft(spectrum, n=len(seg) * factor) * factor


def peak_reference(x, factor=16, chunk=1 << 18, overlap=8192, corrected=True):
    """Finite-block periodic FFT reference, NOT an exact physical true peak.

    Overlap reduces block-edge dependence. File edges are zero-extended.
    Include original samples; even-length Nyquist is split correctly.
    """
    peak = float(np.max(np.abs(x)))
    for channel in x.T:
        for start in range(0, len(x), chunk):
            stop = min(start + chunk, len(x))
            a, b = max(0, start - overlap), min(len(x), stop + overlap)
            seg = channel[a:b]
            left, right = max(0, overlap - start), max(0, stop + overlap - len(x))
            seg = np.pad(seg, (left, right))
            up = interp(seg, factor, corrected)
            lo = (start - a + left) * factor
            hi = lo + (stop - start) * factor
            peak = max(peak, float(np.abs(up[lo:hi]).max()))
    return db(peak)


def float_soxr(path, rate):
    filt = f'aformat=sample_fmts=dbl,aresample={rate*16}:resampler=soxr:precision=33:osf=dbl,astats=metadata=0:reset=0'
    run = subprocess.run(['ffmpeg','-hide_banner','-nostats','-i',str(path),'-af',filt,
                          '-c:a','pcm_f64le','-f','null','-'], capture_output=True, text=True, check=True)
    return float(re.findall(r'Peak level dB: ([-\d.]+)', run.stderr)[-1])


def relative(path):
    for base, prefix in [(AUDIT, 'audit'), (ROOT, 'repo')]:
        try:
            return prefix + '/' + path.resolve().relative_to(base.resolve()).as_posix()
        except ValueError:
            pass
    raise ValueError(path)


def recorded_path(value):
    """Resolve historical Windows paths against restored roots on any machine."""
    value = value.replace('\\', '/')
    for marker, base in [('yes-master-independent-audit-20260915/', AUDIT), ('yes-master/', ROOT)]:
        if marker in value:
            return base / value.split(marker, 1)[1]
    return AUDIT / 'audit-output' / value


def regenerate_witnesses():
    """Claude pruned these intermediates; regenerate without altering its workspace."""
    harness = RECON / ('build/target/release/yes-recon-harness' + ('.exe' if os.name=='nt' else ''))
    dest = OUT / 'witnesses'
    dest.mkdir(exist_ok=True)
    baseline = json.loads((AUDIT / 'audit-output/specs/baseline_export01.json').read_text())
    def run(command, source, filename, spec=None, extra=()):
        output = dest / filename
        if output.exists():
            return output
        args = [str(harness), command, str(source), str(output)]
        if spec is not None:
            settings = dest / (filename + '.spec.json')
            settings.write_text(json.dumps(spec, indent=2)+'\n')
            args += ['--spec', str(settings), '--json', str(dest / (filename+'.receipt.json'))]
        result = subprocess.run(args + list(extra), capture_output=True, text=True, check=True)
        (dest / (filename+'.log')).write_text(result.stdout + result.stderr)
        return output
    owner = run('render', AUDIT/'fixtures/inputs/input01.wav', 'owner_on.wav', baseline)
    assert sha(owner) == '639ac1dd5d049040d8b689636c08bf2890b39929b72601703d1b93938f6022ce'
    source = AUDIT/'fixtures/inputs/input04.wav'
    chain = run('chain', source, 'regrow_input04_chain441.f32.wav', baseline)
    run('src-file',chain,'regrow_input04_prod48.f32.wav',extra=['--to','48000','--mode','prod'])
    no_target = dict(baseline, intensity=.5, lufs_target=None)
    run('render',source,'defect_input04_custom_notarget_24.wav',no_target)
    run('render',source,'defect_input04_custom_notarget_f32.wav',dict(no_target,bit_depth=32))
    run('render',AUDIT/'audit-output/synthetic/clip_50ms.wav','defect_clip50ms_streaming_24.wav',
        dict(intensity=.5,delivery_profile='streaming',bit_depth=24))
    deleted = json.loads((RECON/'results/deleted_renders_hashes.json').read_text())
    checks = []
    for p in dest.glob('*.wav'):
        expected = [r['sha256'] for r in deleted if Path(r['path']).name==p.name]
        if expected:
            checks.append({'file':p.name,'sha256':sha(p),'matches_pruned_record':sha(p) in expected})
    (OUT/'regeneration.json').write_text(json.dumps(checks,indent=2)+'\n')
    print('Fresh owner On reproduced byte-identically; witnesses regenerated', checks, flush=True)
    return dest


def main():
    OUT.mkdir(parents=True, exist_ok=True)
    destination = OUT / 'verification.json'
    if destination.exists():
        raise SystemExit('Preserve completed output; choose a fresh OUT for changed checks.')
    seals = {}
    for name in ['audit-output/SEALED_MANIFEST.json', 'audit-output/reconciliation-20260915/SEALED_MANIFEST.json']:
        manifest = json.loads((AUDIT / name).read_text())
        matches = {p: sha(AUDIT / p) == h for p, h in manifest['files'].items()}
        assert all(matches.values()), matches
        seals[name] = {'sha256': sha(AUDIT / name), 'verified_files': len(matches), 'all_match': True}

    rng = np.random.default_rng(20260915)
    controls = []
    for n in [31, 32, 4095, 4096]:
        for name, x in [('noise', rng.normal(0, .1, n)), ('dc', np.full(n, .25)),
                        ('nyquist', .5 * (-1.) ** np.arange(n))]:
            y = interp(x, 16)
            z = resample(x, n * 16)
            error = float(np.max(np.abs(y[::16] - x)))
            scipy_error = float(np.max(np.abs(y - z)))
            assert error < 1e-12 and scipy_error < 1e-12
            controls.append({'n':n, 'kind':name, 'original_grid_error':error,
                             'scipy_difference':scipy_error,
                             'old_grid_error':float(np.max(np.abs(interp(x,16,False)[::16]-x)))})
    print('Seals and FFT reference controls passed', flush=True)
    witnesses = regenerate_witnesses()

    src = json.loads((RECON / 'results/src_probe_analysis.json').read_text())
    counts = {mode: {'cases': sum(r['mode']==mode for r in src),
                     'exact_lengths':sum(r['mode']==mode and r['length_ok'] for r in src)}
              for mode in ['prod','recon','codex']}
    assert counts == {'prod': {'cases':408,'exact_lengths':368},
                      'recon': {'cases':408,'exact_lengths':408},
                      'codex': {'cases':408,'exact_lengths':408}}
    zero_inputs = [r for r in src if r['frames_in']==1 and r['rms']==0]

    files = []
    for group in ['tp_meters_regrowth_input04.json','tp_meters_blind_defects.json']:
        for row in json.loads((RECON / 'results' / group).read_text()):
            p = recorded_path(row['file'])
            if 'recon48' in p.name or 'swr48' in p.name or 'soxr48' in p.name or p.name.endswith('streaming_f32.wav'):
                continue
            if not p.exists():
                p = witnesses / p.name
            files.append((p, row))
    # All original 25 ceiling failures, plus exploratory centred finalists.
    rows = json.loads((RECON / 'results/tp_meters_failing_set.json').read_text())
    rows += json.loads((RECON / 'results/tp_meters_codex_finalists.json').read_text())
    seen = {str(p.resolve()) for p, _ in files}
    for row in rows:
        p = recorded_path(row['file'])
        if str(p.resolve()) not in seen:
            seen.add(str(p.resolve()))
            files.append((p,row))
    # Synthesized EBU-style controls are not the official downloaded test set.
    for p in sorted((RECON / 'renders/tp_synth').glob('*.wav')):
        files.append((p,{}))

    measurements = []
    for p, old in files:
        x, sr = sf.read(p, dtype='float64', always_2d=True)
        row = {'file':relative(p), 'sha256':sha(p), 'rate':sr, 'frames':len(x),
               'sample_peak_dbfs':db(abs(x).max()),
               'samples_over_full_scale':int(np.sum(abs(x)>1)),
               'corrected_fft16_db':peak_reference(x),
               'old_report_fft16_db':old.get('fftexact16_tp'),
               'old_report_soxr16_db':old.get('soxr16_tp'),
               'native_tp_db':old.get('engine_tp'),
               'ffmpeg_tp_db':old.get('ffmpeg_tp')}
        # Expensive float-SOXR and overlap/factor convergence on critical witnesses.
        if len(measurements)<6 or 'tp_synth' in str(p) or p.name=='drive_t14.wav':
            row['float_soxr16_db'] = float_soxr(p,sr)
            row['corrected_fft32_overlap16384_db'] = peak_reference(x,32,1<<18,16384)
        measurements.append(row)
        print(p.name, row['corrected_fft16_db'], flush=True)
        (OUT / 'measurements.partial.json').write_text(json.dumps(measurements,indent=2)+'\n')

    result = {'python':sys.version, 'numpy':np.__version__, 'seals':seals,
              'fft_controls':controls, 'src_lengths':counts,
              'degenerate_zero_input_rows':len(zero_inputs), 'measurements':measurements,
              'limitations':['Finite-block FFT reconstruction is a reference, not a universal true-peak oracle.',
                             'No production implementation, native playback or listening claim.']}
    destination.write_text(json.dumps(result,indent=2)+'\n')
    print('Saved', destination, flush=True)


if __name__ == '__main__':
    main()
