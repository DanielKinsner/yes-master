"""Check preserved waveform character and compare with target-dependent drive."""
import argparse
import json
import math
from pathlib import Path
import time

from drive_metrics import LIMITS, compare, constraints, measure
from verification_common import sha


def read(path):
    return json.loads(path.read_text(encoding='utf-8'))


def identity_differences(left,right,key=''):
    if isinstance(left,dict):
        assert left.keys() == right.keys()
        return [v for name in left for v in identity_differences(left[name],right[name],key+'.'+name)]
    if isinstance(left,list):
        assert len(left) == len(right)
        return [v for i,(a,b) in enumerate(zip(left,right,strict=True)) for v in identity_differences(a,b,key+f'[{i}]')]
    if left is None or right is None:
        assert left is right
        return []
    assert math.isfinite(left) and math.isfinite(right)
    error = abs(left-right)
    limit = 1e-6 if key.endswith('.correlation') else .0001
    assert error <= limit, (key,error,limit)
    return [dict(metric=key,error=error,limit=limit)]


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--job',type=Path,required=True)
    parser.add_argument('--report',type=Path,required=True)
    parser.add_argument('--output',type=Path,required=True)
    args = parser.parse_args()
    assert not args.output.exists()
    job = read(args.job)
    assert job['experiment'] == 'prepared-target-reuse-v1'
    baseline_path = Path(job['baseline_metrics'])
    assert sha(baseline_path) == job['baseline_metrics_sha256']
    baseline = read(baseline_path)
    assert baseline['status'] == 'complete' and baseline['limits'] == LIMITS
    original = {row['id']:row for row in baseline['rows']}
    facts = {}
    for case in job['cases']:
        path = Path(case['source_metrics'])
        assert sha(path) == case['source_metrics_sha256']
        facts[case['id']] = read(path)['source']
    result = dict(status='running',experiment=job['experiment'],job_sha256=sha(args.job),limits=LIMITS,rows=[])
    checked = {}
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
            assert sha(Path(row['path'])) == row['sha256']
            source = facts[row['id']]
            start = time.perf_counter()
            measured = measure(Path(row['path']),source['anchors'])
            metric_s = time.perf_counter()-start
            delta = compare(source,measured)
            prepared = original[row['prepared_id']]
            assert row['prepared_sha256'] == prepared['sha256']
            differences = identity_differences(prepared['source_delta'],delta)
            control,single = [original[row['id']+'-t9-'+policy] for policy in ('control','single')]
            assert row['normalized_scalar_error'] <= 1e-6 and row['cache_arc_identity_pass'] and row['finalizer_calls'] == 1
            failures = constraints(source,delta,control['source_delta'])
            result['rows'].append(dict(id=row['id'],sha256=row['sha256'],frames=measured['frames'],rate=measured['rate'],
                channels=measured['channels'],source_delta=delta,lufs=row['lufs'],target_error_lu=row['lufs']+9,
                character_failures=failures,prepared_character_failures=prepared['character_failures'],
                max_descriptor_identity_error=max(v['error'] for v in differences),identity_checks=differences,
                metric_s=metric_s,prepared_evaluation_s=prepared['evaluation_s'],
                target_dependent_single=dict(lufs=single['lufs'],character_failures=single['character_failures'],
                    evaluation_s=single['evaluation_s']),
                target_control=dict(lufs=control['lufs'],character_failures=control['character_failures'],
                    evaluation_s=control['evaluation_s'])))
            checked[row['id']] = row['sha256']
            args.output.write_text(json.dumps(result,indent=2,allow_nan=False)+'\n',encoding='utf-8')
            print(row['id'],row['lufs'],failures,flush=True)
        if native['status'] == 'complete':
            assert len(result['rows']) == len(native['rows']) == 4
            result.update(status='complete',native_sha256=sha(args.report),script_sha256=sha(Path(__file__)))
            args.output.write_text(json.dumps(result,indent=2,allow_nan=False)+'\n',encoding='utf-8')
            return
        assert native['status'] == 'running'
        time.sleep(5)


if __name__ == '__main__':
    main()
