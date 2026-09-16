"""Freeze two known sources at two additional preset settings and two targets."""
import argparse
import json
from pathlib import Path
import shutil
import subprocess

import soundfile as sf
from verification_common import sha


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--root',type=Path,required=True)
    parser.add_argument('--specification',type=Path,required=True)
    parser.add_argument('--binary',type=Path,required=True)
    parser.add_argument('--output',type=Path,required=True)
    args = parser.parse_args()
    assert not args.output.exists() and args.binary.is_file()
    cases, estimate = [], 0
    for name,run,metrics in [('funk','c1-funk-recovered-v3','c1-funk-metrics-v3'),
                             ('rich','c1-rich-v2','c1-rich-metrics-v2')]:
        path = args.root/run/'report.json'
        report = json.loads(path.read_text(encoding='utf-8'))
        metrics_path = args.root/metrics/'metrics.json'
        facts = json.loads(metrics_path.read_text(encoding='utf-8'))
        assert report['status'] == facts['status'] == 'complete'
        assert facts['native_sha256'] == sha(path)
        source = Path(report['source'])
        assert sha(source) == report['source_sha256']
        info = sf.info(source)
        assert info.channels == 2
        candidates = []
        for preset in ('universal50','loud75'):
            for target in (-14.,-9.):
                group = f'{name}-{preset}-t{int(-target)}'
                for policy in ('control','single'):
                    rows = [row for row in report['rows'] if row['preset_id'] == preset and row['target'] == target
                            and row['source_gain_db'] == 0 and row['policy'] == policy]
                    assert len(rows) == 1, (name,preset,target,policy)
                    candidates.append(dict(id=rows[0]['id'],offset_db=0.,comparison_group=group,
                                           output_id=f'{group}-{policy}'))
                    estimate += ((info.frames*48000+info.samplerate-1)//info.samplerate)*8+128
        cases.append(dict(id=name,report=str(path.resolve()),report_sha256=sha(path),
            source_sha256=report['source_sha256'],source_metrics=str(metrics_path.resolve()),
            source_metrics_sha256=sha(metrics_path),candidates=candidates))
    free, reserve = shutil.disk_usage(args.output.parent).free, 25*1024**3
    assert free >= reserve+estimate
    job = dict(experiment='broader-first-v1',cases=cases,specification=str(args.specification.resolve()),
        specification_sha256=sha(args.specification),binary=str(args.binary.resolve()),binary_sha256=sha(args.binary),
        preparation_script_sha256=sha(Path(__file__)),estimated_new_wav_bytes=estimate,
        minimum_free_reserve_bytes=reserve,observed_free_bytes=free,
        local_commit=subprocess.check_output(['git','rev-parse','HEAD'],text=True).strip())
    args.output.write_text(json.dumps(job,indent=2,allow_nan=False)+'\n',encoding='utf-8')
    print(f'Frozen eight comparisons / sixteen new WAVs: {estimate} bytes; free {free}')


if __name__ == '__main__':
    main()
