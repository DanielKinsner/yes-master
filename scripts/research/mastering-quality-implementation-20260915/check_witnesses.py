"""Recheck historical A2/B1 witnesses through the current real export API.

Uses restored sources and preserved historical settings. Pruned rate/clip
variants are recreated only under a fresh evidence directory.
"""
import argparse
import hashlib
import importlib.util
import json
from pathlib import Path
import shutil
import subprocess
import numpy as np
import soundfile as sf

ROOT = Path(__file__).resolve().parents[3]
AUDIT = ROOT.parent / 'yes-master-independent-audit-20260915'
RECON = AUDIT / 'audit-output/reconciliation-20260915'
spec = importlib.util.spec_from_file_location('ref', ROOT/'scripts/research/final-review-20260915/verify.py')
ref = importlib.util.module_from_spec(spec)
spec.loader.exec_module(ref)


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    out = args.output.resolve()
    assert not out.exists(), 'preserve evidence'
    out.mkdir(parents=True)
    binary = out / 'mastering_quality_witness.exe'
    shutil.copy2(ROOT/'src-tauri/target/codex-rc/debug/examples/mastering_quality_witness.exe', binary)
    funk = AUDIT/'fixtures/inputs/input04.wav'
    expected = json.loads((ROOT/'scripts/research/mastering-quality-20260915/expected-corpus.json').read_text())
    assert ref.sha(funk) == next(s['sha256'] for s in expected['sources'] if s['id']=='funk')
    variant = out/'input04_48k.wav'
    subprocess.run(['ffmpeg','-n','-nostats','-hide_banner','-loglevel','error','-i',str(funk),
        '-af','aresample=resampler=soxr:precision=28:osr=48000','-c:a','pcm_f32le',str(variant)],check=True)
    x, rate = sf.read(variant, dtype='float32', always_2d=True)
    clip = out/'hotstart_input04_48k_60-120s.wav'
    sf.write(clip, x[60*rate:120*rate], rate, subtype='FLOAT')
    rows=[]
    cases=[('hotstart_input04_48k_60-120s_cd',clip)]
    for name in ['defect_input04_custom_notarget_24','defect_input04_custom_notarget_f32']:
        cases.append((name,funk))
    for name in ['defect_clip50ms_streaming_24','defect_clip50ms_streaming_f32']:
        cases.append((name,AUDIT/'audit-output/synthetic/clip_50ms.wav'))
    for name, source in cases:
        historical = RECON/'renders'/(name+'.json')
        dest=out/name
        subprocess.run([str(binary),str(source),str(historical),str(dest)],check=True)
        new=json.loads((dest/'report.json').read_text())
        old=json.loads(historical.read_text())
        x,rate=sf.read(dest/'delivered.wav',always_2d=True)
        reference,rr=sf.read(dest/'padded-reference.wav',always_2d=True)
        assert len(x)==len(reference)==new['expected_frames'] and rate==rr and x.shape==reference.shape
        assert np.isfinite(x).all()
        gain=np.sum(x*reference)/np.sum(reference*reference)
        residual=x-gain*reference
        bits=new['job']['measurements']['bit_depth']
        tolerance=1.6/2**(bits-1) if bits<32 else 2e-7
        assert np.max(np.abs(residual))<tolerance,(name,np.max(np.abs(residual)),tolerance)
        row={'name':name,'source_sha256':new['source_sha256'],'delivered_sha256':new['delivered_sha256'],
             'frames':len(x),'rate':rate,'bit_depth':bits,'uniform_gain_db':ref.db(gain),
             'max_reference_residual':float(np.max(np.abs(residual))),
             'frame955':x[955].tolist() if len(x)>955 else None,
             'frame955_residual':residual[955].tolist() if len(x)>955 else None,
             'previous_measurements':old['measurements'],'measurements':new['job']['measurements'],
             'fft16_dbtp':ref.peak_reference(x,factor=16),
             'soxr16_dbtp':ref.float_soxr(dest/'delivered.wav',rate)}
        # B1 validates removal of the bypass; estimator qualification is B2.
        assert row['measurements']['true_peak_dbtp']<=-1+1e-3,row
        rows.append(row)
        (out/'comparison.json').write_text(json.dumps({'binary_sha256':ref.sha(binary),'rows':rows},indent=2)+'\n')
        print(name,'frames',len(x),'residual',row['max_reference_residual'],
              'reported TP',row['measurements']['true_peak_dbtp'],'FFT TP',row['fft16_dbtp'],flush=True)


if __name__=='__main__': main()
