"""Independent finite SOXR16/64 measurements, including failed live witnesses."""
import argparse
import json
from pathlib import Path
import re
import subprocess

import numpy as np
import soundfile as sf

from verification_common import sha


def main():
    parser = argparse.ArgumentParser()
    for name in ('report', 'ffmpeg', 'output'):
        parser.add_argument('--' + name, type=Path, required=True)
    args = parser.parse_args()
    assert not args.output.exists(), 'preserve previous evidence'
    report = json.loads(args.report.read_text(encoding='utf-8'))
    assert report['status'] == 'complete'
    args.output.mkdir(parents=True)
    result = dict(source_report_sha256=sha(args.report), ffmpeg_sha256=sha(args.ffmpeg),
                  reference='2048-frame finite zero extension; sample max and SOXR16/64 precision33', rows=[])
    for row in report['rows']:
        witness = row.get('witnesses')
        single = row.get('witness')
        if not witness and not single:
            continue
        if single:
            hashes, paths = {single['sha256']}, [Path(single['path'])]
        else:
            hashes = {v for key, v in witness.items() if key.endswith('sha256')}
            paths = sorted(Path(witness['folder']).glob('*.wav'))
        for path in paths:
            digest = sha(path)
            assert digest in hashes
            x, rate = sf.read(path, dtype='float64', always_2d=True)
            assert np.isfinite(x).all()
            padded = args.output / (path.stem + '-padded.wav')
            sf.write(padded, np.pad(x, ((2048, 2048), (0, 0))), rate, subtype='DOUBLE')
            peaks = dict(samples=(20 * np.log10(np.maximum(np.max(np.abs(x), axis=0), 1e-30))).tolist())
            for factor in (16, 64):
                proc = subprocess.run([str(args.ffmpeg), '-hide_banner', '-nostats', '-i', str(padded),
                    '-af', f'aformat=sample_fmts=dbl,aresample={rate*factor}:resampler=soxr:precision=33:osf=dbl,astats=metadata=0:reset=0',
                    '-c:a', 'pcm_f64le', '-f', 'null', '-'], capture_output=True, text=True, check=True)
                values = [float(v) for v in re.findall(r'Peak level dB: (-inf|[-\d.]+)', proc.stderr)]
                assert len(values) == x.shape[1] + 1
                peaks[f'soxr{factor}'] = values[:-1]
            maximum = max(v for values in peaks.values() for v in values)
            result['rows'].append(dict(path=str(path), sha256=digest, frames=len(x), rate=rate,
                channels=x.shape[1], peaks=peaks, peak_max=maximum,
                ceiling_minus_1_excess_db=max(0., maximum + 1.), fullscale_samples=int((np.abs(x) >= 1).sum())))
            print(path.name, maximum, flush=True)
    assert result['rows'], 'no witnesses found'
    result['status'] = 'complete'
    (args.output / 'comparison.json').write_text(json.dumps(result, indent=2, allow_nan=False), encoding='utf-8')


if __name__ == '__main__':
    main()
