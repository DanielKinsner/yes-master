import concurrent.futures,json
from run_matrix import OUT,run,select
sources=json.loads((OUT/'corpus.json').read_text())['sources']
ss=[s for s in sources if s['id'] in ['coat','piano','rich','imaginal']]
def execute(s,gain):
    d=OUT/('matrix' if gain==0 else f'robustness/gain{abs(gain)}')/s['id']
    source=json.loads((d/'source.json').read_text());center=-18-source['measurements']['lufs']
    vs=[dict(name=f'centered_{i:02}',group='centered',trim=center-12+i*1.5,center=center,offset=-12+i*1.5) for i in range(13)]
    folder=f'centered/gain{abs(gain)}';run(s,folder,vs,gain)
    rows=[json.loads((OUT/folder/s['id']/f"{v['name']}.json").read_text()) for v in vs]
    results=[]
    for target in [-9,-14]:
        r=select(rows,'centered',target);results.append(dict(source=s['id'],gain=gain,target=target,variant=r['variant'],measure=r['pre_landing'],stats=r['stats'],delivery=next(d for d in r['deliveries'] if d['target']==target)))
    (OUT/folder/s['id']/'selection.json').write_text(json.dumps(results,indent=2));return results
with concurrent.futures.ThreadPoolExecutor(max_workers=3) as pool:
    fs=[pool.submit(execute,s,gain) for s in ss for gain in [0,-6,-12]];results=[]
    for f in concurrent.futures.as_completed(fs):results+=f.result()
(OUT/'centered-summary.json').write_text(json.dumps(results,indent=2));print('centered rows',len(results))
