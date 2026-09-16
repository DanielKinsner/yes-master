"""Require all broader whole-output proofs before resolving experimental choices."""
import argparse
from dataclasses import asdict
import json
from pathlib import Path

from drive_metrics import LIMITS, compare, constraints
from preserving_selector import Candidate, select, VERSION
from verification_common import sha


def read(path):
    return json.loads(path.read_text(encoding='utf-8'))


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--root',type=Path,required=True)
    parser.add_argument('--output',type=Path,required=True)
    args = parser.parse_args()
    assert not args.output.exists()
    paths = dict(job=args.root/'broader-job-v1.json',native=args.root/'broader-v1/report.json',
                 metrics=args.root/'broader-metrics-v1.json',independent=args.root/'broader-independent-v1/comparison.json',
                 process=args.root/'broader-v1.process.json')
    data = {key:read(path) for key,path in paths.items()}
    job,native,metrics,independent,process = [data[key] for key in paths]
    assert all(data[key]['status'] == 'complete' for key in ('native','metrics','independent','process'))
    assert job['experiment'] == native['experiment'] == metrics['experiment'] == 'broader-first-v1'
    assert independent['all_pass'] and process['exit_code'] == 0 and metrics['limits'] == LIMITS
    assert native['job_sha256'] == metrics['job_sha256'] == process['job_sha256'] == sha(paths['job'])
    assert independent['native_sha256'] == metrics['native_sha256'] == sha(paths['native'])
    assert process['binary_sha256'] == job['binary_sha256'] == sha(Path(job['binary']))
    assert sha(Path(job['specification'])) == job['specification_sha256']
    assert len(native['rows']) == len(metrics['rows']) == len(independent['rows']) == 16
    source = {}
    for case in job['cases']:
        path = Path(case['source_metrics'])
        assert sha(path) == case['source_metrics_sha256']
        source[case['id']] = read(path)['source']
        report = Path(case['report'])
        assert sha(report) == case['report_sha256']
        assert sha(Path(read(report)['source'])) == case['source_sha256']
    measured = {row['id']:row for row in metrics['rows']}
    checked = {str(Path(row['path']).resolve()):row for row in independent['rows']}
    controls = {row['comparison_group']:row['source_delta'] for row in metrics['rows'] if row['policy'] == 'control'}
    assert len(measured) == len(checked) == 16 and len(controls) == 8
    output,groups = [],{}
    for row in native['rows']:
        metric,check = measured[row['id']],checked[str(Path(row['path']).resolve())]
        assert row['sha256'] == metric['sha256'] == check['sha256'] == sha(Path(row['path']))
        assert row['frames'] == metric['metrics']['frames'] == check['frames']
        assert row['rate'] == metric['metrics']['rate'] == check['rate'] == 48000
        assert row['channels'] == metric['metrics']['channels'] == check['channels'] == 2
        assert row['peak'] <= row['ceiling'] == check['ceiling'] == -1
        assert check['peak_pass'] and check['lufs_pass'] and check['fullscale_samples'] == 0
        assert row['lufs'] == metric['lufs'] == check['native_lufs']
        group = row['comparison_group']
        assert row['policy'] == metric['policy'] and row['preset_id'] == metric['preset_id']
        assert group == metric['comparison_group'] and compare(source[row['case']],metric['metrics']) == metric['source_delta']
        assert constraints(source[row['case']],metric['source_delta'],controls[group]) == metric['character_failures']
        context = sha(paths['job'])+':'+group
        candidate = Candidate(context,row['id'],row['sha256'],row['policy'],row['offset_db'],row['lufs'],
                              metric['target'],row['peak'],row['ceiling'],tuple(metric['character_failures']))
        groups.setdefault(group,[]).append(candidate)
        output.append(dict(id=row['id'],case=row['case'],group=group,preset_id=row['preset_id'],policy=row['policy'],
            sha256=row['sha256'],source_sha256=row['source_sha256'],target=metric['target'],lufs=row['lufs'],
            peak=row['peak'],independent_peak=check['independent_peak'],character_failures=metric['character_failures'],
            source_delta=metric['source_delta'],limiter_max_db=row['render']['limiter_max_db'],
            evaluation_s=metric['evaluation_s'],chain_s=row['render']['chain_s'],src_s=row['src_s'],finalize_s=row['finalize_s'],
            metric_s=metric['metric_s'],reference_s=check['reference_s']))
    selections = [dict(group=name,**asdict(select(rows,rows[0].context))) for name,rows in groups.items()]
    result = dict(status='complete',scope='Two known sources, two presets and two targets; no full C2 or sonic adoption',
        algorithm=VERSION,selector_sha256=sha(Path(__file__).with_name('preserving_selector.py')),
        script_sha256=sha(Path(__file__)),specification_sha256=job['specification_sha256'],
        inputs={key:dict(path=str(path),sha256=sha(path)) for key,path in paths.items()},
        limits=LIMITS,rows=output,selections=selections,process=process,source_preparation=native['source_preparation'],
        phase_work_s={key:sum(row[key] for row in output) for key in ('evaluation_s','metric_s','reference_s')})
    args.output.parent.mkdir(parents=True,exist_ok=True)
    args.output.write_text(json.dumps(result,indent=2,allow_nan=False)+'\n',encoding='utf-8')
    print(json.dumps([dict(group=row['group'],selected=row['selected']['id'] if row['selected'] else None,
        reason=row['reason'],character_qualified=row['character_qualified'],target_feasible=row['target_feasible'])
        for row in selections],indent=2))


if __name__ == '__main__':
    main()
