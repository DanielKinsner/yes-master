"""Separate normalized-response consistency from selector/control dependence.

Development numeric budgets are declared before evaluating gain-copy outcomes.
No changed policy, threshold relaxation or automatic sonic adoption occurs here.
"""
import argparse
import json
from pathlib import Path

import numpy as np
import soundfile as sf

from evaluate_drive import sha, select
from drive_metrics import constraints

NUMERIC_LIMITS = dict(lufs=.001, raw_lufs=.001, operating_drive_db=.001,
                      normalization_remainder_db=.001, character_db=.01, correlation=.00001,
                      maximum_sample_difference=.0001)


def key(row):
    return row['preset_id'], row['target'], row['policy'], row['offset_db']


def difference(a, b):
    if a is None or b is None:
        return None
    return b-a


def maximum_difference(a, b):
    assert len(a) == len(b)
    return max((abs(y-x) for x, y in zip(a, b, strict=True)), default=0.)


def read_pair(run, metrics):
    run, metrics = Path(run), Path(metrics)
    native = json.loads((run/'report.json').read_text(encoding='utf-8'))
    result = json.loads(metrics.read_text(encoding='utf-8'))
    assert native['status'] == result['status'] == 'complete'
    assert sha(run/'report.json') == result['native_sha256']
    assert sha(Path(native['source'])) == native['source_sha256']
    return native, result


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--base', nargs=2, action='append', required=True, metavar=('RUN', 'METRICS'))
    parser.add_argument('--copy', nargs=2, required=True, metavar=('RUN', 'METRICS'))
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    assert not args.output.exists()
    native, copy = read_pair(*args.copy)
    assert native['source_gain_db'] in (-40, -80)
    base_rows, base_selections, descriptors = {}, [], []
    for run, metrics in args.base:
        original_native, original = read_pair(run, metrics)
        # The first Coat run predates the top-level gain field; every retained
        # row still records it. Require all rows to prove this is the original.
        assert original_native.get('source_gain_db', 0) == 0
        assert all(row['source_gain_db'] == 0 for row in original_native['rows'])
        assert original_native['source_sha256'] == native['source_sha256']
        assert original['limits'] == copy['limits']
        assert original['source'] == copy['source'], 'retain exact original-source facts and anchors'
        for row in original['rows']:
            assert key(row) not in base_rows, 'ambiguous original candidate'
            base_rows[key(row)] = row
        base_selections.extend(original['selections'])
        descriptors.append(original_native['descriptor'])
    result = dict(status='running', numeric_limits=NUMERIC_LIMITS,
                  source_sha256=native['source_sha256'], source_gain_db=native['source_gain_db'],
                  copy_metrics_sha256=sha(Path(args.copy[1])),
                  baseline_metrics_sha256=[sha(Path(p[1])) for p in args.base],
                  copy_descriptor=native['descriptor'], baseline_descriptors=descriptors,
                  analysis_scope=native['analysis_scope'], rows=[], selections=[])
    for row in copy['rows']:
        original = base_rows[key(row)]
        x_path, y_path = Path(original['native']['path']), Path(row['native']['path'])
        assert sha(x_path) == original['native']['sha256']
        assert sha(y_path) == row['native']['sha256']
        x, rate = sf.read(x_path, dtype='float64', always_2d=True)
        y, copy_rate = sf.read(y_path, dtype='float64', always_2d=True)
        assert rate == copy_rate and x.shape == y.shape and np.isfinite(x).all() and np.isfinite(y).all()
        delta = y-x
        metrics = dict(maximum_sample_difference=float(np.max(np.abs(delta))),
                       sample_difference_rms=float(np.sqrt(np.mean(delta*delta))),
                       lufs=difference(original['native']['delivery']['lufs'], row['native']['delivery']['lufs']),
                       raw_lufs=difference(original['native']['raw_lufs'], row['native']['raw_lufs']),
                       operating_drive_db=row['native']['operating_drive_db']-original['native']['operating_drive_db'],
                       normalization_remainder_db=row['native']['normalization_db']+native['source_gain_db']-original['native']['normalization_db'])
        for name in ('section_contrast_delta_db', 'attack_median_delta_db', 'attack_p10_delta_db'):
            metrics[name] = difference(original['delta'][name], row['delta'][name])
        for name in ('section_delta_db', 'paired_attack_delta_db', 'band_delta_db'):
            metrics[name+'_max_difference'] = maximum_difference(original['delta'][name], row['delta'][name])
        stereo = original['delta']['stereo_delta'], row['delta']['stereo_delta']
        if all(value is not None for value in stereo):
            for name in ('side_mid_db', 'correlation'):
                metrics[name] = difference(stereo[0][name], stereo[1][name])
        failures = []
        for name, value in metrics.items():
            if name == 'sample_difference_rms':
                continue
            limit = NUMERIC_LIMITS.get(name, NUMERIC_LIMITS['character_db'])
            if value is None or abs(value) > limit:
                failures.append(name)
        invariant = row['policy'] in ('single', 'bounded', 'refinement')
        result['rows'].append(dict(key=key(row), expected_gain_invariant=invariant,
                                  numeric_failures=failures, differences=metrics,
                                  baseline_character_failures=original['character_failures'],
                                  copy_character_failures=row['character_failures'],
                                  resolved_coefficients_identical=original['native']['render']['coefficients'] == row['native']['render']['coefficients'],
                                  technical_pass=original['technical_pass'] and row['technical_pass']))
    copy_by_id = {r['id']: r for r in copy['rows']}
    base_by_id = {r['id']: r for r in base_rows.values()}
    # Diagnostic only: hold the original control's tone/stereo comparator fixed.
    # The declared selector still uses each source-level copy's own control.
    # This distinguishes changed reference limits from changed candidate audio.
    diagnostic_rows = []
    for row in copy['rows']:
        control = next(r for r in base_rows.values() if r['policy'] == 'control'
                       and (r['preset_id'], r['target']) == (row['preset_id'], row['target']))
        diagnostic_rows.append(dict(row, character_failures=constraints(copy['source'], row['delta'], control['delta'])))
    for selected in copy['selections']:
        original = next(s for s in base_selections if (s['preset_id'], s['target']) ==
                        (selected['preset_id'], selected['target']))
        modes = {}
        for mode in ('preserve', 'target_first'):
            base_id, copy_id = original[mode]['selected'], selected[mode]['selected']
            base_key = key(base_by_id[base_id]) if base_id else None
            copy_key = key(copy_by_id[copy_id]) if copy_id else None
            modes[mode] = dict(baseline=original[mode], copy=selected[mode],
                               same_candidate=base_key == copy_key,
                               same_fallback=original[mode]['fallback'] == selected[mode]['fallback'])
        diagnostic_group = [r for r in diagnostic_rows if (r['preset_id'], r['target']) ==
                            (selected['preset_id'], selected['target'])]
        diagnostic_control = next(r for r in diagnostic_group if r['policy'] == 'control')
        diagnostic = select([r for r in diagnostic_group if r['policy'] in ('single', 'bounded', 'refinement')],
                            True, diagnostic_control)
        diagnostic_id = diagnostic['selected']
        original_id = original['preserve']['selected']
        result['selections'].append(dict(preset_id=selected['preset_id'], target=selected['target'], modes=modes,
            diagnostic_fixed_original_control=dict(selection=diagnostic,
                same_candidate=(key(copy_by_id[diagnostic_id]) if diagnostic_id else None) ==
                               (key(base_by_id[original_id]) if original_id else None),
                same_fallback=diagnostic['fallback'] == original['preserve']['fallback'])))
    expected = [r for r in result['rows'] if r['expected_gain_invariant']]
    result['normalized_response_numeric_pass'] = all(not r['numeric_failures'] and r['technical_pass']
        and r['resolved_coefficients_identical'] for r in expected)
    result['selection_invariant'] = all(v['same_candidate'] and v['same_fallback']
                                       for s in result['selections'] for v in s['modes'].values())
    result['status'] = 'complete'
    args.output.write_text(json.dumps(result, indent=2, allow_nan=False), encoding='utf-8')
    print('Normalized responses:', result['normalized_response_numeric_pass'],
          'selector consistency:', result['selection_invariant'], 'rows:', len(expected))


if __name__ == '__main__':
    main()
