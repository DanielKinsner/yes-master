"""Check each completed production WAV while the remaining matrix renders.

Every child manifest describes one fully written, hashed WAV. The final aggregate
requires the producer's complete status and exact declared row count. No partial
file or running producer is promoted to a complete matrix.
"""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess
import sys
import time


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--report', type=Path, required=True)
    parser.add_argument('--ffmpeg', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--expected-rows', type=int, required=True)
    args = parser.parse_args()
    assert args.expected_rows > 0 and not args.output.exists()
    args.output.mkdir(parents=True)
    checked = {}
    result = dict(status='running', rows=[], expected_rows=args.expected_rows,
                  scope='Complete-file independent checks started as each hashed production output finishes; final status requires the complete producer matrix')
    while True:
        try:
            native = json.loads(args.report.read_text(encoding='utf-8'))
        except (FileNotFoundError, json.JSONDecodeError):
            time.sleep(5)
            continue
        assert len(native['rows']) <= args.expected_rows
        for index, row in enumerate(native['rows']):
            key = row['path']
            if key in checked:
                assert checked[key] == row['sha256']
                continue
            assert sha(Path(key)) == row['sha256']
            manifest = args.output / f'case-{index:02d}.json'
            output = args.output / f'case-{index:02d}'
            assert not manifest.exists() and not output.exists()
            manifest.write_text(json.dumps(dict(status='complete', rows=[row],
                producer_snapshot_sha256=sha(args.report)), indent=2), encoding='utf-8')
            with (args.output/f'case-{index:02d}.log').open('x', encoding='utf-8') as log:
                subprocess.run([sys.executable, str(Path(__file__).with_name('check_pcm_experiment.py')),
                    '--report', str(manifest), '--kind', 'device', '--ffmpeg', str(args.ffmpeg),
                    '--output', str(output)], stdout=log, stderr=subprocess.STDOUT, check=True)
            comparison = json.loads((output/'comparison.json').read_text(encoding='utf-8'))
            assert comparison['status'] == 'complete' and len(comparison['rows']) == 1
            verified = comparison['rows'][0]
            assert verified['sha256'] == row['sha256']
            result['rows'].append(verified)
            checked[key] = row['sha256']
            (args.output/'comparison.json').write_text(json.dumps(result, indent=2), encoding='utf-8')
            print(Path(key).name, verified['peak_pass'], verified['lufs_pass'], flush=True)
        if native['status'] == 'complete':
            assert len(checked) == len(native['rows']) == args.expected_rows
            result['status'] = 'complete'
            result['native_sha256'] = sha(args.report)
            result['all_pass'] = all(row['peak_pass'] and row['lufs_pass'] and row['fullscale_samples'] == 0 for row in result['rows'])
            (args.output/'comparison.json').write_text(json.dumps(result, indent=2), encoding='utf-8')
            return
        assert native['status'] == 'running'
        time.sleep(5)


if __name__ == '__main__':
    main()
