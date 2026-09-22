"""Qualify a reference-only static component before diagnostic rescoring."""
import argparse
from collections import Counter
from dataclasses import asdict
import json
from pathlib import Path
import shutil
import time

import numpy as np
from scipy.signal import lfilter
import soundfile as sf
from bidirectional_selector import Candidate, select
from drive_metrics import measure, compare, constraints, LIMITS
from verification_common import sha

ROOT=Path(__file__).resolve().parents[3]
SPEC=ROOT/'docs/reviews/2026-09-21-static-reference-protocol.md'
EVIDENCE=ROOT/'docs/reviews/evidence/2026-09-21-dynamics-research'


def read(path):
    return json.loads(Path(path).read_text(encoding='utf-8-sig'))


def write(path,data):
    path.write_text(json.dumps(data,indent=2,allow_nan=False)+'\n')


def bound(path):
    return dict(path=str(path.resolve()),sha256=sha(path))


def fixtures(directory,rate):
    n=65536
    impulse=np.zeros((n,2),dtype=np.float32)
    impulse[32,0]=.25;impulse[127,1]=-.125
    t=np.arange(n)/rate
    tones=np.column_stack((.1*np.sin(2*np.pi*70*t)+.05*np.sin(2*np.pi*1000*t),
        .07*np.sin(2*np.pi*1000*t+.3)+.03*np.sin(2*np.pi*9000*t))).astype(np.float32)
    state=123456789
    noise=np.empty(n*2,dtype=np.float32)
    for i in range(len(noise)):
        state=(1664525*state+1013904223)&0xffffffff
        noise[i]=((state/4294967296.)*2-1)*.1
    rows=[]
    for name,samples in [('impulse',impulse),('multitone',tones),('noise',noise.reshape(n,2))]:
        path=directory/f'{name}-{rate}.wav'
        sf.write(path,samples,rate,subtype='FLOAT')
        rows.append(dict(name=name,**bound(path)))
    return rows


def prepare(args):
    assert not args.output.exists()
    args.output.mkdir()
    prior=read(ROOT/'test-output/dynamics-research-20260921/quiet-copy/job-v2.json')
    bindings={name:bound(EVIDENCE/file) for name,file in [('original','bidirectional-v1.json'),
        ('quiet','quiet-copy-v2.json'),('availability','block-availability-v1.json')]}
    for binding in bindings.values():
        data=read(binding['path'])
        assert data['status']=='complete'
    original,quiet,availability=[read(bindings[name]['path']) for name in ('original','quiet','availability')]
    assert availability['retained_agreements']==32
    for row in original['rows']+quiet['rows']:
        assert sha(Path(row['native']['path']))==row['native']['sha256']
        assert row['independent']['peak_pass'] and row['independent']['fullscale_samples']==0
    binary=args.output/'probe.exe';shutil.copy2(args.binary,binary)
    sets={rate:fixtures(args.output,rate) for rate in {s['rate'] for s in prior['sources'].values()}}
    cases=[]
    estimate=0
    for case in prior['cases']:
        source=Path(case['source']);assert sha(source)==case['source_sha256']
        facts=prior['sources'][case['id']]
        zero=next(r for r in case['originals'] if r['policy']=='single' and r['offset_db']==0)
        cases.append(dict(id=case['id'],source=str(source),source_sha256=sha(source),rate=facts['rate'],
            frames=facts['frames'],settings=zero['requested_settings'],original_drive=zero['drive_db'],
            original_coefficients=zero['render']['base_coefficients'],fixtures=sets[facts['rate']]))
        estimate+=(facts['frames']*3+65536*3)*8+6*128
    reserve=25*1024**3
    assert shutil.disk_usage(args.output).free>=reserve+estimate
    job=dict(experiment='static-preset-reference-v1',cases=cases,sources=prior['sources'],bindings=bindings,
        binary=str(binary.resolve()),binary_sha256=sha(binary),specification=str(SPEC),specification_sha256=sha(SPEC),
        minimum_free_reserve_bytes=reserve,estimated_new_wav_bytes=estimate,script=bound(Path(__file__)))
    write(args.output/'job.json',job)
    print('Frozen 12 full references and 12 synthetic outputs;',estimate,'estimated bytes')


def cascade(samples,sections):
    result=samples
    for b0,b1,b2,a1,a2 in sections:
        result=lfilter([b0,b1,b2],[1.,a1,a2],result,axis=0).astype(np.float32)
    return result


def independent_reference(samples,topology):
    result=cascade(samples,topology['eq'])
    if topology['compression_active']:
        summed=None
        for sections,gain in zip(topology['branches'],topology['makeup'],strict=True):
            band=cascade(result,sections)*np.float32(gain)
            summed=band if summed is None else summed+band
        result=summed
    width=np.float32(topology['width'])
    if abs(width-np.float32(1.))>1e-5:
        mid=np.float32(.5)*(result[:,0]+result[:,1])
        side=np.float32(.5)*(result[:,0]-result[:,1])*width
        result=np.column_stack((mid+side,mid-side))
    assert np.isfinite(result).all()
    return result


def relative_error(a,b):
    assert a.shape==b.shape
    scale=max(float(np.abs(a).max()),1e-12)
    maximum=0.
    for offset in range(0,len(a),262144):
        delta=a[offset:offset+262144].astype(np.float64)-b[offset:offset+262144]
        maximum=max(maximum,float(np.abs(delta).max())/scale)
    return maximum


def verify(args):
    import hashlib
    job=read(args.job);native=read(args.report)
    assert native['status']=='complete' and native['job_sha256']==sha(args.job) and len(native['rows'])==24
    assert not args.output.exists()
    report=dict(status='running',job=bound(args.job),native=bound(args.report),rows=[])
    originals={}
    for row in native['rows']:
        started=time.perf_counter()
        case=next(c for c in job['cases'] if c['id']==row['case'])
        if row['fixture'] is not None:
            input_path=Path(row['fixture']['path']);assert sha(input_path)==row['fixture']['sha256']
            samples,rate=sf.read(input_path,dtype='float32',always_2d=True)
        else:
            assert sha(Path(case['source']))==case['source_sha256']
            samples,rate=sf.read(case['source'],dtype='float32',always_2d=True)
            samples=(samples.astype(np.float64)*10**(row['source_gain_db']/20)).astype(np.float32)
            samples=(samples.astype(np.float64)/float(np.abs(samples).max())).astype(np.float32)
        assert hashlib.sha256(samples.astype('<f4').tobytes()).hexdigest()==row['input_pcm_sha256']
        path=Path(row['path']);assert sha(path)==row['sha256']
        output,out_rate=sf.read(path,dtype='float32',always_2d=True)
        assert np.isfinite(output).all()
        assert output.shape==samples.shape and (len(output),out_rate,rate,output.shape[1])==(row['frames'],row['rate'],row['rate'],2)
        expected=independent_reference(samples,row['topology'])
        difference=relative_error(output,expected)
        item=dict(id=row['id'],sha256=row['sha256'],independent_relative_pcm_difference=difference,
            independent_pass=difference<=1e-5,copy_relative_difference=None,copy_pass=None,
            work_s=time.perf_counter()-started)
        if row['fixture'] is None:
            if row['source_gain_db']==0:
                originals[row['case']]=output
            else:
                item['copy_relative_difference']=relative_error(originals[row['case']],output)
                item['copy_pass']=item['copy_relative_difference']<=1e-4
                assert abs(item['copy_relative_difference']-row['relative_copy_difference'])<=1e-12
        report['rows'].append(item);write(args.output,report)
        print(row['id'],difference,'copy',item['copy_relative_difference'],flush=True)
    report.update(status='complete',all_pass=all(r['independent_pass'] and r['copy_pass'] is not False for r in report['rows']))
    write(args.output,report)


def score(args):
    job=read(args.job);native=read(args.report);verified=read(args.verified)
    assert not args.output.exists() and verified['status']=='complete' and verified['all_pass']
    assert verified['job']['sha256']==sha(args.job) and verified['native']['sha256']==sha(args.report)
    assert sha(Path(job['specification']))==job['specification_sha256']
    for binding in job['bindings'].values():assert sha(Path(binding['path']))==binding['sha256']
    original,quiet,availability=[read(job['bindings'][name]['path']) for name in ('original','quiet','availability')]
    metrics=[];by_case={}
    for row in native['rows']:
        if row['fixture'] is not None:continue
        started=time.perf_counter();path=Path(row['path'])
        assert sha(path)==row['sha256']
        source=job['sources'][row['case']]
        metric=measure(path,source['anchors'])
        delta=compare(source,metric)
        copy=None
        if row['source_gain_db']==0:
            by_case[row['case']]=metric
        else:
            diff=compare(by_case[row['case']],metric)
            values=dict(section_contrast=abs(diff['section_contrast_delta_db']),
                sections=max(abs(v) for v in diff['section_delta_db']),
                attacks=max(abs(v) for v in diff['paired_attack_delta_db']),
                bands=max(abs(v) for v in diff['band_delta_db']),
                side_mid=abs(diff['stereo_delta']['side_mid_db']),correlation=abs(diff['stereo_delta']['correlation']))
            copy=dict(differences=values,pass_all=all(np.isfinite(v) and v<=(.00001 if k=='correlation' else .01) for k,v in values.items()))
        metrics.append(dict(native=row,metrics=metric,source_delta=delta,copy=copy,work_s=time.perf_counter()-started))
        print('measured',row['id'],flush=True)
    write(args.output.with_suffix('.metrics.json'),dict(status='complete',rows=metrics))
    assert all(r['copy'] is None or r['copy']['pass_all'] for r in metrics)
    groups=[]
    for reference in metrics:
        name=reference['native']['case'];gain=reference['native']['source_gain_db']
        pool=[r for r in (original['rows'] if gain==0 else quiet['rows']) if r['native']['case']==name and
              (gain==0 or r['native']['source_gain_db']==gain)]
        assert len(pool)==4
        context=f'diagnostic-static-reference-v1:{name}:{gain}'
        candidates=[];rows=[]
        for item in pool:
            row=item['native'];technical=[]
            assert sha(Path(row['path']))==row['sha256']
            assert item['independent']['peak_pass'] and item['independent']['fullscale_samples']==0
            if gain==0:
                assert item['independent']['lufs_pass']
            else:
                available=next(r for r in availability['retained'] if r['id']==row['id'])
                assert available['agreement'] and available['file']['sha256']==row['sha256']
            if row['lufs'] is None:technical.append('unavailable_loudness_for_selector')
            failures=constraints(job['sources'][name],item['metric']['source_delta'],reference['source_delta'])
            candidates.append(Candidate(context,row['id'],row['sha256'],row['policy'],row['offset_db'],
                row['lufs'] if row['lufs'] is not None else -float('inf'),-14.,row['peak'],-1.,tuple(failures),tuple(technical)))
            rows.append(dict(id=row['id'],sha256=row['sha256'],historical_failures=item['metric']['character_failures'],
                diagnostic_failures=failures,technical_failures=technical,source_delta=item['metric']['source_delta']))
        selection=asdict(select(candidates,context))
        identity=lambda s:None if s['selected'] is None else (s['selected']['policy'],s['selected']['offset_db'])
        baseline=next(g['selected'] for g in original['groups'] if g['case']==name) if gain==0 else next(
            g['selection'] for g in groups if g['case']==name and g['source_gain_db']==0)
        groups.append(dict(case=name,source_gain_db=gain,reference_id=reference['native']['id'],rows=rows,
            selection=selection,comparison='historical_original' if gain==0 else 'same_reference_original',
            identity_matches=identity(selection)==identity(baseline),
            outcome_matches=(selection['reason'],selection['target_feasible'])==(baseline['reason'],baseline['target_feasible']),
            selected_lufs_change=None if selection['selected'] is None or baseline['selected'] is None else
                selection['selected']['lufs']-baseline['selected']['lufs']))
        print(name,gain,identity(selection),selection['reason'],flush=True)
    result=dict(status='complete',diagnostic_version='diagnostic-static-reference-v1',limits=LIMITS,
        inputs=dict(job=bound(args.job),native=bound(args.report),mechanical=bound(args.verified)),
        references=metrics,groups=groups,script=bound(Path(__file__)),
        original_choices_retained=sum(g['identity_matches'] for g in groups if g['source_gain_db']==0),
        copy_identity_matches=sum(g['identity_matches'] for g in groups if g['source_gain_db']!=0),
        outcomes=dict(Counter(g['selection']['reason'] for g in groups)),
        scope='Reference-only qualified; diagnostic scoring only; no shipping policy or new masters')
    write(args.output,result)


if __name__=='__main__':
    parser=argparse.ArgumentParser();parser.add_argument('mode',choices=('prepare','verify','score'))
    for name in ('output','binary','job','report','verified'):
        parser.add_argument('--'+name,type=Path,required=name=='output')
    args=parser.parse_args();globals()[args.mode](args)
