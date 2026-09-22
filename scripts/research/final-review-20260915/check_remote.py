"""Verify draft status and GitHub asset digests against the committed manifest."""
from pathlib import Path
import hashlib
import json
import subprocess

ROOT=Path(__file__).resolve().parents[3]
OUT=ROOT/'docs/reviews/evidence/2026-09-15-final-review'


def main():
    path=OUT/'RESEARCH_20260915_MANIFEST.json'
    manifest=json.loads(path.read_text())
    release=json.loads(subprocess.check_output(['gh','release','view','research-transfer-2026-09-14',
        '--repo','DanielKinsner/yes-master','--json','isDraft,tagName,assets'],text=True))
    assert release['isDraft'] is True
    assets={a['name']:a for a in release['assets']}
    checked=[]
    for entry in manifest['assets']:
        actual=assets[entry['name']]
        assert actual['size']==entry['bytes']
        assert actual['digest']=='sha256:'+entry['sha256']
        checked.append({'name':entry['name'],'bytes':actual['size'],'sha256':entry['sha256']})
    manifest_digest=hashlib.sha256(path.read_bytes()).hexdigest()
    assert assets['RESEARCH_20260915_MANIFEST.json']['digest']=='sha256:'+manifest_digest
    assert assets['YES-Master-private-research-2026-09-14.zip']['digest']==\
        'sha256:a76a3ae0a95751df0c91e747a363c776ad073cb5ece22bfb54acd6de7e3b8c53'
    result={'tag':release['tagName'],'is_draft':release['isDraft'],
            'checked_assets':checked,'manifest_sha256':manifest_digest,
            'original_owner_archive_digest_unchanged':True}
    (OUT/'github-assets.json').write_text(json.dumps(result,indent=2)+'\n')
    print(json.dumps(result,indent=2))


if __name__=='__main__':main()
