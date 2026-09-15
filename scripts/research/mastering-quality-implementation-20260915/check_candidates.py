"""C0 whole-file checks with retained failures; never a production policy.

Reuses the checked FFT/SOXR references and descriptive metrics, with every
generated artifact redirected into a fresh output directory. No sealed output
or source is modified. LUFS/LRA from FFmpeg are rounded to 0.1 LU.
"""
import argparse
import importlib.util
import json
from pathlib import Path
import time
import numpy as np
import soundfile as sf

ROOT = Path(__file__).resolve().parents[3]


def module(name, path):
    spec = importlib.util.spec_from_file_location(name, ROOT / path)
    result = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(result)
    return result


ref = module('checked_reference', 'scripts/research/final-review-20260915/verify.py')
shape = module('shape', 'scripts/research/mastering-quality-20260915/analyze_music.py')


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--source', type=Path, required=True)
    parser.add_argument('--run', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    assert not args.output.exists(), 'preserve completed results'
    args.output.mkdir(parents=True)
    shape.OUT = args.output  # historical metric function writes only here
    report = json.loads((args.run / 'report.json').read_text())
    assert ref.sha(args.source) == report['source_sha256']
    controls = []
    from scipy.signal import resample
    for n in [31, 32, 4095, 4096]:
        for x in [np.full(n, .25), .5 * (-1.)**np.arange(n), np.random.default_rng(n).normal(size=n)]:
            y = ref.interp(x, 16)
            assert np.max(np.abs(y[::16] - x)) < 1e-12
            assert np.max(np.abs(y - resample(x, n * 16))) < 1e-12
            controls.append(n)
    source = shape.metric(args.source, 'source')
    sections = source['sections10_rms_db']
    active = [i for i, value in enumerate(sections) if value > -40]
    lo, hi = min(active, key=lambda i: sections[i]), max(active, key=lambda i: sections[i])
    result = {'version': 'c0-checked-v1', 'source_sha256': report['source_sha256'],
              'reference_controls_passed': len(controls), 'source': source,
              'section_indices': [lo, hi], 'source_section_contrast_db': sections[hi]-sections[lo],
              'peak_scope': 'FFT16 periodic finite blocks with 8192-frame overlap and zero file edges; float64 SOXR16 precision33; original samples included',
              'rows': []}
    for c in report['candidates']:
        start = time.perf_counter()
        raw_path = args.run / ('raw-' + c['id'] + '.wav')
        x, rate = sf.read(raw_path, dtype='float32', always_2d=True)
        expected = (source['frames'] * rate + source['rate'] - 1) // source['rate']
        assert len(x) == expected and x.shape[1] == source['channels'] and np.isfinite(x).all()
        fft = ref.peak_reference(x, factor=16)
        soxr = ref.float_soxr(raw_path, rate)
        raw_peak = max(fft, soxr, ref.db(np.max(np.abs(x))))
        raw_lufs = c['metrics']['raw_lufs']
        targets = [-14., -9.] if c['id'] == 'control' else [c['target']]
        raw_checked_s = time.perf_counter() - start
        for target in targets:
            start = time.perf_counter()
            gain = min(target - raw_lufs, -1.0 - raw_peak)
            delivered = (x * np.float32(10**(gain/20))).astype(np.float32)
            path = args.output / (c['id'] + f'-{abs(target):g}.wav')
            sf.write(path, delivered, rate, subtype='FLOAT')
            final_fft = ref.peak_reference(delivered, factor=16)
            final_soxr = ref.float_soxr(path, rate)
            final_peak = max(final_fft, final_soxr, ref.db(np.max(np.abs(delivered))))
            final_check_s = time.perf_counter() - start
            m = shape.metric(path, path.stem, source['transient_times'])
            m['section_contrast_db'] = m['sections10_rms_db'][hi]-m['sections10_rms_db'][lo]
            m['section_contrast_change_db'] = m['section_contrast_db']-result['source_section_contrast_db']
            m['band_change_from_source_db'] = [a-b-(m['spectral_rms']-source['spectral_rms'])
                                             for a,b in zip(m['bands_db'], source['bands_db'])]
            row = {'id': c['id'], 'policy': c.get('policy','control'), 'target': target,
                   'extra_drive_db': c['metrics']['extra_drive_db'], 'landing_gain_db': gain,
                   'raw_fft_dbtp': fft, 'raw_soxr_dbtp': soxr,
                   'final_fft_dbtp': final_fft, 'final_soxr_dbtp': final_soxr,
                   'peak_pass': final_peak <= -1.0 + .002,
                   'target_error_lu': m['lufs']-target,
                   'target_pass': abs(m['lufs']-target) <= .2,
                   'render_s': sum(c['metrics'][k] for k in ['chain_s','src_s','meter_s']),
                   'raw_check_s': raw_checked_s, 'final_check_s': final_check_s, 'metrics': m}
            result['rows'].append(row)
            (args.output / 'comparison.json').write_text(json.dumps(result, indent=2)+'\n')
            print(row['id'], target, 'LUFS', m['lufs'], 'peak', final_peak,
                  'crest', m['crest'], 'section delta', m['section_contrast_change_db'], flush=True)
        del x, delivered


if __name__ == '__main__':
    main()
