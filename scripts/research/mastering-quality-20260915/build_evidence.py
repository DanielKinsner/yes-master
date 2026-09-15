from pathlib import Path
import csv,hashlib,json,subprocess,math,platform
import numpy as np
OUT=Path(__file__).resolve().parent
ROOT=OUT.parents[1];E=OUT/'compact-evidence';E.mkdir(parents=True,exist_ok=True)
def portable(data):
    if isinstance(data,float) and not math.isfinite(data):return '-Infinity' if data<0 else 'Infinity' if data>0 else 'NaN'
    if isinstance(data,dict):return {k:portable(v) for k,v in data.items()}
    if isinstance(data,list):return [portable(v) for v in data]
    return data
def dump(name,data):(E/name).write_text(json.dumps(portable(data),indent=2,allow_nan=False),encoding='utf8')
corpus=json.loads((OUT/'corpus.json').read_text());sources=corpus['sources'];rs=[];source_rows=[]
for s in sources:
    rows=json.loads((OUT/'independent'/f"{s['id']}-summary.json").read_text());assert len(rows)==25
    source_rows.append(dict(id=s['id'],split=s['split'],kind=s['kind'],lufs=rows[0]['lufs'],crest=rows[0]['crest'],lra=rows[0]['lra'],tp=rows[0]['tp'],duration=rows[0]['duration'],sha256=rows[0]['sha256']))
    assert rows[0]['sha256']==s['sha256']
    for r in rows[1:]:
        b=next(b for b in rows[1:] if b['name']==f"control_t{abs(r['variant']['target'])}");stats=r['stats'];count=stats['frames']
        band=[a-z-r['spectral_rms']+b['spectral_rms'] for a,z in zip(r['bands_db'],b['bands_db'])]
        d=dict(source=s['id'],split=s['split'],name=r['name'],trim_db=r['variant'].get('trim',0),target=r['variant']['target'],lufs=r['lufs'],tp=r['tp'],target_error=r['target_error'],ceiling_pass=r['tp']<=-.9,crest=r['crest'],crest400=r['crest400_median'],lra=r['lra'],shortterm_spread=r['shortterm_p95_p10'],section_contrast=r['section_contrast_db'],section_delta_from_control=r['section_contrast_db']-b['section_contrast_db'],attack_body=r['transient_median'],tone_max_delta=max(abs(x) for x in band),side_mid=r['side_mid_db'],side_mid_delta=r['side_mid_db']-b['side_mid_db'],correlation=r['correlation'],limiter_max_gr=stats['limiter_gr_max'],limiter_mean_gr=stats['limiter_gr_sum']/count,limiter_active_pct=100*stats['limiter_active']/count,sat_rms_gain=10*np.log10(stats['sat_out_power']/stats['sat_in_power']),sat_peak_reduction=20*np.log10(stats['sat_in_peak']/stats['sat_out_peak']),pre_landing_lufs=r['pre_landing']['lufs'],sha256=r['sha256'])
        for i,label in enumerate(['low','mid','high']):d[f'comp_{label}_max_gr']=stats['comp_gr_max'][i];d[f'comp_{label}_mean_gr']=stats['comp_gr_sum'][i]/count
        rs.append(d)
with (E/'music-results.csv').open('w',newline='') as f:w=csv.DictWriter(f,fieldnames=list(rs[0]));w.writeheader();w.writerows(rs)
dump('source-measurements.json',source_rows)
small=[]
for s in sources:
    item={k:v for k,v in s.items() if k not in ['decoded_parts','path']};item['path']=str(Path('tests for presets')/Path(s['path']).name) if s['id']=='coat' else s['path']
    item['decoded_parts']=[dict(path=x['path'],sha256=x['sha256'],frames=x['frames'],decoded_peak=x['decoded_peak'],format=x['format']['streams'][0]['codec_name'],rate=x['format']['streams'][0]['sample_rate'],channels=x['format']['streams'][0]['channels']) for x in s.get('decoded_parts',[])]
    small.append(item)
dump('corpus.json',dict(acquired_utc=corpus['acquired_utc'],changes_notice='Decoded to float PCM; three stem sums peak-normalized to -6 dBFS without mastering. Experimental renders apply documented YES candidates. No creator endorsement implied.',sources=small))
dump('mechanisms.json',json.loads((OUT/'mechanism-results.json').read_text()))
dump('src-checks.json',json.loads((OUT/'src-check/analysis.json').read_text()))
dump('src-regrowth.json',{p.stem:json.loads(p.read_text()) for p in sorted((OUT/'peak-probe').glob('*.json'))})
dump('oversampling-reference.json',json.loads((OUT/'oversampling-reference.json').read_text()))
dump('peak-verification.json',dict(blocks=json.loads((OUT/'peak-meter-blocks.json').read_text()),soxr=json.loads((OUT/'peak-soxr-verification.json').read_text()),repeat_precision33=json.loads((OUT/'rechecked-peak.json').read_text()) if (OUT/'rechecked-peak.json').exists() else None))
dump('robustness.json',json.loads((OUT/'robustness-summary.json').read_text()))
centered=[]
for r in json.loads((OUT/'centered-independent.json').read_text()):
    r['independent']={k:v for k,v in r['independent'].items() if k not in ['path','shortterm_trace','transient_attack_body_db','transient_times']};centered.append(r)
dump('centered-validation.json',centered)
dump('landr.json',[{k:v for k,v in r.items() if k not in ['path','shortterm_trace','transient_attack_body_db','transient_times']} for r in json.loads((OUT/'landr-current-measurements.json').read_text())])
dump('optional-clips.json',json.loads((OUT/'optional-comparisons/manifest.json').read_text()))
default=OUT/'default-checks.json'
if default.exists():dump('default-intensity.json',[{k:v for k,v in r.items() if k not in ['path','shortterm_trace','transient_attack_body_db','transient_times','native']} for r in json.loads(default.read_text())])
provenance=json.loads((OUT/'source-provenance.json').read_text())
for r in provenance['records']:
    p=Path(r['file']);assert hashlib.sha256(p.read_bytes()).hexdigest()==r['source_sha256'];r['file']=str(p.relative_to(ROOT))
provenance.update(rustc=subprocess.check_output(['rustc','--version'],text=True).strip(),ffmpeg=subprocess.check_output(['ffmpeg','-version'],text=True).splitlines()[0],numpy=np.__version__,reproduction=dict(on='639ac1dd5d049040d8b689636c08bf2890b39929b72601703d1b93938f6022ce',off='d33c8cae3238161a8d7f812884c2137d980189485b3618def8af93a8baab6ea5'),protocol_sha256=hashlib.sha256((ROOT/'docs/reviews/2026-09-15-mastering-quality-protocol.md').read_bytes()).hexdigest(),cargo_lock_sha256=hashlib.sha256((OUT/'Cargo.lock').read_bytes()).hexdigest())
provenance['python']=platform.python_version()
import scipy
provenance['scipy']=scipy.__version__
provenance['verified_reproduction']=json.loads((OUT/'reproduction-proof.json').read_text())
provenance['instrumentation']=json.loads((OUT/'instrumentation-provenance.json').read_text())
provenance['harness_sha256']={str(p.relative_to(OUT)):hashlib.sha256(p.read_bytes()).hexdigest() for p in [OUT/'src/runner.rs',OUT/'src/src_fixed.rs',OUT/'Cargo.toml',OUT/'extend.py']}
stub=ROOT/'web/tryit/wasm/src/stubs.rs'
provenance['records'].append(dict(file=str(stub.relative_to(ROOT)),source_sha256=hashlib.sha256(stub.read_bytes()).hexdigest(),transformation='Direct path inclusion of existing compile stubs.'))
dump('provenance.json',provenance)
print('wrote',len(rs),'music rows; ceiling failures',sum(not r['ceiling_pass'] for r in rs));print(E)
