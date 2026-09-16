"""Synthesize separately identified Tech 3341 cases 15-23 for internal testing.

Not the official distributed EBU test files. Cases 20-23 use zero-crossing
insertion so the amplitude change does not introduce an unintended step. The
earlier exploratory ebu20plus cases are retained; they inserted at nonzero phase.
"""
import argparse
import json
from pathlib import Path
import subprocess
import numpy as np
import soundfile as sf
from scipy.signal import firwin,fftconvolve


def main():
    p=argparse.ArgumentParser();p.add_argument('--output',type=Path,required=True)
    p.add_argument('--binary',type=Path,required=True);p.add_argument('--kernel',type=Path,required=True)
    a=p.parse_args();assert not a.output.exists();a.output.mkdir(parents=True)
    n=4096;t=np.arange(n);fade=np.ones(n)
    fade[:480]=np.linspace(0,1,480);fade[-480:]=np.linspace(1,0,480)
    entries=[]
    for case,ratio,phase,amplitude,expected in [(15,.25,0,.5,-6),(16,.25,45,.5,-6),
            (17,1/6,60,.5,-6),(18,.125,67.5,.5,-6),(19,.25,45,1.41,3)]:
        x=amplitude*np.sin(2*np.pi*ratio*t+np.deg2rad(phase))*fade
        path=a.output/f'synth-{case}.wav';sf.write(path,np.stack([x,x],axis=1),48000,subtype='FLOAT')
        entries.append({'id':f'synth-{case}','path':str(path.resolve()),'expected_dbtp':expected})
    t=np.arange(n*4);start=(len(t)//2//24)*24;stop=start+16
    phase=np.where(t<start,2*np.pi*(t-start)/24,
                   np.where(t<stop,2*np.pi*(t-start)/16,2*np.pi+2*np.pi*(t-stop)/24))
    amplitude=np.where((t>=start)&(t<stop),1.,.5)
    x=amplitude*np.sin(phase)
    x[:1920]*=np.linspace(0,1,1920);x[-1920:]*=np.linspace(1,0,1920)
    assert abs(x[start])<1e-12 and abs(x[stop])<1e-12
    x=fftconvolve(x,firwin(1025,.25,window=('kaiser',12)),mode='same')
    for offset in range(4):
        y=x[offset::4];path=a.output/f'synth-{20+offset}.wav'
        sf.write(path,np.stack([y,y],axis=1),48000,subtype='FLOAT')
        entries.append({'id':f'synth-{20+offset}','path':str(path.resolve()),'expected_dbtp':0})
    manifest=a.output/'inputs.json';manifest.write_text(json.dumps(entries,indent=2)+'\n')
    subprocess.run([str(a.binary.resolve()),str(manifest.resolve()),str((a.output/'native.json').resolve()),str(a.kernel.resolve())],check=True)
    native=json.loads((a.output/'native.json').read_text())['rows'];rows=[]
    denominator=1-np.pi**2/(8*16**2)
    for entry,n in zip(entries,native,strict=True):
        peak=max(max(c['continuous_upper'] for c in n['candidate']),max(n['fir_grid'])/denominator)
        db=float(20*np.log10(peak));expected=entry['expected_dbtp']
        row={'id':entry['id'],'expected_dbtp':expected,'protected_peak_dbtp':db,
             'table_tolerance_pass':expected-.4<=db<=expected+.2}
        rows.append(row);print(row)
    (a.output/'comparison.json').write_text(json.dumps({'scope':__doc__,
        'specification':'https://tech.ebu.ch/docs/tech/tech3341.pdf','rows':rows},indent=2)+'\n')


if __name__=='__main__':main()
