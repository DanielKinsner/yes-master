"""Authorized CC BY downloads; preserve originals; construct labelled test mixes."""
from pathlib import Path
import concurrent.futures, datetime, hashlib, json, subprocess, urllib.request, zipfile,sys,io
import numpy as np
from scipy.io import wavfile

OUT=Path(__file__).resolve().parent
def sha(p):
    with p.open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest()
def fetch(url,p):
    expected=None
    expected_sha=None
    frozen=OUT/'expected-corpus.json'
    if frozen.exists():
        for s in json.loads(frozen.read_text(encoding='utf8'))['sources']:
            for item in s.get('originals',[]):
                if item['url']==url:expected=item['bytes'];expected_sha=item['sha256']
    for meta in (OUT/'pages').glob('*-downloads.json'):
        for f in json.loads(meta.read_text())[0]['files']:
            if f['download_url']==url:expected=f['file_rawsize']
    if p.exists() and (expected is None or p.stat().st_size==expected):
        if expected_sha:assert sha(p)==expected_sha,(p,'original changed')
        return
    subprocess.run(['curl.exe','-L','--fail','--silent','--show-error','--max-time','300','--retry','2','-C','-','--referer','https://ccmixter.org/', '-o',str(p),url],check=True)
    if expected is not None:assert p.stat().st_size==expected
    if expected_sha:assert sha(p)==expected_sha,(p,'download differs from frozen original')
    print('downloaded',p.name,p.stat().st_size,flush=True)
def decode(p,rate=None):
    info=json.loads(subprocess.check_output(['ffprobe','-v','error','-show_format','-show_streams','-of','json',str(p)]))
    sr=rate or int(info['streams'][0]['sample_rate'])
    b=subprocess.check_output(['ffmpeg','-v','error','-i',str(p),'-ar',str(sr),'-ac','2','-f','f32le','pipe:1'])
    return sr,np.frombuffer(b,dtype='<f4').reshape(-1,2).copy(),info

def main():
    specs=[
      dict(id='piano',title='In This Moment',artist='Scott Buckley',split='development',kind='finished_mix',license='https://creativecommons.org/licenses/by/4.0/',page='https://www.scottbuckley.com.au/library/in-this-moment/',urls=['https://www.scottbuckley.com.au/library/wp-content/uploads/2026/08/InThisMoment.mp3']),
      dict(id='aphelion',title='Aphelion',artist='Scott Buckley',split='holdout',kind='finished_mix',license='https://creativecommons.org/licenses/by/4.0/',page='https://www.scottbuckley.com.au/library/aphelion/',urls=['https://www.scottbuckley.com.au/library/wp-content/uploads/2026/04/Aphelion.mp3']),
      dict(id='funk',title='Funkorama',artist='Kevin MacLeod',split='development',kind='finished_mix',license='https://creativecommons.org/licenses/by/4.0/',page='https://incompetech.com/music/royalty-free/index.html?Search=Search&isrc=USUAN1100474',urls=['https://incompetech.com/music/royalty-free/mp3-royaltyfree/Funkorama.mp3']),
      dict(id='metal',title='Metalmania',artist='Kevin MacLeod',split='development',kind='finished_mix',license='https://creativecommons.org/licenses/by/4.0/',page='https://incompetech.com/music/royalty-free/index.html?Search=Search&isrc=USUAN1700023',urls=['https://incompetech.com/music/royalty-free/mp3-royaltyfree/Metalmania.mp3']),
    ]
    for k,title,artist,split in [('baby','Baby Bird','Admiral Bob','development'),('rich','Rich','Hans Atom featuring Adisa McKenzie','holdout'),('imaginal','Imagining Imaginal','SackJo22','holdout')]:
        data=json.loads((OUT/f'pages/{k}-downloads.json').read_text())[0]
        urls=[f['download_url'] for f in data['files'] if f['file_name'].endswith(('.zip','.flac'))]
        specs.append(dict(id=k,title=title,artist=artist,split=split,kind='constructed_unmastered_stem_sum',license='https://creativecommons.org/licenses/by/3.0/',page=data['file_page_url'],urls=urls))
    if '--subset' in sys.argv:
        allowed=sys.argv[sys.argv.index('--subset')+1].split(',');specs=[s for s in specs if s['id'] in allowed]
    jobs=[]
    for s in specs:
        folder=OUT/'originals'/s['id'];folder.mkdir(parents=True,exist_ok=True)
        for u in s['urls']:jobs.append((u,folder/u.rsplit('/',1)[1]))
    errors=[]
    with concurrent.futures.ThreadPoolExecutor(max_workers=4) as pool:
        futures={pool.submit(fetch,*j):j for j in jobs}
        for f in concurrent.futures.as_completed(futures):
            try:f.result()
            except Exception as e:errors.append((futures[f][0],str(e)))
    if errors:raise RuntimeError(errors)
    (OUT/'sources').mkdir(exist_ok=True)
    for s in specs:
        folder=OUT/'originals'/s['id'];s['originals']=[]
        for u in s['urls']:
            p=folder/u.rsplit('/',1)[1]
            s['originals'].append(dict(url=u,path=str(p.relative_to(OUT)),sha256=sha(p),bytes=p.stat().st_size))
            if p.suffix=='.zip':
                dest=folder/p.stem;dest.mkdir(exist_ok=True)
                with zipfile.ZipFile(p) as z:
                    for n in z.namelist():
                        # Some archives use a leading slash; retain only safe components.
                        rel=Path(n.lstrip('/'))
                        assert '..' not in rel.parts and not rel.is_absolute()
                        if n.endswith('/'):continue
                        target=dest/rel;target.parent.mkdir(parents=True,exist_ok=True)
                        if not target.exists():target.write_bytes(z.read(n))
        files=sorted(p for p in folder.rglob('*') if p.suffix.lower() in ['.mp3','.flac'])
        parts=[];s['decoded_parts']=[]
        for p in files:
            sr,x,info=decode(p)
            if parts and sr!=parts[0][0]:sr,x,info=decode(p,parts[0][0])
            parts.append((sr,x))
            s['decoded_parts'].append(dict(path=str(p.relative_to(OUT)),sha256=sha(p),format=info,decoded_peak=float(np.max(np.abs(x))),frames=len(x)))
        sr=parts[0][0]
        if len(parts)==1:
            x=parts[0][1];gain=1.0;s['changes']='FFmpeg decoded to stereo float WAV at original rate; no gain or mastering.'
        else:
            x=np.zeros((max(len(x) for _,x in parts),2),dtype=np.float64)
            for _,part in parts:x[:len(part)]+=part
            gain=10**(-6/20)/np.max(np.abs(x));x=(x*gain).astype(np.float32)
            s['changes']='All available stems summed from time zero at unity relative gain; shorter stems zero padded; scalar peak normalization to -6 dBFS; no bus EQ/compression/limiter. Constructed balance, not artist premaster.'
        s['scalar_gain_db']=float(20*np.log10(gain));p=OUT/f"sources/{s['id']}.wav"
        if p.exists():
            buf=io.BytesIO();wavfile.write(buf,sr,x);assert hashlib.sha256(buf.getvalue()).hexdigest()==sha(p)
        else:wavfile.write(p,sr,x)
        s.update(path=str(p.relative_to(OUT)),sha256=sha(p),rate=sr,frames=len(x),duration_seconds=len(x)/sr)
        print('prepared',s['id'],len(x)/sr,flush=True)
    source=next((OUT.parents[1]/'tests for presets').glob('*original-test.wav'))
    specs.insert(0,dict(id='coat',title="It's a coat",artist='Owner-supplied',split='development',kind='owner_source_mastering_history_unspecified',license='Private owner-authorized research, no redistribution here',path=str(source),sha256=sha(source),changes='Unchanged PCM16 source'))
    frozen=OUT/'expected-corpus.json'
    if frozen.exists():
        expected={s['id']:s for s in json.loads(frozen.read_text(encoding='utf8'))['sources']}
        for s in specs:
            assert s['sha256']==expected[s['id']]['sha256'], (s['id'],'Decoded source differs; retain and investigate versions/bytes before comparing.')
            assert [p['sha256'] for p in s.get('decoded_parts',[])]==[p['sha256'] for p in expected[s['id']].get('decoded_parts',[])]
    (OUT/'corpus.json').write_text(json.dumps(dict(acquired_utc=datetime.datetime.now(datetime.timezone.utc).isoformat(),sources=specs),indent=2),encoding='utf8')
if __name__=='__main__':main()
