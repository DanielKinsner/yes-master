"""Analytical waveshaping reference, not a production oversampler candidate."""
from pathlib import Path
import argparse,json
import numpy as np
from scipy.signal import resample_poly
P=Path(__file__).resolve().parent
parser=argparse.ArgumentParser();parser.add_argument('--output',type=Path,default=P/'oversampling-reference.json');args=parser.parse_args()
assert not args.output.exists(), ('Use a new output path',args.output)
rows=[];drive=1+2*.0715
for sr in [44100,48000,96000]:
    for hz in [1000,7000,11000]:
        for level in [-12,-6,0]:
            x=(10**(level/20)*np.sin(2*np.pi*hz*np.arange(sr*2)/sr)).astype(np.float32).astype(float)
            for factor in [1,4,8]:
                up=x if factor==1 else resample_poly(x,factor,1)
                y=np.tanh(up*drive)/np.tanh(drive)
                if factor!=1:y=resample_poly(y,1,factor)
                # Integer-second rectangular FFT; coherent tones, no window leakage.
                power=abs(np.fft.rfft(y[sr//2:sr//2+sr]))**2
                keep=np.ones(len(power),bool);keep[0]=False;keep[hz::hz]=False
                rows.append(dict(sr=sr,hz=hz,level=level,factor=factor,nonharmonic_dbc=float(10*np.log10(power[keep].sum()/power[hz]))))
args.output.write_text(json.dumps(rows,indent=2));print('Analytical references',len(rows))
