"""Freeze B4 stimuli before encoding; no source or older evidence is modified."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess
import numpy as np
import soundfile as sf


def sha(path):
    with path.open('rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest()


def encodings(rate):
    result = [{'format': 'flac'}, {'format': 'aiff'}]
    if rate in (32000, 44100, 48000):
        result += [{'format': 'mp3', 'bitrate_kbps': b} for b in (128, 192, 256, 320)]
    if rate in (44100, 48000):
        result += [{'format': f, 'bitrate_kbps': b} for f in ('m4a', 'aac') for b in (128, 192, 256, 320)]
        result += [{'format': 'ogg', 'quality': q} for q in (4, 6, 8)]
    return result


def main():
    p = argparse.ArgumentParser()
    for name in ('music', 'ffmpeg', 'output'):
        p.add_argument('--' + name, type=Path, required=True)
    a = p.parse_args()
    assert not a.output.exists()
    assert sha(a.music) == '639ac1dd5d049040d8b689636c08bf2890b39929b72601703d1b93938f6022ce'
    a.output.mkdir(parents=True)
    rows = []
    for kind, rates in [('stress', (32000,44100,48000,88200,96000,176400,192000)),
                        ('short', (32000,44100,48000)), ('music', (32000,44100,48000,96000))]:
        for rate in rates:
            for channels in (1,2):
                identity = f'{kind}-{rate}-{channels}'
                path = a.output / f'{identity}.wav'
                if kind == 'music':
                    subprocess.run([str(a.ffmpeg), '-v','error','-nostdin','-i',str(a.music),
                        '-ss','50','-t','8','-ac',str(channels),'-af',
                        f'aresample={rate}:resampler=soxr:precision=28', '-c:a','pcm_f32le',str(path)],check=True)
                    transform = 'frozen owner On master, seconds 50-58; FFmpeg channel mapping and SOXR precision28'
                else:
                    n = np.arange(round(rate*(.05 if kind == 'short' else 4.)), dtype=np.float64)
                    values = []
                    for channel in range(channels):
                        rng = np.random.default_rng(150926 + channel)
                        if kind == 'short':
                            x = np.sin(2*np.pi*.241*n + np.pi/4 + channel*.7)
                            x[-1] = -1 if channel else 1
                        else:
                            t = n/rate
                            noise = rng.uniform(-1,1,len(n))
                            x = .4*np.sin(2*np.pi*(997+channel*131)*t) + .25*np.sin(2*np.pi*.43*n+channel)
                            x += noise*(.07+.6*np.exp(-300*np.remainder(t+channel*.013,.37)))
                            x[-8:] += .8*(-1.)**np.arange(8)
                        values.append(x)
                    x = np.stack(values,axis=1)
                    x *= 2/np.max(np.abs(x))
                    sf.write(path,x.astype(np.float32),rate,subtype='FLOAT')
                    transform = 'deterministic synthetic float stimulus, global sample peak +6.0206 dBFS; formula in generator'
                rows.append({'id':identity,'path':str(path.resolve()),'sha256':sha(path),
                    'transform':transform,'encodings':encodings(rate)})
    (a.output/'inputs.json').write_text(json.dumps(rows,indent=2),encoding='utf-8')
    version = subprocess.run([str(a.ffmpeg),'-version'],capture_output=True,text=True,check=True).stdout
    (a.output/'provenance.json').write_text(json.dumps({'music_sha256':sha(a.music),
        'ffmpeg_sha256':sha(a.ffmpeg),'ffmpeg_version':version,'stimuli':len(rows),
        'scope':'declared codec/rate/bitrate characterization; full transformed inputs, not full-song mastering/listening proof'},indent=2),encoding='utf-8')
    print(f'Frozen {len(rows)} stimuli: {a.output / "inputs.json"}')


if __name__ == '__main__':
    main()
