"""B2 prototype: cropped FFT reconstruction with a between-grid bound.

Provisional specification, frozen for this run: 262144-frame cores, 32768-frame
overlap and zero extension, power-of-two FFT padding, 16x interpolation, original
samples retained. File-edge exterior ringout is included within the overlap.

For EACH periodic FFT polynomial, Bernstein gives ||p''|| <= pi^2 ||p||
in input-sample time. Linear-interpolation remainder on spacing h=1/L is
<= c ||p||, c=pi^2/(8 L^2). Thus ||p|| <= global_grid_max/(1-c), and a
cropped span's maximum <= cropped_grid_max+c*global_grid_max/(1-c).
This bounds between-grid error of that polynomial, NOT error from truncating
the original file to a finite window. The latter remains a qualification limit.
Primary mathematical background: https://arxiv.org/abs/1903.10801
No production adoption or universal physical-reconstruction guarantee.
"""
import argparse
import json
from pathlib import Path
import time
import numpy as np
from qualify_fir import ref, signals
from qualify_fft import finite_sinc_peak


def fft_bound(x, factor=16, core=262144, guard=32768):
    raw=[];bounded=[]
    for channel in x.T:
        sample=float(np.max(np.abs(channel)))
        raw_peak=sample;upper_peak=sample
        if sample==0:
            raw.append(-300.);bounded.append(-300.);continue
        for start in range(0,len(channel),core):
            stop=min(start+core,len(channel))
            length=1 << (stop-start+2*guard-1).bit_length()
            a=max(0,start-guard);b=min(len(channel),stop+guard)
            seg=np.zeros(length);offset=a-start+guard
            seg[offset:offset+b-a]=channel[a:b]
            up=ref.interp(seg,factor)
            lo=0 if start==0 else guard*factor
            hi=(guard+stop-start+(guard if stop==len(channel) else 0))*factor
            grid=float(np.max(np.abs(up[lo:hi])))
            global_grid=float(np.max(np.abs(up)))
            c=np.pi**2/(8*factor**2)
            raw_peak=max(raw_peak,grid)
            upper_peak=max(upper_peak,grid+c*global_grid/(1-c))
        raw.append(ref.db(raw_peak));bounded.append(ref.db(upper_peak))
    return raw,bounded


def main():
    parser=argparse.ArgumentParser();parser.add_argument('--output',type=Path,required=True)
    a=parser.parse_args();assert not a.output.exists()
    rows=[]
    for ident,x,ratio in signals():
        start=time.perf_counter()
        raw,bounded=fft_bound(x)
        reference=ref.peak_reference(x,factor=32,chunk=16384,overlap=8192)
        row={'id':ident,'raw_channels':raw,'bounded_channels':bounded,
             'reference32':reference,'bound_minus_reference':max(bounded)-reference,
             'seconds':time.perf_counter()-start}
        if ratio is None and len(x)<=1024 and not ident.startswith('ebu'):
            row['finite_sinc32']=finite_sinc_peak(x)
            row['bound_minus_finite_sinc']=max(bounded)-row['finite_sinc32']
        rows.append(row)
    for n in [262143,262144,262145,524289]:
        for kind in ['alternating','edge_burst','random']:
            rng=np.random.default_rng(n)
            x=.5*(-1.)**np.arange(n) if kind=='alternating' else rng.uniform(-.5,.5,n)
            if kind=='edge_burst':
                x[:]=0;start=min(n-19,262140);x[start:start+17]=1.4*np.sin(2*np.pi*.499*np.arange(17)+.6)
            x=x[:,None];start=time.perf_counter();raw,bounded=fft_bound(x)
            reference=ref.peak_reference(x,factor=32,chunk=262144,overlap=8192)
            rows.append({'id':f'{kind}-{n}','raw_channels':raw,'bounded_channels':bounded,
                'reference32':reference,'bound_minus_reference':max(bounded)-reference,'seconds':time.perf_counter()-start})
    a.output.write_text(json.dumps({'version':'fft-bound-v1','scope':__doc__,'rows':rows},indent=2)+'\n')
    vals=[r['bound_minus_reference'] for r in rows]
    print('rows',len(rows),'bound-reference min/max',min(vals),max(vals),flush=True)
    direct=[r['bound_minus_finite_sinc'] for r in rows if 'bound_minus_finite_sinc' in r]
    print('bound-direct sinc min/max',min(direct),max(direct),flush=True)
    for r in rows:
        if r['bound_minus_reference'] < -.002 or r.get('bound_minus_finite_sinc',0) < -.002: print('FAIL',r,flush=True)


if __name__=='__main__':main()
