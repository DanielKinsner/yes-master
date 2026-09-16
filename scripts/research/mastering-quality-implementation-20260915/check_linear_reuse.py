"""Independent finite-signal checks of whole-file residual-based peak reuse.

The WAVs contain the exact PCM produced by the deterministic quantizer, stored
as float only to keep the independent references from adding their own dither.
"""
import argparse
import json
import math
from pathlib import Path
import numpy as np
import soundfile as sf
from check_native_peak_bounds import soxr
from qualify_fft import finite_sinc_peak
from qualify_fir import ref


def main():
    p = argparse.ArgumentParser()
    p.add_argument('--run', type=Path, required=True)
    p.add_argument('--output', type=Path, required=True)
    a = p.parse_args()
    assert not a.output.exists()
    a.output.mkdir(parents=True)
    source = a.run/'comparison.json'
    data = json.loads(source.read_text())
    report = {'version': 'linear-reuse-independent-1', 'input_sha256': ref.sha(source), 'rows': []}
    for index, row in enumerate(data['rows']):
        assert row['intervals_intersect'] and row['path']
        path = Path(row['path'])
        x, rate = sf.read(path, always_2d=True)
        assert rate == row['rate'] and x.shape == (row['frames'], row['channels'])
        padded = a.output/'reference-input.wav'
        sf.write(padded, np.pad(x, ((2048,2048),(0,0))), rate, subtype='DOUBLE')
        references = {f'soxr{factor}': soxr(padded, rate, factor, row['channels']) for factor in [16,64]}
        if len(x) <= 1024:
            references['direct_sinc32'] = [finite_sinc_peak(x[:,c:c+1]) for c in range(x.shape[1])]
        upper = [ref.db(v) for v in row['reused_upper']]
        errors = {name: [u-v for u,v in zip(upper, values, strict=True)] for name,values in references.items()}
        passed = all(error >= -1e-5 for values in errors.values() for error in values)
        report['rows'].append({'id':row['id'], 'gain_db':row['gain_db'], 'bits':row['bits'],
                               'output_sha256':ref.sha(path), 'references':references,
                               'upper_minus_reference_db':errors, 'pass':passed})
        if not passed:
            print('FAIL',row['id'],row['gain_db'],row['bits'],errors,flush=True)
        if index % 25 == 0:
            (a.output/'comparison.json').write_text(json.dumps(report,indent=2)+'\n')
            print('checked',index+1,'of',len(data['rows']),flush=True)
    (a.output/'comparison.json').write_text(json.dumps(report,indent=2)+'\n')
    print('passed',sum(row['pass'] for row in report['rows']),'of',len(report['rows']),flush=True)
    assert all(row['pass'] for row in report['rows'])


if __name__ == '__main__':
    main()
