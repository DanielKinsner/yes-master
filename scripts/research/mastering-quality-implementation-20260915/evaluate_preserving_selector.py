"""Apply the frozen selection rule to hash-bound September 16 complete outputs."""
import argparse
from dataclasses import asdict
import hashlib
import json
from pathlib import Path

from preserving_selector import Candidate, select, VERSION
from drive_metrics import LIMITS
from verification_common import sha


def read(path):
    return json.loads(path.read_text(encoding='utf-8'))


def load_candidates(evidence_path):
    evidence = read(evidence_path)
    assert evidence['status'] == 'complete' and evidence['limits'] == LIMITS
    available = {}
    for phase in ('single_first','flagged_followup'):
        run = evidence[phase]
        assert run['status'] == 'complete'
        for bound in run['inputs'].values():
            assert sha(Path(bound['path'])) == bound['sha256']
        native = read(Path(run['inputs']['native']['path']))
        checks = read(Path(run['inputs']['independent']['path']))
        assert native['status'] == checks['status'] == 'complete' and checks['all_pass']
        by_id = {row['id']:row for row in native['rows']}
        for row in run['rows']:
            if row['case'] == 'coat':
                continue
            original = by_id[row['id']]
            assert sha(Path(original['path'])) == row['sha256'] == original['sha256']
            assert original['requested_settings']['advanced']['lufs_offset_db'] == -14
            assert original['requested_settings']['preset'] == {'kind':'universal'}
            assert original['requested_settings']['intensity'] == .75
            assert row['lufs'] == original['lufs'] and row['qualified_peak'] == original['peak']
            if row['id'] in available:
                assert available[row['id']]['sha256'] == row['sha256']
            available[row['id']] = row
    result = {}
    for name in ('funk','metal','aphelion','rich','baby'):
        rows = [row for row in available.values() if row['case'] == name]
        source = {row['source_sha256'] for row in rows}
        assert len(source) == 1
        # Offline evidence key; a future app cache needs the full request and
        # source-analysis identity specified by C3, not this restricted tuple.
        context = hashlib.sha256(json.dumps(dict(source_sha256=source.pop(),
            preset='universal',intensity=.75,target=-14,rate=48000,channels=2,
            production_base='ab420654',evidence_sha256=sha(evidence_path)),sort_keys=True).encode()).hexdigest()
        result[name] = [Candidate(context,row['id'],row['sha256'],row['policy'],row['offset_db'],
            row['lufs'],-14.,row['qualified_peak'],-1.,tuple(row['character_failures'])) for row in rows]
    return result


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--evidence',type=Path,required=True)
    parser.add_argument('--specification',type=Path,required=True)
    parser.add_argument('--output',type=Path,required=True)
    args = parser.parse_args()
    assert not args.output.exists()
    cases = load_candidates(args.evidence)
    rows = [dict(case=name,**asdict(select(candidates,candidates[0].context))) for name,candidates in cases.items()]
    result = dict(status='complete',algorithm=VERSION,evidence_sha256=sha(args.evidence),
        specification_sha256=sha(args.specification),selector_sha256=sha(Path(__file__).with_name('preserving_selector.py')),
        script_sha256=sha(Path(__file__)),cases=rows,
        scope='Reselection of verified known development files only; no new audio, speedup or production-policy claim')
    args.output.write_text(json.dumps(result,indent=2,allow_nan=False)+'\n',encoding='utf-8')
    print(json.dumps([dict(case=row['case'],selected=row['selected']['id'] if row['selected'] else None,
                         reason=row['reason']) for row in rows],indent=2))


if __name__ == '__main__':
    main()
