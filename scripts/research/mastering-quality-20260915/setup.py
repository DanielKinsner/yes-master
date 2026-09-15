"""Run after copying this package into a fresh ignored experiment directory."""
from pathlib import Path
import hashlib,json,subprocess,sys
P=Path(__file__).resolve().parent
assert P.parent.name=='test-output', 'Copy this package to a fresh test-output/<experiment> directory first.'
assert not (P/'source-provenance.json').exists(), 'Setup requires a fresh experiment directory.'
root=P.parents[1]
for record in json.loads((P/'expected-source.json').read_text())['records']:
    file=root/record['file'].replace('\\','/')
    assert hashlib.sha256(file.read_bytes()).hexdigest()==record['source_sha256'], ('Production source differs from investigated baseline',file)
subprocess.run([sys.executable,str(P/'prepare.py')],check=True)
(P/'src/main.rs').write_text('#![allow(dead_code)]\ninclude!("runner.rs");\n')
subprocess.run([sys.executable,str(P/'extend.py')],check=True)
head=(P/'src/runner.rs').read_text().split('fn main(){')[0].replace('use sha2::{Digest,Sha256};','')
for name,template in [('mechanisms','mechanisms-main.txt'),('probe','probe-main.txt'),('src_sweep','src-sweep-main.txt'),('src_check','src-check-main.txt'),('peak_meter','peak-meter-main.txt')]:
    (P/f'src/{name}.rs').write_text('#![allow(dead_code)]\n'+head+(P/template).read_text())
print('Generated private modules and probes; production files unchanged.')
