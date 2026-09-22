"""Supplementary screen of Universal intensity 50; no retuning."""
import concurrent.futures,json
from run_matrix import OUT,run
from analyze_music import metric
ss=json.loads((OUT/'corpus.json').read_text())['sources']
def execute(s):
    vs=[dict(name=f'universal50_t{t}',intensity=.5,target=-t,write=True) for t in [9,14]]
    run(s,'default-checks',vs);rows=[]
    for v in vs:
        file=OUT/'default-checks'/s['id']/(v['name']+'.wav');r=metric(file,s['id']+'_'+v['name']);r['variant']=v;r['source_id']=s['id'];r['native']=json.loads(file.with_suffix('.json').read_text());rows.append(r)
    return rows
with concurrent.futures.ThreadPoolExecutor(max_workers=2) as pool:
    fs=[pool.submit(execute,s) for s in ss];rows=[]
    for f in concurrent.futures.as_completed(fs):rows+=f.result()
(OUT/'default-checks.json').write_text(json.dumps(rows,indent=2));print('default checks',len(rows))
