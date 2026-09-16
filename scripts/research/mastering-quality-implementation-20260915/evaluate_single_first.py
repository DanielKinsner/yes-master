"""Evaluate complete new outputs with frozen source anchors and fresh controls."""
import argparse
import json
from pathlib import Path
import time
from drive_metrics import LIMITS, compare, constraints, measure
from verification_common import sha


def read(path):
    return json.loads(path.read_text(encoding='utf-8'))


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--job', type=Path, required=True)
    parser.add_argument('--report', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    assert not args.output.exists()
    job = read(args.job)
    facts = {}
    for case in job['cases']:
        path = Path(case['source_metrics'])
        assert sha(path) == case['source_metrics_sha256']
        facts[case['id']] = read(path)['source']
    result = dict(status='running',job_sha256=sha(args.job),limits=LIMITS,rows=[],cases=[],
                  scope='Five known development sources plus two exact Coat reproduction anchors; no holdout or sonic adoption')
    checked, controls = {}, {}
    while True:
        try:
            native = read(args.report)
        except (FileNotFoundError,json.JSONDecodeError):
            time.sleep(5)
            continue
        assert native['job_sha256'] == sha(args.job)
        for row in native['rows']:
            if row['id'] in checked:
                assert checked[row['id']] == row['sha256']
                continue
            path = Path(row['path'])
            assert sha(path) == row['sha256']
            source = facts[row['case']]
            started = time.perf_counter()
            metrics = measure(path,source['anchors'])
            metric_s = time.perf_counter()-started
            assert (metrics['frames'],metrics['rate'],metrics['channels']) == (row['frames'],48000,2)
            delta = compare(source,metrics)
            if row['policy'] == 'control':
                controls[row['case']] = delta
            failures = constraints(source,delta,controls[row['case']])
            target = row['requested_settings']['advanced']['lufs_offset_db']
            assert row['peak'] <= row['ceiling'] == -1
            evaluated = dict(case=row['case'],policy=row['policy'],id=row['id'],sha256=row['sha256'],
                metrics=metrics,source_delta=delta,character_failures=failures,
                target=target,lufs=row['lufs'],target_error_lu=row['lufs']-target,
                peak=row['peak'],metric_s=metric_s,limiter_max_db=row['render']['limiter_max_db'],
                evaluation_s=row['render']['chain_s']+row['src_s']+row['finalize_s'],
                chain_s=row['render']['chain_s'],src_s=row['src_s'],finalize_s=row['finalize_s'])
            result['rows'].append(evaluated)
            checked[row['id']] = row['sha256']
            args.output.write_text(json.dumps(result,indent=2,allow_nan=False)+'\n',encoding='utf-8')
            print(row['id'],evaluated['lufs'],failures,flush=True)
        if native['status'] == 'complete':
            assert len(result['rows']) == len(native['rows']) == 12
            for name in ('funk','metal','aphelion','rich','baby'):
                rows = {r['policy']:r for r in result['rows'] if r['case']==name}
                single, control = rows['single'], rows['control']
                result['cases'].append(dict(case=name,single_qualifies_existing_limits=
                    not single['character_failures'] and abs(single['target_error_lu']) <= LIMITS['target_error_lu'],
                    single_character_failures=single['character_failures'],target_error_lu=single['target_error_lu'],
                    single_evaluation_s=single['evaluation_s'],control_evaluation_s=control['evaluation_s'],
                    initial_control_plus_candidate_s=single['evaluation_s']+control['evaluation_s']))
            result.update(status='complete',native_sha256=sha(args.report))
            args.output.write_text(json.dumps(result,indent=2,allow_nan=False)+'\n',encoding='utf-8')
            return
        assert native['status'] == 'running'
        time.sleep(5)


if __name__ == '__main__':
    main()
