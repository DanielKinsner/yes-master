"""Hash-then-delete intermediate renders whose measurements are already in results/."""
import hashlib, os, json, glob, sys
def sha(p):
    h = hashlib.sha256()
    with open(p, 'rb') as f:
        for c in iter(lambda: f.read(1 << 20), b''):
            h.update(c)
    return h.hexdigest()
patterns = sys.argv[1:]
rec_path = 'results/deleted_renders_hashes.json'
rec = json.load(open(rec_path)) if os.path.exists(rec_path) else {}
freed = 0; n = 0
for pat in patterns:
    for p in glob.glob(pat):
        key = p.replace(os.sep, '/')
        rec[key] = {'sha256': sha(p), 'bytes': os.path.getsize(p)}
        freed += os.path.getsize(p); n += 1
        os.remove(p)
json.dump(rec, open(rec_path, 'w'), indent=1)
print(f"hashed and deleted {n} files, {freed/1e9:.1f} GB; hashes retained in {rec_path}")
