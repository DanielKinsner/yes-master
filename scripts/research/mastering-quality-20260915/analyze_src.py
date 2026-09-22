"""Reproduce isolated SRC measurements; preserve an existing analysis file."""
from pathlib import Path
import argparse,json
import numpy as np
from scipy.io import wavfile
P=Path(__file__).resolve().parent
parser=argparse.ArgumentParser()
parser.add_argument('--input',type=Path,default=P/'src-check')
parser.add_argument('--output',type=Path)
args=parser.parse_args();out=args.output or args.input/'analysis.json'
assert not out.exists(), ('Use a new output path',out)
rows=json.loads((args.input/'native.json').read_text())
for r in rows:
    sr,x=wavfile.read(args.input/r['file']);x=x[:,0].astype(float)
    # The Rust generator rounds positive half-frame ties away from zero.
    frames=int(np.floor(r['from']*r['duration']+.5))
    r['expected']=(frames*r['to']+r['from']-1)//r['from']
    r['length_pass']=len(x)==r['expected']
    assert np.isfinite(x).all()
    if r['duration']>=.1:
        # Ignore 20 ms at either end; retain the demonstrated frame-955 error.
        t=np.arange(len(x))/sr;keep=slice(int(.02*sr),-int(.02*sr))
        a=np.column_stack([np.sin(2*np.pi*r['hz']*t[keep]),np.cos(2*np.pi*r['hz']*t[keep])])
        fit=a@np.linalg.lstsq(a,x[keep],rcond=None)[0];residual=x[keep]-fit
        r['residual_dbc']=float(10*np.log10(np.mean(residual**2)/np.mean(fit**2)))
        r['max_error']=float(np.max(np.abs(residual)))
    else:
        ref=.5*np.sin(2*np.pi*r['hz']*np.arange(r['expected'])/sr)
        r['rms']=float(np.sqrt(np.mean(x*x)))
        r['reference_rms']=float(np.sqrt(np.mean(ref*ref)))
    if r['mode']=='fixed':assert r['length_pass'],r
out.write_text(json.dumps(rows,indent=2))
print('SRC checks',len(rows),'fixed exact lengths',sum(r['mode']=='fixed' and r['length_pass'] for r in rows))
