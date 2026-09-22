"""Join both frozen C1 checkpoints only after all whole-file checks pass."""
import argparse
import json
from pathlib import Path

from drive_metrics import LIMITS, compare, constraints
from verification_common import sha


def read(path):
    return json.loads(path.read_text(encoding='utf-8'))


def checkpoint(root, mode, followup_version):
    followup = mode == 'flagged'
    paths = dict(job=root/(f'flagged-job-{followup_version}.json' if followup else 'single-first-job-v1.json'),
                 native=root/(f'flagged-{followup_version}/report.json' if followup else 'single-first-v1/report.json'),
                 metrics=root/(f'flagged-metrics-{followup_version}.json' if followup else 'metrics-v1.json'),
                 independent=root/(f'flagged-independent-{followup_version}/comparison.json' if followup else 'independent-v1/comparison.json'),
                 process=root/(f'flagged-{followup_version}.process.json' if followup else 'single-first-v1.process.json'))
    data = {key: read(path) for key,path in paths.items()}
    job, native, metrics, checks, process = [data[key] for key in paths]
    assert all(data[key]['status'] == 'complete' for key in ('native','metrics','independent','process'))
    assert process['exit_code'] == 0 and checks['all_pass'] and metrics['limits'] == LIMITS
    assert native['job_sha256'] == metrics['job_sha256'] == process['job_sha256'] == sha(paths['job'])
    assert metrics['native_sha256'] == checks['native_sha256'] == sha(paths['native'])
    assert process['binary_sha256'] == job['binary_sha256'] == sha(Path(job['binary']))
    assert sha(Path(job['specification'])) == job['specification_sha256']
    expected = 10 if followup else 12
    assert len(native['rows']) == len(metrics['rows']) == len(checks['rows']) == expected
    measured = {row['id']: row for row in metrics['rows']}
    checked = {str(Path(row['path']).resolve()): row for row in checks['rows']}
    assert len(measured) == len(checked) == expected
    facts = {}
    for case in job['cases']:
        source_metrics = Path(case['source_metrics'])
        assert sha(source_metrics) == case['source_metrics_sha256']
        facts[case['id']] = read(source_metrics)['source']
        report = Path(case['report'])
        assert sha(report) == case['report_sha256']
        assert sha(Path(read(report)['source'])) == case['source_sha256']
    controls = {row['case']: row['source_delta'] for row in metrics['rows'] if row['policy'] == 'control'}
    rows = []
    for row in native['rows']:
        metric = measured[row['id']]
        check = checked[str(Path(row['path']).resolve())]
        assert row['sha256'] == metric['sha256'] == check['sha256'] == sha(Path(row['path']))
        assert row['frames'] == metric['metrics']['frames'] == check['frames']
        assert row['rate'] == metric['metrics']['rate'] == check['rate'] == 48000
        assert row['channels'] == metric['metrics']['channels'] == check['channels'] == 2
        assert check['peak_pass'] and check['lufs_pass'] and check['fullscale_samples'] == 0
        assert row['peak'] <= row['ceiling'] == check['ceiling'] == -1
        assert row['lufs'] == metric['lufs'] == check['native_lufs']
        source = facts[row['case']]
        assert compare(source,metric['metrics']) == metric['source_delta']
        assert constraints(source,metric['source_delta'],controls[row['case']]) == metric['character_failures']
        reuse = row.get('reused_validated_baseline',False) if followup else row['reused_exact_anchor']
        assert reuse == ('verification_reused_from' in check)
        rows.append(dict(id=row['id'],case=row['case'],policy=row['policy'],offset_db=row['offset_db'],
                         sha256=row['sha256'],source_sha256=row['source_sha256'],frames=row['frames'],
                         lufs=row['lufs'],target_error_lu=metric['target_error_lu'],
                         qualified_peak=row['peak'],independent_peak=check['independent_peak'],
                         character_failures=metric['character_failures'],source_delta=metric['source_delta'],
                         limiter_max_db=row['render']['limiter_max_db'],
                         evaluation_s=metric['evaluation_s'],chain_s=row['render']['chain_s'],
                         src_s=row['src_s'],finalize_s=row['finalize_s'],metric_s=metric['metric_s'],
                         reused_output=reuse,fresh_reference_s=0 if reuse else check['reference_s']))
    assert sum(row['reused_output'] for row in rows) == (4 if followup else 2)
    return dict(status='complete',inputs={key:dict(path=str(path),sha256=sha(path)) for key,path in paths.items()},
                specification_sha256=job['specification_sha256'],binary_sha256=job['binary_sha256'],
                process=process,source_preparation=native['source_preparation'],rows=rows,cases=metrics['cases'],
                fresh_metric_work_s=sum(row['metric_s'] for row in rows),
                fresh_independent_work_s=sum(row['fresh_reference_s'] for row in rows),
                new_wavs=sum(not row['reused_output'] for row in rows))


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--root',type=Path,required=True)
    parser.add_argument('--output',type=Path,required=True)
    parser.add_argument('--followup-version',default='v2')
    args = parser.parse_args()
    assert not args.output.exists()
    first, followup = checkpoint(args.root,'single',args.followup_version), checkpoint(args.root,'flagged',args.followup_version)
    followup_job = read(Path(followup['inputs']['job']['path']))
    for key, name in [('native','report'),('metrics','metrics'),('independent','independent'),('job','job')]:
        assert followup_job[f'baseline_{name}_sha256'] == first['inputs'][key]['sha256']
    originals = {row['id']:row for row in first['rows']}
    for row in followup['rows']:
        if row['reused_output']:
            original = originals[row['id']]
            assert all(row[key] == original[key] for key in ('sha256','source_sha256','frames','lufs',
                       'target_error_lu','source_delta','character_failures','evaluation_s'))
    simple_cost = sum(row['initial_control_plus_candidate_s'] for row in first['cases'])
    extra_cost = sum(row['additional_three_evaluation_s'] for row in followup['cases'])
    result = dict(status='complete',scope='C1 known development sources only; no production sonic adoption',
                  script_sha256=sha(Path(__file__)),limits=LIMITS,single_first=first,flagged_followup=followup,
                  initial_five_source_control_plus_single_evaluation_s=simple_cost,
                  additional_two_source_search_evaluation_s=extra_cost,
                  conditional_five_source_evaluation_s=simple_cost+extra_cost,
                  cost_exclusions='Evaluation sums exclude original source analysis, decode/normalization, I/O/hash, metrics and references. '
                  'Native child process totals include decode/render/I/O/hash but exclude separate metrics/references. '
                  'Runs had overlapping work and do not prove app latency or an all-track-grid speedup.')
    # Interpretation after the frozen experiment, not a retroactive winner edit:
    # the candidate-only preserving selector can choose a quieter result even
    # when current processing already satisfies every character constraint.
    result['post_experiment_control_comparison'] = []
    for case in followup['cases']:
        control = next(row for row in followup['rows'] if row['id'] == case['case']+'-control')
        chosen = next(row for row in followup['rows'] if row['id'] == case['preserving']['id'])
        result['post_experiment_control_comparison'].append(dict(case=case['case'],
            current_control_qualifies_character=not control['character_failures'],
            preserving_choice=chosen['id'],
            current_control_is_closer_to_target_and_character_qualified=(not control['character_failures']
                and abs(control['target_error_lu']) < abs(chosen['target_error_lu'])),
            current_control_lufs=control['lufs'],preserving_choice_lufs=chosen['lufs'],
            note='Compare the corrected control as an eligible result in a future frozen selector; existing outcomes remain unchanged.'))
    args.output.parent.mkdir(parents=True,exist_ok=True)
    args.output.write_text(json.dumps(result,indent=2,allow_nan=False)+'\n',encoding='utf-8')
    print(json.dumps(dict(single=first['cases'],followup=followup['cases'],
                         conditional_evaluation_s=simple_cost+extra_cost),indent=2))


if __name__ == '__main__':
    main()
