"""Prove streaming finite padding matches retained double-WAV reference samples."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess

import numpy as np
import soundfile as sf


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--ffmpeg', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--source', type=Path, action='append', default=[])
    args = parser.parse_args()
    assert not args.output.exists(), 'preserve evidence'
    args.output.mkdir(parents=True)
    paths = list(args.source)
    for channels, frames in [(1, 1), (2, 3), (1, 2053), (2, 4097)]:
        path = args.output / f'synthetic-{channels}-{frames}.wav'
        sf.write(path, np.random.default_rng(frames).uniform(-1.1, 1.1, (frames, channels)),
                 44100, subtype='FLOAT')
        paths.append(path)

    def render(path, filters):
        return subprocess.run([str(args.ffmpeg), '-v', 'error', '-i', str(path), '-af', filters,
                               '-c:a', 'pcm_f64le', '-f', 'f64le', '-'],
                              capture_output=True, check=True).stdout

    rows = []
    for index, path in enumerate(paths):
        x, rate = sf.read(path, always_2d=True, dtype='float64')
        assert len(x) <= 200000, 'use a short witness for bytewise 64x comparison'
        expected = np.pad(x, ((2048, 2048), (0, 0)))
        padded = args.output / f'{index}-padded.wav'
        sf.write(padded, expected, rate, subtype='DOUBLE')
        padding_filter = 'aformat=sample_fmts=dbl,adelay=2048S:all=1,apad=pad_len=2048'
        assert render(path, padding_filter) == expected.astype('<f8').tobytes()
        for factor in (16, 64):
            resample = f'aresample={rate*factor}:resampler=soxr:precision=33:osf=dbl'
            disk = render(padded, 'aformat=sample_fmts=dbl,' + resample)
            streamed = render(path, padding_filter + ',' + resample)
            assert disk == streamed, (path, factor)
            rows.append(dict(path=str(path), factor=factor, output_bytes=len(disk),
                             sha256=hashlib.sha256(disk).hexdigest(), bit_exact=True))
    report = dict(status='complete', rows=rows,
                  ffmpeg_sha256=hashlib.sha256(args.ffmpeg.read_bytes()).hexdigest())
    (args.output / 'report.json').write_text(json.dumps(report, indent=2), encoding='utf-8')
    print('PASS', len(rows), 'bit-exact SOXR outputs; finite padding also exact')


if __name__ == '__main__':
    main()
