"""Hash-check and safely restore the September 15 private research supplement.

Standard library only. Existing differing files are refused; nothing is deleted.
Both roots are explicit, so this works outside the original Windows paths.
"""
from pathlib import Path, PurePosixPath
import argparse
import hashlib
import json
import os
import tempfile
import zipfile


def sha(path):
    with path.open('rb') as stream:
        return hashlib.file_digest(stream,'sha256').hexdigest()


def target_path(target, roots):
    relative=PurePosixPath(target)
    if relative.is_absolute() or len(relative.parts)<2 or any(p in {'.','..'} or ':' in p or '\\' in p for p in relative.parts):
        raise ValueError(f'Unsafe target: {target}')
    root=roots[relative.parts[0]]
    path=root.joinpath(*relative.parts[1:]).resolve()
    if not path.is_relative_to(root):
        raise ValueError(f'Target escapes root: {target}')
    return path


def restore(download, manifest_path, roots, verify_only=False):
    manifest=json.loads(manifest_path.read_text())
    assert manifest['schema']==1
    roots={k:Path(v).resolve() for k,v in roots.items()}
    expected_assets={a['name']:a for a in manifest['assets']}
    all_destinations=set()
    for item in manifest['files']:
        assert item['member']=='objects/'+item['sha256']
        assert item['asset'] in expected_assets
        for name in item['targets']:
            path=target_path(name,roots)
            if path in all_destinations: raise ValueError(f'Duplicate target: {name}')
            all_destinations.add(path)
            if not verify_only and path.exists() and (not path.is_file() or sha(path)!=item['sha256']):
                raise ValueError(f'Refusing to overwrite existing different file: {path}')
    for name, asset in expected_assets.items():
        if PurePosixPath(name).name!=name: raise ValueError(name)
        path=download/name
        if path.stat().st_size!=asset['bytes'] or sha(path)!=asset['sha256']:
            raise ValueError(f'Archive integrity failed: {name}')
    verified=restored=already_present=0
    for asset_name in expected_assets:
        with zipfile.ZipFile(download/asset_name) as archive:
            entries=[item for item in manifest['files'] if item['asset']==asset_name]
            names=archive.namelist()
            if len(names)!=len(set(names)) or set(names)!={e['member'] for e in entries}:
                raise ValueError(f'Unexpected or duplicate ZIP members: {asset_name}')
            for item in entries:
                with archive.open(item['member']) as source:
                    digest=hashlib.file_digest(source,'sha256').hexdigest()
                if digest!=item['sha256'] or archive.getinfo(item['member']).file_size!=item['bytes']:
                    raise ValueError(f'Member integrity failed: {item["member"]}')
                verified+=1
                if verify_only: continue
                for name in item['targets']:
                    destination=target_path(name,roots)
                    if destination.exists():
                        if sha(destination)!=digest: raise ValueError(f'File changed during restore: {destination}')
                        already_present+=1
                        continue
                    destination.parent.mkdir(parents=True,exist_ok=True)
                    # Exclusive creation: a concurrently created file cannot be replaced.
                    with archive.open(item['member']) as source, destination.open('xb') as output:
                        while chunk:=source.read(1<<20): output.write(chunk)
                    if sha(destination)!=digest: raise ValueError(f'Restored hash failed: {destination}')
                    restored+=1
        print(f'Checked {asset_name}: {verified} unique files so far',flush=True)
    result={'unique_files_verified':verified,'paths_restored':restored,
            'existing_identical_paths':already_present,'verify_only':verify_only,
            'assets_verified':len(expected_assets),'manifest_sha256':sha(manifest_path)}
    print(json.dumps(result,indent=2),flush=True)
    return result


if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--download-dir',required=True,type=Path)
    parser.add_argument('--manifest',type=Path)
    parser.add_argument('--repo',required=True,type=Path)
    parser.add_argument('--audit',required=True,type=Path)
    parser.add_argument('--verify-only',action='store_true')
    parser.add_argument('--proof',type=Path)
    args=parser.parse_args()
    result=restore(args.download_dir,args.manifest or args.download_dir/'RESEARCH_20260915_MANIFEST.json',
                   {'repo':args.repo,'audit':args.audit},args.verify_only)
    if args.proof:
        if args.proof.exists(): raise SystemExit('Refuse overwriting an existing proof')
        args.proof.write_text(json.dumps(result,indent=2)+'\n')
