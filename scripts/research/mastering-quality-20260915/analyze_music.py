"""Independent delivery and shape measurements; no perceptual score."""
from pathlib import Path
import concurrent.futures,hashlib,json,math,re,subprocess,sys
import numpy as np
from scipy.io import wavfile
from scipy.signal import welch,resample_poly,find_peaks
OUT=Path(__file__).resolve().parent
BANDS=[(20,60),(60,120),(120,250),(250,500),(500,1000),(1000,2000),(2000,4000),(4000,8000),(8000,16000)]
def db(v):return float(10*np.log10(max(float(v),1e-30)))
def load(path):
    sr,x=wavfile.read(path);x=x.astype(float)/(32768 if x.dtype==np.int16 else 2147483648 if x.dtype==np.int32 else 1)
    if x.ndim==1:x=np.stack([x,x],axis=1)
    assert np.isfinite(x).all()
    return sr,x
def metric(path,key,anchors=None):
    folder=OUT/'independent';folder.mkdir(exist_ok=True);saved=folder/(key+'.json')
    if saved.exists():return json.loads(saved.read_text())
    sr,x=load(path);original_frames=len(x)
    h=hashlib.file_digest(path.open('rb'),'sha256').hexdigest()
    ff=subprocess.run(['ffmpeg','-hide_banner','-nostats','-i',str(path),'-af','ebur128=peak=true','-f','null','-'],capture_output=True,text=True,check=True).stderr
    (folder/(key+'.ffmpeg.txt')).write_text(ff)
    summary=ff.rsplit('Summary:',1)[-1]
    def get(p):return float(re.search(p,summary)[1])
    lufs=get(r'I:\s*([-\d.]+) LUFS');tp=get(r'Peak:\s*([-\d.]+) dBFS');lra=get(r'LRA:\s*([-\d.]+) LU')
    short=[]
    for line in ff.splitlines():
        m=re.search(r't:\s*([\d.]+).*?M:\s*([-\d.]+)\s+S:\s*([-\d.]+)',line)
        if m and float(m[1])>=3:short.append([float(m[1]),float(m[3])])
    short=np.array(short);active=short[short[:,1]>-40,1]
    row=dict(key=key,path=str(path),sha256=h,lufs=lufs,tp=tp,lra=lra,rate=sr,frames=original_frames,duration=original_frames/sr,channels=x.shape[1],peak=db(abs(x).max()**2),rms=db(np.mean(x*x)),fullscale=int(np.sum(abs(x)>=1)),shortterm_p95_p10=float(np.percentile(active,95)-np.percentile(active,10)))
    row['crest']=row['peak']-row['rms']
    if sr!=48000:g=math.gcd(sr,48000);x=resample_poly(x,48000//g,sr//g,axis=0)
    sr=48000;mid=x.mean(axis=1);side=(x[:,0]-x[:,1])/2
    row['side_mid_db']=db(np.mean(side**2)/max(np.mean(mid**2),1e-30));row['correlation']=float(np.corrcoef(x.T)[0,1]);row['spectral_rms']=db(np.mean(x*x))
    f,px=welch(x,fs=sr,nperseg=8192,noverlap=4096,axis=0);px=px.mean(axis=1);df=f[1]-f[0]
    row['bands_db']=[db(px[(f>=lo)&(f<hi)].sum()*df) for lo,hi in BANDS]
    _,pm=welch(mid,fs=sr,nperseg=8192,noverlap=4096);_,ps=welch(side,fs=sr,nperseg=8192,noverlap=4096)
    row['band_side_mid_db']=[db(ps[(f>=lo)&(f<hi)].sum()/max(pm[(f>=lo)&(f<hi)].sum(),1e-30)) for lo,hi in BANDS]
    chunks=x[:len(x)//19200*19200].reshape(-1,19200,2);rp=np.mean(chunks**2,axis=(1,2));pk=abs(chunks).max(axis=(1,2));crest=20*np.log10(np.maximum(pk,1e-30))-10*np.log10(np.maximum(rp,1e-30));mask=10*np.log10(np.maximum(rp,1e-30))>-40
    row['crest400_median']=float(np.median(crest[mask]));row['rms400_spread']=float(10*np.log10(np.percentile(rp[mask],95)/np.percentile(rp[mask],10)))
    # Fixed 10-second sections, preserve trace for explicit source anchored contrasts.
    sections=x[:len(x)//480000*480000].reshape(-1,480000,2);row['sections10_rms_db']=[db(v) for v in np.mean(sections**2,axis=(1,2))]
    if anchors is None:
        block=x[:len(x)//480*480].reshape(-1,480,2);env=np.sqrt(np.mean(block**2,axis=(1,2)));novelty=np.maximum(np.diff(env,prepend=env[0]),0)
        peaks,_=find_peaks(novelty,distance=20);peaks=sorted(peaks,key=lambda k:novelty[k],reverse=True)[:40];anchors=sorted([float(k*.01) for k in peaks if k*.01>.05 and k*.01<len(x)/sr-.2])
    ratios=[]
    for t in anchors:
        a=x[int((t-.02)*sr):int((t+.03)*sr)];b=x[int((t+.03)*sr):int((t+.15)*sr)]
        if len(a) and len(b):ratios.append(db(abs(a).max()**2/ max(np.mean(b*b),1e-30)))
    row['transient_times']=anchors;row['transient_attack_body_db']=ratios;row['transient_median']=float(np.median(ratios)) if ratios else None
    row['shortterm_trace']=short.tolist()
    saved.write_text(json.dumps(row,indent=2));return row
def analyze_source(s):
    p=Path(s['path']);p=p if p.is_absolute() else OUT/p
    source=metric(p,s['id']+'_source');rows=[source]
    for p in sorted((OUT/'finalists'/s['id']).glob('*.wav')):
        r=metric(p,s['id']+'_'+p.stem,source['transient_times']);r['source_id']=s['id'];r['name']=p.stem
        native=json.loads(p.with_suffix('.json').read_text());r['variant']=native['variant'];r['native']=native['delivered_native'];r['stats']=native['stats'];r['pre_landing']=native['pre_landing']
        assert r['frames']==round(source['duration']*48000) or abs(r['frames']-source['duration']*48000)<=1.01
        assert abs(r['lufs']-r['native'][0])<.11,(r['key'],r['lufs'],r['native'])
        # Keep the predeclared criterion; retain failing results for the report.
        r['ceiling_check_pass']=r['tp']<=-.9 and r['fullscale']==0
        if not r['ceiling_check_pass']:print('CEILING FAILURE',r['key'],r['tp'],r['native'],flush=True)
        assert abs(r['crest']-r['pre_landing']['crest_db'])<1e-4
        r['target_error']=r['variant']['target']-r['lufs']
        r['band_change_from_source']=[a-b-(r['spectral_rms']-source['spectral_rms']) for a,b in zip(r['bands_db'],source['bands_db'])]
        active=[i for i,x in enumerate(source['sections10_rms_db']) if x>-40]
        lo=min(active,key=lambda i:source['sections10_rms_db'][i]);hi=max(active,key=lambda i:source['sections10_rms_db'][i]);r['section_contrast_db']=r['sections10_rms_db'][hi]-r['sections10_rms_db'][lo];r['source_section_contrast_db']=source['sections10_rms_db'][hi]-source['sections10_rms_db'][lo];r['section_indices']=[lo,hi]
        rows.append(r)
    (OUT/'independent'/f"{s['id']}-summary.json").write_text(json.dumps(rows,indent=2));print('measured',s['id'],len(rows),flush=True)
if __name__=='__main__':
    corpus=json.loads((OUT/'corpus.json').read_text())['sources'];split=sys.argv[1]
    with concurrent.futures.ThreadPoolExecutor(max_workers=2) as pool:
        for r in pool.map(analyze_source,[s for s in corpus if s['split']==split]):pass
