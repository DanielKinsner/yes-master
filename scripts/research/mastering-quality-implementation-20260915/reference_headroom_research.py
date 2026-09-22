"""Frozen headroom candidate and separately labeled normalized-pool diagnostic."""
import argparse
from collections import Counter
import json
import math
from pathlib import Path
import shutil
import time

import numpy as np
from drive_metrics import measure, compare, constraints, LIMITS
from verification_common import sha

ROOT = Path(__file__).resolve().parents[3]
EVIDENCE = ROOT/'docs/reviews/evidence/2026-09-21-dynamics-research'
SPEC = ROOT/'docs/reviews/2026-09-21-reference-headroom-protocol.md'


def read(path):
    return json.loads(Path(path).read_text(encoding='utf-8-sig'))


def write(path, data):
    Path(path).write_text(json.dumps(data, indent=2, allow_nan=False)+'\n', encoding='utf-8')


def bound(path):
    return dict(path=str(Path(path).resolve()), sha256=sha(Path(path)))


def prepare(args):
    assert not args.output.exists()
    args.output.mkdir()
    prior = read(ROOT/'test-output/dynamics-research-20260921/quiet-copy/job-v2.json')
    bindings = {name: bound(EVIDENCE/file) for name, file in [
        ('original', 'bidirectional-v1.json'), ('quiet', 'quiet-copy-v2.json'),
        ('availability', 'block-availability-v1.json'), ('static', 'static-reference-v1.json'),
        ('static_verification', 'static-reference-verification-v1.json')]}
    original, quiet, availability, static, verification = [read(b['path']) for b in bindings.values()]
    assert all(d['status'] == 'complete' for d in (original, quiet, availability, static, verification))
    mechanical = read(static['inputs']['mechanical']['path'])
    assert sha(Path(static['inputs']['mechanical']['path'])) == static['inputs']['mechanical']['sha256']
    assert mechanical['status'] == 'complete' and mechanical['all_pass']
    assert len(static['references']) == 12 and availability['retained_agreements'] == 32
    for row in original['rows']+quiet['rows']:
        assert sha(Path(row['native']['path'])) == row['native']['sha256']
        assert row['independent']['peak_pass'] and row['independent']['fullscale_samples'] == 0
    binary = args.output/'probe.exe'
    shutil.copy2(args.binary, binary)
    cases = []
    estimate = 0
    for case in prior['cases']:
        zero = next(r for r in case['originals'] if r['policy'] == 'single' and r['offset_db'] == 0)
        references = [r['native'] for r in static['references'] if r['native']['case'] == case['id']]
        assert [r['source_gain_db'] for r in references] == [0, -40, -80]
        for r in references:
            assert sha(Path(r['path'])) == r['sha256']
        assert sha(Path(case['source'])) == case['source_sha256']
        cases.append(dict(id=case['id'], source=case['source'], source_sha256=case['source_sha256'],
            settings=zero['requested_settings'], original_drive=zero['drive_db'],
            original_coefficients=zero['render']['base_coefficients'], preset_push_db=case['preset_push_db'],
            saturation_small_signal_db=case['saturation_small_signal_db'],
            delivered_frames=zero['frames'], references=references))
        estimate += zero['frames']*2*4*3+128*3
    reserve = 25*1024**3
    assert shutil.disk_usage(args.output).free >= reserve+estimate
    job = dict(experiment='reference-headroom-candidate-v1', cases=cases, sources=prior['sources'],
        bindings=bindings, binary=str(binary.resolve()), binary_sha256=sha(binary),
        specification=str(SPEC), specification_sha256=sha(SPEC), script=bound(Path(__file__)),
        minimum_free_reserve_bytes=reserve, estimated_new_wav_bytes=estimate)
    write(args.output/'job.json', job)
    print('Frozen twelve complete masters;', estimate, 'estimated bytes')


def verify_bindings(job):
    assert sha(Path(job['specification'])) == job['specification_sha256']
    assert sha(Path(job['binary'])) == job['binary_sha256']
    for b in job['bindings'].values():
        assert sha(Path(b['path'])) == b['sha256']


def copy_check(original, candidate, delta):
    values = dict(pcm=candidate['maximum_copy_difference'],
        lufs=None if candidate['lufs'] is None or original['lufs'] is None else abs(candidate['lufs']-original['lufs']),
        drive=abs(candidate['drive_db']-original['drive_db']),
        section_contrast=abs(delta['section_contrast_delta_db']),
        sections=max(abs(v) for v in delta['section_delta_db']),
        attacks=max(abs(v) for v in delta['paired_attack_delta_db']),
        bands=max(abs(v) for v in delta['band_delta_db']),
        side_mid=abs(delta['stereo_delta']['side_mid_db']),
        correlation=abs(delta['stereo_delta']['correlation']))
    tolerances = dict(pcm=1e-4, lufs=.001, drive=.001, section_contrast=.01,
        sections=.01, attacks=.01, bands=.01, side_mid=.01, correlation=1e-5)
    failures = [k for k, v in values.items() if v is None or not math.isfinite(v) or v > tolerances[k]]
    return dict(differences=values, tolerances=tolerances, failures=failures, pass_all=not failures)


def evaluate(args):
    job, native = read(args.job), read(args.report)
    verify_bindings(job)
    assert native['status'] == 'complete' and native['job_sha256'] == sha(args.job) and len(native['rows']) == 12
    assert not args.output.exists()
    static, original, quiet = [read(job['bindings'][n]['path']) for n in ('static', 'original', 'quiet')]
    result = dict(status='running', job=bound(args.job), native=bound(args.report), limits=LIMITS, rows=[])
    originals = {}
    for row in native['rows']:
        started = time.perf_counter()
        name, gain = row['case'], row['source_gain_db']
        path = Path(row['path'])
        assert sha(path) == row['sha256']
        source = job['sources'][name]
        reference = next(r for r in static['references'] if r['native']['case'] == name and r['native']['source_gain_db'] == gain)
        assert (row['reference_id'], row['reference_sha256']) == (reference['native']['id'], reference['native']['sha256'])
        controls = original['rows'] if gain == 0 else quiet['rows']
        control = next(r for r in controls if r['native']['case'] == name and r['native']['policy'] == 'control' and
            (gain == 0 or r['native']['source_gain_db'] == gain))
        metric = measure(path, source['anchors'])
        delta = compare(source, metric)
        copied = None
        if gain == 0:
            originals[name] = (row, metric)
        else:
            copied = copy_check(originals[name][0], row, compare(originals[name][1], metric))
        item = dict(native=row, metrics=metric, source_delta=delta,
            static_reference_failures=constraints(source, delta, reference['source_delta']),
            matched_current_failures=constraints(source, delta, control['metric']['source_delta']),
            target_error_lu=None if row['lufs'] is None else row['lufs']+14.,
            copy=copied, work_s=time.perf_counter()-started)
        result['rows'].append(item)
        write(args.output, result)
        print(row['id'], row['lufs'], item['static_reference_failures'], 'copy', None if copied is None else copied['pass_all'], flush=True)
    result.update(status='complete', all_copy_pass=all(r['copy'] is None or r['copy']['pass_all'] for r in result['rows']))
    write(args.output, result)


def normalized_pool(pool):
    """This new diagnostic cannot change the frozen production/control selectors."""
    assert len(pool) == 4 and {r['rank'] for r in pool} == {0, 1, 2, 3}
    eligible = [r for r in pool if r['technical_pass'] and not r['failures'] and
                r['lufs'] is not None and math.isfinite(r['lufs'])]
    hits = [r for r in eligible if abs(r['lufs']+14.) <= LIMITS['target_error_lu']]
    shortfalls = [r for r in eligible if r['lufs'] < -14.-LIMITS['target_error_lu']]
    selected = min(hits, key=lambda r: r['rank']) if hits else min(
        shortfalls, key=lambda r: (abs(r['lufs']+14.), r['rank']), default=None)
    return dict(selected=selected, reason='qualified_target' if hits else
        ('qualified_shortfall' if selected is not None else 'no_eligible_normalized_candidate'))


def summarize(args):
    job, native, metric, independent = [read(p) for p in (args.job, args.report, args.metrics, args.independent)]
    verify_bindings(job)
    assert all(d['status'] == 'complete' for d in (native, metric, independent))
    assert native['job_sha256'] == sha(args.job) and metric['job']['sha256'] == sha(args.job)
    assert metric['native']['sha256'] == sha(args.report) and independent['native_sha256'] == sha(args.report)
    assert len(native['rows']) == len(metric['rows']) == len(independent['rows']) == 12
    static, original, quiet, availability = [read(job['bindings'][n]['path']) for n in ('static', 'original', 'quiet', 'availability')]
    rows, groups = [], []
    for measured in metric['rows']:
        row = measured['native']; name, gain = row['case'], row['source_gain_db']
        reference = next(r for r in independent['rows'] if r['sha256'] == row['sha256'] and r['path'] == row['path'])
        assert sha(Path(row['path'])) == row['sha256']
        technical = reference['peak_pass'] and reference['lufs_pass'] and reference['fullscale_samples'] == 0 and row['peak'] <= -1.
        rows.append(dict(**measured, independent=reference, technical_pass=technical))
        old = [r for r in (original['rows'] if gain == 0 else quiet['rows']) if r['native']['case'] == name and
               (gain == 0 or r['native']['source_gain_db'] == gain)]
        static_group = next(g for g in static['groups'] if g['case'] == name and g['source_gain_db'] == gain)
        pool = []
        comparisons = []
        for retained in old:
            r = retained['native']; verification = retained['independent']
            assert sha(Path(r['path'])) == r['sha256']
            classified = next(x for x in static_group['rows'] if x['id'] == r['id'] and x['sha256'] == r['sha256'])
            lufs_pass = verification['lufs_pass'] if gain == 0 else next(
                a['agreement'] for a in availability['retained'] if a['id'] == r['id'] and a['file']['sha256'] == r['sha256'])
            valid = verification['peak_pass'] and verification['fullscale_samples'] == 0 and lufs_pass and r['peak'] <= -1.
            item = dict(id=r['id'], sha256=r['sha256'], policy=r['policy'], offset_db=r['offset_db'],
                lufs=r['lufs'], technical_pass=valid, failures=classified['diagnostic_failures'],
                matched_current_failures=retained['metric']['character_failures'], source_delta=classified['source_delta'])
            comparisons.append(item)
            if r['policy'] == 'single':
                pool.append(dict(**item, rank={0: 0, -3: 1, 3: 2}[r['offset_db']]))
        new = dict(id=row['id'], sha256=row['sha256'], policy='headroom', offset_db=None,
            lufs=row['lufs'], technical_pass=technical, failures=measured['static_reference_failures'],
            matched_current_failures=measured['matched_current_failures'], source_delta=measured['source_delta'])
        comparisons.append(new)
        pool.append(dict(**new, rank=3))
        selection = normalized_pool(pool)
        identity = lambda s: None if s['selected'] is None else (s['selected']['policy'], s['selected']['offset_db'])
        baseline = selection if gain == 0 else next(g['selection'] for g in groups if g['case'] == name and g['source_gain_db'] == 0)
        groups.append(dict(case=name, source_gain_db=gain, candidates=comparisons, selection=selection,
            identity_matches_original=identity(selection) == identity(baseline),
            prior_static_selection=static_group['selection']))
        print(name, gain, identity(selection), selection['reason'], flush=True)
    process = read(args.report.parent.with_suffix('.process.json'))
    result = dict(status='complete', experiment=job['experiment'], diagnostic='normalized-pool-v1', limits=LIMITS,
        inputs={n:bound(p) for n,p in [('job',args.job),('native',args.report),('metrics',args.metrics),('independent',args.independent)]},
        bindings=job['bindings'], script=bound(Path(__file__)), rows=rows, groups=groups,
        process=process, all_technical_pass=all(r['technical_pass'] for r in rows), all_copy_pass=metric['all_copy_pass'],
        copy_identity_matches=sum(g['identity_matches_original'] for g in groups if g['source_gain_db'] != 0),
        outcomes=dict(Counter(g['selection']['reason'] for g in groups)), new_files=12,
        new_audio_bytes=sum(Path(r['native']['path']).stat().st_size for r in rows),
        scope='Development diagnostic only. Existing selectors and shipping audio unchanged.')
    assert not args.output.exists()
    write(args.output, result)


if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument('mode', choices=('prepare', 'evaluate', 'summarize'))
    for name in ('output', 'binary', 'job', 'report', 'metrics', 'independent'):
        parser.add_argument('--'+name, type=Path, required=name == 'output')
    args = parser.parse_args()
    globals()[args.mode](args)
