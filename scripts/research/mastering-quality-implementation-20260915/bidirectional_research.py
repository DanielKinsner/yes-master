"""Hash-bound preparation and whole-file scoring for a seven-render checkpoint."""
import argparse
from collections import Counter
from dataclasses import asdict
import json
from pathlib import Path
import shutil
import struct
import subprocess
import time

from bidirectional_selector import Candidate, VERSION, select
from drive_metrics import LIMITS, compare, constraints, measure
from verification_common import sha

ROOT = Path(__file__).resolve().parents[3]
OLD = ROOT/'test-output/dynamics-research-20260918'
EARLIER = ROOT/'test-output/dynamics-research-20260916'
SPEC = ROOT/'docs/reviews/2026-09-21-bidirectional-search-protocol.md'
NAMES = ('piano', 'imaginal', 'metal', 'rich')


def read(path):
    return json.loads(Path(path).read_text(encoding='utf-8'))


def bound(path):
    return dict(path=str(Path(path).resolve()), sha256=sha(Path(path)))


def typed(value):
    if isinstance(value, float):
        return struct.unpack('f', struct.pack('f', value))[0]
    if isinstance(value, dict):
        return {k: typed(v) for k, v in value.items()}
    if isinstance(value, list):
        return [typed(v) for v in value]
    return value


def verified(row, metric, check, source, control):
    assert row['sha256'] == metric['sha256'] == check['sha256'] == sha(Path(row['path']))
    assert row['frames'] == metric['metrics']['frames'] == check['frames']
    assert row['rate'] == metric['metrics']['rate'] == check['rate'] == 48000
    assert row['channels'] == metric['metrics']['channels'] == check['channels'] == 2
    assert row['peak'] <= row['ceiling'] == check['ceiling'] == -1
    assert check['peak_pass'] and check['lufs_pass'] and check['fullscale_samples'] == 0
    assert row['lufs'] == metric['lufs'] == check['native_lufs']
    assert compare(source, metric['metrics']) == metric['source_delta']
    assert constraints(source, metric['source_delta'], control) == metric['character_failures']


def prepare(args):
    assert not args.output.exists()
    bundles = {
        'universal': (OLD/'universal50-v1/report.json', OLD/'universal50-metrics-v1.json',
                      OLD/'universal50-independent-v1/comparison.json', OLD/'universal50-job-v1.json'),
        'broader': (EARLIER/'broader-v1/report.json', EARLIER/'broader-metrics-v1.json',
                    EARLIER/'broader-independent-v1/comparison.json', EARLIER/'broader-job-v1.json'),
    }
    retained, bindings, cases, sources = [], {}, [], {}
    estimate = 0
    for name in NAMES:
        paths = bundles['broader' if name == 'rich' else 'universal']
        native, metrics, independent, job = [read(p) for p in paths]
        assert native['status'] == metrics['status'] == independent['status'] == 'complete'
        assert independent['all_pass'] and metrics['limits'] == LIMITS
        assert metrics['native_sha256'] == independent['native_sha256'] == sha(paths[0])
        assert native['job_sha256'] == metrics['job_sha256'] == sha(paths[3])
        bindings[name] = [bound(p) for p in paths]
        case = next(c for c in job['cases'] if c['id'] == name)
        assert sha(Path(case['report'])) == case['report_sha256']
        assert sha(Path(case['source_metrics'])) == case['source_metrics_sha256']
        prior, facts = read(case['report']), read(case['source_metrics'])
        assert facts['status'] == prior['status'] == 'complete'
        assert facts['limits'] == LIMITS and facts['native_sha256'] == case['report_sha256']
        assert sha(Path(prior['source'])) == case['source_sha256'] == prior['source_sha256']
        sources[name] = facts['source']
        pair = [r for r in native['rows'] if r['case'] == name and r['preset_id'] == 'universal50'
                and r['requested_settings']['advanced']['lufs_offset_db'] == -14]
        assert len(pair) == 2 and {r['policy'] for r in pair} == {'control', 'single'}
        control = next(r for r in metrics['rows'] if r['id'] == next(r['id'] for r in pair if r['policy'] == 'control'))
        for row in pair:
            assert row['source_sha256'] == case['source_sha256'] and row['offset_db'] == 0
            settings = row['requested_settings']
            assert settings['preset'] == {'kind': 'universal'} and settings['intensity'] == .5
            historic = next(r for r in prior['rows'] if r['policy'] == row['policy']
                            and r['preset_id'] == 'universal50' and r['target'] == -14 and r['source_gain_db'] == 0)
            assert typed(settings) == typed(historic['requested_settings'])
            assert row['render']['base_coefficients'] == historic['render']['coefficients']
            assert typed(row['drive_db']) == typed(historic['operating_drive_db'])
            metric = next(r for r in metrics['rows'] if r['id'] == row['id'])
            check = next(r for r in independent['rows'] if r['path'] == row['path'])
            verified(row, metric, check, facts['source'], control['source_delta'])
            retained.append(dict(native=row, metric=metric, independent=check, retained=True))
        single = next(r for r in prior['rows'] if r['policy'] == 'single' and r['preset_id'] == 'universal50'
                      and r['target'] == -14 and r['source_gain_db'] == 0)
        offsets = [-3.] if name == 'metal' else [-3., 3.]
        candidates = [dict(id=single['id'], offset_db=o, comparison_group=f'{name}-universal50-t14',
                          output_id=f"{name}-universal50-t14-single-{'m' if o < 0 else 'p'}3") for o in offsets]
        estimate += len(candidates)*(pair[0]['frames']*8+128)
        cases.append({k: case[k] for k in ('id', 'report', 'report_sha256', 'source_sha256',
                                          'source_metrics', 'source_metrics_sha256')})
        cases[-1]['candidates'] = candidates
    positive_path = ROOT/'docs/reviews/evidence/2026-09-18-dynamics-research/metal-positive-v1.json'
    positive = read(positive_path)
    assert positive['status'] == 'complete' and positive['limits'] == LIMITS
    bindings['positive_summary'] = [bound(positive_path)]
    positive_native_path = OLD/'metal-positive-v1/report.json'
    positive_checks_path = OLD/'metal-positive-independent-v1/comparison.json'
    positive_native, positive_checks = read(positive_native_path), read(positive_checks_path)
    assert positive_native['status'] == positive_checks['status'] == 'complete' and positive_checks['all_pass']
    assert positive_checks['native_sha256'] == sha(positive_native_path)
    bindings['positive_raw'] = [bound(positive_native_path), bound(positive_checks_path)]
    row = next(r for r in positive_native['rows'] if r['preset_id'] == 'universal50')
    # The earlier Metal summary stores character fields alongside native data.
    item = next(r for r in positive['rows'] if r['native']['id'] == row['id'])
    assert item['native'] == row
    zero = next(r for r in retained if r['native']['case'] == 'metal' and r['native']['policy'] == 'single')
    assert row['offset_db'] == 3 and row['source_sha256'] == zero['native']['source_sha256']
    assert typed(row['requested_settings']) == typed(zero['native']['requested_settings'])
    assert row['render']['base_coefficients'] == zero['native']['render']['base_coefficients']
    assert row['drive_db'] == zero['native']['drive_db']
    control = next(r['metric']['source_delta'] for r in retained if r['native']['case'] == 'metal' and r['native']['policy'] == 'control')
    check = next(r for r in positive_checks['rows'] if r['path'] == row['path'])
    metric = dict(id=row['id'], sha256=row['sha256'], lufs=row['lufs'],
                  **{k: item[k] for k in ('metrics', 'source_delta', 'character_failures', 'metric_s')})
    assert check == item['independent']
    verified(row, metric, check, sources['metal'], control)
    retained.append(dict(native=row, metric=metric, independent=check, retained=True))
    assert len(retained) == 9
    reserve, free = 25*1024**3, shutil.disk_usage(args.output.parent).free
    assert free >= reserve+estimate
    job = dict(experiment='bidirectional-development-v1', cases=cases, retained=retained, sources=sources,
               retained_bindings=bindings, specification=str(SPEC), specification_sha256=sha(SPEC),
               binary=str(args.binary.resolve()), binary_sha256=sha(args.binary),
               preparation_script_sha256=sha(Path(__file__)), estimated_new_wav_bytes=estimate,
               minimum_free_reserve_bytes=reserve, observed_free_bytes=free,
               local_commit=subprocess.check_output(['git','rev-parse','HEAD'], text=True).strip())
    args.output.write_text(json.dumps(job, indent=2, allow_nan=False)+'\n', encoding='utf-8')
    print(f'Frozen seven new outputs, nine verified retained files; {estimate} estimated new bytes')


def evaluate(args):
    assert not args.output.exists()
    job, native = read(args.job), read(args.report)
    assert native['status'] == 'complete' and native['experiment'] == job['experiment'] == 'bidirectional-development-v1'
    assert native['job_sha256'] == sha(args.job) and len(native['rows']) == 7
    assert sha(Path(job['specification'])) == job['specification_sha256']
    for bindings in job['retained_bindings'].values():
        for binding in bindings:
            assert sha(Path(binding['path'])) == binding['sha256']
    rows = []
    for row in native['rows']:
        assert sha(Path(row['path'])) == row['sha256']
        zero = next(r['native'] for r in job['retained'] if r['native']['case'] == row['case']
                    and r['native']['policy'] == 'single' and r['native']['offset_db'] == 0)
        assert typed(row['requested_settings']) == typed(zero['requested_settings'])
        assert row['source_sha256'] == zero['source_sha256'] and row['drive_db'] == zero['drive_db']
        assert row['render']['base_coefficients'] == zero['render']['base_coefficients']
        source = job['sources'][row['case']]
        started = time.perf_counter()
        metrics = measure(Path(row['path']), source['anchors'])
        metric_s = time.perf_counter()-started
        control = next(r['metric']['source_delta'] for r in job['retained']
                       if r['native']['case'] == row['case'] and r['native']['policy'] == 'control')
        delta = compare(source, metrics)
        rows.append(dict(id=row['id'], sha256=row['sha256'], metrics=metrics, source_delta=delta,
                         character_failures=constraints(source, delta, control), lufs=row['lufs'], metric_s=metric_s))
        print(row['id'], row['lufs'], rows[-1]['character_failures'], flush=True)
    args.output.write_text(json.dumps(dict(status='complete', limits=LIMITS, rows=rows,
        native_sha256=sha(args.report), job_sha256=sha(args.job)), indent=2, allow_nan=False)+'\n', encoding='utf-8')


def summarize(args):
    assert not args.output.exists()
    paths = dict(job=args.job, native=args.report, metrics=args.metrics, independent=args.independent, process=args.process)
    job, native, metrics, independent, process = [read(p) for p in paths.values()]
    assert all(r['status'] == 'complete' for r in (native, metrics, independent, process))
    assert metrics['limits'] == LIMITS and independent['all_pass'] and process['exit_code'] == 0
    assert native['job_sha256'] == metrics['job_sha256'] == process['job_sha256'] == sha(args.job)
    assert metrics['native_sha256'] == independent['native_sha256'] == sha(args.report)
    assert process['binary_sha256'] == job['binary_sha256'] == sha(Path(job['binary']))
    assert sha(Path(job['specification'])) == job['specification_sha256']
    for bindings in job['retained_bindings'].values():
        for binding in bindings:
            assert sha(Path(binding['path'])) == binding['sha256']
    assert len(native['rows']) == len(metrics['rows']) == len(independent['rows']) == 7
    rows = list(job['retained'])
    for row in native['rows']:
        metric = next(r for r in metrics['rows'] if r['id'] == row['id'])
        check = next(r for r in independent['rows'] if r['path'] == row['path'])
        rows.append(dict(native=row, metric=metric, independent=check, retained=False))
    groups = []
    for name in NAMES:
        group = [r for r in rows if r['native']['case'] == name]
        assert len(group) == 4
        control = next(r['metric']['source_delta'] for r in group if r['native']['policy'] == 'control')
        context = sha(args.job)+':'+name
        candidates = []
        for item in group:
            row, metric, check = [item[k] for k in ('native', 'metric', 'independent')]
            verified(row, metric, check, job['sources'][name], control)
            candidates.append(Candidate(context, row['id'], row['sha256'], row['policy'], row['offset_db'],
                row['lufs'], -14., row['peak'], row['ceiling'], tuple(metric['character_failures'])))
        initial = select([r for r in candidates if r.offset_db == 0], context)
        assert initial.needs_more_search
        result = select(candidates, context)
        cost = lambda r: r['native']['render']['chain_s']+r['native']['src_s']+r['native']['finalize_s']
        baseline = sum(cost(r) for r in group if r['native']['offset_db'] == 0)
        extra = sum(cost(r) for r in group if r['native']['offset_db'] != 0)
        groups.append(dict(case=name, initial=asdict(initial), selected=asdict(result), baseline_evaluation_s=baseline,
                           extra_evaluation_s=extra, extra_to_baseline_ratio=extra/baseline,
                           evaluation_budget_pass=extra <= 1.5*baseline))
    result = dict(status='complete', algorithm=VERSION, limits=LIMITS, inputs={k:bound(p) for k,p in paths.items()},
        specification_sha256=job['specification_sha256'], selector_sha256=sha(Path(__file__).with_name('bidirectional_selector.py')),
        script_sha256=sha(Path(__file__)), groups=groups, rows=rows, process=process, source_preparation=native['source_preparation'],
        outcome_counts=dict(Counter(g['selected']['reason'] for g in groups)), new_files=7, retained_files=9,
        new_audio_bytes=sum(Path(r['native']['path']).stat().st_size for r in rows if not r['retained']),
        memory_budget_pass=process['peak_working_set_bytes'] <= 2*1024**3,
        phase_work_s={k:sum(r['native'][k] if k != 'chain_s' else r['native']['render'][k]
                           for r in rows if not r['retained']) for k in ('chain_s','src_s','finalize_s')},
        metric_work_s=sum(r['metric']['metric_s'] for r in rows if not r['retained']),
        reference_work_s=sum(r['independent']['reference_s'] for r in rows if not r['retained']),
        scope='Four known unresolved Universal 50/-14 contexts; no holdout, listening or production adoption')
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(result, indent=2, allow_nan=False)+'\n', encoding='utf-8')
    print(json.dumps(dict(outcomes=result['outcome_counts'], groups=groups), indent=2))


if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument('mode', choices=('prepare', 'evaluate', 'summarize'))
    for name in ('binary','job','report','metrics','independent','process','output'):
        parser.add_argument('--'+name, type=Path, required=name == 'output')
    args = parser.parse_args()
    globals()[args.mode](args)
