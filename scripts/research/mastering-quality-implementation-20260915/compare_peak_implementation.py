"""Check optimized-meter parity against a retained native run and its references.

The retained independent reference measurements are reused only after identical
source hashes/layout and tight numerical agreement have been established.
"""
import argparse
import json
import math
from pathlib import Path
from qualify_fir import ref


def main():
    parser = argparse.ArgumentParser()
    for name in ['before', 'after', 'references', 'output']:
        parser.add_argument('--' + name, type=Path, required=True)
    args = parser.parse_args()
    assert not args.output.exists()
    before = json.loads(args.before.read_text())
    after = json.loads(args.after.read_text())
    references = json.loads(args.references.read_text())
    result = {'version': 'peak-implementation-parity-1',
              'before_sha256': ref.sha(args.before), 'after_sha256': ref.sha(args.after),
              'references_sha256': ref.sha(args.references), 'rows': []}
    denominator = 1 - math.pi**2 / (8 * 16**2)
    for old, new, checked in zip(before['rows'], after['rows'], references['rows'], strict=True):
        assert old['id'] == new['id'] == checked['id']
        for key in ['source_sha256', 'rate', 'channels', 'frames']:
            assert old[key] == new[key], (new['id'], key)
        assert checked['source_sha256'] == new['source_sha256']
        maximum_error = 0.
        for left, right in zip(old['candidate'], new['candidate'], strict=True):
            assert left.keys() == right.keys()
            for key in left:
                error = abs(left[key] - right[key])
                maximum_error = max(maximum_error, error)
                assert error <= 1e-10 * max(1., abs(left[key])), (new['id'], key, error)
        for left, right in zip(old['fir_grid'], new['fir_grid'], strict=True):
            maximum_error = max(maximum_error, abs(left - right))
            assert abs(left - right) <= 1e-10 * max(1., abs(left))
        upper = [ref.db(max(c['continuous_upper'], (f + 1e-10*c['sample_peak'])/denominator))
                 for c, f in zip(new['candidate'], new['fir_grid'], strict=True)]
        errors = {name: [u-v for u, v in zip(upper, values, strict=True)]
                  for name, values in checked['zero_extended_references'].items()}
        width = [ref.db(c['continuous_upper'])-ref.db(c['grid_lower']) for c in new['candidate']]
        assert all(e >= -1e-5 for values in errors.values() for e in values)
        assert max(width) <= .0501
        result['rows'].append({'id': new['id'], 'max_absolute_error': maximum_error,
                               'upper_minus_reference_db': errors, 'width_db': width})
    args.output.write_text(json.dumps(result, indent=2) + '\n')
    print('passed', len(result['rows']), 'maximum amplitude error',
          max(r['max_absolute_error'] for r in result['rows']))


if __name__ == '__main__':
    main()
