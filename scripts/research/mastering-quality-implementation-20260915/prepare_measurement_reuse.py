"""Copy the built release unit probe, then bind four verified cached-buffer pairs."""
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
    parser.add_argument('--build-messages',type=Path,required=True)
    parser.add_argument('--specification',type=Path,required=True)
    parser.add_argument('--binary',type=Path,required=True)
    parser.add_argument('--output',type=Path,required=True)
    args = parser.parse_args()
    assert not args.output.exists() and not args.binary.exists()
    baseline = read(args.baseline)
    assert baseline['status'] == 'complete' and len(baseline['rows']) == 4
    for bound in baseline['inputs'].values():
        assert sha(Path(bound['path'])) == bound['sha256']
    original_job = read(Path(baseline['inputs']['job']['path']))
    native = read(Path(baseline['inputs']['native']['path']))
    independent = read(Path(baseline['inputs']['independent']['path']))
    assert independent['status'] == 'complete' and independent['all_pass'] and native['status'] == 'complete'
    records = [json.loads(line) for line in args.build_messages.read_text(encoding='utf-8-sig').splitlines() if line.strip()]
    binaries = {record['executable'] for record in records if record.get('reason') == 'compiler-artifact'
                and record.get('executable') and record['target']['name']=='yes_master_lib' and record['profile']['test']}
    assert len(binaries) == 1
    shutil.copyfile(binaries.pop(),args.binary)
    cases = []
    for case in original_job['cases']:
        row = next(row for row in native['rows'] if row['id'] == case['id'])
        assert sha(Path(case['prepared_path'])) == case['prepared_sha256']
        assert sha(Path(row['path'])) == row['sha256']
        cases.append(dict(id=case['id'],prepared_path=case['prepared_path'],prepared_sha256=case['prepared_sha256'],
            expected_path=row['path'],expected_sha256=row['sha256'],expected_lufs=row['lufs'],
            expected_peak=row['peak'],expected_gain=row['gain']))
    reserve,free = 25*1024**3,shutil.disk_usage(args.output.parent).free
    assert free >= reserve
    job = dict(experiment='prepared-measurement-reuse-v1',cases=cases,
        baseline_summary=str(args.baseline.resolve()),baseline_summary_sha256=sha(args.baseline),
        specification=str(args.specification.resolve()),specification_sha256=sha(args.specification),
        binary=str(args.binary.resolve()),binary_sha256=sha(args.binary),preparation_script_sha256=sha(Path(__file__)),
        minimum_free_reserve_bytes=reserve,estimated_new_wav_bytes=0,observed_free_bytes=free,
        local_commit=subprocess.check_output(['git','rev-parse','HEAD'],text=True).strip())
    args.output.write_text(json.dumps(job,indent=2,allow_nan=False)+'\n',encoding='utf-8')
    print(f'Frozen four paired paths / sixteen exact outputs, zero new WAVs; free {free}')


if __name__ == '__main__':
    main()
