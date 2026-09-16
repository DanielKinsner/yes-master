"""Compare complete production source-analysis observations without reanalysis."""
import argparse
import json
from pathlib import Path

from evaluate_drive import sha


def flatten(value, prefix=''):
    if not isinstance(value, dict):
        return {prefix: value}
    return {key: item for name, child in value.items()
            for key, item in flatten(child, f'{prefix}.{name}' if prefix else name).items()}


def compare(a, b):
    x, y = flatten(a), flatten(b)
    return {key: dict(before=x.get(key), after=y.get(key),
                     delta=y[key]-x[key] if isinstance(x.get(key), (float, int))
                     and isinstance(y.get(key), (float, int)) else None)
            for key in sorted(x.keys() | y.keys()) if x.get(key) != y.get(key)}


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--report', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    assert not args.output.exists()
    native = json.loads(args.report.read_text(encoding='utf-8'))
    assert native['status'] == 'complete'
    groups = {}
    for row in native['rows']:
        assert sha(Path(row['path'])) == row['sha256']
        key = (row['source_gain_db'], row['normalized'])
        group = groups.setdefault(row['source_id'], {})
        assert key not in group
        group[key] = row
    result = dict(status='complete', native_sha256=sha(args.report), rows=[], source_normalization=[],
        scope='Complete source-analysis comparison; exact coefficient identity is diagnostic; no new audio render, selector, gate or source-analysis policy')
    for source, rows in groups.items():
        assert set(rows) == {(gain, normalized) for gain in [0, -40, -80] for normalized in [False, True]}
        assert len({r['source_sha256'] for r in rows.values()}) == 1
        original, normalized_original = rows[(0, False)], rows[(0, True)]
        result['source_normalization'].append(dict(source=source,
            profile_differences=compare(original['resolved']['profile'], normalized_original['resolved']['profile']),
            coefficient_identity=[a['coefficients'] == b['coefficients']
                for a, b in zip(original['resolved']['rows'], normalized_original['resolved']['rows'], strict=True)]))
        for normalized in [False, True]:
            base = rows[(0, normalized)]
            for gain in [-40, -80]:
                copy = rows[(gain, normalized)]
                before, after = base['measured_input_lufs'], copy['measured_input_lufs']
                expected_shift = 0 if normalized else gain
                lufs_error = after-before-expected_shift if before is not None and after is not None else None
                coefficients = []
                for a, b in zip(base['resolved']['rows'], copy['resolved']['rows'], strict=True):
                    assert (a['preset'], a['intensity']) == (b['preset'], b['intensity'])
                    coefficients.append(dict(preset=a['preset'], intensity=a['intensity'],
                        exactly_identical=a['coefficients'] == b['coefficients']))
                result['rows'].append(dict(source=source, gain_db=gain, normalized=normalized,
                    profile_differences=compare(base['resolved']['profile'], copy['resolved']['profile']),
                    analysis_differences=compare(
                        {k:v for k,v in base['analysis'].items() if k not in ('track_id','measured_at_iso')},
                        {k:v for k,v in copy['analysis'].items() if k not in ('track_id','measured_at_iso')}),
                    stand_down_differences=compare(base['resolved']['stand_down_diagnostic'], copy['resolved']['stand_down_diagnostic']),
                    measured_input_lufs=after, serialized_source_lufs=copy['analysis']['lufs_integrated'],
                    expected_lufs_shift=expected_shift, lufs_shift_error=lufs_error,
                    coefficients=coefficients, analysis_s=copy['analysis_s']))
    result['times'] = [dict(id=r['id'], decode_s=r['decode_s'],
                           transform_write_s=r['transform_write_s'], analysis_s=r['analysis_s'])
                       for r in native['rows']]
    args.output.write_text(json.dumps(result, indent=2, allow_nan=False)+'\n', encoding='utf-8')
    print('Compared', len(groups), 'complete source-analysis groups')


if __name__ == '__main__':
    main()
