"""Join immutable-buffer reuse, character identity and independent whole-file proof."""
import argparse
import json
from pathlib import Path

from drive_metrics import LIMITS
from verification_common import sha


def read(path):
    return json.loads(path.read_text(encoding='utf-8'))


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--root',type=Path,required=True)
    parser.add_argument('--output',type=Path,required=True)
    args = parser.parse_args()
    assert not args.output.exists()
    paths = dict(job=args.root/'target-reuse-job-v1.json',native=args.root/'target-reuse-v1/report.json',
        metrics=args.root/'target-reuse-metrics-v1.json',independent=args.root/'target-reuse-independent-v1/comparison.json',
        process=args.root/'target-reuse-v1.process.json')
    data = {key:read(path) for key,path in paths.items()}
    job,native,metrics,checks,process = [data[key] for key in paths]
    assert all(data[key]['status'] == 'complete' for key in ('native','metrics','independent','process'))
    assert process['exit_code'] == 0 and checks['all_pass'] and metrics['limits'] == LIMITS
    assert native['job_sha256'] == metrics['job_sha256'] == process['job_sha256'] == sha(paths['job'])
    assert checks['native_sha256'] == metrics['native_sha256'] == sha(paths['native'])
    assert process['binary_sha256'] == job['binary_sha256'] == sha(Path(job['binary']))
    assert sha(Path(job['specification'])) == job['specification_sha256']
    assert sha(Path(job['baseline_summary'])) == job['baseline_summary_sha256']
    assert sha(Path(job['baseline_metrics'])) == job['baseline_metrics_sha256']
    assert len(native['rows']) == len(metrics['rows']) == len(checks['rows']) == 4
    measured = {row['id']:row for row in metrics['rows']}
    checked = {str(Path(row['path']).resolve()):row for row in checks['rows']}
    result = dict(status='complete',scope='Offline prepared final-rate PCM reuse; no raw-buffer equivalence or app latency claim',
        inputs={key:dict(path=str(path),sha256=sha(path)) for key,path in paths.items()},
        specification_sha256=job['specification_sha256'],baseline_summary_sha256=job['baseline_summary_sha256'],
        script_sha256=sha(Path(__file__)),limits=LIMITS,process=process,rows=[])
    for row in native['rows']:
        metric,check = measured[row['id']],checked[str(Path(row['path']).resolve())]
        case = next(case for case in job['cases'] if case['id'] == row['id'])
        assert sha(Path(case['prepared_path'])) == case['prepared_sha256'] == row['prepared_sha256']
        assert row['sha256'] == metric['sha256'] == check['sha256'] == sha(Path(row['path']))
        assert row['frames'] == metric['frames'] == check['frames'] == case['frames']
        assert row['rate'] == metric['rate'] == check['rate'] == 48000
        assert row['channels'] == metric['channels'] == check['channels'] == 2
        assert row['lufs'] == metric['lufs'] == check['native_lufs']
        assert row['peak'] <= row['ceiling'] == check['ceiling'] == -1
        assert check['peak_pass'] and check['lufs_pass'] and check['fullscale_samples'] == 0
        assert row['normalized_scalar_error'] <= 1e-6 and row['finalizer_calls'] == 1 and row['cache_arc_identity_pass']
        assert all(entry['error'] <= entry['limit'] for entry in metric['identity_checks'])
        combined = dict(row)
        combined.pop('path')
        combined.update(character_failures=metric['character_failures'],source_delta=metric['source_delta'],
            prepared_character_failures=metric['prepared_character_failures'],
            prepared_evaluation_s=metric['prepared_evaluation_s'],
            max_descriptor_identity_error=metric['max_descriptor_identity_error'],
            target_dependent_single=metric['target_dependent_single'],target_control=metric['target_control'],
            independent_peak=check['independent_peak'],metric_s=metric['metric_s'],reference_s=check['reference_s'])
        result['rows'].append(combined)
    result['phase_work_s'] = {key:sum(row[key] for row in result['rows'])
                              for key in ('decode_s','new_target_s','metric_s','reference_s')}
    args.output.parent.mkdir(parents=True,exist_ok=True)
    args.output.write_text(json.dumps(result,indent=2,allow_nan=False)+'\n',encoding='utf-8')
    print(json.dumps([dict(id=row['id'],lufs=row['lufs'],new_target_s=row['new_target_s'],
        target_dependent_single=row['target_dependent_single'],failures=row['character_failures'])
        for row in result['rows']],indent=2))


if __name__ == '__main__':
    main()
