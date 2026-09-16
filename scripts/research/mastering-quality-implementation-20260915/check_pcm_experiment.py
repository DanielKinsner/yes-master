"""Independent complete-file PCM checks; diagnostic misses remain in the report."""
import argparse
import json
from pathlib import Path
import re
import subprocess
import time

import numpy as np
import soundfile as sf
from evaluate_drive import sha


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--report', type=Path, required=True)
    parser.add_argument('--kind', choices=('device', 'wide'), required=True)
    parser.add_argument('--ffmpeg', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    native = json.loads(args.report.read_text())
    assert native['status'] == 'complete'
    assert not args.output.exists()
    args.output.mkdir(parents=True)
    rows = native['rows'] if args.kind == 'device' else [r for r in native['gain_copies'] if r['wide'] and r['source_gain_db'] in (0, -80)]
    report = dict(status='running', native_sha256=sha(args.report), ffmpeg_sha256=sha(args.ffmpeg),
                  reference='finite double PCM, 2048 leading/trailing zero frames, SOXR16/64 precision33 plus sample peak',
                  peak_tolerance_db=.002, lufs_tolerance_lu=.11, rows=[])
    for row in rows:
        path = Path(row['path'])
        assert sha(path) == row['sha256']
        samples, rate = sf.read(path, always_2d=True, dtype='float64')
        assert np.isfinite(samples).all()
        started = time.perf_counter()
        peaks = dict(samples=[float(20*np.log10(max(p, 1e-30))) for p in np.max(np.abs(samples), axis=0)])
        for factor in (16, 64):
            process = subprocess.run([str(args.ffmpeg), '-hide_banner', '-nostats', '-i', str(path), '-af',
                f'aformat=sample_fmts=dbl,adelay=2048S:all=1,apad=pad_len=2048,aresample={rate*factor}:resampler=soxr:precision=33:osf=dbl,astats=metadata=0:reset=0',
                '-c:a', 'pcm_f64le', '-f', 'null', '-'], capture_output=True, text=True, check=True)
            values = [float(v) for v in re.findall(r'Peak level dB: (-inf|[-\d.]+)', process.stderr)]
            assert len(values) == samples.shape[1] + 1
            peaks[f'soxr{factor}'] = values[:-1]
        process = subprocess.run([str(args.ffmpeg), '-hide_banner', '-nostats', '-i', str(path), '-af',
            'ebur128=peak=true', '-f', 'null', '-'], capture_output=True, text=True, check=True)
        summary = process.stderr.rsplit('Summary:', 1)[-1]
        lufs = float(re.search(r'I:\s*([-\d.]+) LUFS', summary)[1])
        native_lufs = row['lufs'] if args.kind == 'device' else row['delivered_lufs']
        ceiling = row['ceiling'] if args.kind == 'device' else native['requested_settings']['advanced']['ceiling_dbtp']
        peak = max(v for values in peaks.values() for v in values)
        report['rows'].append(dict(path=str(path), sha256=sha(path), frames=len(samples), rate=rate,
            channels=samples.shape[1], peaks=peaks, ceiling=ceiling, independent_peak=peak,
            peak_pass=peak <= ceiling+.002, fullscale_samples=int((np.abs(samples) >= 1).sum()),
            native_lufs=native_lufs, independent_lufs=lufs, lufs_pass=abs(lufs-native_lufs) <= .11,
            reference_s=time.perf_counter()-started))
        (args.output/'comparison.json').write_text(json.dumps(report, indent=2))
        print(path.name, 'peak', peak, 'pass', peak <= ceiling+.002, flush=True)
    report['status'] = 'complete'
    report['all_pass'] = all(r['peak_pass'] and r['lufs_pass'] and r['fullscale_samples'] == 0 for r in report['rows'])
    (args.output/'comparison.json').write_text(json.dumps(report, indent=2))


if __name__ == '__main__':
    main()
