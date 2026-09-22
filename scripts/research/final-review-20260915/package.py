"""Preserve portable research text and selected private evidence. No deletions.

Run inventory first; build only into a new output directory. Research archives
are for the existing private DRAFT release, never Git or a published release.
"""
from pathlib import Path
import argparse
import hashlib
import json
import os
import shutil
import zipfile

ROOT = Path(__file__).resolve().parents[3]
AUDIT = ROOT.parent / 'yes-master-independent-audit-20260915'
EXPERIMENT = ROOT / 'test-output/mastering-quality-20260915'
FINAL = ROOT / 'test-output/mastering-quality-final-review-20260915'
PUBLIC = ROOT / 'docs/reviews/evidence/2026-09-15-claude-audit'
DEST = ROOT / 'test-output/research-transfer-20260915'
TEXT_EXT = {'.json','.csv','.txt','.log','.md','.py','.rs','.toml','.lock','.ps1'}
SKIP = {'target','pylibs','venv','__pycache__','.git','node_modules'}


def sha(path):
    with path.open('rb') as stream:
        return hashlib.file_digest(stream,'sha256').hexdigest()


def walk(root):
    for folder, dirs, names in os.walk(root):
        dirs[:] = [d for d in dirs if d not in SKIP]
        for name in names:
            yield Path(folder)/name


def collect():
    paths = set()
    for root in [EXPERIMENT, AUDIT/'audit-output']:
        for p in walk(root):
            if p.suffix.lower() in TEXT_EXT and 'refs' not in p.relative_to(root).parts:
                paths.add(p.resolve())
    snapshot = json.loads((AUDIT/'SOURCE_SNAPSHOT.json').read_text())
    paths.update((AUDIT/r['file']).resolve() for r in snapshot['files'])
    paths.update(p.resolve() for p in AUDIT.iterdir() if p.is_file())
    for root in [AUDIT/'fixtures',EXPERIMENT/'originals',EXPERIMENT/'sources',
                 AUDIT/'audit-output/synthetic',
                 AUDIT/'audit-output/reconciliation-20260915/results',
                 AUDIT/'audit-output/reconciliation-20260915/renders',FINAL/'witnesses']:
        paths.update(p.resolve() for p in walk(root) if p.suffix.lower() not in {'.pyc','.pdf'})
    # Every independently re-metered failing pilot plus current comparison set.
    for name in ['tp_meters_failing_set.json','tp_meters_codex_finalists.json']:
        for row in json.loads((AUDIT/'audit-output/reconciliation-20260915/results'/name).read_text()):
            p = Path(row['file'])
            if p.is_file():
                paths.add(p.resolve())
    for root in [EXPERIMENT/'optional-comparisons',
                 EXPERIMENT/'src-check',EXPERIMENT/'src-check-integer']:
        paths.update(p.resolve() for p in walk(root))
    for p in [AUDIT/'audit-output/build/target/release/yes-audit-harness.exe',
              AUDIT/'audit-output/reconciliation-20260915/build/target/release/yes-recon-harness.exe']:
        if p.exists(): paths.add(p.resolve())
    return sorted(paths)


def mapped(path):
    for root,label in [(AUDIT,'audit'),(ROOT,'repo')]:
        try:
            return label+'/'+path.relative_to(root.resolve()).as_posix()
        except ValueError:
            pass
    raise ValueError(path)


def preserve_text():
    entries=[]
    for p in walk(AUDIT/'audit-output'):
        if p.suffix.lower() not in TEXT_EXT or 'refs' in p.relative_to(AUDIT).parts:
            continue
        relative=p.relative_to(AUDIT)
        destination=PUBLIC/relative
        destination.parent.mkdir(parents=True,exist_ok=True)
        if destination.exists() and sha(destination)!=sha(p):
            raise RuntimeError(f'Refuse changing archived evidence {relative}')
        shutil.copyfile(p,destination)
        entries.append({'path':relative.as_posix(),'bytes':p.stat().st_size,'sha256':sha(p)})
    # Coordinator snapshot and fixture provenance, without the music itself.
    for name in ['SOURCE_SNAPSHOT.json','REVIEW_INPUTS.md','START_REVIEW.md','FOLLOWUP_REVIEW.md','fixtures/manifest.json']:
        p=AUDIT/name
        destination=PUBLIC/name
        destination.parent.mkdir(parents=True,exist_ok=True)
        shutil.copyfile(p,destination)
        entries.append({'path':name,'bytes':p.stat().st_size,'sha256':sha(p)})
    (PUBLIC/'ARCHIVED_FILES.json').write_text(json.dumps(entries,indent=2)+'\n')
    print('Archived original text byte-for-byte:',len(entries),flush=True)


def build(paths):
    if DEST.exists(): raise SystemExit('Choose a fresh package destination; preserve existing archive.')
    DEST.mkdir(parents=True)
    objects={}
    for p in paths:
        digest=sha(p)
        item=objects.setdefault(digest,{'sha256':digest,'bytes':p.stat().st_size,'targets':[],'source':p})
        item['targets'].append(mapped(p))
    print('Unique objects:',len(objects),'bytes:',sum(x['bytes'] for x in objects.values()),flush=True)
    # Size bound on uncompressed content means every asset stays below 2 GB.
    groups=[]
    current=[]
    size=0
    for item in objects.values():
        if current and size+item['bytes']>1_600_000_000:
            groups.append(current);current=[];size=0
        current.append(item);size+=item['bytes']
    if current: groups.append(current)
    assets=[]
    manifest={'schema':1,'source_baseline':'a4fb621d88a95b8af549467fb499943acb4274d5',
              'private_draft_tag':'research-transfer-2026-09-14',
              'scope':'Selected reproducibility bundle; excludes bulk derivable renders/build caches. Nothing deleted.',
              'files':[],'assets':assets}
    for i,items in enumerate(groups,1):
        name=f'YES-Master-research-2026-09-15-part{i:02d}.zip'
        archive=DEST/name
        with zipfile.ZipFile(archive,'w',compression=zipfile.ZIP_DEFLATED,compresslevel=1,allowZip64=True) as z:
            for item in items:
                member='objects/'+item['sha256']
                z.write(item['source'],member)
                manifest['files'].append({k:v for k,v in item.items() if k!='source'}|{'asset':name,'member':member})
        assets.append({'name':name,'bytes':archive.stat().st_size,'sha256':sha(archive)})
        print('Built',name,archive.stat().st_size,flush=True)
    (DEST/'RESEARCH_20260915_MANIFEST.json').write_text(json.dumps(manifest,indent=2)+'\n')
    print('Archive manifest ready',len(objects),'unique files',len(paths),'restore paths',flush=True)


if __name__=='__main__':
    parser=argparse.ArgumentParser()
    parser.add_argument('mode',choices=['inventory','text','build'])
    args=parser.parse_args()
    if args.mode=='text': preserve_text()
    else:
        files=collect()
        print(len(files),'files',sum(p.stat().st_size for p in files),'uncompressed bytes',flush=True)
        if args.mode=='build':build(files)
