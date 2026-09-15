"""B2 finite-block FFT alternatives. Exploration only; no meter adoption.

Change block padding/support independently of oversampling and retain every
signed discrepancy against the previously checked reference. Direct sinc sums
provide a separate small-signal reference on selected finite edge cases.
"""
import argparse
import json
import time
import numpy as np
from qualify_fir import ref, signals


def finite_sinc_peak(x, factor=32):
    # Exact cardinal-sinc sum at the declared sample grid; includes two input
    # frames of exterior ringout. No FFT interpolation or windowed FIR here.
    peak=float(np.max(np.abs(x)))
    positions=np.arange(-2*factor,(len(x)+2)*factor)/factor
    for start in range(0,len(positions),256):
        matrix=np.sinc(positions[start:start+256,None]-np.arange(len(x))[None,:])
        peak=max(peak,float(np.max(np.abs(matrix @ x))))
    return ref.db(peak)


def main():
    p=argparse.ArgumentParser();p.add_argument('--output',required=True)
    a=p.parse_args()
    from pathlib import Path
    out=Path(a.output);assert not out.exists()
    rows=[]
    for ident,x,ratio in signals():
        start=time.perf_counter()
        reference=ref.peak_reference(x,factor=32,chunk=16384,overlap=8192)
        row={'id':ident,'reference32':reference,'candidates':[]}
        for factor,overlap in [(16,16384),(32,16384),(32,32768)]:
            before=time.perf_counter()
            value=ref.peak_reference(x,factor=factor,chunk=65536,overlap=overlap)
            row['candidates'].append({'factor':factor,'overlap':overlap,'peak':value,
                'error_db':value-reference,'seconds':time.perf_counter()-before})
        # Finite-sinc controls cover odd/even, DC, impulse, alternating and
        # random (including >0 dBFS); this cost is qualification, not callback work.
        if ratio is None and len(x)<=1024 and not ident.startswith('ebu'):
            row['finite_sinc32']=finite_sinc_peak(x)
        row['seconds']=time.perf_counter()-start;rows.append(row)
    result={'version':'fft-exploration-v1','rows':rows}
    out.write_text(json.dumps(result,indent=2)+'\n')
    for factor,overlap in [(16,16384),(32,16384),(32,32768)]:
        vals=[c['error_db'] for r in rows for c in r['candidates'] if c['factor']==factor and c['overlap']==overlap]
        print(factor,overlap,'min/max',min(vals),max(vals))
    for r in rows:
        if 'finite_sinc32' in r:
            print(r['id'],'sinc',r['finite_sinc32'],'reference',r['reference32'],
                  'candidate',r['candidates'][-1]['peak'])


if __name__=='__main__':main()
