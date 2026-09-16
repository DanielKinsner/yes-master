"""Frozen finite reference for the native polyphase saturation cost prototype."""
import argparse
import json
from pathlib import Path
import platform

import numpy as np
import scipy
from scipy.signal import upfirdn

from evaluate_drive import sha


def prepare(output, protocol):
    assert not output.exists()
    output.mkdir(parents=True)
    evidence = output.parent
    filters = []
    for length in [128, 512]:
        folder = evidence / f'e-isolated-filter{length}-v2'
        report = json.loads((folder/'report.json').read_text(encoding='utf-8'))
        path = folder/'fir-8x.f64le'
        assert report['status'] == 'complete'
        assert sha(path) == report['kernels']['8']['sha256']
        filters.append(dict(id=f'filter{length}',path=str(path.resolve()),sha256=sha(path)))
    n = 32768
    t = np.arange(n)/44100
    impulses = np.zeros((n, 2)); impulses[0, 0] = 1.4; impulses[-1, 1] = -1.4
    step = np.tile([.8, 1.4], (n, 1))
    tone = np.sin(2*np.pi*997*t)[:,None]*np.array([.4, .7])
    burst = np.sin(2*np.pi*19000*t)[:,None]*np.array([1.4, -.9]); burst[:n-8192] = 0
    signals = []
    for name, x in [('impulses',impulses),('step',step),('tone',tone),('burst',burst)]:
        path=output/f'{name}.f64le'
        x.astype('<f8').tofile(path)
        signals.append(dict(id=name,path=str(path.resolve()),sha256=sha(path)))
    job=dict(protocol_sha256=sha(protocol),script_sha256=sha(Path(__file__)),
             rate=44100,amount=.0715,channels=2,filters=filters,signals=signals)
    (output/'job.json').write_text(json.dumps(job,indent=2)+'\n',encoding='utf-8')


def check(job_path, native_path, output):
    assert not output.exists()
    job=json.loads(job_path.read_text(encoding='utf-8'))
    native=json.loads(native_path.read_text(encoding='utf-8'))
    assert native['status']=='complete' and native['job_sha256']==sha(job_path)
    assert len(native['rows'])==len(job['filters'])*len(job['signals'])==8
    result=dict(status='running',native_sha256=sha(native_path),job_sha256=sha(job_path),
        script_sha256=sha(Path(__file__)),python=platform.python_version(),numpy=np.__version__,
        scipy=scipy.__version__,maximum_absolute_error=1e-10,rows=[],
        scope='Independent finite upfirdn traversal with frozen FIR/positive curve; no analytic frequency requalification or production/device claim')
    for row in native['rows']:
        filt=next(f for f in job['filters'] if f['id']==row['filter'])
        signal=next(s for s in job['signals'] if s['id']==row['signal'])
        for info in [filt,signal,row]:assert sha(Path(info['path']))==info['sha256']
        kernel=np.fromfile(filt['path'],dtype='<f8')
        x=np.fromfile(signal['path'],dtype='<f8').reshape(-1,2)
        up=upfirdn(kernel*8,x,up=8,axis=0)
        drive=1+2*job['amount']
        wet=np.tanh(up*drive)/np.tanh(drive)
        expected=upfirdn(kernel,wet,down=8,axis=0)
        actual=np.fromfile(row['path'],dtype='<f8').reshape(-1,2)
        assert len(actual)==len(expected)==row['frames_out']==row['frames_in']+2*row['delay_frames']
        maximum=float(np.max(np.abs(expected-actual)))
        block_ms=np.concatenate([t['block_256_ns'] for t in row['trials']])/1e6
        result['rows'].append(dict(filter=row['filter'],signal=row['signal'],maximum_error=maximum,
            pass_error=maximum<=1e-10,delay_ms=row['delay_frames']/44100*1000,
            preparation_s=row['preparation_s'],state_bytes=row['state_bytes'],kernel_bytes=row['kernel_bytes'],
            median_processing_s=float(np.median([t['processing_s'] for t in row['trials']])),
            block_ms_median=float(np.median(block_ms)),block_ms_p95=float(np.percentile(block_ms,95)),
            block_ms_max=float(np.max(block_ms)),block_audio_interval_ms=256/44100*1000))
    result['status']='complete'
    result['all_pass']=all(r['pass_error'] for r in result['rows'])
    output.write_text(json.dumps(result,indent=2)+'\n',encoding='utf-8')
    assert result['all_pass'], 'preserved native/reference discrepancy'
    print('Verified all eight native finite-filter cases')


def main():
    parser=argparse.ArgumentParser()
    sub=parser.add_subparsers(dest='mode',required=True)
    p=sub.add_parser('prepare');p.add_argument('--output',type=Path,required=True);p.add_argument('--protocol',type=Path,required=True)
    c=sub.add_parser('check');c.add_argument('--job',type=Path,required=True);c.add_argument('--report',type=Path,required=True);c.add_argument('--output',type=Path,required=True)
    args=parser.parse_args()
    if args.mode=='prepare':prepare(args.output,args.protocol)
    else:check(args.job,args.report,args.output)


if __name__=='__main__':main()
