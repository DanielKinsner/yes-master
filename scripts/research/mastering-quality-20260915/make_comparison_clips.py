"""Optional targeted comparisons, equal excerpt LUFS using gain alone."""
import json,re,subprocess
from pathlib import Path
from analyze_music import OUT
folder=OUT/'optional-comparisons';folder.mkdir(exist_ok=True);rows=[]
for source,start in [('metal',60),('piano',60)]:
    for variant in ['control','drive']:
        input=OUT/f'finalists/{source}/{variant}_t14.wav'
        cmd=['ffmpeg','-hide_banner','-nostats','-ss',str(start),'-t','20','-i',str(input)]
        log=subprocess.run(cmd+['-af','ebur128=peak=true','-f','null','-'],capture_output=True,text=True,check=True).stderr
        summary=log.rsplit('Summary:',1)[-1];lufs=float(re.search(r'I:\s*([-\d.]+)',summary)[1]);gain=-18-lufs
        output=folder/f'{source}_{variant}_minus18.wav'
        if not output.exists():subprocess.run(cmd+['-af',f'volume={gain}dB','-c:a','pcm_f32le',str(output)],capture_output=True,check=True)
        log=subprocess.run(['ffmpeg','-hide_banner','-nostats','-i',str(output),'-af','ebur128=peak=true','-f','null','-'],capture_output=True,text=True,check=True).stderr
        summary=log.rsplit('Summary:',1)[-1];achieved=float(re.search(r'I:\s*([-\d.]+)',summary)[1]);peak=float(re.search(r'Peak:\s*([-\d.]+)',summary)[1]);assert abs(achieved+18)<=.1 and peak<-1
        rows.append(dict(source=source,variant=variant,start=start,duration=20,excerpt_input_lufs=lufs,gain_only_db=gain,achieved_lufs=achieved,tp=peak,file=output.name))
(folder/'manifest.json').write_text(json.dumps(rows,indent=2))
(folder/'README.md').write_text('''# Optional targeted comparisons

No response or listening ranking is needed to complete the engineering recommendation.
All four excerpts cover 60-80 seconds, measure -18.0 LUFS, and use scalar gain only.
No added limiter, EQ, loudness normalization processor, fades, or time alignment.

- Metalmania: compare repeated drum attacks and dense guitar sustain. This tests
  the cost in dense preset character when targeting -14 with less input drive.
- In This Moment: compare piano attacks and decay. This tests whether the gentler
  -14 candidate loses desired density despite retaining more measured contrast.

These do not decide the -9 piano tradeoff, validate aliases by ear, or certify a
perceptual winner. Labels deliberately identify the engineering comparison.

Credits: Metalmania by Kevin MacLeod (incompetech.com), CC BY 4.0;
In This Moment by Scott Buckley (www.scottbuckley.com.au), CC BY 4.0.
License: https://creativecommons.org/licenses/by/4.0/ .
Changes: decoded original MP3, experimental YES mastering, excerpting and scalar
playback gain. The original creators do not endorse YES or these experiments.
''')
print(rows)
