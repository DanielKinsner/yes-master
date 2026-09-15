"""Independent whole-file checks of selected C1 outcomes and corrected controls."""
import argparse
import hashlib
import json
from pathlib import Path
import re
import subprocess
import time
import numpy as np
import soundfile as sf


def sha(path):
    with path.open('rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest()


def main():
    parser = argparse.ArgumentParser()
    for name in ('metrics', 'ffmpeg', 'output'):
        parser.add_argument('--'+name, type=Path, required=True)
    a = parser.parse_args()
    assert not a.output.exists()
    a.output.mkdir(parents=True)
    metrics = json.loads(a.metrics.read_text(encoding='utf-8'))
    assert metrics['status'] == 'complete'
    ids = {r['id'] for r in metrics['rows'] if r['policy'] == 'control'}
    for selection in metrics['selections']:
        for mode in ('preserve', 'target_first'):
            if selection[mode]['selected'] is not None:
                ids.add(selection[mode]['selected'])
    report = dict(metrics_sha256=sha(a.metrics), ffmpeg_sha256=sha(a.ffmpeg), rows=[],
                  reference='finite zero extension, 2048 input frames at both edges; SOXR16/64 precision33 plus sample peak; 0.002 dB independent numerical tolerance')
    for row in metrics['rows']:
        if row['id'] not in ids:
            continue
        path = Path(row['native']['path'])
        assert sha(path) == row['native']['sha256']
        x, rate = sf.read(path, always_2d=True, dtype='float64')
        assert np.isfinite(x).all() and len(x) == row['metrics']['frames']
        padded = a.output / (row['id'] + '-padded.wav')
        sf.write(padded, np.pad(x, ((2048,2048),(0,0))), rate, subtype='DOUBLE')
        peaks = dict(samples=[float(20*np.log10(max(v,1e-30))) for v in np.max(np.abs(x),axis=0)])
        start = time.perf_counter()
        for factor in (16,64):
            process = subprocess.run([str(a.ffmpeg),'-hide_banner','-nostats','-i',str(padded),'-af',
                f'aformat=sample_fmts=dbl,aresample={rate*factor}:resampler=soxr:precision=33:osf=dbl,astats=metadata=0:reset=0',
                '-c:a','pcm_f64le','-f','null','-'], capture_output=True,text=True,check=True)
            values = [float(v) for v in re.findall(r'Peak level dB: (-inf|[-\d.]+)',process.stderr)]
            assert len(values) == x.shape[1]+1
            peaks[f'soxr{factor}'] = values[:-1]
        process = subprocess.run([str(a.ffmpeg),'-hide_banner','-nostats','-i',str(path),'-af',
                                  'ebur128=peak=true','-f','null','-'],capture_output=True,text=True,check=True)
        summary = process.stderr.rsplit('Summary:',1)[-1]
        integrated = float(re.search(r'I:\s*([-\d.]+) LUFS',summary)[1])
        peak_pass = max(v for values in peaks.values() for v in values) <= row['native']['ceiling']+.002
        native_lufs = row['native']['delivery']['lufs']
        lufs_pass = native_lufs is not None and abs(integrated-native_lufs) <= .11
        record = dict(id=row['id'],sha256=sha(path),reference=peaks,
                      independent_lufs_rounded_0_1=integrated,native_lufs=native_lufs,
                      fullscale_samples=int((np.abs(x)>=1).sum()),
                      peak_pass=peak_pass,lufs_pass=lufs_pass,reference_s=time.perf_counter()-start)
        report['rows'].append(record)
        (a.output/'comparison.json').write_text(json.dumps(report,indent=2),encoding='utf-8')
        print('Checked',row['id'],'peak',peak_pass,'LUFS',lufs_pass,flush=True)
        assert peak_pass and lufs_pass and record['fullscale_samples']==0
    report['status'] = 'complete'
    (a.output/'comparison.json').write_text(json.dumps(report,indent=2),encoding='utf-8')


if __name__ == '__main__':
    main()
