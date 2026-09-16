"""Measure completed E whole-chain files using fixed original C1 source anchors."""
import argparse
import json
from pathlib import Path
import time

import soundfile as sf

from drive_metrics import LIMITS, compare, constraints, measure
from evaluate_drive import sha


def main():
    parser=argparse.ArgumentParser()
    parser.add_argument('--report',type=Path,required=True)
    parser.add_argument('--metrics-map',type=Path,required=True)
    parser.add_argument('--output',type=Path,required=True)
    args=parser.parse_args()
    assert not args.output.exists()
    mapping=json.loads(args.metrics_map.read_text(encoding='utf-8'))
    sources={}
    for case,path in mapping.items():
        path=Path(path)
        metrics=json.loads(path.read_text(encoding='utf-8'))
        assert metrics['status']=='complete'
        sources[case]=dict(facts=metrics['source'],metrics_sha256=sha(path))
    result=dict(status='running',rows=[],c1_proposal_limits=LIMITS,
        metrics_map_sha256=sha(args.metrics_map),source_metrics={k:v['metrics_sha256'] for k,v in sources.items()},
        scope='Matched whole-chain mechanism changes using original C1 source anchors; C1 proposal constraints are descriptive, not E voicing approval')
    controls={}
    checked={}
    while True:
        try:native=json.loads(args.report.read_text(encoding='utf-8'))
        except (FileNotFoundError,json.JSONDecodeError):time.sleep(5);continue
        for row in native['rows']:
            key=row['path']
            if key in checked:
                assert checked[key]==row['sha256'];continue
            path=Path(key)
            assert sha(path)==row['sha256'] and sha(Path(row['source']))==row['source_sha256']
            info=sf.info(row['source'])
            source=sources[row['case']]['facts']
            metrics=measure(path,source['anchors'])
            assert metrics['rate']==48000 and metrics['channels']==info.channels==2
            assert metrics['frames']==(info.frames*48000+info.samplerate-1)//info.samplerate
            delta=compare(source,metrics)
            if row['variant']=='current':controls[row['case']]=(metrics,delta)
            assert row['case'] in controls
            control,control_delta=controls[row['case']]
            target=row['requested_settings']['advanced']['lufs_offset_db']
            result['rows'].append(dict(case=row['case'],variant=row['variant'],path=key,sha256=row['sha256'],
                metrics=metrics,source_delta=delta,matched_current_delta=compare(control,metrics),
                c1_proposal_failures=constraints(source,delta,control_delta),
                target_error_lu=row['lufs']-target,render=row['render'],gain=row['gain']))
            checked[key]=row['sha256']
            args.output.write_text(json.dumps(result,indent=2,allow_nan=False)+'\n',encoding='utf-8')
            print('Measured',row['case'],row['variant'],flush=True)
        if native['status']=='complete':
            assert len(result['rows'])==16
            result['status']='complete';result['native_sha256']=sha(args.report)
            args.output.write_text(json.dumps(result,indent=2,allow_nan=False)+'\n',encoding='utf-8')
            return
        assert native['status']=='running'
        time.sleep(5)


if __name__=='__main__':main()
