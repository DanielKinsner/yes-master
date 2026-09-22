"""Compare independent SRC peak regrowth; do not treat filter residual as distortion."""
from pathlib import Path
import json,subprocess
import numpy as np
from scipy.io import wavfile
from scipy.signal import resample_poly
P=Path(__file__).resolve().parent;D=P/'peak-probe'
for stem in ['funk_control','funk_drive_12','metal_control']:
    output=D/f'{stem}_independent_src.json'
    assert not output.exists(), ('Use a fresh experiment directory',output)
    source=D/f'{stem}_before_src.wav';sr,x=wavfile.read(source)
    _,actual=wavfile.read(D/f'{stem}_after_src.wav')
    poly=resample_poly(x.astype(float),160,147,axis=0)
    data=subprocess.check_output(['ffmpeg','-v','error','-i',str(source),'-af','aresample=48000:resampler=soxr:precision=33','-f','f32le','pipe:1'])
    soxr=np.frombuffer(data,dtype='<f4').reshape(-1,2)
    result={}
    for name,y in [('rubato',actual),('polyphase',poly),('soxr',soxr)]:
        index=np.argmax(np.abs(y));result[name]=dict(peak=float(20*np.log10(np.max(np.abs(y)))),time=float(index//2/48000))
    n=min(len(soxr),len(actual));result['rms_error']=float(np.sqrt(np.mean((actual[:n].astype(float)-soxr[:n])**2)))
    output.write_text(json.dumps(result,indent=2));print(stem,result)
