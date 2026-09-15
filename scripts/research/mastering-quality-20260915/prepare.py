from pathlib import Path
import hashlib,json,subprocess
out=Path(__file__).resolve().parent
root=out.parents[1]
records=[]
for name in ['confidence','guardrails']:
    p=root/f'src-tauri/src/{name}.rs'
    data=p.read_bytes()
    old=b'#[cfg(not(target_arch = "wasm32"))]'
    count=data.count(old)
    assert count==(2 if name=='confidence' else 5),(name,count)
    modified=data.replace(old,b'#[cfg(any())]')
    (out/f'src/{name}.rs').write_bytes(modified)
    records.append(dict(file=str(p),source_sha256=hashlib.sha256(data).hexdigest(),diagnostic_sha256=hashlib.sha256(modified).hexdigest(),transformation='Disable six existing non-WASM-only IPC wrappers and one related import using cfg(any()); no algorithm edits.',sections=count))
for name in ['dsp','types','analysis','deep_analysis','sample_rate','wav_writer','export_format']:
    p=root/f'src-tauri/src/{name}.rs'
    records.append(dict(file=str(p),source_sha256=hashlib.sha256(p.read_bytes()).hexdigest(),transformation='Direct path inclusion, unchanged'))
(out/'source-provenance.json').write_text(json.dumps(dict(head=subprocess.check_output(['git','rev-parse','HEAD'],text=True,cwd=root).strip(),records=records),indent=2))
