"""Whole-file reference extension with explicit below-gate availability evidence."""
import argparse
import json
import math
from pathlib import Path
import re
import subprocess
import time

import numpy as np
import soundfile as sf
from verification_common import sha


def loudness_summary(stderr):
    summary = stderr.rsplit('Summary:', 1)[-1]
    integrated = summary.split('Integrated loudness:', 1)[1].split('Loudness range:', 1)[0]
    return (float(re.search(r'I:\s*([-\d.]+) LUFS', integrated)[1]),
            float(re.search(r'Threshold:\s*([-\d.]+) LUFS', integrated)[1]))


def loudness_agreement(native, independent, threshold, peak):
    no_blocks = independent == -70. and threshold == 0. and peak < -70.
    if native is None:
        return dict(native_lufs=None, independent_lufs=None if no_blocks else independent,
                    integrated_loudness_available=not no_blocks,
                    lufs_pass=no_blocks, loudness_comparison='both_unavailable' if no_blocks else 'availability_mismatch')
    return dict(native_lufs=native, independent_lufs=None if no_blocks else independent,
                integrated_loudness_available=not no_blocks,
                lufs_pass=not no_blocks and math.isfinite(native) and abs(independent-native) <= .11,
                loudness_comparison='finite_agreement' if not no_blocks else 'availability_mismatch')


def main():
    parser = argparse.ArgumentParser()
    for name in ('report','ffmpeg','output'):
        parser.add_argument('--'+name, type=Path, required=True)
    args = parser.parse_args()
    native = json.loads(args.report.read_text())
    assert native['status'] == 'complete' and len(native['rows']) == 32
    assert not args.output.exists()
    args.output.mkdir(parents=True)
    result = dict(status='running', native_sha256=sha(args.report), ffmpeg_sha256=sha(args.ffmpeg),
                  reference='2048 zero frames, SOXR16/64 precision33 double; explicit integrated availability',
                  peak_tolerance_db=.002, lufs_tolerance_lu=.11, rows=[])
    for index,row in enumerate(native['rows']):
        path = Path(row['path'])
        assert sha(path) == row['sha256']
        samples,rate = sf.read(path, always_2d=True, dtype='float64')
        assert np.isfinite(samples).all()
        assert (len(samples),rate,samples.shape[1]) == (row['frames'],row['rate'],row['channels'])
        started = time.perf_counter()
        peaks = dict(samples=[float(20*np.log10(max(p,1e-30))) for p in np.max(np.abs(samples),axis=0)])
        for factor in (16,64):
            process = subprocess.run([str(args.ffmpeg),'-hide_banner','-nostats','-i',str(path),'-af',
                f'aformat=sample_fmts=dbl,adelay=2048S:all=1,apad=pad_len=2048,aresample={rate*factor}:resampler=soxr:precision=33:osf=dbl,astats=metadata=0:reset=0',
                '-c:a','pcm_f64le','-f','null','-'],capture_output=True,text=True,check=True)
            (args.output/f'{index:02d}-soxr{factor}.log').write_text(process.stderr,encoding='utf-8')
            values = [float(v) for v in re.findall(r'Peak level dB: (-inf|[-\d.]+)',process.stderr)]
            assert len(values) == samples.shape[1]+1
            peaks[f'soxr{factor}'] = values[:-1]
        process = subprocess.run([str(args.ffmpeg),'-hide_banner','-nostats','-i',str(path),'-af',
            'ebur128=peak=true','-f','null','-'],capture_output=True,text=True,check=True)
        (args.output/f'{index:02d}-lufs.log').write_text(process.stderr,encoding='utf-8')
        lufs,threshold = loudness_summary(process.stderr)
        peak = max(v for values in peaks.values() for v in values)
        agreement = loudness_agreement(row['lufs'],lufs,threshold,peak)
        if row['lufs'] is None:
            assert row['loudness_unavailable_reason'] == 'below_integrated_gate'
        result['rows'].append(dict(path=str(path),sha256=sha(path),frames=len(samples),rate=rate,
            channels=samples.shape[1],peaks=peaks,ceiling=row['ceiling'],independent_peak=peak,
            peak_pass=peak<=row['ceiling']+.002,fullscale_samples=int((np.abs(samples)>=1).sum()),
            ffmpeg_integrated_display=lufs,ffmpeg_integrated_threshold=threshold,
            **agreement,reference_s=time.perf_counter()-started))
        (args.output/'comparison.json').write_text(json.dumps(result,indent=2,allow_nan=False)+'\n')
        print(row['id'],result['rows'][-1]['peak_pass'],agreement['lufs_pass'],agreement['loudness_comparison'],flush=True)
    result['status']='complete'
    result['all_pass']=all(r['peak_pass'] and r['lufs_pass'] and r['fullscale_samples']==0 for r in result['rows'])
    (args.output/'comparison.json').write_text(json.dumps(result,indent=2,allow_nan=False)+'\n')


if __name__ == '__main__':
    main()
