import concurrent.futures,json
from run_matrix import OUT,run
from analyze_music import metric
ss={s['id']:s for s in json.loads((OUT/'corpus.json').read_text())['sources']}
rs=json.loads((OUT/'centered-summary.json').read_text())
def execute(source,gain):
    rows=[r for r in rs if r['source']==source and r['gain']==gain];vs=[]
    for r in rows:vs.append(dict(r['variant'],name=f"selected_t{abs(r['target'])}",target=r['target'],write=True))
    folder=f'centered-finalists/gain{abs(gain)}';run(ss[source],folder,vs,gain);result=[]
    for r,v in zip(rows,vs):
        p=OUT/folder/source/(v['name']+'.wav');m=metric(p,f"centered_{source}_gain{abs(gain)}_{v['name']}")
        result.append(dict(r,independent=m,ceiling_pass=m['tp']<=-.9,target_pass=abs(m['lufs']-r['target'])<=.2))
    return result
with concurrent.futures.ThreadPoolExecutor(max_workers=2) as pool:
    fs=[pool.submit(execute,s,g) for s in ss if s in ['coat','piano','rich','imaginal'] for g in [0,-6,-12]];out=[]
    for f in concurrent.futures.as_completed(fs):out+=f.result()
(OUT/'centered-independent.json').write_text(json.dumps(out,indent=2));print('measured centered',len(out))
