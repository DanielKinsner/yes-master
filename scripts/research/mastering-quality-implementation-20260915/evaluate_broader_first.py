"""Evaluate matched preset/target controls without changing the frozen C1 limits."""
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
    parser.add_argument('--job',type=Path,required=True)
    parser.add_argument('--report',type=Path,required=True)
    parser.add_argument('--output',type=Path,required=True)
    args = parser.parse_args()
    assert not args.output.exists()
    job = read(args.job)
    assert job['experiment'] == 'broader-first-v1'
    sources = {}
    for case in job['cases']:
        path = Path(case['source_metrics'])
        assert sha(path) == case['source_metrics_sha256']
        sources[case['id']] = read(path)['source']
    result = dict(status='running',experiment=job['experiment'],job_sha256=sha(args.job),limits=LIMITS,rows=[])
    checked, controls = {}, {}
    while True:
        try:
            native = read(args.report)
        except (FileNotFoundError,json.JSONDecodeError):
            time.sleep(5)
            continue
        assert native['job_sha256'] == sha(args.job) and native['experiment'] == job['experiment']
        for row in native['rows']:
            if row['id'] in checked:
                assert checked[row['id']] == row['sha256']
                continue
            path = Path(row['path'])
            assert sha(path) == row['sha256']
            source = sources[row['case']]
            start = time.perf_counter()
            measured = measure(path,source['anchors'])
            metric_s = time.perf_counter()-start
            assert (measured['frames'],measured['rate'],measured['channels']) == (row['frames'],48000,2)
            delta = compare(source,measured)
            group = row['comparison_group']
            if row['policy'] == 'control':
                assert group not in controls
                controls[group] = delta
            target = row['requested_settings']['advanced']['lufs_offset_db']
            assert target in (-14.,-9.) and row['offset_db'] == 0
            failures = constraints(source,delta,controls[group])
            result['rows'].append(dict(id=row['id'],case=row['case'],comparison_group=group,
                policy=row['policy'],preset_id=row['preset_id'],sha256=row['sha256'],metrics=measured,
                source_delta=delta,character_failures=failures,lufs=row['lufs'],peak=row['peak'],
                target=target,target_error_lu=row['lufs']-target,metric_s=metric_s,
                evaluation_s=row['render']['chain_s']+row['src_s']+row['finalize_s']))
            checked[row['id']] = row['sha256']
            args.output.write_text(json.dumps(result,indent=2,allow_nan=False)+'\n',encoding='utf-8')
            print(row['id'],row['lufs'],failures,flush=True)
        if native['status'] == 'complete':
            assert len(result['rows']) == len(native['rows']) == 16 and len(controls) == 8
            result.update(status='complete',native_sha256=sha(args.report),script_sha256=sha(Path(__file__)))
            args.output.write_text(json.dumps(result,indent=2,allow_nan=False)+'\n',encoding='utf-8')
            return
        assert native['status'] == 'running'
        time.sleep(5)


if __name__ == '__main__':
    main()
