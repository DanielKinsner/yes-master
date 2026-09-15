"""B2 candidate exploration, independent of the production meter. No adoption.

Compare finite Hann-windowed sinc polyphase candidates with the corrected FFT
reference. Keep complete outputs and failed cases. See the implementation ledger.
"""
import argparse
import importlib.util
import json
from pathlib import Path
import time
import numpy as np
from scipy.signal import upfirdn, resample

ROOT = Path(__file__).resolve().parents[3]
spec = importlib.util.spec_from_file_location('checked_reference', ROOT/'scripts/research/final-review-20260915/verify.py')
ref = importlib.util.module_from_spec(spec)
spec.loader.exec_module(ref)

def fir_peak(x, taps, factor):
    t = np.arange(-taps*factor//2, taps*factor//2 + 1) / factor
    h = np.sinc(t) * (0.5 + 0.5*np.cos(np.pi*t/(taps/2)))
    # DC unity per phase; phase zero retains the original grid exactly.
    for phase in range(factor):
        h[phase::factor] /= h[phase::factor].sum()
    y = upfirdn(h, x, up=factor, axis=0)
    # File boundaries are zero-extended. Include exterior filter ringout, too.
    return ref.db(max(np.max(np.abs(x)),np.max(np.abs(y))))

def signals():
    rng=np.random.default_rng(20260915)
    for n in [1,2,17,1023,1024,4095,4096]:
        for kind in ['dc','alternating','random','start','tail','silence']:
            x={'dc':np.full(n,.5),'alternating':.5*(-1.)**np.arange(n),
                'random':rng.uniform(-1.4,1.4,n),'silence':np.zeros(n)}.get(kind)
            if x is None:
                x=np.zeros(n);x[0 if kind=='start' else -1]=1.4
            yield f'{kind}-{n}',np.stack([x,x[::-1]*-.7],axis=1),None
    for rate in [44100,48000,96000,192000]:
        n=4096
        for ratio in [1/8,1/6,1/4,.35,.4,.45,.48,.49,.499]:
            for phase in [0,15,45,60,67.5,90]:
                for burst in [False,True]:
                    t=np.arange(n)
                    x=1.41*np.sin(2*np.pi*ratio*t+np.deg2rad(phase))
                    if burst: x[:n//2]=0;x[n//2+17:]=0
                    else:
                        fade=min(round(rate*.01),n//4)
                        x[:fade]*=np.linspace(0,1,fade);x[-fade:]*=np.linspace(1,0,fade)
                    yield f'tone-{rate}-{ratio:.6f}-{phase}-{burst}',x[:,None],ratio
    # EBU 20-23 construction: continuous phase, one fs/4 cycle inserted into
    # fs/6 at 4*fs; antialias before each of the four downsample offsets.
    # Synthesized interpretations, not the official downloaded compliance set.
    from scipy.signal import firwin, fftconvolve
    n=16384;t=np.arange(n);steps=np.full(n,2*np.pi/24)
    start=n//2;steps[start:start+16]=2*np.pi/16
    phase=np.cumsum(steps)-steps[0]
    x=.5*np.sin(phase);x[start:start+16]*=2
    x[:1920]*=np.linspace(0,1,1920);x[-1920:]*=np.linspace(1,0,1920)
    x=fftconvolve(x,firwin(1025,0.25,window=('kaiser',12)),mode='same')
    for offset in range(4): yield f'ebu20plus-{offset}',x[offset::4,None],None

def main():
    parser=argparse.ArgumentParser();parser.add_argument('--output',type=Path,required=True)
    args=parser.parse_args();assert not args.output.exists()
    controls=[]
    for n in [31,32,1023,1024]:
        for x in [np.ones(n)*.5,(-1.)**np.arange(n)*.5,np.random.default_rng(n).normal(size=n)]:
            y=ref.interp(x,16)
            assert np.max(np.abs(y[::16]-x))<1e-12
            assert np.max(np.abs(y-resample(x,len(x)*16)))<1e-12
            controls.append(n)
    rows=[];start=time.perf_counter()
    for ident,x,ratio in signals():
        fft=ref.peak_reference(x,factor=32,chunk=16384,overlap=8192)
        row={'id':ident,'frames':len(x),'channels':x.shape[1],'ratio':ratio,'fft32_dbtp':fft,'candidates':[]}
        for taps,factor in [(48,4),(64,8),(128,16),(512,16),(2048,16)]:
            now=time.perf_counter();peak=fir_peak(x,taps,factor)
            row['candidates'].append({'taps':taps,'factor':factor,'peak':peak,'error_db':peak-fft,'seconds':time.perf_counter()-now})
        rows.append(row)
    result={'version':'fir-exploration-v1','reference_controls_passed':len(controls),'seconds':time.perf_counter()-start,'rows':rows}
    args.output.parent.mkdir(parents=True,exist_ok=True)
    args.output.write_text(json.dumps(result,indent=2)+'\n')
    for pair in [(48,4),(64,8),(128,16),(512,16),(2048,16)]:
        vals=[c['error_db'] for r in rows for c in r['candidates'] if (c['taps'],c['factor'])==pair]
        print(pair,'min/max',min(vals),max(vals),'under -.05',sum(v<-.05 for v in vals))

if __name__=='__main__':main()
