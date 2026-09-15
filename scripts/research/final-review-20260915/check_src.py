"""Recheck retained SRC waveforms against a fixed analytic tone, not a fitted one."""
from pathlib import Path
import hashlib
import json
import numpy as np
import soundfile as sf

ROOT=Path(__file__).resolve().parents[3]
PROBES=ROOT.parent/'yes-master-independent-audit-20260915/audit-output/reconciliation-20260915/results/src-probe'
OUT=ROOT/'test-output/mastering-quality-final-review-20260915/src-focused.json'


def main():
    if OUT.exists(): raise SystemExit('Preserve existing evidence; choose a new OUT.')
    results=[]
    for source,dest,delay in [(48000,44100,955),(96000,44100,514),(44100,48000,1120)]:
        waves={}
        for mode in ['prod','recon','codex']:
            path=PROBES/f'{source}_{dest}_{source*2}_100_{mode}.wav'
            x,sr=sf.read(path,always_2d=True)
            assert sr==dest and x.shape[1]==2
            if mode!='prod': assert len(x)==dest*2
            offset=.5 if (source,dest) in [(48000,44100),(96000,44100)] else 0
            # Positive delay means y[n] follows the source at n - delay.
            expected=.5*np.sin(2*np.pi*100*(np.arange(len(x))-offset)/dest)
            # Keep the faulty sample even where a conventional edge guard excludes it.
            interior=np.arange(round(sr*.02),len(x)-round(sr*.02))
            error=x[:,0]-expected
            row={'source':source,'dest':dest,'mode':mode,'frames':len(x),'expected_frames':dest*2,
                 'file':path.name,'sha256':hashlib.sha256(path.read_bytes()).hexdigest(),
                 'fixed_reference_offset_samples':offset,
                 'error_at_delay':float(error[delay]),
                 'interior_rms_error':float(np.sqrt(np.mean(error[interior]**2))),
                 'interior_max_error':float(abs(error[interior]).max())}
            if mode!='prod': assert abs(row['error_at_delay'])<1e-5
            waves[mode]=x
            results.append(row)
        assert np.max(np.abs(waves['recon']-waves['codex']))<1e-6
        if source in [48000,96000]: assert abs(results[-3]['error_at_delay'])>.1
    OUT.write_text(json.dumps(results,indent=2)+'\n')
    print(json.dumps(results,indent=2))


if __name__=='__main__':main()
