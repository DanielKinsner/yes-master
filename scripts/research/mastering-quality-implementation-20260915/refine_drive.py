"""At most two measured local refinements; original development evidence stays immutable."""
import argparse
import copy
import json
from pathlib import Path
import subprocess
import sys

from evaluate_drive import LIMITS, select, sha


def next_refinement(rows):
    candidates = sorted((r for r in rows if r['policy'] in ('single', 'bounded', 'refinement')
                         and r['technical_pass'] and r['target_error_lu'] is not None),
                        key=lambda r: r['offset_db'])
    qualified = [r for r in candidates if not r['character_failures']]
    if not qualified or any(abs(r['target_error_lu']) <= LIMITS['target_error_lu'] for r in qualified):
        return None
    best = min(qualified, key=lambda r: (abs(r['target_error_lu']), abs(r['offset_db']), r['id']))
    index = candidates.index(best)
    neighbors = [candidates[i] for i in (index - 1, index + 1) if 0 <= i < len(candidates)]
    improving = [r for r in neighbors if abs(r['target_error_lu']) < abs(best['target_error_lu'])]
    if not improving:
        return None
    neighbor = min(improving, key=lambda r: (abs(r['target_error_lu']), abs(r['offset_db']), r['id']))
    return (best['offset_db'] + neighbor['offset_db']) / 2


def main():
    parser = argparse.ArgumentParser()
    for name in ('metrics', 'run', 'exe', 'output'):
        parser.add_argument('--' + name, type=Path, required=True)
    args = parser.parse_args()
    assert not args.output.exists(), 'preserve previous refinement evidence'
    base = json.loads(args.metrics.read_text(encoding='utf-8'))
    assert base['status'] == 'complete'
    args.output.mkdir(parents=True)
    result = copy.deepcopy(base)
    result['status'] = 'refinements-running'
    result['base_metrics'] = str(args.metrics.resolve())
    result['base_metrics_sha256'] = sha(args.metrics)
    result['refinement_executable_sha256'] = sha(args.exe)
    result['refinement_runs'] = []
    result['selections'] = []
    native_path = args.run / 'report.json'
    assert sha(native_path) == base['native_sha256']
    native = json.loads(native_path.read_text(encoding='utf-8'))
    assert native['status'] == 'complete'
    source = Path(native['source'])
    assert sha(source) == native['source_sha256']
    script = Path(__file__).with_name('evaluate_drive.py')
    for selection in base['selections']:
        preset, target = selection['preset_id'], selection['target']
        rows = [r for r in result['rows'] if r['preset_id'] == preset and r['target'] == target]
        control = next(r for r in rows if r['policy'] == 'control')
        for iteration in (1, 2):
            offset = next_refinement(rows)
            if offset is None:
                break
            label = f'{preset}-t{-target:g}-r{iteration}'
            folder = args.output / label
            folder.mkdir()
            settings = control['native']['requested_settings']
            # This runner deliberately covers C1's automatic Track groups.
            # Do not silently discard later Manual/EQ/voicing-control cases.
            assert settings['output_gain_db'] == 0 and not settings['volume_match']
            assert all(settings[key] == 0 for key in settings if key.startswith('eq_') and key.endswith('_db'))
            advanced = settings['advanced']
            assert advanced['compression_mode'] == 'preset' and advanced['adaptive_strength'] == .5
            assert all(value is None for key, value in advanced.items()
                       if key.startswith('compression_') and key != 'compression_mode')
            assert all(advanced[key] is None for key in ('warmth', 'width', 'presence_air'))
            job = dict(source=str(source), source_sha256=sha(source), targets=[target],
                       presets=[dict(id=preset, preset=settings['preset'], intensity=settings['intensity'])],
                       source_gain_db=control['source_gain_db'], manual_input_db=settings['input_gain_db'],
                       phase='refinement', candidate_offsets=[offset],
                       protocol='C1 development: measured local midpoint; at most two refinements per group')
            job_path = folder / 'job.json'
            job_path.write_text(json.dumps(job, indent=2), encoding='utf-8')
            with (folder / 'render.log').open('w', encoding='utf-8') as log:
                subprocess.run([str(args.exe.resolve()), str(job_path.resolve()), str((folder / 'native').resolve())],
                               stdout=log, stderr=subprocess.STDOUT, check=True)
            with (folder / 'metrics.log').open('w', encoding='utf-8') as log:
                subprocess.run([sys.executable, str(script), '--run', str(folder / 'native'),
                                '--output', str(folder / 'measured')], stdout=log, stderr=subprocess.STDOUT, check=True)
            metric_path = folder / 'measured' / 'metrics.json'
            measured = json.loads(metric_path.read_text(encoding='utf-8'))
            assert measured['status'] == 'complete'
            assert measured['source']['anchors'] == base['source']['anchors']
            new_control = next(r for r in measured['rows'] if r['policy'] == 'control')
            assert new_control['native']['render']['coefficients'] == control['native']['render']['coefficients']
            new_rows = [r for r in measured['rows'] if r['policy'] == 'refinement']
            assert len(new_rows) == 1
            row = new_rows[0]
            row['id'] = label + '-' + row['id']
            rows.append(row)
            result['rows'].append(row)
            result['refinement_runs'].append(dict(label=label, offset_db=offset,
                metrics=str(metric_path), metrics_sha256=sha(metric_path)))
            (args.output / 'metrics.json').write_text(json.dumps(result, indent=2, allow_nan=False), encoding='utf-8')
            print(label, offset, 'target error', row['target_error_lu'], 'constraints', row['character_failures'], flush=True)
        candidates = [r for r in rows if r['policy'] in ('single', 'bounded', 'refinement')]
        result['selections'].append(dict(preset_id=preset, target=target,
            preserve=select(candidates, True, control), target_first=select(candidates, False, control),
            candidate_count=len(candidates)))
    result['status'] = 'complete'
    (args.output / 'metrics.json').write_text(json.dumps(result, indent=2, allow_nan=False), encoding='utf-8')


if __name__ == '__main__':
    main()
