"""Bounded controlled qualification followed by additive retained-WAV metering."""
import argparse
import json
from pathlib import Path
import shutil
import subprocess
import time

import numpy as np
import soundfile as sf
from block_availability import classify, agreement, VERSION
from verification_common import sha

ROOT = Path(__file__).resolve().parents[3]
FFMPEG = Path('C:/Program Files (x86)/Common Files/AutoPod/ffmpeg/bin/ffmpeg.exe')
FFMPEG_SHA = '67abd24645ae7cf73e1fad8b092de73b064b62ae61819751fc31e79e91e842d3'
BASE = ROOT/'docs/reviews/evidence/2026-09-21-dynamics-research/quiet-copy-v2.json'
PROTOCOL = ROOT/'docs/reviews/2026-09-21-loudness-availability-protocol.md'


def read(path):
    return json.loads(Path(path).read_text(encoding='utf-8-sig'))


def write(path, data):
    path.write_text(json.dumps(data, indent=2, allow_nan=False)+'\n')


def bound(path):
    return dict(path=str(path.resolve()), sha256=sha(path))


def fixtures(directory):
    specs = [('silence', 0., 96000, None), ('quiet', 10**(-90/20), 96000, None),
             ('burst_quiet', 10**(-60/20), 96000, (38400,38880)),
             ('burst_finite', 10**(-40/20), 96000, (38400,38880))]
    specs += [('tone_'+str(db), 10**(db/20), 96000, None)
              for db in (-72., -70.2, -70.04, -69.96, -69.8, -68.)]
    specs += [('short', .1, 14400, None), ('tail_burst', 10**(-60/20), 96000, (95520,96000))]
    rows = []
    for name, amplitude, frames, interval in specs:
        signal = amplitude*np.sin(2*np.pi*1000*np.arange(frames)/48000)
        if interval is not None:
            signal[:interval[0]] = 0.
            signal[interval[1]:] = 0.
        samples = np.repeat(signal[:,None],2,axis=1).astype(np.float32)
        path = directory/(name+'.wav')
        sf.write(path,samples,48000,subtype='FLOAT')
        rows.append(dict(id=name,**bound(path),frames=frames,amplitude=amplitude,
                         active_interval=interval,sample_peak_dbfs=None if amplitude==0 else float(20*np.log10(np.abs(samples).max()))))
    return rows


def native(binary, directory, name, files):
    job = directory/(name+'-job.json')
    output = directory/(name+'-native.json')
    write(job,dict(version=VERSION,files=files,binary=bound(binary),protocol=bound(PROTOCOL)))
    start=time.perf_counter()
    proc=subprocess.run([str(binary),str(job),str(output)],capture_output=True,text=True)
    (directory/(name+'-native.log')).write_text(proc.stdout+'\n'+proc.stderr)
    assert proc.returncode==0,proc.stderr
    report=read(output)
    assert report['status']=='complete' and len(report['rows'])==len(files)
    assert report['job_sha256']==sha(job)
    return report,time.perf_counter()-start


def reference(directory, name, files, native_report):
    result=[]
    for index,file in enumerate(files):
        start=time.perf_counter()
        assert sha(Path(file['path']))==file['sha256']
        proc=subprocess.run([str(FFMPEG),'-hide_banner','-nostats','-i',file['path'],
            '-af','ebur128=peak=true','-f','null','-'],capture_output=True,text=True,check=True)
        log=directory/f'{name}-{index:02d}-lufs.log'
        log.write_text(proc.stderr,encoding='utf-8')
        independent=classify(proc.stderr,file['frames'])
        meter=next(r for r in native_report['rows'] if r['id']==file['id'])
        assert meter['sha256']==file['sha256'] and meter['frames']==file['frames']
        item=dict(id=file['id'],file=file,native=meter,independent=independent,
            agreement=agreement(meter['integrated_state'],meter['lufs'],independent),
            log=bound(log),reference_s=time.perf_counter()-start)
        result.append(item)
        write(directory/(name+'-reference.json'),dict(status='running',rows=result))
        print(name,file['id'],meter['integrated_state'],independent['state'],item['agreement'],flush=True)
    write(directory/(name+'-reference.json'),dict(status='complete',rows=result))
    return result


def main():
    parser=argparse.ArgumentParser()
    parser.add_argument('--binary',type=Path,required=True)
    parser.add_argument('--output',type=Path,required=True)
    args=parser.parse_args()
    assert not args.output.exists()
    assert sha(FFMPEG)==FFMPEG_SHA
    assert shutil.disk_usage(args.output.parent).free>25*1024**3
    baseline=read(BASE)
    for item in baseline['inputs'].values():
        assert sha(Path(item['path']))==item['sha256']
    assert baseline['independent_failures']==['metal-copy-80-control-offset0']
    directory=args.output.resolve()
    directory.mkdir()
    binary=directory/'meter-v1.exe'
    shutil.copy2(args.binary,binary)
    controls=fixtures(directory)
    start=time.perf_counter()
    control_meter,control_native_s=native(binary,directory,'controls',controls)
    controlled=reference(directory,'controls',controls,control_meter)
    unknown=[]
    for item in controlled:
        if item['independent']['state']=='unknown':
            assert item['id'] in ('short','tone_-70.04','tone_-69.96'),item
            unknown.append(item['id'])
            assert not item['agreement']
        else:
            assert item['agreement'],item
        if item['native']['integrated_state']=='unavailable':
            assert not agreement('finite',-70.,item['independent'])
    assert next(r for r in controlled if r['id']=='burst_quiet')['independent']['state']=='unavailable'
    assert next(r for r in controlled if r['id']=='short')['independent']['state']=='unknown'
    control_wall_s=time.perf_counter()-start
    files=[]
    for item in baseline['rows']:
        row=item['native']
        assert item['independent']['peak_pass'] and item['independent']['fullscale_samples']==0
        assert sha(Path(row['path']))==row['sha256']
        info=sf.info(row['path'])
        assert (info.frames,info.samplerate,info.channels,info.subtype)==(row['frames'],48000,2,'FLOAT')
        files.append(dict(id=row['id'],path=row['path'],sha256=row['sha256'],frames=row['frames']))
    retained_meter,retained_native_s=native(binary,directory,'retained',files)
    retained=reference(directory,'retained',files,retained_meter)
    for row,previous in zip(retained,baseline['rows'],strict=True):
        assert row['id']==previous['native']['id']
        old_native=previous['native']['lufs']
        current=row['native']['lufs']
        assert (old_native is None)==(current is None)
        if current is not None:
            assert abs(current-old_native)<=.001,(row['id'],current,old_native)
        row['retained_peak_format_evidence']=previous['independent']
    result=dict(status='complete',classifier=VERSION,protocol=bound(PROTOCOL),baseline=bound(BASE),
        binary=bound(binary),ffmpeg=bound(FFMPEG),scripts={name:bound(Path(__file__).parent/name)
            for name in ('block_availability.py','block_availability_research.py')},
        controls=controlled,control_unknowns=unknown,retained=retained,
        retained_agreements=sum(r['agreement'] for r in retained),
        retained_unavailable=sum(r['independent']['state']=='unavailable' for r in retained),
        retained_unknowns=[r['id'] for r in retained if r['independent']['state']=='unknown'],
        control_native_wall_s=control_native_s,control_total_wall_s=control_wall_s,
        retained_native_wall_s=retained_native_s,
        retained_reference_work_s=sum(r['reference_s'] for r in retained),
        scope='Additive availability diagnostic only; old qualification and selector unchanged')
    write(directory/'result-v1.json',result)
    print('Retained agreements:',result['retained_agreements'],'unknown:',result['retained_unknowns'])


if __name__=='__main__':
    main()
