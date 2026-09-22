"""Record the post-power-recovery device and muted callback checks separately."""
import argparse
import json
from pathlib import Path
import numpy as np

from verification_common import sha


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--root',type=Path,required=True)
    parser.add_argument('--binary',type=Path,required=True)
    parser.add_argument('--source',type=Path,required=True)
    parser.add_argument('--output',type=Path,required=True)
    args = parser.parse_args()
    assert not args.output.exists()
    paths = dict(devices=args.root/'post-power-device-open.json',
                 gain=args.root/'post-power-realtek-gain.json',eq=args.root/'post-power-realtek-eq.json')
    data = {key:json.loads(path.read_text(encoding='utf-8')) for key,path in paths.items()}
    assert data['devices']['status'] == 'complete'
    default = next(row for row in data['devices']['rows'] if row['is_default'])
    assert default['device'] == 'Speakers (Realtek(R) Audio)' and default['result']['opened']
    rows = []
    for kind in ('gain','eq'):
        report = data[kind]
        assert report['device'] == default['device']
        assert report['requested_buffer_policy'] == 'device_default' and report['requested_frames'] is None
        assert report['source_rate'] == report['chain_rate'] == report['sample_rate'] == 48000
        assert report['file_rate_override'] == '96000'
        assert all(report[key] == 0 for key in ('deadline_misses','device_errors','streaming_error_code','exhausted_samples'))
        assert report['combined_gain_edits'] == (1200 if kind=='gain' else 0)
        assert report['coefficient_edits'] == (120 if kind=='eq' else 0)
        samples = [row[0]/1e6 for row in report['callback_rows_ns_frames_start_ns']]
        assert len(samples) == report['callbacks'] and samples
        rows.append(dict(kind=kind,device=report['device'],callbacks=report['callbacks'],
            gain_edits=report['combined_gain_edits'],eq_edits=report['coefficient_edits'],
            granted_frames_min=report['granted_frames_min'],granted_frames_max=report['granted_frames_max'],
            median_ms=float(np.median(samples)),p95_ms=float(np.percentile(samples,95)),max_ms=max(samples),
            deadline_misses=0,device_errors=0,converter_errors=0,exhausted_samples=0,
            cancellation_join_s=report['join_after_cancel_s']))
    result = dict(status='complete',scope='Post-power interruption: muted actual Realtek callbacks, 48-to-96-to-48 kHz, device-default buffer; no fixed-256, installed or listening claim',
        source_sha256=sha(args.source),binary_sha256=sha(args.binary),script_sha256=sha(Path(__file__)),
        inputs={key:dict(path=str(path),sha256=sha(path)) for key,path in paths.items()},
        default_open=default,rows=rows,
        limitations='Earlier endpoint-open failure remains historical with cause unresolved. An initial wrong exact test filter ran zero tests and is not counted; the corrected filter ran one test.')
    args.output.write_text(json.dumps(result,indent=2,allow_nan=False)+'\n',encoding='utf-8')
    print(json.dumps(rows,indent=2))


if __name__ == '__main__':
    main()
