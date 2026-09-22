"""Table: current (0) vs candidate A / B per source and preset."""
import json
import os
import sys

AUDIT = os.path.abspath(os.path.join(os.path.dirname(__file__), ".."))
which = sys.argv[1] if len(sys.argv) > 1 else "dev"
rows = json.load(open(os.path.join(AUDIT, "results", f"candidate_{which}.json")))
by = {}
for r in rows:
    if "error" in r:
        print("ERR", r["id"], r["error"][:200])
        continue
    by.setdefault((r["source"], r["preset"]), {})[r["variant"]] = r

print(f"{'source/preset':24s} {'var':>3} {'pregain':>7} {'LUFS':>7} {'TP':>6} {'crest':>6} {'psr':>6} {'LRA':>6} {'p95p10':>6} {'specdev':>7} notes")
agg = {"A": [], "B": []}
for (s, p), v in sorted(by.items()):
    base = v.get("0")
    for var in ("0", "A", "B"):
        r = v.get(var)
        if not r:
            continue
        note = ",".join(c for c in r["checks"] if c in ("target_not_reached", "true_peak_high", "streaming_headroom_low"))
        print(f"{s+'/'+p:24s} {var:>3} {r['pregain_db']:+7.2f} {r['lufs']:7.2f} {r['tp']:6.2f} {r['crest_db']:6.2f} {r['psr_db']:6.2f} {r['lra']:6.2f} {r['p95_p10_db']:6.2f} {r['spec_dev_db']:7.2f} {note}")
        if var != "0" and base:
            agg[var].append({
                "id": f"{s}/{p}",
                "d_crest": r["crest_db"] - base["crest_db"], "d_psr": r["psr_db"] - base["psr_db"],
                "d_lra": r["lra"] - base["lra"], "d_lufs_err": abs(r["lufs"] + 14) - abs(base["lufs"] + 14),
                "d_specdev": r["spec_dev_db"] - base["spec_dev_db"], "tp_ok": r["tp"] <= -0.9,
            })
for var, lst in agg.items():
    if not lst:
        continue
    print(f"\n== candidate {var} vs current, per case deltas (positive crest/psr/lra = more dynamics preserved; negative lufs_err = closer to target)")
    for a in lst:
        print(f"  {a['id']:24s} crest {a['d_crest']:+5.2f}  psr {a['d_psr']:+5.2f}  LRA {a['d_lra']:+5.2f}  |LUFS err| {a['d_lufs_err']:+5.2f}  specdev {a['d_specdev']:+5.2f}  tp_ok {a['tp_ok']}")
    n = len(lst)
    print(f"  cases with crest preserved better: {sum(a['d_crest']>0.05 for a in lst)}/{n}; closer to target: {sum(a['d_lufs_err']<-0.05 for a in lst)}/{n}; further from target: {sum(a['d_lufs_err']>0.05 for a in lst)}/{n}; LRA higher: {sum(a['d_lra']>0.05 for a in lst)}/{n}; spectral deviation lower: {sum(a['d_specdev']<-0.05 for a in lst)}/{n}; TP ok: {sum(a['tp_ok'] for a in lst)}/{n}")
