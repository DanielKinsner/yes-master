"""Summarize completed C0 trials without combining preparation and edit waits."""
import argparse
import json
from pathlib import Path
from statistics import median


def stats(values):
    return {'median': median(values), 'worst': max(values), 'trials': len(values)}


def summarize(directory):
    report = json.loads((directory / 'report.json').read_text())
    process = json.loads(directory.with_suffix('.process.json').read_text())
    assert process['exit_code'] == 0 and len(report['sessions']) == 12
    groups = {}
    for session in report['sessions']:
        groups.setdefault(session['strategy'], []).append(session)
    result = {k: report[k] for k in ['source_sha256', 'source_rate', 'frames',
              'channels', 'hash_s', 'decode_s', 'analysis_s', 'pcm_bytes', 'deep_bytes']}
    result['source_preparation_s'] = sum(report[k] for k in ['hash_s', 'decode_s', 'analysis_s', 'passage_preparation_s'])
    result['whole_probe_process'] = {'wall_s': process['wall_s'],
        'cpu_s_last_sample': max(s['cpu_s'] for s in process['samples']),
        'peak_working_set_bytes': max(s['peak_working_set_bytes'] for s in process['samples'])}
    result['strategies'] = {}
    # The simulations differ only in reuse. Their final measurements must match.
    expected = {}
    for name, sessions in groups.items():
        assert len(sessions) == 3
        rows = {edit['edit']: [] for edit in sessions[0]['edits']}
        for s in sessions:
            for edit in s['edits']:
                rows[edit['edit']].append(edit['elapsed_s'])
                if 'delivered_lufs' in edit:
                    values = (edit['delivered_lufs'], edit['provisional_peak'])
                    previous = expected.setdefault(edit['edit'], values)
                    assert max(abs(a-b) for a,b in zip(values,previous)) < 2e-5
        result['strategies'][name] = {
            'session_work_s': stats([s['total_session_s'] for s in sessions]),
            'settings_preparation_s': stats([s.get('settings_preparation_s',0) for s in sessions]),
            'max_raw_cache_bytes': max(s.get('max_raw_cache_bytes',0) for s in sessions),
            'edits_s': {edit: stats(values) for edit,values in rows.items()},
            'scalar_hits_per_trial': [sum(e.get('scalar_hit',False) for e in s['edits']) for s in sessions],
            'raw_hits_per_trial': [sum(e.get('raw_hit',False) for e in s['edits']) for s in sessions],
        }
    result['matching_simulation_outputs'] = expected
    result['production_witness_hashes'] = {w['name']:w['sha256'] for w in report['production_witnesses']}
    return result


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--runs', nargs='+', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    assert not args.output.exists()
    result = {run.name:summarize(run) for run in args.runs}
    args.output.write_text(json.dumps(result, indent=2)+'\n')
    for name, r in result.items():
        print(name, 'source preparation', r['source_preparation_s'])
        for strategy, s in r['strategies'].items():
            print(strategy, 'session', s['session_work_s'], 'prep', s['settings_preparation_s'],
                  'initial', s['edits_s']['initial'], 'target', s['edits_s']['target'],
                  'cache MiB', s['max_raw_cache_bytes']/2**20)


if __name__ == '__main__':
    main()
