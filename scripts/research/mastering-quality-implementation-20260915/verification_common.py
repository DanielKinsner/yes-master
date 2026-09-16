"""Verification helpers shared without importing experimental drive selectors.

The hash and peak algorithms are unchanged from the frozen research snapshot.
The FFT reference retains its documented finite-block limitations.
"""
import hashlib
import importlib.util
from pathlib import Path
import re
import subprocess

import numpy as np


def sha(path):
    with path.open('rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest()


def channel_peaks(x, path, rate):
    root = Path(__file__).resolve().parents[3]
    spec = importlib.util.spec_from_file_location(
        'checked_reference', root / 'scripts/research/final-review-20260915/verify.py')
    ref = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(ref)
    fft = [ref.peak_reference(x[:, i:i+1], factor=16) for i in range(x.shape[1])]
    filt = f'aformat=sample_fmts=dbl,aresample={rate*16}:resampler=soxr:precision=33:osf=dbl,astats=metadata=0:reset=0'
    run = subprocess.run(['ffmpeg', '-hide_banner', '-nostats', '-i', str(path), '-af', filt,
                          '-c:a', 'pcm_f64le', '-f', 'null', '-'], capture_output=True, text=True, check=True)
    values = [float(v) for v in re.findall(r'Peak level dB: (-inf|[-\d.]+)', run.stderr)]
    assert len(values) == x.shape[1] + 1
    soxr = [max(-300., v) for v in values[:-1]]
    sample = [ref.db(np.max(np.abs(x[:, i]))) for i in range(x.shape[1])]
    return {'fft_dbtp': fft, 'soxr_dbtp': soxr, 'sample_dbfs': sample}
