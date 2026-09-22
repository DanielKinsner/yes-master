"""Reuse independent PCM/reference facts only for unchanged encoded bytes."""
import argparse
import copy
import json
from pathlib import Path
from verification_common import sha


def main():
    p=argparse.ArgumentParser()
    for name in ('baseline','native','independent','output'):
        p.add_argument('--'+name,type=Path,required=True)
    a=p.parse_args();assert not a.output.exists();a.output.mkdir(parents=True)
    old=json.loads((a.baseline/'report.json').read_text(encoding='utf-8'))
    new=json.loads((a.native/'report.json').read_text(encoding='utf-8'))
    proof=json.loads((a.independent/'comparison.json').read_text(encoding='utf-8'))
    assert new['status']==proof['status']=='complete'
    assert new['prior_report_sha256']==proof['native_sha256']==sha(a.baseline/'report.json')
    result=copy.deepcopy(proof)
    result['prior_independent_sha256']=sha(a.independent/'comparison.json')
    result['native_sha256']=sha(a.native/'report.json')
    result['mp3_decoder']=new['mp3_decoder']
    result['reuse_scope']='same encoded file hashes; independently decoded PCM and finite references reused without another encode or reconstruction'
    for row in result['rows']:
        index=row['index'];prior=old['rows'][index];current=new['rows'][index]
        assert prior['sha256']==current['sha256']==sha(Path(current['path']))
        assert prior['input_sha256']==current['input_sha256']==sha(Path(current['input']))
        decoded=a.independent/(Path(current['path']).name+'.wav')
        assert sha(decoded)==row['decoded_sha256']
        row['prior_native_peak']=row['native_peak']
        row['native_peak']=current['decoded_peak']
        row['measurement_pass']=current['decoded_peak']>=row['independent_peak']-.002
        row['pass']=row['frames_pass'] and row['measurement_pass'] and (row['exact_lossless_pcm'] is None or (row['exact_lossless_pcm'] and row['independent_ceiling_pass']))
    (a.output/'comparison.json').write_text(json.dumps(result,indent=2),encoding='utf-8')
    print('Technical checks',sum(r['pass'] for r in result['rows']),'/',len(result['rows']))
    assert all(r['pass'] for r in result['rows'])


if __name__=='__main__':main()
