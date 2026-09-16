"""Targeted independent checks of newly exposed finite-reference scope limits.

Never replaces the frozen references. Compares direct SOXR with zero padding,
the measured interior impulse response, and direct finite alternating sums.
"""
import argparse
import json
from pathlib import Path
import subprocess
import numpy as np
import soundfile as sf
from scipy.signal import fftconvolve


def main():
    p=argparse.ArgumentParser();p.add_argument('--inputs',type=Path,required=True)
    p.add_argument('--kernel',type=Path,required=True);p.add_argument('--output',type=Path,required=True)
    a=p.parse_args();assert not a.output.exists();a.output.mkdir(parents=True)
    h=np.load(a.kernel/'kernel.npy');radius=(len(h)-1)//2
    rows=[]
    for ident in ['dc-1','dc-17','random-2','start-4096','tail-4096','tone-44100-0.450000-45-True']:
        x,rate=sf.read(a.inputs/(ident+'.wav'),always_2d=True)
        padded=np.pad(x,((2048,2048),(0,0)))
        src=a.output/(ident+'-padded.wav');out=a.output/(ident+'-soxr.wav')
        sf.write(src,padded,rate,subtype='DOUBLE')
        subprocess.run(['ffmpeg','-v','error','-i',str(src),'-af',
            f'aformat=sample_fmts=dbl,aresample={rate*16}:resampler=soxr:precision=33:osf=dbl',
            '-c:a','pcm_f64le',str(out)],check=True)
        y,_=sf.read(out,always_2d=True);errors=[];peaks=[]
        for c in range(x.shape[1]):
            z=np.zeros(len(padded)*16);z[::16]=padded[:,c]
            expected=fftconvolve(z,h)[radius:radius+len(z)]
            errors.append(float(np.max(np.abs(y[:,c]-expected))))
            peaks.append(float(np.max(np.abs(y[:,c]))))
        rows.append({'id':ident,'padded_soxr_channels':peaks,'max_sample_error_vs_kernel':errors})
    alternating=[]
    # Separate O(N*positions) reciprocal sums, no FFT, window, or moment tree.
    # For alternating x[n]=.5*(-1)^n, y[n]=.5 and sinc sum is
    # .5*sin(pi*t)/pi * sum(n=0..N-1) 1/(t-n).
    positions=np.arange(-1024,1025)/1024
    positions=positions[positions!=np.floor(positions)]
    for n in [262143,262144,262145]:
        indices=np.arange(n);peak=.5;where=None
        for t in positions:
            v=abs(.5*np.sin(np.pi*t)/np.pi*np.sum(1/(t-indices)))
            if v>peak:peak=float(v);where=float(t)
        alternating.append({'frames':n,'peak_near_left_edge':peak,'position':where,
                            'peak_db':float(20*np.log10(peak))})
    (a.output/'comparison.json').write_text(json.dumps({'padded_soxr':rows,'direct_alternating':alternating},indent=2)+'\n')
    print(json.dumps({'padded_soxr':rows,'direct_alternating':alternating},indent=2))


if __name__=='__main__':main()
