"""Compare finite-cardinal + measured SOXR FIR envelopes on retained inputs.

The finite-cardinal interval is a different reconstruction from periodic FFT
blocks. Every discrepancy remains in this result; none is silently waived.
"""
import argparse
import json
from pathlib import Path
import time
import numpy as np
import soundfile as sf
from scipy.signal import fftconvolve
from qualify_sinc_bound import measure_channel
from qualify_fir import ref


def main():
    p=argparse.ArgumentParser();p.add_argument('--inputs',type=Path,required=True)
    p.add_argument('--kernel',type=Path,required=True);p.add_argument('--output',type=Path,required=True)
    a=p.parse_args();assert not a.output.exists()
    kernel=np.load(a.kernel/'kernel.npy')
    proof=json.loads((a.kernel/'provenance.json').read_text())
    assert ref.sha(a.kernel/'kernel.npy')==proof['kernel_sha256']
    assert max(r['discarded_l1']+r['kernel_l1_error'] for r in proof['rows'])<2e-13
    inputs=json.loads((a.inputs/'inputs.json').read_text())
    old=json.loads((a.inputs/'comparison.json').read_text())['rows']
    result={'version':'hybrid-envelope-experiment-1','kernel_sha256':proof['kernel_sha256'],'rows':[]}
    c=np.pi**2/(8*16**2)
    for entry,control in zip(inputs,old,strict=True):
        x,rate=sf.read(entry['path'],always_2d=True);now=time.perf_counter();channels=[]
        for channel in x.T:
            finite=measure_channel(channel)
            stuffed=np.zeros(len(channel)*16);stuffed[::16]=channel
            convolution=fftconvolve(stuffed,kernel)
            peak=float(np.max(np.abs(convolution)))
            # Recorded kernel tail/shift residual is ~2e-13 L1. Include a larger
            # explicit floating arithmetic allowance, not a empirical dB offset.
            numerical=1e-10*float(np.max(np.abs(channel)))
            soxr_upper=(peak+numerical)/(1-c)
            channels.append({'finite':finite,'soxr_grid':peak,'soxr_upper':soxr_upper,
                'upper':max(finite['continuous_upper'],soxr_upper)})
        upper=[ref.db(c['upper']) for c in channels]
        errors={key:[u-v for u,v in zip(upper,values)] for key,values in control['reference'].items()}
        row={'id':entry['id'],'channels':channels,'upper_dbtp':upper,
             'reference':control['reference'],'upper_minus_reference_db':errors,
             'all_references_pass':all(v>=-1e-5 for values in errors.values() for v in values),
             'seconds':time.perf_counter()-now}
        result['rows'].append(row)
        a.output.write_text(json.dumps(result,indent=2)+'\n')
        if not row['all_references_pass']:print('DISCREPANCY',entry['id'],errors,flush=True)
    print('rows',len(result['rows']),'all reference passes',sum(r['all_references_pass'] for r in result['rows']),flush=True)


if __name__=='__main__':main()
