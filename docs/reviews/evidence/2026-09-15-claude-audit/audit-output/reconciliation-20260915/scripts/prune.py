"""Hash-then-delete intermediate renders (disk is nearly full). Appends
{path, sha256, bytes} to results/deleted_renders_hashes.json so every
deleted file is reproducible and identifiable.

Usage: prune.py <glob> [<glob> ...]
"""
import glob
import hashlib
import json
import os
import sys

RECON = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
LEDGER = os.path.join(RECON, "results", "deleted_renders_hashes.json")


def sha(path):
    h = hashlib.sha256()
    with open(path, "rb") as f:
        for chunk in iter(lambda: f.read(1 << 20), b""):
            h.update(chunk)
    return h.hexdigest()


def prune(patterns):
    rows = json.load(open(LEDGER)) if os.path.exists(LEDGER) else []
    freed = 0
    for pat in patterns:
        for p in sorted(glob.glob(pat)):
            if not os.path.isfile(p):
                continue
            size = os.path.getsize(p)
            rows.append({"path": os.path.relpath(p, RECON).replace("\\", "/"), "sha256": sha(p), "bytes": size})
            os.remove(p)
            freed += size
    json.dump(rows, open(LEDGER, "w"), indent=1)
    return freed


if __name__ == "__main__":
    freed = prune(sys.argv[1:])
    print(f"pruned {freed/1e9:.2f} GB; ledger has {len(json.load(open(LEDGER)))} entries")
