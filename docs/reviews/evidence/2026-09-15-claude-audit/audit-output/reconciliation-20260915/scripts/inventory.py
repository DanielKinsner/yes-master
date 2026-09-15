"""Cleanup inventory (no deletion): sizes of reproducible build caches,
duplicate inputs across the two workspaces, unique evidence, and the minimum
archive needed to replicate on another machine. Writes results/cleanup_inventory.json
and prints a table."""
import hashlib
import json
import os

RECON = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
AUDIT = os.path.dirname(RECON)
ROOT = os.path.dirname(AUDIT)
MAIN = os.path.join(os.path.dirname(ROOT), "yes-master")
CODEX = os.path.join(MAIN, "test-output", "mastering-quality-20260915")


def du(path):
    total = 0
    for dp, _, fns in os.walk(path):
        for fn in fns:
            try:
                total += os.path.getsize(os.path.join(dp, fn))
            except OSError:
                pass
    return total


def sha(path):
    h = hashlib.sha256()
    with open(path, "rb") as f:
        for chunk in iter(lambda: f.read(1 << 20), b""):
            h.update(chunk)
    return h.hexdigest()


gb = lambda b: round(b / 1e9, 2)
inv = {"reproducible_build_caches": {}, "duplicate_inputs": [], "unique_evidence": {}, "minimum_archive": [], "intermediate_renders_retained": {}}

for name, p in {
    "audit-output/build/target (blind harness cargo target)": os.path.join(AUDIT, "build", "target"),
    "audit-output/reconciliation-20260915/build/target (recon harness cargo target)": os.path.join(RECON, "build", "target"),
    "yes-master/test-output/mastering-quality-20260915/target (Codex cargo target)": os.path.join(CODEX, "target"),
    "yes-master/test-output/mastering-quality-20260915/pylibs + venv (Codex python libs)": os.path.join(CODEX, "pylibs"),
}.items():
    if os.path.exists(p):
        inv["reproducible_build_caches"][name] = gb(du(p))

# duplicate inputs: fixtures vs Codex sources vs owner corpus
pairs = []
codex_sources = os.path.join(CODEX, "sources")
fixtures = {f"input0{i}": os.path.join(ROOT, "fixtures", "inputs", f"input0{i}.wav") for i in range(1, 9)}
fx_hash = {k: sha(v) for k, v in fixtures.items()}
if os.path.isdir(codex_sources):
    for fn in os.listdir(codex_sources):
        p = os.path.join(codex_sources, fn)
        if fn.lower().endswith(".wav"):
            h = sha(p)
            for k, v in fx_hash.items():
                if v == h:
                    inv["duplicate_inputs"].append({"fixture": f"fixtures/inputs/{k}.wav", "duplicate": f"yes-master/test-output/mastering-quality-20260915/sources/{fn}", "bytes": os.path.getsize(p)})
owner = os.path.join(MAIN, "tests for presets")
if os.path.isdir(owner):
    for dp, _, fns in os.walk(owner):
        for fn in fns:
            if fn.lower().endswith(".wav"):
                p = os.path.join(dp, fn)
                if os.path.getsize(p) == os.path.getsize(fixtures["input01"]) and sha(p) == fx_hash["input01"]:
                    inv["duplicate_inputs"].append({"fixture": "fixtures/inputs/input01.wav", "duplicate": os.path.relpath(p, MAIN).replace("\\", "/"), "bytes": os.path.getsize(p)})
# blind-phase gain/rate variants derivable from fixtures
inv["derivable_variants"] = {"audit-output/variants": gb(du(os.path.join(AUDIT, "variants"))), "audit-output/synthetic": gb(du(os.path.join(AUDIT, "synthetic"))),
                            "audit-output/reconciliation-20260915/renders/policy_inputs": gb(du(os.path.join(RECON, "renders", "policy_inputs")))}

inv["unique_evidence"] = {
    "blind sealed set (18 files) + results/ + specs/ + cases/": gb(du(os.path.join(AUDIT, "results")) + du(os.path.join(AUDIT, "specs")) + du(os.path.join(AUDIT, "cases")) + os.path.getsize(os.path.join(AUDIT, "INDEPENDENT_REPORT.md")) + os.path.getsize(os.path.join(AUDIT, "PROTOCOL.md"))),
    "blind retained renders (audit-output/renders)": gb(du(os.path.join(AUDIT, "renders"))),
    "reconciliation results/ refs/ scripts/ harness src": gb(du(os.path.join(RECON, "results")) + du(os.path.join(RECON, "refs")) + du(os.path.join(RECON, "scripts")) + du(os.path.join(RECON, "harness", "src"))),
    "reconciliation retained renders": gb(du(os.path.join(RECON, "renders"))),
    "Codex compact evidence (docs/reviews/evidence, in git)": gb(du(os.path.join(MAIN, "docs", "reviews", "evidence", "2026-09-15-mastering-quality"))),
    "Codex full private evidence (test-output/mastering-quality-20260915, ignored)": gb(du(CODEX)),
}
inv["minimum_archive"] = [
    "fixtures/ (8 inputs + baseline export01.wav + manifest.json), REVIEW_INPUTS.md, SOURCE_SNAPSHOT.json, START_REVIEW.md, FOLLOWUP_REVIEW.md",
    "src-tauri/ source snapshot (or the yes-master checkout at a4fb621)",
    "audit-output/: INDEPENDENT_REPORT.md, PROTOCOL.md, SEALED_MANIFEST.json, EVIDENCE_MANIFEST.json, harness/ (src + Cargo.*), scripts/, specs/, cases/, results/",
    "audit-output/reconciliation-20260915/: RECONCILIATION.md, SEALED_MANIFEST.json, EVIDENCE_MANIFEST.json, harness/ (src + Cargo.*), scripts/, results/, refs/",
    "toolchain notes: Rust 1.95.0 with the locked crates (cargo fetches them), Python 3.13 + numpy/scipy/soundfile, FFmpeg 7.1.1 with soxr",
    "NOT needed: build/target trees (rebuild), renders/ (reproducible from cases/ and scripts; hashes in results/deleted_renders_hashes.json), variants/ and synthetic/ (regenerated by the blind scripts), Codex target/ and pylibs/",
]
json.dump(inv, open(os.path.join(RECON, "results", "cleanup_inventory.json"), "w"), indent=1)
print(json.dumps(inv, indent=1))
