"""Validate the final handoff's links, sealed bytes, source scope and helper syntax."""
from pathlib import Path
import ast
import hashlib
import json
import re
import subprocess

ROOT=Path(__file__).resolve().parents[3]
EVIDENCE=ROOT/'docs/reviews/evidence/2026-09-15-final-review'
CLAUDE=ROOT/'docs/reviews/evidence/2026-09-15-claude-audit'


def sha(path):return hashlib.sha256(path.read_bytes()).hexdigest()


def main():
    entries=json.loads((CLAUDE/'ARCHIVED_FILES.json').read_text())
    for row in entries:
        p=CLAUDE/row['path']
        assert p.stat().st_size==row['bytes'] and sha(p)==row['sha256'], row['path']
    index_output=subprocess.check_output(['git','ls-files','--stage','-z','--',
        'docs/reviews/evidence/2026-09-15-claude-audit'],cwd=ROOT)
    index={}
    for record in index_output.split(b'\0'):
        if record:
            metadata,path=record.split(b'\t',1)
            index[path.decode('utf-8')]=metadata.split()[1].decode('ascii')
    for row in entries:
        p=CLAUDE/row['path'];data=p.read_bytes()
        object_id=hashlib.sha1(b'blob '+str(len(data)).encode('ascii')+b'\0'+data).hexdigest()
        assert index.get(p.relative_to(ROOT).as_posix())==object_id, f'Missing/changed Git archive bytes: {p}'
    seals={}
    for name in ['audit-output/SEALED_MANIFEST.json','audit-output/reconciliation-20260915/SEALED_MANIFEST.json']:
        data=json.loads((CLAUDE/name).read_text())
        for path,digest in data['files'].items():assert sha(CLAUDE/path)==digest,path
        seals[name]=len(data['files'])
    assert (ROOT/'AGENTS.md').read_bytes()==(ROOT/'CLAUDE.md').read_bytes()
    scope=subprocess.check_output(['git','diff','--name-only','a4fb621','--','src','src-tauri'],cwd=ROOT,text=True).strip()
    assert not scope,scope
    new_documents=[ROOT/'docs/reviews'/name for name in [
        '2026-09-15-mastering-quality-final-synthesis.md',
        '2026-09-15-mastering-quality-final-verification.md',
        '2026-09-15-mastering-quality-transfer.md']]
    new_documents += [ROOT/'docs/prompts/2026-09-15-mastering-quality-planning-handoff.md',
                      CLAUDE/'README.md',ROOT/'scripts/research/final-review-20260915/README.md']
    link_count=0
    for p in new_documents:
        for target in re.findall(r'\]\(([^)]+)\)',p.read_text(encoding='utf-8')):
            if re.match(r'^[a-zA-Z]+:',target) or target.startswith('#'):continue
            target=target.split('#',1)[0]
            assert (p.parent/target).exists(),(p,target)
            link_count+=1
    paths=[p for root in [CLAUDE,ROOT/'scripts/research/final-review-20260915']
           for p in root.rglob('*') if p.is_file() and '__pycache__' not in p.parts]
    secret_patterns=[r'gh[pousr]_[A-Za-z0-9]{30,}',r'sk-[A-Za-z0-9]{30,}',
                     r'AKIA[A-Z0-9]{16}',r'xox[baprs]-[A-Za-z0-9-]{20,}',
                     r'-----BEGIN (?:RSA |OPENSSH )?PRIVATE KEY-----']
    python_count=0
    for path in paths:
        text=path.read_text(encoding='utf-8',errors='strict')
        assert not any(re.search(pattern,text) for pattern in secret_patterns),f'Secret-like pattern in {path}'
        if path.suffix=='.py':
            ast.parse(text,filename=str(path));python_count+=1
    manifest=json.loads((EVIDENCE/'RESEARCH_20260915_MANIFEST.json').read_text())
    verification=json.loads((EVIDENCE/'verification.json').read_text())
    assert verification['src_lengths']['prod']=={'cases':408,'exact_lengths':368}
    assert len(manifest['files'])==3543
    assert sum(len(r['targets']) for r in manifest['files'])==4507
    assert (EVIDENCE/'github-download-restore.json').exists()
    result={'archived_text_files_verified':len(entries),'archived_files_verified_in_git_index':len(entries),'seals_verified':seals,
            'application_diff_since_baseline':scope,'agent_instructions_identical':True,
            'local_links_checked':link_count,'python_files_parsed':python_count,
            'secret_pattern_scan_files':len(paths),'secret_pattern_matches':0,
            'research_only':True,'new_production_build_or_listening_claim':False}
    (EVIDENCE/'handoff-validation.json').write_text(json.dumps(result,indent=2)+'\n')
    print(json.dumps(result,indent=2))


if __name__=='__main__':main()
