"""Freeze and verify two corrected-chain reproductions of a retained C1 lead."""
import argparse
import json
from pathlib import Path
import shutil
import subprocess
import time

from drive_metrics import LIMITS, compare, constraints, measure
from verification_common import sha


def read(path):
    return json.loads(path.read_text(encoding='utf-8'))


def bound(path):
    return dict(path=str(path.resolve()), sha256=sha(path))


def retained(paths, source, source_sha):
    data = {}
    for key, item in paths.items():
        path = Path(item['path'])
        assert sha(path) == item['sha256']
        data[key] = read(path)
    native, metrics, independent = [data[key] for key in ('native', 'metrics', 'independent')]
    assert all(row['status'] == 'complete' for row in data.values())
    assert independent['all_pass'] and metrics['limits'] == LIMITS
    assert independent['native_sha256'] == metrics['native_sha256'] == paths['native']['sha256']
    result = []
    for row in native['rows']:
        if row['case'] != 'metal' or row['requested_settings']['advanced']['lufs_offset_db'] != -14:
            continue
        metric = next(x for x in metrics['rows'] if x['id'] == row['id'])
        check = next(x for x in independent['rows'] if Path(x['path']).resolve() == Path(row['path']).resolve())
        assert row['sha256'] == metric['sha256'] == check['sha256'] == sha(Path(row['path']))
        assert row['source_sha256'] == source_sha and row['offset_db'] == 0
        assert row['lufs'] == metric['lufs'] == check['native_lufs']
        assert row['frames'] == metric['metrics']['frames'] == check['frames']
        assert row['rate'] == check['rate'] == 48000 and row['channels'] == check['channels'] == 2
        assert row['peak'] <= row['ceiling'] == check['ceiling'] == -1
        assert check['peak_pass'] and check['lufs_pass'] and check['fullscale_samples'] == 0
        assert compare(source, metric['metrics']) == metric['source_delta']
        result.append(dict(native=row, metric=metric, independent=check))
    assert len(result) == 2 and {x['native']['policy'] for x in result} == {'control', 'single'}
    control = next(x['metric']['source_delta'] for x in result if x['native']['policy'] == 'control')
    for row in result:
        assert constraints(source, row['metric']['source_delta'], control) == row['metric']['character_failures']
    return result


def prepare(args):
    original = args.original/'c1-metal-v2/report.json'
    original_metrics = args.original/'c1-metal-metrics-v2/metrics.json'
    prior, facts = read(original), read(original_metrics)
    assert prior['status'] == facts['status'] == 'complete' and facts['native_sha256'] == sha(original)
    assert facts['limits'] == LIMITS and sha(Path(prior['source'])) == prior['source_sha256']
    history = [row for row in facts['rows'] if row['preset_id'] in ('universal50', 'universal75')
               and row['target'] == -14 and row['source_gain_db'] == 0 and row['offset_db'] in (3, 6)]
    assert len(history) == 4
    assert all(not row['character_failures'] if row['offset_db'] == 3 else
               'attack_median_delta_db' in row['character_failures'] for row in history)
    baselines, candidates, estimate = {}, [], 0
    for preset, paths in {
        'universal50': dict(native=args.new/'universal50-v1/report.json', metrics=args.new/'universal50-metrics-v1.json',
                            independent=args.new/'universal50-independent-v1/comparison.json'),
        'universal75': dict(native=args.previous/'single-first-v1/report.json', metrics=args.previous/'metrics-v1.json',
                            independent=args.previous/'independent-v1/comparison.json'),
    }.items():
        baselines[preset] = {key: bound(path) for key, path in paths.items()}
        verified = retained(baselines[preset], facts['source'], prior['source_sha256'])
        for row in verified:
            native = row['native']
            # The earlier single-first report predates the convenience label.
            # Always verify the actual settings; never infer identity from a
            # missing label or silently replace a contradictory one.
            settings = native['requested_settings']
            assert settings['preset'] == {'kind': 'universal'}
            assert settings['intensity'] == {'universal50': .5, 'universal75': .75}[preset]
            if 'preset_id' in native:
                assert native['preset_id'] == preset
            else:
                assert preset == 'universal75'
                assert read(Path(baselines[preset]['native']['path']))['experiment'] == 'single-first-development-v1'
        original_single = next(row for row in prior['rows'] if row['preset_id'] == preset and
                               row['target'] == -14 and row['source_gain_db'] == 0 and row['policy'] == 'single')
        zero = next(row['native'] for row in verified if row['native']['policy'] == 'single')
        assert zero['render']['base_coefficients'] == original_single['render']['coefficients']
        candidates.append(dict(id=original_single['id'], offset_db=3.,
            comparison_group=f'metal-{preset}-t14', output_id=f'metal-{preset}-t14-single-p3'))
        estimate += zero['frames']*8+128
    reserve, free = 25*1024**3, shutil.disk_usage(args.output.parent).free
    assert free >= reserve+estimate
    job = dict(experiment='metal-positive-drive-v1', cases=[dict(id='metal', report=str(original.resolve()),
        report_sha256=sha(original), source_metrics=str(original_metrics.resolve()),
        source_metrics_sha256=sha(original_metrics), source_sha256=prior['source_sha256'], candidates=candidates)],
        baselines=baselines, historical_leads=[dict(id=row['id'], offset_db=row['offset_db'],
            failures=row['character_failures']) for row in history],
        specification=str(args.specification.resolve()), specification_sha256=sha(args.specification),
        binary=str(args.binary.resolve()), binary_sha256=sha(args.binary),
        preparation_script_sha256=sha(Path(__file__)), estimated_new_wav_bytes=estimate,
        minimum_free_reserve_bytes=reserve, observed_free_bytes=free,
        local_commit=subprocess.check_output(['git','rev-parse','HEAD'], text=True).strip())
    args.output.write_text(json.dumps(job, indent=2, allow_nan=False)+'\n', encoding='utf-8')
    print(f'Frozen two new outputs, four verified retained rows; {estimate} new WAV bytes')


def summarize(args):
    paths = dict(job=args.new/'metal-positive-job-v1.json', native=args.new/'metal-positive-v1/report.json',
        independent=args.new/'metal-positive-independent-v1/comparison.json', process=args.new/'metal-positive-v1.process.json')
    job, native, independent, process = [read(path) for path in paths.values()]
    assert native['status'] == independent['status'] == process['status'] == 'complete'
    assert native['experiment'] == job['experiment'] == 'metal-positive-drive-v1'
    assert independent['all_pass'] and process['exit_code'] == 0
    assert native['job_sha256'] == process['job_sha256'] == sha(paths['job'])
    assert independent['native_sha256'] == sha(paths['native'])
    assert job['binary_sha256'] == process['binary_sha256'] == sha(Path(job['binary']))
    assert sha(Path(job['specification'])) == job['specification_sha256']
    case = job['cases'][0]
    assert sha(Path(case['report'])) == case['report_sha256']
    assert sha(Path(case['source_metrics'])) == case['source_metrics_sha256']
    assert sha(Path(read(Path(case['report']))['source'])) == case['source_sha256']
    source = read(Path(case['source_metrics']))['source']
    baselines = {preset: retained(items, source, case['source_sha256']) for preset, items in job['baselines'].items()}
    assert len(native['rows']) == len(independent['rows']) == 2
    assert {row['preset_id'] for row in native['rows']} == set(baselines)
    rows = []
    for row in native['rows']:
        preset = row['preset_id']
        zero = next(x['native'] for x in baselines[preset] if x['native']['policy'] == 'single')
        control = next(x['metric']['source_delta'] for x in baselines[preset] if x['native']['policy'] == 'control')
        assert row['requested_settings'] == zero['requested_settings'] and row['offset_db'] == 3
        assert row['source_sha256'] == zero['source_sha256'] == case['source_sha256']
        assert row['drive_db'] == zero['drive_db'] and row['render']['base_coefficients'] == zero['render']['base_coefficients']
        check = next(x for x in independent['rows'] if Path(x['path']).resolve() == Path(row['path']).resolve())
        assert sha(Path(row['path'])) == row['sha256'] == check['sha256']
        assert row['frames'] == zero['frames'] == check['frames']
        assert row['rate'] == check['rate'] == 48000 and row['channels'] == check['channels'] == 2
        assert row['peak'] <= row['ceiling'] == check['ceiling'] == -1
        assert row['lufs'] == check['native_lufs'] and check['peak_pass'] and check['lufs_pass'] and check['fullscale_samples'] == 0
        started = time.perf_counter()
        measured = measure(Path(row['path']), source['anchors'])
        metric_s = time.perf_counter()-started
        assert (measured['frames'], measured['rate'], measured['channels']) == (row['frames'], 48000, 2)
        delta = compare(source, measured)
        failures = constraints(source, delta, control)
        rows.append(dict(native=row, independent=check, metrics=measured, source_delta=delta,
            character_failures=failures, target_error_lu=row['lufs']+14,
            character_qualified=not failures, target_feasible=abs(row['lufs']+14) <= LIMITS['target_error_lu'],
            metric_s=metric_s, evaluation_s=row['render']['chain_s']+row['src_s']+row['finalize_s']))
    result = dict(status='complete', scope='Two positive-offset development reproductions; outside the frozen preserving selector grid, no production adoption',
        inputs={key: bound(path) for key, path in paths.items()}, limits=LIMITS, rows=rows,
        retained=baselines, process=process, source_preparation=native['source_preparation'],
        script_sha256=sha(Path(__file__)), specification_sha256=job['specification_sha256'])
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(result, indent=2, allow_nan=False)+'\n', encoding='utf-8')
    print(json.dumps([dict(id=row['native']['id'], lufs=row['native']['lufs'],
                          failures=row['character_failures']) for row in rows], indent=2))


def main():
    parser = argparse.ArgumentParser()
    sub = parser.add_subparsers(dest='action', required=True)
    prep = sub.add_parser('prepare')
    for key in ('original', 'new', 'previous', 'specification', 'binary', 'output'):
        prep.add_argument('--'+key, type=Path, required=True)
    summary = sub.add_parser('summarize')
    for key in ('new', 'output'):
        summary.add_argument('--'+key, type=Path, required=True)
    args = parser.parse_args()
    assert not args.output.exists()
    (prepare if args.action == 'prepare' else summarize)(args)


if __name__ == '__main__':
    main()
