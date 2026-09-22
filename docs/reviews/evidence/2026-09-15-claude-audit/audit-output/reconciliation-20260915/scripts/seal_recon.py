"""Seal the reconciliation separately from the blind phase: write
EVIDENCE_MANIFEST.json (hash of every retained reconciliation artefact and of
the external inputs relied on) and SEALED_MANIFEST.json (report, scripts,
harness source, evidence manifest). Also re-verifies the 18 original sealed
files and records that result. Run last."""
import datetime
import hashlib
import json
import os

RECON = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
AUDIT = os.path.dirname(RECON)
ROOT = os.path.dirname(AUDIT)
MAIN = os.path.join(os.path.dirname(ROOT), "yes-master")


def sha(path):
    h = hashlib.sha256()
    with open(path, "rb") as f:
        for chunk in iter(lambda: f.read(1 << 20), b""):
            h.update(chunk)
    return h.hexdigest()


def walk(base, exts=None, skip=()):
    out = {}
    for dp, dns, fns in os.walk(base):
        dns[:] = [d for d in dns if d not in skip]
        for fn in sorted(fns):
            if exts and not fn.lower().endswith(exts):
                continue
            p = os.path.join(dp, fn)
            out[os.path.relpath(p, ROOT).replace("\\", "/")] = {"sha256": sha(p), "bytes": os.path.getsize(p)}
    return out


# original seal re-verification
orig = json.load(open(os.path.join(AUDIT, "SEALED_MANIFEST.json")))
orig_check = {k: sha(os.path.join(ROOT, k)) == v for k, v in orig["files"].items()}

codex_evidence = os.path.join(MAIN, "docs", "reviews", "evidence", "2026-09-15-mastering-quality")
codex_files = {}
for fn in ["music-results.csv", "src-checks.json", "peak-verification.json", "src-regrowth.json", "mechanisms.json",
           "centered-validation.json", "robustness.json", "default-intensity.json", "corpus.json", "provenance.json"]:
    p = os.path.join(codex_evidence, fn)
    if os.path.exists(p):
        codex_files["yes-master/docs/reviews/evidence/2026-09-15-mastering-quality/" + fn] = {"sha256": sha(p), "bytes": os.path.getsize(p)}
for rel in ["docs/reviews/2026-09-15-mastering-quality-recommendation.md", "docs/reviews/2026-09-15-mastering-quality-protocol.md",
            "docs/reviews/2026-09-14-dynamics-stage-investigation.md", "scripts/research/mastering-quality-20260915/src/src_fixed.rs",
            "src-tauri/src/sample_rate.rs", "src-tauri/src/dsp.rs", "src-tauri/src/engine.rs", "src-tauri/Cargo.lock"]:
    p = os.path.join(MAIN, rel)
    if os.path.exists(p):
        codex_files["yes-master/" + rel] = {"sha256": sha(p), "bytes": os.path.getsize(p)}
finalists = os.path.join(MAIN, "test-output", "mastering-quality-20260915", "finalists")
for rel in ["imaginal/control_t14.wav", "imaginal/adapt0_t14.wav", "imaginal/drive_t14.wav", "imaginal/sat_off_t14.wav",
            "funk/continuous_drive_t14.wav", "funk/control_t14.wav", "funk/drive_t14.wav", "coat/control_t14.wav",
            "metal/control_t14.wav", "piano/control_t14.wav", "rich/control_t14.wav", "aphelion/control_t14.wav", "baby/control_t14.wav"]:
    p = os.path.join(finalists, rel)
    if os.path.exists(p):
        codex_files["yes-master/test-output/mastering-quality-20260915/finalists/" + rel] = {"sha256": sha(p), "bytes": os.path.getsize(p)}

rubato = None
for base in [os.path.expanduser("~/.cargo/registry/src")]:
    for dp, dns, fns in os.walk(base):
        if os.path.basename(dp) == "rubato-1.0.1" and "src" in dns:
            rubato = {"path": dp, "lib.rs_sha256": sha(os.path.join(dp, "src", "lib.rs"))}
            break

evidence = {
    "generated_at_utc": datetime.datetime.now(datetime.timezone.utc).isoformat(),
    "original_seal_verified": {"all_18_match": all(orig_check.values()), "per_file": orig_check, "sealed_at_utc": orig["sealed_at_utc"]},
    "fixture_inputs": {f"fixtures/inputs/input0{i}.wav": {"sha256": sha(os.path.join(ROOT, "fixtures", "inputs", f"input0{i}.wav"))} for i in range(1, 9)},
    "blind_harness_binary": {"audit-output/build/target/release/yes-audit-harness.exe": {"sha256": sha(os.path.join(AUDIT, "build", "target", "release", "yes-audit-harness.exe"))}},
    "recon_harness": walk(os.path.join(RECON, "harness"), (".rs", ".toml", ".lock")),
    "recon_harness_binary": {"audit-output/reconciliation-20260915/build/target/release/yes-recon-harness.exe": {"sha256": sha(os.path.join(RECON, "build", "target", "release", "yes-recon-harness.exe"))}},
    "scripts": walk(os.path.join(RECON, "scripts"), (".py",)),
    "results": walk(os.path.join(RECON, "results")),
    "refs": walk(os.path.join(RECON, "refs")),
    "retained_renders": walk(os.path.join(RECON, "renders"), (".wav", ".json")),
    "codex_materials_relied_on": codex_files,
    "rubato_locked_source": rubato,
}
with open(os.path.join(RECON, "EVIDENCE_MANIFEST.json"), "w") as f:
    json.dump(evidence, f, indent=1)

sealed_files = ["audit-output/reconciliation-20260915/RECONCILIATION.md", "audit-output/reconciliation-20260915/EVIDENCE_MANIFEST.json",
                "audit-output/reconciliation-20260915/harness/src/main.rs", "audit-output/reconciliation-20260915/harness/src/recon.rs",
                "audit-output/reconciliation-20260915/harness/src/codex_src_fixed.rs", "audit-output/reconciliation-20260915/harness/Cargo.toml",
                "audit-output/reconciliation-20260915/harness/Cargo.lock"]
sealed_files += sorted(evidence["scripts"])
sealed = {
    "sealed_at_utc": datetime.datetime.now(datetime.timezone.utc).isoformat(),
    "baseline_git_revision": json.load(open(os.path.join(ROOT, "SOURCE_SNAPSHOT.json")))["baseline_git_revision"],
    "original_blind_seal": {"file": "audit-output/SEALED_MANIFEST.json", "sha256": sha(os.path.join(AUDIT, "SEALED_MANIFEST.json")), "all_18_verified": all(orig_check.values())},
    "files": {k: sha(os.path.join(ROOT, k)) for k in sealed_files},
    "note": "Reconciliation seal, separate from the blind seal. The 18 blind-phase files were not modified.",
}
with open(os.path.join(RECON, "SEALED_MANIFEST.json"), "w") as f:
    json.dump(sealed, f, indent=1)
print(json.dumps(sealed, indent=1))
