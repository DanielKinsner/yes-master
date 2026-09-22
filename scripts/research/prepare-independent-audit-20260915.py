"""Coordinator-only preparation. Do not include this script in the review snapshot."""
from pathlib import Path,PurePosixPath
import hashlib,json,subprocess,zipfile,shutil

root=Path(__file__).resolve().parents[2]
dest=root.parent/'yes-master-independent-audit-20260915'
baseline='a4fb621d88a95b8af549467fb499943acb4274d5'
assert not dest.exists(), ('Choose a fresh review directory',dest)
archive=root/'test-output/independent-source-snapshot-20260915.zip'
assert not archive.exists()
roots=['src','src-tauri','apps','web','public','index.html','package.json','package-lock.json','tsconfig.json','tsconfig.node.json','vite.config.ts','LICENSE','THIRD_PARTY_NOTICES.md']
subprocess.run(['git','archive','--format=zip',f'--output={archive}',baseline,'--',*roots],cwd=root,check=True)
dest.mkdir();records=[]
with zipfile.ZipFile(archive) as z:
    for entry in z.infolist():
        if entry.is_dir():continue
        rel=PurePosixPath(entry.filename)
        assert not rel.is_absolute() and '..' not in rel.parts
        # Retain production code, build inputs and legal notices, not prose reviews.
        if any(part.lower() in ['docs','test-output','research','graphify-out'] for part in rel.parts):continue
        if rel.suffix.lower()=='.md' and rel.name not in ['THIRD_PARTY_NOTICES.md','LICENSE.md']:continue
        if rel.name in ['AGENTS.md','CLAUDE.md']:continue
        data=z.read(entry);target=dest/rel;target.parent.mkdir(parents=True,exist_ok=True);target.write_bytes(data)
        records.append(dict(file=rel.as_posix(),sha256=hashlib.sha256(data).hexdigest()))
(dest/'SOURCE_SNAPSHOT.json').write_text(json.dumps(dict(baseline_git_revision=baseline,source='Canonical Git file bytes; line endings may differ from the working checkout. No implementation edits.',files=records),indent=2))

prompt=(root/'docs/prompts/2026-09-15-independent-mastering-audit.md').read_bytes()
(dest/'START_REVIEW.md').write_bytes(prompt)
instructions=b'''# Independent YES Master review workspace

Use START_REVIEW.md as the current user task. This source snapshot has no Git
history and is deliberately separate from the active application checkout.
Stay inside this workspace for YES source, fixtures and project context. Do not
open other YES folders, previous project memories, research notes or task history.
Public primary technical references and appropriately licensed new inputs are
permitted. No previous result is an acceptance criterion for this assessment.

Read REVIEW_INPUTS.md and fixtures/manifest.json. Treat production source as
read-only and write experiments/results under audit-output/. Preserve defaults,
preset calibration, source files and disabled feature gates. Do not deploy,
publish, upload audio, purchase services or change the active application.

Design your own measurements and isolated harness using the real implementation.
The snapshot omits prose project documentation and general automation scripts;
their absence is a packaging limit of this review, not an application defect.
It is suitable for source and native DSP investigation; full desktop packaging
may require additional build assets. Do not turn this into a packaging audit.
If a specific source/build input is missing, request that input alone without
requesting earlier conclusions or broad project-history access.

When finished, seal your protocol, independent report and evidence manifest as
specified by START_REVIEW.md. Any later reconciliation belongs in an addendum.
AGENTS.md and CLAUDE.md must remain byte-identical.
'''
(dest/'AGENTS.md').write_bytes(instructions);(dest/'CLAUDE.md').write_bytes(instructions)

def sha(p):
    with p.open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest()
out=root/'test-output/mastering-quality-20260915'
corpus=json.loads((out/'corpus.json').read_text())['sources'];rows=[]
(dest/'fixtures/inputs').mkdir(parents=True)
(dest/'fixtures/baseline').mkdir()
for index,s in enumerate(corpus,1):
    original=Path(s['path']);original=original if original.is_absolute() else out/original
    assert sha(original)==s['sha256']
    rel=f'fixtures/inputs/input{index:02}.wav';target=dest/rel
    shutil.copyfile(original,target);assert sha(target)==s['sha256']
    row=dict(id=f'input{index:02}',file=rel,sha256=s['sha256'],title=s['title'],artist=s['artist'],kind=s['kind'],license=s['license'],preparation=s['changes'])
    if s.get('page'):row['credit_url']=s['page']
    if s['id']=='rich':row['additional_credit_url']='https://ccmixter.org/files/adisa/28618'
    rows.append(row)
baseline_source=root/'tests for presets/YES-compressor-on.wav'
shutil.copyfile(baseline_source,dest/'fixtures/baseline/export01.wav')
reference=dict(file='fixtures/baseline/export01.wav',source_id='input01',sha256=sha(baseline_source),purpose='Production export reproduction control; not a preferred-sound target.',settings=dict(preset='Universal',intensity=0.75,compressor_mode='Preset',compression_density='Auto (named-preset default)',adapt_strength=0.5,delivery_profile='Custom',lufs_target=-9,true_peak_ceiling_dbtp=-1,output_sample_rate_hz=48000,output_format='WAV PCM24',volume_match=False,input_gain_db=0,output_gain_db=0,manual_eq='Neutral',other_advanced_settings='Defaults'))
(dest/'fixtures/manifest.json').write_text(json.dumps(dict(inputs=rows,baseline_exports=[reference]),indent=2),encoding='utf8')
(dest/'REVIEW_INPUTS.md').write_text('''# Review inputs

This folder supplies a production source snapshot, eight audio inputs and one
recorded production export. Source hashes and the baseline revision are in
SOURCE_SNAPSHOT.json. The supplied source has no intentional implementation edits.
The repository history, project reports and earlier experiment tooling are absent.

The fixtures manifest records source identity, licenses, preparation and baseline
export settings. It contains no expected audio measurements or preferred candidate.
The baseline export exists to check faithful engine reproduction, not to dictate
the audit conclusion. Do not infer current defect status from fixture selection.

Four public inputs are finished MP3 downloads decoded to floating-point WAV;
three are constructed sums of supplied stems. The latter are not artist-approved
premaster balances and may contain previously processed individual stems. One
input is owner-supplied, with earlier processing history unspecified. This is a
convenience set with limited creator coverage, not a representative population.
Original creator credit and license links are in fixtures/manifest.json. Sources
were decoded without clipping float headroom; the stem sums use the documented
scalar preparation. No creator endorsement of YES or experimental outputs is implied.

The supplied baseline was generated on Windows. Use existing local Rust, Python,
FFmpeg/FFprobe and other standard tools as needed, recording exact versions.
The application dependency locks are included. Measure with independent tools
where useful and investigate disagreements instead of assuming either tool is right.

Public licensing permits the documented local analysis/adaptations with credit;
the owner source/export is authorized only for this private investigation. No
public distribution, third-party audio upload or paid service is authorized.

Create audit-output/ for your own protocol, experiments and sealed report.
Follow START_REVIEW.md. Do not consult another YES workspace for context.
''',encoding='utf8')
assert not (dest/'.git').exists()
assert (dest/'AGENTS.md').read_bytes()==(dest/'CLAUDE.md').read_bytes()
print(json.dumps(dict(workspace=str(dest),production_files=len(records),audio_inputs=len(rows),baseline_exports=1,source_bytes=sum((dest/r['file']).stat().st_size for r in records),audio_bytes=sum((dest/r['file']).stat().st_size for r in rows)+(dest/reference['file']).stat().st_size),indent=2))
