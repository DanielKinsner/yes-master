"""Finite cardinal-sinc reconstruction with an explicit omitted-tail bound.

Development experiment, not a qualified/production meter. Unlike a periodic
FFT interpolator, local FFT convolution here evaluates a finite sinc SUM exactly
on the selected grid (to floating precision). Summation by parts bounds omitted
tails using extrema of alternating-sample prefix sums. Outside the examined
domain, L1/(pi*distance) bounds the entire finite signal below its sample peak.

For finite cardinal-sinc f, Bernstein gives ||f''||inf <= pi^2 ||f||inf.
Linear interpolation on spacing 1/L then bounds the continuous supremum by
G/(1-pi^2/(8*L^2)), where G bounds ALL grid samples, including exterior tails.
Primary background: https://d-nb.info/1158599315/34 (Bernstein inequalities).
Floating-point numerical error is recorded separately; this experiment is not
a formal interval-arithmetic proof. No fixed empirical dB safety margin.
"""
import argparse
from functools import lru_cache
import json
from pathlib import Path
import time
import numpy as np
from scipy.fft import rfft, irfft, next_fast_len
from qualify_fir import signals, ref
from qualify_fft import finite_sinc_peak


@lru_cache(maxsize=8)
def kernels(size, factor):
    length = next_fast_len(2*size-1)
    # Circular kernel indices represent every possible input/output difference;
    # padding to >= 2M-1 prevents aliasing into any of the M requested outputs.
    indices = np.arange(-(size-1), size)
    result = []
    for phase in range(1, factor):
        h = np.zeros(length)
        h[indices % length] = np.sinc(indices+phase/factor)
        result.append(rfft(h))
    return length, result


def local_grid(x, start, stop, guard, factor, far=None):
    lo = start-guard
    size = stop-start+2*guard
    segment = np.zeros(size)
    a, b = min(len(x),max(0,lo)), max(0,min(len(x),stop+guard))
    if b > a:
        segment[a-lo:b-lo] = x[a:b]
    length, filters = kernels(size, factor)
    spectrum = rfft(segment, n=length)
    peaks = [float(np.max(np.abs(segment[guard:guard+stop-start])))]
    for filt in filters:
        y = irfft(spectrum*filt, n=length)
        values=y[guard:guard+stop-start]
        if far is not None:
            phase=len(peaks)/factor
            indices=np.arange(start,stop)
            v=(indices+phase-(start+(stop-start)/2))/((stop-start)/2)
            values=values+np.sin(np.pi*phase)*(-1.)**indices/np.pi*np.polynomial.polynomial.polyval(v,far)
        peaks.append(float(np.max(np.abs(values))))
    return max(peaks), a, b


class PrefixExtrema:
    """One extrema pair per fixed input block; no full-file prefix array."""
    def __init__(self, x, block):
        self.block = block
        self.frames = len(x)
        self.minima, self.maxima, self.ends = [], [], [0.]
        offset = 0.
        compensation = 0.
        for start in range(0,len(x),block):
            chunk = x[start:start+block]
            p = np.r_[0.,np.cumsum(chunk*(-1.)**np.arange(start,start+len(chunk)))]
            self.minima.append(offset+float(np.min(p)))
            self.maxima.append(offset+float(np.max(p)))
            increment = float(p[-1])-compensation
            total = offset+increment
            compensation = (total-offset)-increment
            offset = total
            self.ends.append(offset)
        self.minima,self.maxima,self.ends = map(np.array,[self.minima,self.maxima,self.ends])

    @property
    def bytes(self):
        return self.minima.nbytes+self.maxima.nbytes+self.ends.nbytes

    def bound(self, a, b, nearest, left):
        if b <= a:
            return 0.
        assert a % self.block == 0 and (b % self.block == 0 or b == self.frames)
        lo,hi = a//self.block,(b+self.block-1)//self.block
        anchor = self.ends[hi if left else lo]
        amplitude = max(abs(float(np.min(self.minima[lo:hi]))-anchor),
                        abs(float(np.max(self.maxima[lo:hi]))-anchor))
        distance = nearest-(b-1) if left else a-nearest
        assert distance > 0
        return amplitude/distance/np.pi

    def tail(self, a, b, nearest, left, width):
        # Outward dyadic spans retain cancellation within each span. Sum of their
        # bounds is also a bound; compare with the whole-span bound explicitly.
        whole = self.bound(a,b,nearest,left)
        parts = 0.
        while b > a:
            if left:
                c = max(a,((b-width)//self.block)*self.block)
                parts += self.bound(c,b,nearest,True)
                b = c
            else:
                c = min(b,((a+width)//self.block)*self.block)
                parts += self.bound(a,c,nearest,False)
                a = c
            width *= 2
        return min(whole,parts)


def measure_channel(x, factor=16, core=4096, guard=4096):
    sample = float(np.max(np.abs(x), initial=0))
    if sample == 0:
        return {'grid_estimate':0., 'grid_lower':0., 'continuous_upper':0., 'blocks':0}
    assert guard % core == 0
    prefixes = PrefixExtrema(x,core)
    l1 = float(np.sum(np.abs(x)))
    extent = int(np.ceil(l1/(np.pi*sample))) + 1
    first = -(int(np.ceil(extent/core))*core)
    last = int(np.ceil((len(x)+extent)/core))*core
    lower, upper, estimate = sample, sample, sample
    worst = None
    blocks = 0
    records=[]
    for start in range(first,last,core):
        stop = start+core
        value,a,b = local_grid(x,start,stop,guard,factor)
        # Full phase-dependent sin(pi*t) is bounded by 1 here. Distances use
        # closest evaluation position in the complete core interval.
        error = prefixes.tail(0,a,start,True,guard) if a > 0 else 0.
        error += prefixes.tail(b,len(x),stop-1/factor,False,guard) if b < len(x) else 0.
        estimate = max(estimate,value)
        lower = max(lower,value-error)
        if value+error > upper:
            upper = value+error
            worst = {'start':start,'local_peak':value,'tail_bound':error}
        blocks += 1
        records.append((start,value,error,a,b))
    refined=0
    if upper/(1-np.pi**2/(8*factor**2)) > lower*10**(.05/20):
        from sinc_tail_moments import tree,far_polynomial
        root=tree(x,core)
        bounds=[]
        for start,value,error,a,b in records:
            if (value+error)/(1-np.pi**2/(8*factor**2)) > lower*10**(.05/20):
                poly,remainder=far_polynomial(root,a,b,start,core)
                value,_,_=local_grid(x,start,start+core,guard,factor,poly)
                error=remainder/np.pi
                lower=max(lower,value-error)
                refined+=1
            bounds.append(value+error)
        upper=max(sample,max(bounds))
    return {'grid_estimate':estimate,'grid_lower':lower,
        'continuous_upper':upper/(1-np.pi**2/(8*factor**2)),
        'blocks':blocks,'worst':worst,'exterior_extent':extent,
        'refined_blocks':refined,
        'prefix_bytes':prefixes.bytes}


def main():
    parser=argparse.ArgumentParser()
    parser.add_argument('--output',type=Path,required=True)
    parser.add_argument('--music',type=Path)
    parser.add_argument('--max-seconds',type=float,default=10)
    args=parser.parse_args();assert not args.output.exists()
    cases = list(signals()) if args.music is None else []
    if args.music:
        import soundfile as sf
        info=sf.info(args.music)
        x,_=sf.read(args.music,frames=int(args.max_seconds*info.samplerate),always_2d=True)
        cases=[('music-excerpt',x,None)]
    else:
        for n in [262143,262144,262145]:
            cases.append((f'long-alternating-{n}',(.5*(-1.)**np.arange(n))[:,None],None))
    rows=[]
    for ident,x,ratio in cases:
        now=time.perf_counter()
        channels=[measure_channel(c) for c in x.T]
        row={'id':ident,'channels':channels,'seconds':time.perf_counter()-now,
             'upper_dbtp':ref.db(max(c['continuous_upper'] for c in channels)),
             'lower_dbtp':ref.db(max(c['grid_lower'] for c in channels)),
             'legacy_fft32':ref.peak_reference(x,factor=32)}
        if len(x)<=1024 and ratio is None:
            row['direct_sinc32']=finite_sinc_peak(x)
        rows.append(row)
        args.output.write_text(json.dumps({'version':'finite-sinc-bound-v1','scope':__doc__,'rows':rows},indent=2)+'\n')
        print(ident,row['lower_dbtp'],row['upper_dbtp'],'legacy',row['legacy_fft32'],row['seconds'],flush=True)


if __name__=='__main__':main()
