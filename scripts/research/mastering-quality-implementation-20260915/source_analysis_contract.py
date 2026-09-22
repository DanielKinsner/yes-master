"""Fresh source-profile sensitivity audit; no mastering or automatic adoption."""
import argparse
from collections import Counter
import json
import math
from pathlib import Path
import re
import shutil
from verification_common import sha

ROOT = Path(__file__).resolve().parents[3]
SPEC = ROOT/'docs/reviews/2026-09-21-source-analysis-contract-protocol.md'
NUMBER = re.compile(r'(?<![A-Za-z_])[-+]?(?:\d*\.\d+|\d+)(?:e[-+]?\d+)?')


def read(path):
    return json.loads(Path(path).read_text(encoding='utf-8-sig'))


def write(path, data):
    Path(path).write_text(json.dumps(data, indent=2, allow_nan=False)+'\n', encoding='utf-8')


def bound(path):
    return dict(path=str(Path(path).resolve()), sha256=sha(Path(path)))


def prepare(args):
    assert not args.output.exists()
    args.output.mkdir()
    prior_path = ROOT/'test-output/dynamics-research-20260921/reference-headroom-v1/job.json'
    prior = read(prior_path)
    for case in prior['cases']:
        assert sha(Path(case['source'])) == case['source_sha256']
    binary = args.output/'probe.exe'
    shutil.copy2(args.binary, binary)
    estimate = sum(c['references'][0]['frames']*8*5+128*5 for c in prior['cases'])
    reserve = 25*1024**3
    assert shutil.disk_usage(args.output).free >= estimate+reserve
    analysis_source = ROOT/'src-tauri/src/analysis.rs'
    job = dict(experiment='source-analysis-contract-v1', cases=prior['cases'], sources=prior['sources'],
        prior_job=bound(prior_path), binary=str(binary.resolve()), binary_sha256=sha(binary),
        analysis_source=str(analysis_source), analysis_source_sha256=sha(analysis_source),
        specification=str(SPEC), specification_sha256=sha(SPEC), script=bound(Path(__file__)),
        minimum_free_reserve_bytes=reserve, estimated_new_wav_bytes=estimate)
    write(args.output/'job.json', job)
    print('Frozen 24 analyses / 20 derived inputs;', estimate, 'estimated bytes')


def flatten(data, prefix=''):
    if isinstance(data, dict):
        result = {}
        for key, value in data.items():
            result.update(flatten(value, prefix+'.'+key if prefix else key))
        return result
    return {prefix: data}


def profile_comparison(a, b):
    if a is None or b is None:
        return dict(pass_all=False, failures=['profile_unavailable'], differences=None, presence_matches=(a is None) == (b is None))
    av, bv = flatten(a), flatten(b)
    if av.keys() != bv.keys():
        return dict(pass_all=False, failures=['profile_shape'], differences=None, presence_matches=True)
    differences, failures = {}, []
    for key, v in av.items():
        other = bv[key]
        if v is None or other is None or not math.isfinite(v) or not math.isfinite(other):
            differences[key] = None
            failures.append(key+':unavailable')
        else:
            differences[key] = abs(v-other)
            limit = .01 if key.startswith('dynamic_range_') else .00001
            if differences[key] > limit:
                failures.append(key)
    return dict(pass_all=not failures, failures=failures, differences=differences, presence_matches=True)


def coefficient_comparison(a, b):
    structure = NUMBER.sub('#', a) == NUMBER.sub('#', b)
    av, bv = [float(x) for x in NUMBER.findall(a)], [float(x) for x in NUMBER.findall(b)]
    numeric = structure and len(av) == len(bv) and all(math.isfinite(x) for x in av+bv) and not re.search(r'\b(?:NaN|inf|infinity)\b', a+b, re.IGNORECASE)
    return dict(exact_identity=a == b, structure_identity=structure,
        maximum_absolute_numeric_difference=max((abs(x-y) for x,y in zip(av,bv,strict=True)), default=0.) if numeric else None,
        numeric_comparison_available=numeric, scope='Coefficient sensitivity only; no audible-equivalence claim')


def summarize(args):
    job, native = read(args.job), read(args.report)
    assert native['status'] == 'complete' and len(native['rows']) == 24 and native['job_sha256'] == sha(args.job)
    for path, expected in [(job['specification'],job['specification_sha256']),
            (job['analysis_source'],job['analysis_source_sha256']), (job['binary'],job['binary_sha256']),
            (job['prior_job']['path'],job['prior_job']['sha256'])]:
        assert sha(Path(path)) == expected
    comparisons, normalization, derived = [], [], []
    for row in native['rows']:
        assert sha(Path(row['path'])) == row['sha256']
        case = next(c for c in job['cases'] if c['id'] == row['case'])
        if row['domain'] == 'normalized':
            ref = next(r for r in case['references'] if r['source_gain_db'] == row['source_gain_db'])
            assert row['pcm_sha256'] == ref['input_pcm_sha256']
        if not (row['domain'] == 'actual' and row['source_gain_db'] == 0):
            derived.append(bound(row['path']))
        original = next(r for r in native['rows'] if r['case'] == row['case'] and r['domain'] == row['domain'] and r['source_gain_db'] == 0)
        if row['source_gain_db'] != 0:
            comparisons.append(dict(case=row['case'], domain=row['domain'], source_gain_db=row['source_gain_db'],
                profile=profile_comparison(original['profile'],row['profile']),
                coefficients=coefficient_comparison(original['coefficients'],row['coefficients']),
                native_availability=row['native_availability'], native_lufs=row['native_lufs'],
                analysis_lufs=row['analysis']['lufs_integrated'], original_deep=original['deep'] is not None, deep=row['deep'] is not None))
        elif row['domain'] == 'normalized':
            actual = next(r for r in native['rows'] if r['case'] == row['case'] and r['domain'] == 'actual' and r['source_gain_db'] == 0)
            normalization.append(dict(case=row['case'], profile=profile_comparison(actual['profile'],row['profile']),
                coefficients=coefficient_comparison(actual['coefficients'],row['coefficients'])))
    assert len(derived) == 20 and len(comparisons) == 16 and len(normalization) == 4
    process = read(args.report.parent.with_suffix('.process.json'))
    result = dict(status='complete', experiment=job['experiment'], inputs=dict(job=bound(args.job),native=bound(args.report)),
        analysis_source=bound(job['analysis_source']), rows=native['rows'], preparation=native['preparation'],
        comparisons=comparisons, normalization=normalization, process=process,
        memory_budget_pass=process['peak_working_set_bytes'] <= 2*1024**3,
        profile_copy_passes={d:sum(c['profile']['pass_all'] for c in comparisons if c['domain']==d) for d in ('actual','normalized')},
        coefficient_copy_identities={d:sum(c['coefficients']['exact_identity'] for c in comparisons if c['domain']==d) for d in ('actual','normalized')},
        availability=dict(Counter(r['domain']+':'+r['native_availability'] for r in native['rows'])),
        derived_inputs=derived,new_input_bytes=sum(Path(x['path']).stat().st_size for x in derived),
        script=bound(Path(__file__)),scope='Fresh production source-analysis audit only. No new masters or shipping changes.')
    assert not args.output.exists()
    write(args.output,result)
    print(json.dumps({k:result[k] for k in ('profile_copy_passes','coefficient_copy_identities','availability','memory_budget_pass')},indent=2))


if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument('mode',choices=('prepare','summarize'))
    for name in ('output','job','report','binary'):
        parser.add_argument('--'+name,type=Path,required=name=='output')
    args = parser.parse_args()
    globals()[args.mode](args)
