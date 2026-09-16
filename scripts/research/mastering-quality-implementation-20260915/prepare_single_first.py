"""Freeze existing development inputs and corrected-chain reproduction anchors."""
import argparse
import json
from pathlib import Path
import shutil
import subprocess

import soundfile as sf
from verification_common import sha


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--root', type=Path, required=True)
    parser.add_argument('--specification', type=Path, required=True)
    parser.add_argument('--binary', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    assert not args.output.exists() and args.binary.is_file()
    baseline_path = args.root/'e-whole-v1/report.json'
    baseline = json.loads(baseline_path.read_text(encoding='utf-8'))
    assert baseline['status'] == 'complete'
    cases, estimated_bytes = [], 0
    for name, run, metrics in [
        ('coat','c1-coat-v1','c1-coat-metrics-v1'),
        ('funk','c1-funk-recovered-v3','c1-funk-metrics-v3'),
        ('metal','c1-metal-v2','c1-metal-metrics-v2'),
        ('aphelion','c1-aphelion-v2','c1-aphelion-metrics-v2'),
        ('rich','c1-rich-v2','c1-rich-metrics-v2'),
        ('baby','c1-baby-v2','c1-baby-metrics-v2')]:
        report_path = args.root/run/'report.json'
        report = json.loads(report_path.read_text(encoding='utf-8'))
        assert report['status'] == 'complete'
        source = Path(report['source'])
        assert sha(source) == report['source_sha256']
        metrics_path = args.root/metrics/'metrics.json'
        measured = json.loads(metrics_path.read_text(encoding='utf-8'))
        assert measured['status'] == 'complete' and measured['native_sha256'] == sha(report_path)
        candidates = []
        for policy in ('control','single'):
            target = -9. if name == 'coat' and policy == 'control' else -14.
            rows = [r for r in report['rows'] if r['policy'] == policy and r['target'] == target
                    and r['preset_id'] == 'universal75' and r['source_gain_db'] == 0]
            assert len(rows) == 1, (name, policy, len(rows))
            candidate = dict(id=rows[0]['id'], offset_db=0.)
            if name == 'coat':
                key = f'coat-{policy}-t{int(-target)}'
                reference = next(r for r in baseline['rows'] if r['case'] == key and r['variant'] == 'current')
                assert sha(Path(reference['path'])) == reference['sha256']
                candidate.update(expected_path=reference['path'], expected_sha256=reference['sha256'])
            else:
                info = sf.info(source)
                estimated_bytes += ((info.frames*48000 + info.samplerate-1)//info.samplerate)*8 + 128
            candidates.append(candidate)
        cases.append(dict(id=name, report=str(report_path.resolve()), report_sha256=sha(report_path),
                          source_sha256=report['source_sha256'], source_metrics=str(metrics_path.resolve()),
                          source_metrics_sha256=sha(metrics_path), candidates=candidates))
    reserve = 25*1024**3
    free = shutil.disk_usage(args.output.parent).free
    assert free >= reserve + estimated_bytes, (free, reserve, estimated_bytes)
    job = dict(experiment='single-first-development-v1', cases=cases,
               specification=str(args.specification.resolve()), specification_sha256=sha(args.specification),
               baseline_report=str(baseline_path.resolve()), baseline_report_sha256=sha(baseline_path),
               binary=str(args.binary.resolve()), binary_sha256=sha(args.binary),
               preparation_script_sha256=sha(Path(__file__)), estimated_new_wav_bytes=estimated_bytes,
               minimum_free_reserve_bytes=reserve, observed_free_bytes=free,
               local_commit=subprocess.check_output(['git','rev-parse','HEAD'],text=True).strip())
    args.output.write_text(json.dumps(job,indent=2,allow_nan=False)+'\n',encoding='utf-8')
    print(f'Frozen 2 reproduction anchors and 10 new outputs: {estimated_bytes} bytes; free {free}')


if __name__ == '__main__':
    main()
