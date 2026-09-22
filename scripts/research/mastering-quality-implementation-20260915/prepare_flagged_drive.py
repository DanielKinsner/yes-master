"""Freeze six extra candidates only for the two independently verified flags."""
import argparse
import json
from pathlib import Path
import shutil
import subprocess

from drive_metrics import LIMITS
from verification_common import sha


def read(path):
    return json.loads(path.read_text(encoding='utf-8'))


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--root', type=Path, required=True)
    parser.add_argument('--specification', type=Path, required=True)
    parser.add_argument('--binary', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    assert not args.output.exists() and args.binary.is_file()
    paths = dict(job=args.root/'single-first-job-v1.json',
                 report=args.root/'single-first-v1/report.json',
                 metrics=args.root/'metrics-v1.json',
                 independent=args.root/'independent-v1/comparison.json')
    prior = {key: read(path) for key, path in paths.items()}
    assert all(prior[key]['status'] == 'complete' for key in ('report', 'metrics', 'independent'))
    assert prior['report']['job_sha256'] == prior['metrics']['job_sha256'] == sha(paths['job'])
    assert prior['metrics']['native_sha256'] == prior['independent']['native_sha256'] == sha(paths['report'])
    assert prior['metrics']['limits'] == LIMITS and prior['independent']['all_pass']
    assert len(prior['report']['rows']) == len(prior['metrics']['rows']) == len(prior['independent']['rows']) == 12
    flagged = {row['case'] for row in prior['metrics']['cases'] if not row['single_qualifies_existing_limits']}
    assert flagged == {'metal', 'rich'}, flagged
    checks = {str(Path(row['path']).resolve()): row for row in prior['independent']['rows']}
    metrics = {row['id']: row for row in prior['metrics']['rows']}
    cases, estimated_bytes = [], 0
    for name in ('metal', 'rich'):
        case = next(row.copy() for row in prior['job']['cases'] if row['id'] == name)
        original = read(Path(case['report']))
        assert sha(Path(case['report'])) == case['report_sha256']
        assert sha(Path(original['source'])) == case['source_sha256'] == original['source_sha256']
        retained = case['candidates']
        case['candidates'] = []
        for policy in ('control', 'single'):
            row = next(row for row in prior['report']['rows'] if row['id'] == f'{name}-{policy}')
            check = checks[str(Path(row['path']).resolve())]
            assert sha(Path(row['path'])) == row['sha256'] == check['sha256'] == metrics[row['id']]['sha256']
            assert check['peak_pass'] and check['lufs_pass'] and check['fullscale_samples'] == 0
            assert row['frames'] == check['frames'] == metrics[row['id']]['metrics']['frames']
            assert row['rate'] == check['rate'] == 48000 and row['channels'] == check['channels'] == 2
            assert row['lufs'] == check['native_lufs'] and row['peak'] <= row['ceiling'] == check['ceiling'] == -1
            candidate = next(c.copy() for c in retained if
                             next(r for r in original['rows'] if r['id'] == c['id'])['policy'] == policy)
            assert candidate['offset_db'] == 0 and row['source_sha256'] == case['source_sha256']
            candidate['reuse_row_id'] = row['id']
            case['candidates'].append(candidate)
            if policy == 'single':
                for offset in (-3., -6., -12.):
                    case['candidates'].append(dict(id=candidate['id'], offset_db=offset))
                    estimated_bytes += row['frames']*8 + 128
        cases.append(case)
    free, reserve = shutil.disk_usage(args.output.parent).free, 25*1024**3
    assert free >= reserve + estimated_bytes
    job = dict(experiment='flagged-lower-drive-v1', cases=cases,
               specification=str(args.specification.resolve()), specification_sha256=sha(args.specification),
               binary=str(args.binary.resolve()), binary_sha256=sha(args.binary),
               baseline_report=str(paths['report'].resolve()), baseline_report_sha256=sha(paths['report']),
               baseline_metrics=str(paths['metrics'].resolve()), baseline_metrics_sha256=sha(paths['metrics']),
               baseline_independent=str(paths['independent'].resolve()), baseline_independent_sha256=sha(paths['independent']),
               baseline_job=str(paths['job'].resolve()), baseline_job_sha256=sha(paths['job']),
               preparation_script_sha256=sha(Path(__file__)), estimated_new_wav_bytes=estimated_bytes,
               minimum_free_reserve_bytes=reserve, observed_free_bytes=free,
               local_commit=subprocess.check_output(['git','rev-parse','HEAD'],text=True).strip())
    args.output.write_text(json.dumps(job,indent=2,allow_nan=False)+'\n',encoding='utf-8')
    print(f'Frozen 4 verified reused outputs and 6 new WAVs: {estimated_bytes} bytes; free {free}')


if __name__ == '__main__':
    main()
