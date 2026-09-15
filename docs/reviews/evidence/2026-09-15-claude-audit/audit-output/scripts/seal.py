"""Build EVIDENCE_MANIFEST.json (hash of every retained artefact) and
SEALED_MANIFEST.json (hashes of the report, protocol, scripts, harness source
and the evidence manifest itself). Run last."""
import hashlib
import json
import os
import sys
import datetime

AUDIT = os.path.abspath(os.path.join(os.path.dirname(__file__), ".."))
ROOT = os.path.dirname(AUDIT)


def sha(path):
    h = hashlib.sha256()
    with open(path, "rb") as f:
        for chunk in iter(lambda: f.read(1 << 20), b""):
            h.update(chunk)
    return h.hexdigest()


def walk(rel, exts=None):
    out = {}
    base = os.path.join(AUDIT, rel)
    for dp, _, fns in os.walk(base):
        for fn in sorted(fns):
            if exts and not fn.lower().endswith(exts):
                continue
            p = os.path.join(dp, fn)
            out[os.path.relpath(p, ROOT).replace("\\", "/")] = {"sha256": sha(p), "bytes": os.path.getsize(p)}
    return out


evidence = {
    "generated_at_utc": datetime.datetime.now(datetime.timezone.utc).isoformat(),
    "inputs": {k: {"sha256": sha(os.path.join(ROOT, k))} for k in [
        f"fixtures/inputs/input0{i}.wav" for i in range(1, 9)] + ["fixtures/baseline/export01.wav", "fixtures/manifest.json", "SOURCE_SNAPSHOT.json", "REVIEW_INPUTS.md", "START_REVIEW.md"]},
    "source_crate": {k: {"sha256": sha(os.path.join(ROOT, k))} for k in [
        "src-tauri/Cargo.lock", "src-tauri/Cargo.toml", "src-tauri/src/dsp.rs", "src-tauri/src/engine.rs",
        "src-tauri/src/guardrails.rs", "src-tauri/src/analysis.rs", "src-tauri/src/wav_writer.rs",
        "src-tauri/src/sample_rate.rs", "src-tauri/src/decode.rs", "src-tauri/src/exports.rs",
        "src-tauri/src/types.rs", "src-tauri/src/profile_store.rs", "src-tauri/src/confidence.rs"]},
    "harness": walk("harness", (".rs", ".toml", ".lock")),
    "scripts": walk("scripts", (".py",)),
    "specs_and_cases": {**walk("specs"), **walk("cases")},
    "results": walk("results"),
    "synthetic_inputs": walk("synthetic", (".wav",)),
    "variant_inputs": walk("variants", (".wav",)),
    "renders": walk("renders", (".wav", ".json")),
    "harness_binary": {"audit-output/build/target/release/yes-audit-harness.exe": {"sha256": sha(os.path.join(AUDIT, "build", "target", "release", "yes-audit-harness.exe"))}},
}
with open(os.path.join(AUDIT, "EVIDENCE_MANIFEST.json"), "w") as f:
    json.dump(evidence, f, indent=1)

sealed_files = ["audit-output/INDEPENDENT_REPORT.md", "audit-output/PROTOCOL.md", "audit-output/EVIDENCE_MANIFEST.json",
                "audit-output/harness/src/main.rs", "audit-output/harness/Cargo.toml", "audit-output/harness/Cargo.lock"]
sealed_files += sorted(k for k in evidence["scripts"])
sealed = {
    "sealed_at_utc": datetime.datetime.now(datetime.timezone.utc).isoformat(),
    "baseline_git_revision": json.load(open(os.path.join(ROOT, "SOURCE_SNAPSHOT.json")))["baseline_git_revision"],
    "files": {k: sha(os.path.join(ROOT, k)) for k in sealed_files},
    "note": "Any later reconciliation with other work is to be written as a separate addendum; these files are not to be edited after sealing.",
}
with open(os.path.join(AUDIT, "SEALED_MANIFEST.json"), "w") as f:
    json.dump(sealed, f, indent=1)
print(json.dumps(sealed, indent=1))
