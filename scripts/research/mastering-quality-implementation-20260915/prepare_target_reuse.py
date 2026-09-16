"""Bind four independently verified -14 selections for the target reuse probe."""
import argparse
import json
from pathlib import Path
import shutil
import subprocess

from verification_common import sha


def read(path):
    return json.loads(path.read_text(encoding='utf-8'))


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--baseline',type=Path,required=True)
    parser.add_argument('--specification',type=Path,required=True)
    parser.add_argument('--binary',type=Path,required=True)
    parser.add_argument('--output',type=Path,required=True)
    args = parser.parse_args()
    assert not args.output.exists() and args.binary.is_file()
    summary = read(args.baseline)
    assert summary['status'] == 'complete'
    assert sha(Path(__file__).with_name('preserving_selector.py')) == summary['selector_sha256']
    for bound in summary['inputs'].values():
        assert sha(Path(bound['path'])) == bound['sha256']
    native = read(Path(summary['inputs']['native']['path']))
    baseline_job = read(Path(summary['inputs']['job']['path']))
    independent = read(Path(summary['inputs']['independent']['path']))
    assert native['status'] == independent['status'] == 'complete' and independent['all_pass']
    assert len(native['rows']) == len(independent['rows']) == 16
    cases,estimated = [],0
    for selection in summary['selections']:
        if not selection['group'].endswith('-t14'):
            continue
        selected = selection['selected']
        assert selected is not None and selected['target_lufs'] == -14
        row = next(row for row in native['rows'] if row['id'] == selected['id'])
        assert row['sha256'] == selected['sha256'] == sha(Path(row['path']))
        source = next(case for case in baseline_job['cases'] if case['id'] == row['case'])
        cases.append(dict(id=selection['group'].removesuffix('-t14'),case=row['case'],preset_id=row['preset_id'],
            prepared_id=row['id'],prepared_path=row['path'],prepared_sha256=row['sha256'],
            frames=row['frames'],source_sha256=row['source_sha256'],source_metrics=source['source_metrics'],
            source_metrics_sha256=source['source_metrics_sha256'],baseline_selection_reason=selection['reason']))
        estimated += row['frames']*8+128
    assert len(cases) == 4
    reserve,free = 25*1024**3,shutil.disk_usage(args.output.parent).free
    assert free >= reserve+estimated
    job = dict(experiment='prepared-target-reuse-v1',cases=cases,
        baseline_summary=str(args.baseline.resolve()),baseline_summary_sha256=sha(args.baseline),
        baseline_metrics=summary['inputs']['metrics']['path'],baseline_metrics_sha256=summary['inputs']['metrics']['sha256'],
        specification=str(args.specification.resolve()),specification_sha256=sha(args.specification),
        binary=str(args.binary.resolve()),binary_sha256=sha(args.binary),preparation_script_sha256=sha(Path(__file__)),
        minimum_free_reserve_bytes=reserve,estimated_new_wav_bytes=estimated,observed_free_bytes=free,
        local_commit=subprocess.check_output(['git','rev-parse','HEAD'],text=True).strip())
    args.output.write_text(json.dumps(job,indent=2,allow_nan=False)+'\n',encoding='utf-8')
    print(f'Frozen four prepared buffers / new WAVs: {estimated} bytes; free {free}')


if __name__ == '__main__':
    main()
