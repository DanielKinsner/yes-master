"""Keep direct historical references and separately check finite zero extension.

This is a new qualification record, never a rewrite of frozen results. SOXR
gets 2048 zero input frames at each edge so every internal stage reaches the
declared zero-extended response before program material. Factors 16/32/64 remain
separate measurements. The candidate must satisfy the stated envelope tests.
"""
import argparse
import json
from pathlib import Path
import re
import subprocess
import numpy as np
import soundfile as sf
from qualify_fir import ref
from qualify_fft import finite_sinc_peak


def soxr(path,rate,factor,channels):
    filt=f'aformat=sample_fmts=dbl,aresample={rate*factor}:resampler=soxr:precision=33:osf=dbl,astats=metadata=0:reset=0'
    run=subprocess.run(['ffmpeg','-hide_banner','-nostats','-i',str(path),'-af',filt,
        '-c:a','pcm_f64le','-f','null','-'],capture_output=True,text=True,check=True)
    values=[max(-300.,float(v)) for v in re.findall(r'Peak level dB: (-inf|[-\d.]+)',run.stderr)]
    assert len(values)==channels+1
    return values[:-1]


def main():
    p=argparse.ArgumentParser();p.add_argument('--native',type=Path,required=True)
    p.add_argument('--inputs',type=Path,required=True);p.add_argument('--historical',type=Path)
    p.add_argument('--output',type=Path,required=True)
    a=p.parse_args();assert not a.output.exists();a.output.mkdir(parents=True)
    native=json.loads(a.native.read_text());entries=json.loads(a.inputs.read_text())
    old={r['id']:r for r in json.loads(a.historical.read_text())['rows']} if a.historical else {}
    result={'version':'finite-zero-extension-qualification-1','native_version':native['version'],
            'native_sha256':ref.sha(a.native),'rows':[]}
    denominator=1-np.pi**2/(8*16**2)
    for entry,n in zip(entries,native['rows'],strict=True):
        assert entry['id']==n['id']
        path=Path(entry['path']);assert ref.sha(path)==n['source_sha256']
        x,rate=sf.read(path,always_2d=True)
        padded=a.output/(entry['id']+'-padded.wav')
        sf.write(padded,np.pad(x,((2048,2048),(0,0))),rate,subtype='DOUBLE')
        references={f'soxr{factor}':soxr(padded,rate,factor,x.shape[1]) for factor in [16,32,64]}
        if len(x)<=1024:references['direct_sinc32']=[finite_sinc_peak(x[:,c:c+1]) for c in range(x.shape[1])]
        finite=[c['continuous_upper'] for c in n['candidate']]
        # This lowpass-model grid allowance must itself pass higher-factor checks;
        # it is not a claim that any oversampling factor guarantees every DAC peak.
        fir=[(v+1e-10*c['sample_peak'])/denominator for v,c in zip(n['fir_grid'],n['candidate'],strict=True)]
        upper=[ref.db(max(f,g)) for f,g in zip(finite,fir,strict=True)]
        errors={name:[u-v for u,v in zip(upper,values,strict=True)] for name,values in references.items()}
        width=[ref.db(c['continuous_upper'])-ref.db(c['grid_lower']) for c in n['candidate']]
        row={'id':entry['id'],'source_sha256':n['source_sha256'],'channels':n['candidate'],
             'fir_grid':n['fir_grid'],'protected_upper_dbtp':upper,'finite_width_db':width,
             'zero_extended_references':references,'upper_minus_reference_db':errors,
             'reference_pass':all(e>=-1e-5 for values in errors.values() for e in values),
             'width_pass':max(width)<=.0501,
             'historical_direct_reference':old.get(entry['id'],{}).get('reference')}
        result['rows'].append(row)
        (a.output/'comparison.json').write_text(json.dumps(result,indent=2)+'\n')
        if not row['reference_pass'] or not row['width_pass']:print('FAIL',entry['id'],errors,width,flush=True)
    print('rows',len(result['rows']),'reference passes',sum(r['reference_pass'] for r in result['rows']),
          'width passes',sum(r['width_pass'] for r in result['rows']),flush=True)


if __name__=='__main__':main()
