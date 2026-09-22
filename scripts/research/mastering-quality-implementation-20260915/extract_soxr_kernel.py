"""Measure the reviewed SOXR reconstruction kernel for a local FIR comparison.

No audio processing policy change. The retained kernel is experimental numeric
data, with generator/binary/version provenance and independently varied impulse
positions/lengths. Keep finite discarded-tail L1 and alignment errors explicit.
"""
import argparse
import json
from pathlib import Path
import subprocess
import shutil
import numpy as np
import soundfile as sf
from qualify_fir import ref

FACTOR=16
SUPPORT=512


def main():
    p=argparse.ArgumentParser();p.add_argument('--output',type=Path,required=True)
    a=p.parse_args();assert not a.output.exists();a.output.mkdir(parents=True)
    fftmpeg=Path(shutil.which('ffmpeg'))
    version=subprocess.run([str(fftmpeg),'-version'],text=True,capture_output=True,check=True).stdout
    report={'factor':FACTOR,'input_support_each_side':SUPPORT,'ffmpeg_sha256':ref.sha(fftmpeg),
            'ffmpeg_version':version,'filter':'aformat=sample_fmts=dbl,aresample=768000:resampler=soxr:precision=33:osf=dbl',
            'rows':[]}
    reference=None
    for n,position in [(8192,2048),(16385,8193),(32768,16384)]:
        x=np.zeros(n);x[position]=1.
        source=a.output/f'impulse-{n}-{position}.wav';sf.write(source,x,48000,subtype='DOUBLE')
        out=a.output/f'upsampled-{n}-{position}.wav'
        subprocess.run([str(fftmpeg),'-v','error','-i',str(source),'-af',report['filter'],
                        '-c:a','pcm_f64le',str(out)],check=True)
        y,rate=sf.read(out);assert len(y)==n*FACTOR and rate==48000*FACTOR
        lo=(position-SUPPORT)*FACTOR;hi=(position+SUPPORT)*FACTOR+1
        h=y[lo:hi].copy()
        if reference is None:reference=h
        discarded=np.r_[y[:lo],y[hi:]]
        row={'frames':n,'position':position,'peak_index':int(np.argmax(np.abs(y))),
             'discarded_l1':float(np.sum(np.abs(discarded))),
             'kernel_max_error':float(np.max(np.abs(h-reference))),
             'kernel_l1_error':float(np.sum(np.abs(h-reference)))}
        report['rows'].append(row)
    np.save(a.output/'kernel.npy',reference)
    reference.astype('<f8').tofile(a.output/'kernel.f64le')
    report['kernel_sha256']=ref.sha(a.output/'kernel.npy')
    (a.output/'provenance.json').write_text(json.dumps(report,indent=2)+'\n')
    print(json.dumps(report['rows'],indent=2))


if __name__=='__main__':main()
