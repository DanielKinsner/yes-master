"""Join freshly verified and revalidated retained Universal 50 development pairs."""
import argparse
from collections import Counter
import json
from pathlib import Path

from drive_metrics import LIMITS
from preserving_selector import VERSION
from verification_common import sha


def read(path):
    return json.loads(path.read_text(encoding='utf-8'))


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--fresh', type=Path, required=True)
    parser.add_argument('--retained', type=Path, required=True)
    parser.add_argument('--revalidated', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    assert not args.output.exists()
    fresh, retained, revalidated = [read(path) for path in (args.fresh, args.retained, args.revalidated)]
    for data in (fresh, retained, revalidated):
        assert data['status'] == 'complete' and data['limits'] == LIMITS and data['algorithm'] == VERSION
        for item in data['inputs'].values():
            assert sha(Path(item['path'])) == item['sha256']
    # Revalidation must preserve the old measurements, selections and timing exactly.
    for key in ('rows', 'selections', 'process', 'source_preparation', 'phase_work_s'):
        assert retained[key] == revalidated[key], key
    assert len(fresh['rows']) == 24 and len(retained['rows']) == 16
    rows = [dict(row, evidence='fresh') for row in fresh['rows']]
    rows += [dict(row, evidence='retained') for row in retained['rows'] if row['preset_id'] == 'universal50']
    expected_sources = {'coat', 'piano', 'imaginal', 'metal', 'aphelion', 'baby', 'funk', 'rich'}
    expected = {(source, target, policy) for source in expected_sources
                for target in (-14., -9.) for policy in ('control', 'single')}
    assert len(rows) == len({row['id'] for row in rows}) == 32
    assert {(row['case'], row['target'], row['policy']) for row in rows} == expected
    assert all(row['preset_id'] == 'universal50' for row in rows)
    selections = fresh['selections'] + [row for row in retained['selections']
                                        if '-universal50-' in row['group']]
    assert len(selections) == len({row['group'] for row in selections}) == 16
    by_key = {(row['case'], row['target'], row['policy']): row for row in rows}
    effects = []
    for source in sorted(expected_sources):
        single14, single9 = [by_key[source, target, 'single'] for target in (-14., -9.)]
        effects.append(dict(source=source,
            single_loudness_increase_lu=single9['lufs']-single14['lufs'],
            single_additional_loss_db={key: single14['source_delta'][key]-single9['source_delta'][key]
                for key in ('section_contrast_delta_db', 'attack_median_delta_db', 'attack_p10_delta_db')},
            single14_failures=single14['character_failures'], single9_failures=single9['character_failures']))
    result = dict(status='complete', scope='Eight known sources at Universal 50 and two targets; original level only, no holdout or production adoption',
        inputs={key: dict(path=str(path), sha256=sha(path)) for key, path in
                [('fresh', args.fresh), ('retained', args.retained), ('revalidated', args.revalidated)]},
        script_sha256=sha(Path(__file__)), limits=LIMITS, algorithm=VERSION, rows=rows,
        selections=selections, outcome_counts=dict(Counter(row['reason'] for row in selections)),
        single_target_effects=effects,
        timing_scope='Only fresh evidence has new cost observations. Retained rows keep original measurements; no combined process/session time is inferred.')
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(result, indent=2, allow_nan=False)+'\n', encoding='utf-8')
    print(json.dumps(dict(outcomes=result['outcome_counts'], single_target_effects=effects), indent=2))


if __name__ == '__main__':
    main()
