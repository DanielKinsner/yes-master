"""B4 experiment: measured synthetic headroom versus bounded downward re-encode.

Always derives PCM from the original protected float master. No cumulative
lossy transcode and no changed ceiling, sonic constant or shipping policy.
"""
import argparse
import json
from pathlib import Path
import numpy as np
import soundfile as sf
from make_codec_inputs import sha


def main():
    p=argparse.ArgumentParser()
    p.add_argument('--baseline',type=Path,required=True)
    p.add_argument('--output',type=Path,required=True)
    p.add_argument('--previous',type=Path)
    a=p.parse_args();assert not a.output.exists();a.output.mkdir(parents=True)
    baseline=json.loads((a.baseline/'report.json').read_text(encoding='utf-8'))
    assert baseline['status']=='complete' and all('error' not in r for r in baseline['rows'])
    headroom={}
    for row in baseline['rows']:
        fmt=row['encoding']['format']
        if fmt not in ('flac','aiff') and row['id'].startswith(('short-','stress-')):
            headroom[fmt]=max(headroom.get(fmt,0.),row['growth_db'])
    selected=[]
    if a.previous:
        previous=json.loads((a.previous/'report.json').read_text(encoding='utf-8'))
        for row in previous['rows']:
            e=row['experiment']
            if e['strategy']=='measured_reencode' and row['ceiling_miss']:
                assert e['round']==1, 'at most two corrective encodes'
                selected.append((e['baseline_index'],'measured_reencode',2,e['gain_db']-1.-row['decoded_peak']))
    else:
        for i,row in enumerate(baseline['rows']):
            fmt=row['encoding']['format']
            if fmt in ('flac','aiff'):continue
            selected.append((i,'synthetic_headroom',0,-headroom[fmt]))
            if row['ceiling_miss']:
                selected.append((i,'measured_reencode',1,-1.-row['decoded_peak']))
    inputs=[]
    for index,strategy,iteration,gain_db in selected:
        row=baseline['rows'][index]
        assert row['bits']==32 and gain_db<=0
        source=Path(row['input']);assert sha(source)==row['input_sha256']
        x,rate=sf.read(source,always_2d=True,dtype='float32')
        identity=f'{strategy}-r{iteration}-{index}'
        path=a.output/(identity+'.wav')
        sf.write(path,x*np.float32(10**(gain_db/20)),rate,subtype='FLOAT')
        inputs.append({'id':identity,'path':str(path.resolve()),'sha256':sha(path),
            'encodings':[row['encoding']],'preserve_level':True,
            'experiment':{'baseline_index':index,'strategy':strategy,'round':iteration,'gain_db':gain_db,
                'baseline_peak':row['decoded_peak'],'baseline_lufs':row['decoded_lufs']}})
    (a.output/'inputs.json').write_text(json.dumps(inputs,indent=2),encoding='utf-8')
    (a.output/'protocol.json').write_text(json.dumps({'baseline_sha256':sha(a.baseline/'report.json'),
        'previous_sha256':sha(a.previous/'report.json') if a.previous else None,
        'synthetic_headroom_db':headroom,'cases':len(inputs),
        'rule':'headroom=max(0,observed synthetic peak growth) per codec, all rates/channels/qualities; measured correction removes exact decoded excess; at most two re-encodes from retained original float PCM',
        'exhaustion':'retain the actual valid output and an explicit ceiling-miss/advisory; no enforced-codec claim',
        'scope':'development comparison, not unseen validation or adoption'},indent=2),encoding='utf-8')
    print('Frozen',len(inputs),'correction inputs; headroom:',headroom)


if __name__=='__main__':main()
