"""Prepare, measure and qualify the frozen corrected-policy quiet-copy matrix."""
import argparse
from collections import Counter
from dataclasses import asdict
import json
from pathlib import Path
import shutil
import subprocess
import time

import numpy as np
from bidirectional_selector import Candidate, select
from bidirectional_research import verified, typed
from drive_metrics import LIMITS, compare, constraints, measure
from verification_common import sha

ROOT=Path(__file__).resolve().parents[3]
SPEC=ROOT/'docs/reviews/2026-09-21-quiet-copy-protocol.md'
BASE=ROOT/'docs/reviews/evidence/2026-09-21-dynamics-research/bidirectional-v1.json'
NAMES=('piano','imaginal','metal','rich')


def read(path):
    return json.loads(Path(path).read_text(encoding='utf-8-sig'))


def bound(path):
    return dict(path=str(Path(path).resolve()),sha256=sha(Path(path)))


def prepare(args):
    assert not args.output.exists()
    baseline=read(BASE)
    assert baseline['status']=='complete' and baseline['limits']==LIMITS
    for item in baseline['inputs'].values():
        assert sha(Path(item['path']))==item['sha256']
    prior_job=read(baseline['inputs']['job']['path'])
    for bindings in prior_job['retained_bindings'].values():
        for item in bindings:
            assert sha(Path(item['path']))==item['sha256']
    cases=[]
    estimate=0
    for name in NAMES:
        old_case=next(c for c in prior_job['cases'] if c['id']==name)
        assert sha(Path(old_case['report']))==old_case['report_sha256']
        assert sha(Path(old_case['source_metrics']))==old_case['source_metrics_sha256']
        historic=read(old_case['report'])
        assert sha(Path(historic['source']))==old_case['source_sha256']
        rows=[r for r in baseline['rows'] if r['native']['case']==name]
        assert len(rows)==4 and {(r['native']['policy'],r['native']['offset_db']) for r in rows}=={
            ('control',0.),('single',0.),('single',-3.),('single',3.)}
        source=prior_job['sources'][name]
        control=next(r['metric']['source_delta'] for r in rows if r['native']['policy']=='control')
        single=next(r for r in historic['rows'] if r['preset_id']=='universal50' and r['target']==-14
                    and r['source_gain_db']==0 and r['policy']=='single')
        for item in rows:
            row=item['native']
            verified(row,item['metric'],item['independent'],source,control)
            assert row['source_sha256']==old_case['source_sha256']
            assert typed(row['requested_settings'])==typed(single['requested_settings'])
            if row['policy']=='single':
                assert row['render']['base_coefficients']==single['render']['coefficients']
                assert typed(row['drive_db'])==typed(single['operating_drive_db'])
        cases.append(dict(id=name,source=historic['source'],source_sha256=old_case['source_sha256'],
            report=old_case['report'],report_sha256=old_case['report_sha256'],
            normalized_lufs=historic['descriptor']['normalized_lufs'],normalization_db=historic['descriptor']['normalization_db'],
            preset_push_db=single['preset_push_db'],saturation_small_signal_db=single['saturation_small_signal_db'],
            originals=[r['native'] for r in rows]))
        estimate+=8*(rows[0]['native']['frames']*8+128)
    reserve,free=25*1024**3,shutil.disk_usage(args.output.parent).free
    assert free>=reserve+estimate
    job=dict(experiment='quiet-copy-bidirectional-v1',cases=cases,baseline=bound(BASE),
        sources=prior_job['sources'],specification=str(SPEC),specification_sha256=sha(SPEC),
        binary=str(args.binary.resolve()),binary_sha256=sha(args.binary),preparation_script_sha256=sha(Path(__file__)),
        minimum_free_reserve_bytes=reserve,estimated_new_wav_bytes=estimate,observed_free_bytes=free,
        local_commit=subprocess.check_output(['git','rev-parse','HEAD'],text=True).strip())
    args.output.write_text(json.dumps(job,indent=2,allow_nan=False)+'\n')
    print(f'Frozen 32 new outputs, 16 verified originals; estimated new bytes {estimate}')


def compare_numeric(row,metric,original,case):
    if row['policy']=='control':
        return None
    delta=compare(original['metric']['metrics'],metric)
    descriptor=row['descriptor']
    differences=dict(maximum_pcm_difference=row['maximum_delivered_copy_difference'],
        delivered_lufs_difference=abs(row['lufs']-original['native']['lufs']),
        normalized_lufs_difference=abs(descriptor['normalized_lufs']-case['normalized_lufs']),
        drive_db_difference=abs(row['drive_db']-original['native']['drive_db']),
        normalization_db_difference=abs(descriptor['normalization_db']+row['source_gain_db']-case['normalization_db']),
        section_contrast_difference=abs(delta['section_contrast_delta_db']),
        maximum_section_difference=max(abs(v) for v in delta['section_delta_db']),
        maximum_attack_difference=max(abs(v) for v in delta['paired_attack_delta_db']),
        maximum_band_difference=max(abs(v) for v in delta['band_delta_db']),
        side_mid_difference=abs(delta['stereo_delta']['side_mid_db']),
        correlation_difference=abs(delta['stereo_delta']['correlation']))
    budgets=dict(maximum_pcm_difference=.0001,delivered_lufs_difference=.001,normalized_lufs_difference=.001,
        drive_db_difference=.001,normalization_db_difference=.001,section_contrast_difference=.01,
        maximum_section_difference=.01,maximum_attack_difference=.01,maximum_band_difference=.01,
        side_mid_difference=.01,correlation_difference=.00001)
    failures=[k for k,v in differences.items() if not np.isfinite(v) or v>budgets[k]]
    return dict(differences=differences,budgets=budgets,failures=failures,pass_all=not failures)


def evaluate(args):
    assert not args.output.exists()
    job,native=read(args.job),read(args.report)
    assert native['status']=='complete' and len(native['rows'])==32
    assert native['job_sha256']==sha(args.job) and native['experiment']==job['experiment']
    assert sha(Path(job['baseline']['path']))==job['baseline']['sha256']
    baseline=read(job['baseline']['path'])
    result=dict(status='running',limits=LIMITS,job_sha256=sha(args.job),native_sha256=sha(args.report),rows=[])
    controls={}
    for row in native['rows']:
        assert sha(Path(row['path']))==row['sha256']
        original=next(r for r in baseline['rows'] if r['native']['id']==row['original_id'])
        assert row['original_sha256']==original['native']['sha256']
        assert typed(row['requested_settings'])==typed(original['native']['requested_settings'])
        assert row['source_sha256']==original['native']['source_sha256']
        source=job['sources'][row['case']]
        start=time.perf_counter()
        metric=measure(Path(row['path']),source['anchors'])
        metric_s=time.perf_counter()-start
        assert (metric['frames'],metric['rate'],metric['channels'])==(row['frames'],48000,2)
        delta=compare(source,metric)
        key=(row['case'],row['source_gain_db'])
        if row['policy']=='control':
            assert key not in controls
            controls[key]=delta
        fixed=next(r['metric']['source_delta'] for r in baseline['rows']
                   if r['native']['case']==row['case'] and r['native']['policy']=='control')
        item=dict(id=row['id'],sha256=row['sha256'],metrics=metric,source_delta=delta,metric_s=metric_s,
            character_failures=constraints(source,delta,controls[key]),
            fixed_original_reference_failures=constraints(source,delta,fixed),
            numeric_consistency=compare_numeric(row,metric,original,next(c for c in job['cases'] if c['id']==row['case'])))
        result['rows'].append(item)
        args.output.write_text(json.dumps(result,indent=2,allow_nan=False)+'\n')
        print(row['id'],row['lufs'],item['character_failures'],
              'numeric='+str(item['numeric_consistency']['pass_all']) if item['numeric_consistency'] else 'level-dependent control',flush=True)
    assert len(controls)==8
    result['status']='complete'
    args.output.write_text(json.dumps(result,indent=2,allow_nan=False)+'\n')


def summarize(args):
    assert not args.output.exists()
    paths=dict(job=args.job,native=args.report,metrics=args.metrics,independent=args.independent,process=args.process)
    job,native,metrics,independent,process=[read(p) for p in paths.values()]
    assert all(r['status']=='complete' for r in (native,metrics,independent,process))
    assert native['job_sha256']==metrics['job_sha256']==process['job_sha256']==sha(args.job)
    assert metrics['native_sha256']==independent['native_sha256']==sha(args.report)
    assert process['exit_code']==0 and metrics['limits']==LIMITS
    assert independent['ffmpeg_sha256']=='67abd24645ae7cf73e1fad8b092de73b064b62ae61819751fc31e79e91e842d3'
    assert sha(Path(job['binary']))==process['binary_sha256']==job['binary_sha256']
    assert sha(Path(job['specification']))==job['specification_sha256']
    assert sha(Path(job['baseline']['path']))==job['baseline']['sha256']
    baseline=read(job['baseline']['path'])
    assert len(native['rows'])==len(metrics['rows'])==len(independent['rows'])==32
    assert independent['all_pass']==all(r['peak_pass'] and r['lufs_pass'] and
                                       r['fullscale_samples']==0 for r in independent['rows'])
    rows=[]
    for row in native['rows']:
        metric=next(r for r in metrics['rows'] if r['id']==row['id'])
        check=next(r for r in independent['rows'] if r['path']==row['path'])
        assert row['sha256']==metric['sha256']==check['sha256']==sha(Path(row['path']))
        assert row['frames']==check['frames'] and row['rate']==check['rate']==48000
        assert row['channels']==check['channels']==2 and row['peak']<=row['ceiling']==check['ceiling']==-1.
        assert row['lufs']==check['native_lufs']
        rows.append(dict(native=row,metric=metric,independent=check))
    groups=[]
    for name in NAMES:
        original=next(g['selected'] for g in baseline['groups'] if g['case']==name)
        identity=lambda r:None if r['selected'] is None else (r['selected']['policy'],r['selected']['offset_db'])
        for gain in (-40.,-80.):
            group=[r for r in rows if r['native']['case']==name and r['native']['source_gain_db']==gain]
            assert len(group)==4
            results={}
            for key,label in [('character_failures','matched_current'),('fixed_original_reference_failures','fixed_original_diagnostic')]:
                context=sha(args.job)+':'+name+':'+str(gain)+':'+label
                candidates=[]
                for item in group:
                    row=item['native']
                    available=row['lufs'] is not None
                    check=item['independent']
                    technical=[] if available else ['unavailable_loudness_for_selector']
                    if not check['peak_pass']:
                        technical.append('independent_peak_failed')
                    if not check['lufs_pass']:
                        technical.append('independent_loudness_failed')
                    if check['fullscale_samples']:
                        technical.append('fullscale_samples')
                    candidates.append(Candidate(context,row['id'],row['sha256'],row['policy'],row['offset_db'],
                        row['lufs'] if available else -float('inf'),-14.,row['peak'],-1.,tuple(item['metric'][key]),
                        tuple(technical)))
                selection=asdict(select(candidates,context))
                results[label]=dict(selection=selection,identity_matches_original=identity(selection)==identity(original),
                    outcome_matches_original=(selection['reason'],selection['target_feasible'])==(original['reason'],original['target_feasible']),
                    selected_lufs_change_lu=None if selection['selected'] is None else
                        selection['selected']['lufs']-original['selected']['lufs'],
                    level_sensitive_control_selected=selection['selected'] is not None and selection['selected']['policy']=='control')
            groups.append(dict(case=name,source_gain_db=gain,original=original,**results))
    consistency=[r['metric']['numeric_consistency'] for r in rows if r['native']['policy']=='single']
    result=dict(status='complete',scope='Corrected known-source quiet copies; original analysis/guards retained; no sonic adoption',
        independent_qualification='passed' if independent['all_pass'] else 'failed',
        all_whole_file_checks_pass=independent['all_pass'],
        independent_passes=sum(r['independent']['peak_pass'] and r['independent']['lufs_pass'] and
                              r['independent']['fullscale_samples']==0 for r in rows),
        independent_failures=[r['native']['id'] for r in rows if not (r['independent']['peak_pass'] and
                              r['independent']['lufs_pass'] and r['independent']['fullscale_samples']==0)],
        inputs={k:bound(p) for k,p in paths.items()},specification_sha256=job['specification_sha256'],limits=LIMITS,
        script_sha256=sha(Path(__file__)),rows=rows,groups=groups,process=process,source_preparation=native['source_preparation'],
        normalized_comparisons=len(consistency),normalized_passes=sum(r['pass_all'] for r in consistency),
        matched_identity_passes=sum(g['matched_current']['identity_matches_original'] for g in groups),
        fixed_reference_identity_passes=sum(g['fixed_original_diagnostic']['identity_matches_original'] for g in groups),
        matched_outcomes=dict(Counter(g['matched_current']['selection']['reason'] for g in groups)),
        fixed_reference_outcomes=dict(Counter(g['fixed_original_diagnostic']['selection']['reason'] for g in groups)),
        matched_outcome_passes=sum(g['matched_current']['outcome_matches_original'] for g in groups),
        fixed_reference_outcome_passes=sum(g['fixed_original_diagnostic']['outcome_matches_original'] for g in groups),
        unavailable_deliveries=sum(r['native']['lufs'] is None for r in rows),
        new_audio_bytes=sum(Path(r['native']['path']).stat().st_size for r in rows),
        memory_budget_pass=process['peak_working_set_bytes']<=2*1024**3,
        phase_work_s={k:sum(r['native'][k] if k!='chain_s' else r['native']['render'][k] for r in rows)
                      for k in ('chain_s','src_s','finalize_s')},
        metric_work_s=sum(r['metric']['metric_s'] for r in rows),reference_work_s=sum(r['independent']['reference_s'] for r in rows))
    args.output.parent.mkdir(parents=True,exist_ok=True)
    args.output.write_text(json.dumps(result,indent=2,allow_nan=False)+'\n')
    print(json.dumps({k:result[k] for k in ('independent_qualification','independent_passes','independent_failures','normalized_comparisons','normalized_passes','matched_identity_passes',
        'fixed_reference_identity_passes','matched_outcomes','unavailable_deliveries','memory_budget_pass')},indent=2))


if __name__=='__main__':
    parser=argparse.ArgumentParser()
    parser.add_argument('mode',choices=('prepare','evaluate','summarize'))
    for name in ('job','report','binary','metrics','independent','process','output'):
        parser.add_argument('--'+name,type=Path,required=name=='output')
    args=parser.parse_args()
    globals()[args.mode](args)
