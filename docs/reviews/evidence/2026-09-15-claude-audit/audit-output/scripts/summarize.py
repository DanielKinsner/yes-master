"""Compact tables from a ledger: python summarize.py <ledger> [--spectrum]"""
import json
import os
import sys

AUDIT = os.path.abspath(os.path.join(os.path.dirname(__file__), ".."))
name = sys.argv[1]
rows = json.load(open(os.path.join(AUDIT, "results", f"{name}.json")))
show_spec = "--spectrum" in sys.argv


def f(v, w=6, d=2):
    if v is None:
        return " " * (w - 3) + "n/a"
    try:
        return f"{v:{w}.{d}f}"
    except Exception:
        return str(v)[:w].rjust(w)


hdr = f"{'id':40s} {'srcL':>6} {'tgt':>6} {'rcL':>6} {'indL':>6} {'Lerr':>6} {'ceil':>5} {'rcTP':>6} {'ffTP':>6} {'ind16':>6} {'srcLRA':>6} {'LRA':>6} {'crest':>6} {'psr':>6} {'p95p10':>6} {'corr':>6} {'sm_dB':>6} {'lenD':>4} {'lag':>4} checks"
print(hdr)
for r in rows:
    if "error" in r:
        print(f"{r['id']:40s} ERROR {r['error'][:100]}")
        continue
    checks = ",".join(c for c in r["checks"] if c not in ("dynamic_range_low", "comp_density_on_compressed_source", "export_ok"))
    print(f"{r['id']:40s} {f(r['src_lufs'])} {f(r['target_lufs'])} {f(r['receipt_lufs'])} {f(r['ind_lufs'])} {f(r['A2_lufs_err'])} {f(r['ceiling_dbtp'],5,1)} {f(r['receipt_tp'])} {f(r['ff_Peak'])} {f(r.get('ind_tp16'))} {f(r['src_lra'])} {f(r['ind_lra'])} {f(r['crest_db'])} {f(r['psr_db'])} {f(r['p95_p10_db'])} {f(r['corr'])} {f(r['side_mid_db'])} {f(r['length_delta_frames'],4,0)} {f(r['lag_samples'],4,0)} {checks}")
    if show_spec and r.get("third_octave_delta_db"):
        d = r["third_octave_delta_db"]
        keys = ["31.5", "63", "125", "250", "500", "1000", "2000", "4000", "8000", "16000"]
        print("      spectrum delta dB @", " ".join(f"{k}:{f(d.get(k),5,1)}" for k in keys))
