from pathlib import Path
import json,re,subprocess
import numpy as np
from scipy.io import wavfile
OUT=Path(__file__).resolve().parent
D=OUT/'mechanisms-v2'
rows=json.loads((D/'native.json').read_text())
results=[]
for r in rows:
    sr,x=wavfile.read(D/r['input']);_,y=wavfile.read(D/r['output']);x=x.astype(float);y=y.astype(float)
    row={k:r[k] for k in ['rate','case','mode','amount','limited','seconds','subnormal_samples']}
    row['native_tp']=r['measure']['true_peak_dbtp'];row['limiter_gr_max']=r['stats']['limiter_gr_max']
    if r['meta']['type']=='tone':
        f=r['meta']['hz'];n=sr;yy=y[sr//2:sr//2+n,0];xx=x[sr//2:sr//2+n,0]
        z=np.fft.rfft(yy);power=abs(z)**2
        fundamental=power[f];harmonics=[k*f for k in range(2,1+sr//(2*f))]
        row['fundamental_gain_db']=float(10*np.log10(fundamental/abs(np.fft.rfft(xx)[f])**2))
        row['harmonic_power_dbc']=float(10*np.log10(max(float(power[harmonics].sum()),1e-30)/fundamental)) if harmonics else None
        keep=np.ones(len(power),bool);keep[0]=False;keep[f]=False;keep[harmonics]=False
        row['nonharmonic_power_dbc']=float(10*np.log10(max(float(power[keep].sum()),1e-30)/fundamental))
        row['peak']=float(abs(yy).max())
        if not r['limited']:
            a=r['amount'];expected=xx if a==0 else np.tanh(xx*(1+2*a))/np.tanh(1+2*a) if a>0 else np.tanh(xx*(-2*a))/(-2*a)
            row['analytic_max_error']=float(abs(yy-expected).max())
    if r['case']=='burst':
        # Steady low-level carrier recovery, compare RMS to matching no-burst gain.
        for begin,end,label in [(0.8,.95,'before'),(1.,1.02,'burst'),(1.025,1.045,'after5ms'),(1.12,1.14,'after100ms'),(1.52,1.54,'after500ms')]:
            sl=slice(int(begin*sr),int(end*sr));row[label+'_gain_db']=float(10*np.log10(np.mean(y[sl,0]**2)/np.mean(x[sl,0]**2)))
        # linked gain must preserve instantaneous L:R on pure limiter test.
        if r['mode']=='limiter_only':row['stereo_ratio_error_max']=float(abs(y[:,0]-2*y[:,1]).max())
    if r['case'] in ['burst','impulse','step','silence','tiny_tail','multitone'] and r['limited']:
        log=subprocess.run(['ffmpeg','-hide_banner','-nostats','-i',str(D/r['output']),'-af','ebur128=peak=true','-f','null','-'],capture_output=True,text=True,check=True).stderr
        (D/(r['output']+'.ffmpeg.txt')).write_text(log)
        match=re.search(r'True peak:\s*Peak:\s*(-?[\d.]+|-inf) dBFS',log);row['ffmpeg_tp']=float(match[1]) if match else None
    row['tail_rms']=float(np.sqrt(np.mean(y[-sr//2:]**2)))
    results.append(row)
(OUT/'mechanism-results.json').write_text(json.dumps(results,indent=2))
for r in results:
    if (r['case']=='tone_11000_0' and r['rate']==44100) or (r['case']=='burst' and r['rate']==48000 and r['limited']):print(r)
print('max analytic error',max(r.get('analytic_max_error',0) for r in results))
print('max limited independent tp',max(r['ffmpeg_tp'] for r in results if r.get('ffmpeg_tp') is not None))
print('subnormals',sum(r['subnormal_samples'] for r in results))
