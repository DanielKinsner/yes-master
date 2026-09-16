"""Measure only new bounded-grid outputs using unchanged source/control limits."""
import argparse
import json
from pathlib import Path
import time

from drive_metrics import LIMITS, compare, constraints, measure
from verification_common import sha


def read(path):
    return json.loads(path.read_text(encoding='utf-8'))


def select(rows, control, preserving):
    eligible = [row for row in rows if not preserving or not row['character_failures']]
    fallback = not eligible
    if fallback:
        eligible = [control]
    hits = [row for row in eligible if abs(row['target_error_lu']) <= LIMITS['target_error_lu']]
    winner = min(hits, key=lambda row: abs(row['offset_db'])) if hits else min(
        eligible, key=lambda row: (abs(row['target_error_lu']), abs(row['offset_db'])))
    return dict(id=winner['id'], offset_db=winner['offset_db'], lufs=winner['lufs'],
                target_error_lu=winner['target_error_lu'], character_failures=winner['character_failures'],
                current_control_fallback=fallback,
                reason='no_character_qualified_candidate' if fallback else
                       'target_hit' if hits else 'candidate_with_target_shortfall')


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--job', type=Path, required=True)
    parser.add_argument('--report', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    assert not args.output.exists()
    job = read(args.job)
    assert job['experiment'] == 'flagged-lower-drive-v1'
    baseline_path = Path(job['baseline_metrics'])
    assert sha(baseline_path) == job['baseline_metrics_sha256']
    baseline = read(baseline_path)
    assert baseline['status'] == 'complete' and baseline['limits'] == LIMITS
    retained = {row['id']: row for row in baseline['rows']}
    facts = {}
    for case in job['cases']:
        path = Path(case['source_metrics'])
        assert sha(path) == case['source_metrics_sha256']
        facts[case['id']] = read(path)['source']
    result = dict(status='running',experiment=job['experiment'],job_sha256=sha(args.job),
                  baseline_metrics_sha256=sha(baseline_path),limits=LIMITS,rows=[],cases=[],
                  scope='Two flagged known development sources; no holdout or sonic adoption')
    checked = {}
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
            assert sha(Path(row['path'])) == row['sha256']
            source = facts[row['case']]
            reused = row['reused_validated_baseline']
            if reused:
                assert retained[row['id']]['sha256'] == row['sha256'] and row['offset_db'] == 0
                metrics, metric_s = retained[row['id']]['metrics'], 0.
            else:
                started = time.perf_counter()
                metrics = measure(Path(row['path']), source['anchors'])
                metric_s = time.perf_counter()-started
            assert (metrics['frames'],metrics['rate'],metrics['channels']) == (row['frames'],48000,2)
            delta = compare(source,metrics)
            control = retained[f"{row['case']}-control"]['source_delta']
            failures = constraints(source,delta,control)
            assert row['requested_settings']['advanced']['lufs_offset_db'] == -14
            evaluated = dict(id=row['id'],case=row['case'],policy=row['policy'],offset_db=row['offset_db'],
                sha256=row['sha256'],metrics=metrics,source_delta=delta,character_failures=failures,
                lufs=row['lufs'],target_error_lu=row['lufs']+14,peak=row['peak'],metric_s=metric_s,
                limiter_max_db=row['render']['limiter_max_db'],reused_validated_baseline=reused,
                evaluation_s=row['render']['chain_s']+row['src_s']+row['finalize_s'])
            result['rows'].append(evaluated)
            checked[row['id']] = row['sha256']
            args.output.write_text(json.dumps(result,indent=2,allow_nan=False)+'\n',encoding='utf-8')
            print(row['id'], evaluated['lufs'], failures, flush=True)
        if native['status'] == 'complete':
            assert len(result['rows']) == len(native['rows']) == 10
            for name in ('metal','rich'):
                rows = [row for row in result['rows'] if row['case'] == name and row['policy'] == 'single']
                assert {row['offset_db'] for row in rows} == {0., -3., -6., -12.}
                control = next(row for row in result['rows'] if row['id'] == f'{name}-control')
                single = next(row for row in rows if row['offset_db'] == 0)
                extra = sum(row['evaluation_s'] for row in rows if not row['reused_validated_baseline'])
                result['cases'].append(dict(case=name,preserving=select(rows,control,True),
                    target_first=select(rows,control,False),
                    prior_control_plus_single_s=control['evaluation_s']+single['evaluation_s'],
                    additional_three_evaluation_s=extra,
                    control_plus_four_candidates_s=control['evaluation_s']+single['evaluation_s']+extra))
            result.update(status='complete',native_sha256=sha(args.report))
            args.output.write_text(json.dumps(result,indent=2,allow_nan=False)+'\n',encoding='utf-8')
            print(json.dumps(result['cases'],indent=2),flush=True)
            return
        assert native['status'] == 'running'
        time.sleep(5)


if __name__ == '__main__':
    main()
