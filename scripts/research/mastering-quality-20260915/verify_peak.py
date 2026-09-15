"""Independent 4x/16x soxr reconstruction; no normalization or limiter."""
from pathlib import Path
import argparse,json,re,subprocess
P=Path(__file__).resolve().parent
parser=argparse.ArgumentParser();parser.add_argument('--output',type=Path,default=P/'peak-soxr-verification.json');args=parser.parse_args()
assert not args.output.exists(), ('Use a new output path',args.output)
rows=[]
for file in ['finalists/funk/continuous_drive_t14.wav','finalists/imaginal/adapt0_t14.wav','finalists/imaginal/control_t14.wav']:
    for rate in [192000,768000]:
        log=subprocess.run(['ffmpeg','-hide_banner','-nostats','-i',str(P/file),'-af',f'aresample={rate}:resampler=soxr:precision=33,astats=metadata=0:reset=0','-f','null','-'],capture_output=True,text=True,check=True).stderr
        peak=float(re.findall(r'Peak level dB: ([-\d.]+)',log)[-1])
        rows.append(dict(file=file,rate=rate,peak=peak))
args.output.write_text(json.dumps(rows,indent=2));print(rows)
