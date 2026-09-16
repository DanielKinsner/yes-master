"""Evaluate frozen C1 full-song outputs; select with individually visible limits."""
import argparse
import hashlib
import json
from pathlib import Path
import time
from drive_metrics import LIMITS, measure, compare, constraints


def sha(path):
    with path.open('rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest()


def select(rows, preserve, control):
    valid = [r for r in rows if r['technical_pass'] and r['target_error_lu'] is not None
             and (not preserve or not r['character_failures'])]
    if not valid:
        return {'selected': control['id'] if control['technical_pass'] else None,
                'fallback': True, 'fallback_technical_pass': control['technical_pass'],
                'reason': 'no candidate with all required metrics/constraints; corrected control fallback',
                'target_error_lu': control['target_error_lu'],
                'character_failures': control['character_failures']}
    hits = [r for r in valid if abs(r['target_error_lu']) <= LIMITS['target_error_lu']]
    if hits:
        chosen = min(hits, key=lambda r: (abs(r['offset_db']), r['id']))
        reason = 'target and constraints feasible' if preserve else 'target feasible; character tradeoffs reported'
    else:
        chosen = min(valid, key=lambda r: (abs(r['target_error_lu']), abs(r['offset_db']), r['id']))
        reason = 'target infeasible within the frozen candidate budget'
    return dict(selected=chosen['id'], fallback=False, reason=reason, target_error_lu=chosen['target_error_lu'],
                character_failures=chosen['character_failures'])


def main():
    p = argparse.ArgumentParser()
    p.add_argument('--run', type=Path, required=True)
    p.add_argument('--output', type=Path, required=True)
    p.add_argument('--allow-running', action='store_true', help='explicit partial development snapshot only')
    a = p.parse_args()
    assert not a.output.exists(), 'preserve existing evidence'
    a.output.mkdir(parents=True)
    native = json.loads((a.run / 'report.json').read_text(encoding='utf-8'))
    assert native['status'] == 'complete' or a.allow_running
    source_path = Path(native['source'])
    assert sha(source_path) == native['source_sha256']
    start = time.perf_counter()
    source = measure(source_path)
    result = dict(protocol='c1-development-v1', limits=LIMITS, source=source,
                  native_sha256=sha(a.run / 'report.json'),
                  source_measure_s=time.perf_counter() - start, rows=[], selections=[])
    controls = {}
    for row in native['rows']:
        path = Path(row['path'])
        assert sha(path) == row['sha256']
        start = time.perf_counter()
        metrics = measure(path, source['anchors'])
        delta = compare(source, metrics)
        key = (row['preset_id'], row['target'])
        if row['policy'] == 'control':
            controls[key] = delta
        assert key in controls, 'measure corrected control before candidates'
        failures = constraints(source, delta, controls[key])
        target_error = row['delivery']['lufs'] - row['target'] if row['delivery']['lufs'] is not None else None
        technical = (metrics['frames'] == native['delivery_frames'] and
                     metrics['rate'] == native['delivery_rate'] and metrics['channels'] == native['channels'] and
                     row['delivery']['true_peak_dbtp'] <= row['ceiling'] + 1e-5)
        result['rows'].append(dict(id=row['id'], preset_id=row['preset_id'], policy=row['policy'],
                                  target=row['target'], offset_db=row['offset_db'],
                                  source_gain_db=row['source_gain_db'],
                                  technical_pass=technical, target_error_lu=target_error,
                                  character_failures=failures, metrics=metrics, delta=delta,
                                  measure_s=time.perf_counter()-start, native=row))
        (a.output / 'metrics.json').write_text(json.dumps(result, indent=2, allow_nan=False), encoding='utf-8')
        print('Measured', row['id'], 'target error', target_error, 'constraints', failures, flush=True)
    for preset, target in controls:
        candidates = [r for r in result['rows'] if r['preset_id'] == preset and r['target'] == target
                      and r['policy'] in ('single', 'bounded', 'refinement')
                      and r['source_gain_db'] == native.get('source_gain_db', 0)]
        control = next(r for r in result['rows'] if r['preset_id'] == preset and r['target'] == target and r['policy'] == 'control')
        result['selections'].append(dict(preset_id=preset, target=target,
            preserve=select(candidates, True, control), target_first=select(candidates, False, control),
            candidate_count=len(candidates)))
    result['status'] = 'complete' if native['status'] == 'complete' else 'partial-development-snapshot'
    (a.output / 'metrics.json').write_text(json.dumps(result, indent=2, allow_nan=False), encoding='utf-8')


if __name__ == '__main__':
    main()
