"""Independent B2 qualification driver for the standalone native experiment.

Retains float source files, exact executable hash, source hashes, native/library
results, and independent Python FFT/SOXR/direct-sum checks. No production caller.
The synthetic interpretations of EBU cases do not establish official compliance.
"""
import argparse
import json
from pathlib import Path
import shutil
import subprocess
import time
import numpy as np
import soundfile as sf
from qualify_fir import signals, ref
from qualify_fft import finite_sinc_peak
from check_candidates import channel_peaks


def main():
    p=argparse.ArgumentParser()
    p.add_argument('--binary',type=Path,required=True)
    p.add_argument('--output',type=Path,required=True)
    p.add_argument('--music',type=Path,nargs='*')
    a=p.parse_args();assert not a.output.exists();a.output.mkdir(parents=True)
    binary=a.output/'peak-probe.exe';shutil.copy2(a.binary,binary)
    entries=[]
    if a.music:
        for i,path in enumerate(a.music):entries.append({'id':f'music-{i}-{path.stem}','path':str(path.resolve())})
    else:
        for ident,x,ratio in signals():
            rate=int(ident.split('-')[1]) if ident.startswith('tone-') else 48000
            path=a.output/(ident+'.wav')
            sf.write(path,x.astype(np.float32),rate,subtype='FLOAT')
            entries.append({'id':ident,'path':str(path.resolve())})
        for n in [262143,262144,262145]:
            path=a.output/f'long-alternating-{n}.wav'
            sf.write(path,(.5*(-1.)**np.arange(n)).astype(np.float32),48000,subtype='FLOAT')
            entries.append({'id':path.stem,'path':str(path.resolve())})
    manifest=a.output/'inputs.json';manifest.write_text(json.dumps(entries,indent=2)+'\n')
    with (a.output/'native.log').open('w') as log:
        subprocess.run([str(binary.resolve()),str(manifest.resolve()),str((a.output/'native.json').resolve())],
                       stdout=log,stderr=subprocess.STDOUT,check=True)
    native=json.loads((a.output/'native.json').read_text())['rows']
    result={'version':'b2-native-qualification-1','binary_sha256':ref.sha(binary),'rows':[]}
    for entry,row in zip(entries,native,strict=True):
        now=time.perf_counter();path=Path(entry['path'])
        assert entry['id']==row['id'] and ref.sha(path)==row['source_sha256']
        x,rate=sf.read(path,always_2d=True)
        checked=channel_peaks(x,path,rate)
        upper=[ref.db(c['continuous_upper']) for c in row['candidate']]
        lower=[ref.db(c['grid_lower']) for c in row['candidate']]
        if len(x)<=1024:
            checked['direct_sinc32_dbtp']=[finite_sinc_peak(x[:,i:i+1]) for i in range(x.shape[1])]
        # Invariance is checked on the reviewed stateful library as well; explicit
        # finite flushing is kept separate from current application behavior.
        library=row['ebur128_0_1_10']
        for l in library[1:]:
            assert max(abs(v-w) for v,w in zip(l['flushed'],library[0]['flushed']))<1e-12
        errors={name:[u-v for u,v in zip(upper,values,strict=True)] for name,values in checked.items()}
        result['rows'].append({'id':entry['id'],'native':row,'reference':checked,
            'upper_dbtp':upper,'lower_dbtp':lower,'upper_minus_reference_db':errors,
            'all_reference_bounds_pass':all(v>=-1e-5 for vals in errors.values() for v in vals),
            'reference_seconds':time.perf_counter()-now})
        (a.output/'comparison.json').write_text(json.dumps(result,indent=2)+'\n')
        print(entry['id'],'upper',upper,'width',[u-l for u,l in zip(upper,lower)],
              'pass',result['rows'][-1]['all_reference_bounds_pass'],flush=True)


if __name__=='__main__':main()
