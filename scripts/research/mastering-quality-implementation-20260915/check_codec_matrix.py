"""Independent FFmpeg decode, exact lossless PCM and finite SOXR peak checks."""
import argparse
import hashlib
import json
from pathlib import Path
import re
import subprocess
import numpy as np
import soundfile as sf


def sha(path):
    with path.open('rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest()


def main():
    parser = argparse.ArgumentParser()
    for name in ('run','ffmpeg','output'):
        parser.add_argument('--'+name,type=Path,required=True)
    parser.add_argument('--decoder',type=Path)
    a=parser.parse_args()
    decoder=a.decoder or a.ffmpeg
    assert not a.output.exists()
    a.output.mkdir(parents=True)
    native=json.loads((a.run/'report.json').read_text(encoding='utf-8'))
    assert native['status']=='complete'
    report={'native_sha256':sha(a.run/'report.json'),'decoder_sha256':sha(decoder),'reference_ffmpeg_sha256':sha(a.ffmpeg),
            'decoder_version':subprocess.run([str(decoder),'-version'],capture_output=True,text=True,check=True).stdout,
            'scope':'independent full decoded PCM; finite zero extension 2048 samples per edge; SOXR16/64 precision33; no enforced lossy ceiling claim',
            'rows':[]}
    refs={}
    def peaks(x,rate):
        padded=a.output/'reference-input.wav'
        sf.write(padded,np.pad(x,((2048,2048),(0,0))),rate,subtype='DOUBLE')
        result={'samples':[float(20*np.log10(max(np.max(np.abs(x[:,c])),1e-15))) for c in range(x.shape[1])]}
        for factor in (16,64):
            filt=f'aformat=sample_fmts=dbl,aresample={rate*factor}:resampler=soxr:precision=33:osf=dbl,astats=metadata=0:reset=0'
            process=subprocess.run([str(a.ffmpeg),'-hide_banner','-nostats','-i',str(padded),'-af',filt,
                '-c:a','pcm_f64le','-f','null','-'],capture_output=True,text=True,check=True)
            values=[float(v) for v in re.findall(r'Peak level dB: (-inf|[-\d.]+)',process.stderr)]
            assert len(values)==x.shape[1]+1
            result[f'soxr{factor}']=values[:-1]
        return result
    for index,row in enumerate(native['rows']):
        if 'error' in row:
            report['rows'].append({'index':index,'error':row['error'],'pass':False})
            continue
        encoded=Path(row['path']);expected_path=Path(row['input'])
        assert sha(encoded)==row['sha256'] and sha(expected_path)==row['input_sha256']
        decoded=a.output/(encoded.name+'.wav')
        process=subprocess.run([str(decoder),'-v','error','-nostdin','-i',str(encoded),'-c:a','pcm_f32le',str(decoded)],capture_output=True,text=True,check=True)
        x,rate=sf.read(decoded,always_2d=True,dtype='float32')
        expected,expected_rate=sf.read(expected_path,always_2d=True,dtype='float32')
        assert np.isfinite(x).all() and len(x)>0
        assert rate==expected_rate==row['rate'] and x.shape[1]==row['channels']
        fmt=row['encoding']['format'];lossless=fmt in ('flac','aiff')
        frames_pass=(len(x)==row['frames']) if fmt!='aac' else (row['frames']+1024<=len(x)<=row['frames']+3071)
        parity=bool(x.shape==expected.shape and np.array_equal(x,expected)) if lossless else None
        reference=peaks(x,rate)
        if str(expected_path) not in refs: refs[str(expected_path)]=peaks(expected,rate)
        input_reference=refs[str(expected_path)]
        maximum=max(v for values in reference.values() for v in values)
        input_maximum=max(v for values in input_reference.values() for v in values)
        measurement_pass=row['decoded_peak'] >= maximum-.002
        ceiling_pass=maximum<=-1+.002
        record={'index':index,'id':row['id'],'encoding':row['encoding'],'bits':row['bits'],
            'rate':rate,'channels':x.shape[1],'frames':len(x),'frames_pass':frames_pass,
            'decoded_sha256':sha(decoded),'exact_lossless_pcm':parity,'reference':reference,
            'input_reference':input_reference,'independent_peak':maximum,'independent_growth_db':maximum-input_maximum,
            'native_peak':row['decoded_peak'],'measurement_pass':measurement_pass,
            'independent_ceiling_pass':ceiling_pass,
            'pass':frames_pass and measurement_pass and (not lossless or (parity and ceiling_pass))}
        # Valid lossy misses remain explicit observations, never forced passes
        # for a requested ceiling or reasons to hide working output formats.
        report['rows'].append(record)
        (a.output/'comparison.json').write_text(json.dumps(report,indent=2),encoding='utf-8')
        if not record['pass']: print('FAIL',record,flush=True)
        elif index%20==0: print('Checked',index+1,'of',len(native['rows']),flush=True)
    report['status']='complete'
    (a.output/'comparison.json').write_text(json.dumps(report,indent=2),encoding='utf-8')
    print('Technical checks passed',sum(r['pass'] for r in report['rows']),'/',len(report['rows']),flush=True)
    assert all(r['pass'] for r in report['rows'])


if __name__=='__main__':main()
