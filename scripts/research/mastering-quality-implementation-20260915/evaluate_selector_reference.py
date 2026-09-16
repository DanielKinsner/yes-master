"""Re-score retained C1 metrics against the declared normalized single reference.

No audio is rendered and no old evidence or production policy is modified.
"""
import argparse
import json
from pathlib import Path

from drive_metrics import LIMITS, constraints
from evaluate_drive import select, sha


def candidate_key(row):
    return [row['preset_id'], row['target'], row['policy'], row['offset_db']]


def read_metrics(paths):
    rows, facts, provenance = [], None, []
    for path in paths:
        path = Path(path)
        result = json.loads(path.read_text(encoding='utf-8'))
        assert result['status'] == 'complete' and result['limits'] == LIMITS
        if facts is None:
            facts = result['source']
        assert facts == result['source']
        rows.extend(result['rows'])
        provenance.append(dict(path=str(path), sha256=sha(path),
                               native_sha256=result['native_sha256']))
    keys = [tuple(candidate_key(row)) for row in rows]
    assert len(keys) == len(set(keys)), 'ambiguous candidate'
    assert rows
    return rows, facts, provenance


def score(rows, facts):
    groups = sorted({(row['preset_id'], row['target']) for row in rows})
    results = []
    for preset, target in groups:
        group = [row for row in rows if (row['preset_id'], row['target']) == (preset, target)]
        references = [row for row in group if row['policy'] == 'single']
        controls = [row for row in group if row['policy'] == 'control']
        assert len(references) == len(controls) == 1
        reference, control = references[0], controls[0]
        scored = [dict(row, character_failures=constraints(facts, row['delta'], reference['delta']))
                  for row in group]
        old_candidates = [r for r in group if r['policy'] in ('single', 'bounded', 'refinement')]
        candidates = [r for r in scored if r['policy'] in ('single', 'bounded', 'refinement')]
        new_control = next(r for r in scored if r['policy'] == 'control')
        by_id = {r['id']: r for r in scored}
        assert len(by_id) == len(scored)
        selections = {}
        for mode, preserve in [('preserve', True), ('target_first', False)]:
            selections[mode] = {}
            for version, pool, fallback in [('old', old_candidates, control), ('normalized', candidates, new_control)]:
                selected = select(pool, preserve, fallback)
                row = by_id[selected['selected']] if selected['selected'] is not None else None
                selections[mode][version] = dict(selected,
                    candidate_key=candidate_key(row) if row else None,
                    delta=row['delta'] if row else None,
                    technical_pass=row['technical_pass'] if row else False)
        results.append(dict(preset_id=preset, target=target,
            reference_key=candidate_key(reference), selections=selections,
            candidates=[dict(candidate_key=candidate_key(row),
                old_failures=old['character_failures'], new_failures=row['character_failures'],
                target_error_lu=row['target_error_lu'], technical_pass=row['technical_pass'],
                delta=row['delta']) for old, row in zip(group, scored, strict=True)]))
    return results


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--manifest', required=True, type=Path)
    parser.add_argument('--specification', required=True, type=Path)
    parser.add_argument('--output', required=True, type=Path)
    args = parser.parse_args()
    assert not args.output.exists(), 'preserve existing evidence'
    manifest = json.loads(args.manifest.read_text(encoding='utf-8'))
    assert manifest['specification_sha256'] == sha(args.specification)
    result = dict(status='running', protocol='c1-normalized-reference-development-v2',
        limits=LIMITS, manifest_sha256=sha(args.manifest), script_sha256=sha(Path(__file__)),
        specification_sha256=sha(args.specification), sources=[],
        scope='Re-scoring completed f32-chain metrics; no new PCM verification, source analysis, holdout or production adoption')
    for source in manifest['sources']:
        levels, source_facts = {}, None
        assert set(source['levels']) == {'0', '-40', '-80'}
        for level, paths in source['levels'].items():
            rows, facts, provenance = read_metrics(paths)
            assert all(row['source_gain_db'] == float(level) for row in rows)
            if source_facts is None:
                source_facts = facts
            assert source_facts == facts, 'this experiment retains original source facts/anchors'
            levels[level] = dict(metrics=provenance, groups=score(rows, facts))
        consistency = []
        baseline = levels['0']['groups']
        for level in ['-40', '-80']:
            copy = levels[level]['groups']
            assert [(r['preset_id'], r['target']) for r in baseline] == [(r['preset_id'], r['target']) for r in copy]
            for before, after in zip(baseline, copy, strict=True):
                for mode in ['preserve', 'target_first']:
                    comparison = {}
                    for version in ['old', 'normalized']:
                        a, b = before['selections'][mode][version], after['selections'][mode][version]
                        comparison[version] = dict(same_candidate=a['candidate_key'] == b['candidate_key'],
                            same_fallback=a['fallback'] == b['fallback'])
                    consistency.append(dict(level=level, preset_id=before['preset_id'], target=before['target'],
                                            mode=mode, comparison=comparison))
        result['sources'].append(dict(id=source['id'], levels=levels, consistency=consistency))
    result['status'] = 'complete'
    args.output.write_text(json.dumps(result, indent=2, allow_nan=False)+'\n', encoding='utf-8')
    print('Completed normalized-reference comparison:', len(result['sources']), 'sources')


if __name__ == '__main__':
    main()
