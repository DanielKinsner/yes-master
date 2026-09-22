from pathlib import Path
import hashlib,json,subprocess
P=Path(__file__).resolve().parent;root=P.parents[1]
src=next((root/'tests for presets').glob('*original-test.wav'))
variants=[dict(name='on',write=True),dict(name='off',comp='off',write=True),dict(name='chunk257',chunk=257,write=True)]
job=dict(source=str(src),output=str(P/'verified-reproduction'),variants=variants)
j=P/'verify-job.json';j.write_text(json.dumps(job))
subprocess.run([str(P/'target/release/yes-stage-ablation.exe'),str(j)],check=True)
proof={}
for v,owner in [('on','YES-compressor-on.wav'),('off','YES-compressor-off.wav'),('chunk257','YES-compressor-on.wav')]:
    actual=hashlib.file_digest((P/f'verified-reproduction/{v}.wav').open('rb'),'sha256').hexdigest()
    expected=hashlib.file_digest((root/'tests for presets'/owner).open('rb'),'sha256').hexdigest()
    assert actual==expected,(v,actual,expected);proof[v]=actual
(P/'reproduction-proof.json').write_text(json.dumps(proof,indent=2));print('PASS',proof)
