"""Frozen grids; targets share raw renders only after Rust coefficient equality assertion."""
from pathlib import Path
import concurrent.futures,json,subprocess,sys,time
OUT=Path(__file__).resolve().parent
EXE=OUT/'target/release/yes-stage-ablation.exe'
def variants():
    rows=[]
    for group,sat,adapt in [('drive','current',.5),('continuous_drive','continuous',.5),('adapt_drive','current',1.)]:
        for i in range(13):
            trim=-12+i*1.5
            rows.append(dict(name=f'{group}_{i:02}',group=group,sat=sat,adapt=adapt,trim=trim))
    for name,extras in [('control',{}),('sat_half',dict(sat='half')),('sat_off',dict(sat='off')),('continuous',dict(sat='continuous')),('density25',dict(density=.25)),('density75',dict(density=.75)),('adapt0',dict(adapt=0)),('adapt100',dict(adapt=1)),('comp_off',dict(comp='off'))]:
        rows.append(dict(name=name,group='isolated',**extras))
    return rows
def run(s,folder,vs,gain=0):
    p=Path(s['path']);p=p if p.is_absolute() else OUT/p
    output=OUT/folder/s['id'];output.mkdir(parents=True,exist_ok=True)
    todo=[v for v in vs if not (output/f"{v['name']}.json").exists()]
    if not todo:return
    job=dict(source=str(p),output=str(output),source_gain_db=gain,variants=todo)
    jobs=OUT/'jobs';jobs.mkdir(exist_ok=True);j=jobs/(folder.replace('/','_')+'_'+s['id']+'.json');j.write_text(json.dumps(job,indent=2))
    with (output/'run.log').open('a') as log:subprocess.run([str(EXE),str(j)],stdout=log,stderr=log,check=True)
    print('completed',folder,s['id'],len(todo),flush=True)
def select(rows,group,target):
    rs=[r for r in rows if r['variant'].get('group')==group]
    def error(r):return next(d['target_error'] for d in r['deliveries'] if d['target']==target)
    good=[r for r in rs if error(r)<=.2]
    if good:return min(good,key=lambda r:r['variant']['trim'])
    best=min(error(r) for r in rs)
    return min([r for r in rs if error(r)<=best+.05],key=lambda r:r['variant']['trim'])
def finalists(s):
    rows=[json.loads(p.read_text()) for p in (OUT/'matrix'/s['id']).glob('*.json') if p.name not in ['source.json','selection.json']]
    assert len(rows)==48, (s['id'],'incomplete grid',len(rows))
    vs=[];selection=[]
    for target in [-9,-14]:
        for group in ['drive','continuous_drive','adapt_drive']:
            r=select(rows,group,target);v=dict(r['variant']);v.update(name=f'{group}_t{abs(target)}',write=True,target=target)
            vs.append(v);selection.append(dict(source=s['id'],target=target,group=group,selected=r['name'],trim=v['trim'],delivery=next(d for d in r['deliveries'] if d['target']==target)))
        for r in rows:
            if r['variant'].get('group')=='isolated':vs.append(dict(r['variant'],name=f"{r['name']}_t{abs(target)}",write=True,target=target))
    (OUT/'matrix'/s['id']/'selection.json').write_text(json.dumps(selection,indent=2))
    run(s,'finalists',vs)
if __name__=='__main__':
    mode=sys.argv[1];corpus=json.loads((OUT/'corpus.json').read_text())['sources']
    split=sys.argv[2] if len(sys.argv)>2 else 'development';ss=[s for s in corpus if s['split']==split]
    with concurrent.futures.ThreadPoolExecutor(max_workers=3) as pool:
        fs=[pool.submit(finalists,s) if mode=='finalists' else pool.submit(run,s,'matrix',variants()) for s in ss]
        for f in concurrent.futures.as_completed(fs):f.result()
