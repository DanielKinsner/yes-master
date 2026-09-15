from pathlib import Path
import concurrent.futures,json
from run_matrix import OUT,run,variants,select
sources=json.loads((OUT/'corpus.json').read_text())['sources']
# Two development songs with contrasting level/dynamics, and two held-out mixes.
selected=[s for s in sources if s['id'] in ['coat','piano','rich','imaginal']]
vs=[v for v in variants() if v.get('group')=='drive' or v['name']=='control']
with concurrent.futures.ThreadPoolExecutor(max_workers=2) as pool:
    fs=[pool.submit(run,s,f'robustness/gain{abs(g)}',vs,g) for s in selected for g in [-6,-12]]
    for f in concurrent.futures.as_completed(fs):f.result()
results=[]
for s in selected:
    for gain in [0,-6,-12]:
        d=OUT/('matrix' if gain==0 else f'robustness/gain{abs(gain)}')/s['id']
        rows=[json.loads(p.read_text()) for p in d.glob('*.json') if p.name not in ['source.json','selection.json']]
        assert len([r for r in rows if r['variant'].get('group')=='drive'])==13
        for target in [-9,-14]:
            for label,r in [('control',next(r for r in rows if r['name']=='control')),('drive',select(rows,'drive',target))]:
                results.append(dict(source=s['id'],gain=gain,target=target,mode=label,variant=r['variant'],measure=r['pre_landing'],stats=r['stats'],delivery=next(d for d in r['deliveries'] if d['target']==target)))
(OUT/'robustness-summary.json').write_text(json.dumps(results,indent=2));print('robustness rows',len(results))
