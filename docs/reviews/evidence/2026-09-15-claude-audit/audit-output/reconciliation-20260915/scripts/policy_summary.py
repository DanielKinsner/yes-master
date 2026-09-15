"""Tables from results/policy_compare.json: per condition (source, level,
target) the P / A / A2 / C rows, and aggregate deltas vs production."""
import json
import os
import sys

import numpy as np

RECON = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
d = json.load(open(os.path.join(RECON, "results", "policy_compare.json")))
rows = d["finalists"]
out = []


def p(s=""):
    out.append(s)
    print(s)


by = {}
for r in rows:
    by.setdefault((r["source"], r["level_db"], r["target"]), {})[r["policy"]] = r

p("== per condition: trim | delivered LUFS (receipt) | TP engine / exact | crest | LRA | ST spread | section contrast (src) | attack/body | tone max|dev| | M/S | lim max/mean GR | comp max GR low | render s")
order = ["coat", "piano", "rich", "imaginal", "funk", "metal", "aphelion", "baby"]
for key in sorted(by, key=lambda k: (order.index(k[0]), -k[1], -k[2])):
    src, lvl, tgt = key
    for pol in ("P", "A", "A2", "C"):
        r = by[key].get(pol)
        if not r:
            continue
        sel = d["selection"].get(f"{src}_{lvl}_t{int(-tgt)}", {})
        how = sel.get("C", {}).get("how", "") if pol == "C" else ""
        p(f"{src:8s} {lvl:+3d} dB t{int(-tgt):2d} {pol:2s} {r['trim_db']:+6.2f} | {r['receipt_lufs']:6.2f} | {r['tp_engine']:5.2f}/{r['tp_fftexact16']:5.2f} | {r['crest_db']:5.2f} | {r['lra_ffmpeg']:4.1f} | {r['shortterm_p95_p10'] if r['shortterm_p95_p10'] is not None else float('nan'):4.1f} | {r['section_contrast_db'] if r['section_contrast_db'] is not None else float('nan'):5.2f} ({r['source_section_contrast_db']:5.2f}) | {r['attack_body_median_db'] if r['attack_body_median_db'] is not None else float('nan'):5.2f} | {r['tone_max_abs_delta_db'] if r['tone_max_abs_delta_db'] is not None else float('nan'):4.2f} | {r['side_mid_db'] if r['side_mid_db'] is not None else float('nan'):6.2f} | {r['limiter_max_gr_db']:5.2f}/{r['limiter_mean_gr_db']:4.2f} | {r['comp_max_gr_db'][0]:4.2f} | {r['render_secs']:5.1f} {how}")

p("\n== aggregates vs production (P) per policy and target")
for pol in ("A", "A2", "C"):
    for tgt in (-14.0, -9.0):
        deltas = []
        for key, v in by.items():
            if key[2] != tgt or pol not in v or "P" not in v:
                continue
            a, b = v[pol], v["P"]
            deltas.append({
                "key": key, "d_lufs_err": abs(a["receipt_lufs"] - tgt) - abs(b["receipt_lufs"] - tgt),
                "hit": abs(a["receipt_lufs"] - tgt) <= 0.2, "hit_P": abs(b["receipt_lufs"] - tgt) <= 0.2,
                "tp_exact_ok": a["tp_fftexact16"] <= -0.9, "tp_exact_ok_P": b["tp_fftexact16"] <= -0.9,
                "d_crest": a["crest_db"] - b["crest_db"], "d_lra": a["lra_ffmpeg"] - b["lra_ffmpeg"],
                "d_st": (a["shortterm_p95_p10"] or 0) - (b["shortterm_p95_p10"] or 0),
                "d_contrast": (a["section_contrast_db"] or 0) - (b["section_contrast_db"] or 0),
                "d_attack": (a["attack_body_median_db"] or 0) - (b["attack_body_median_db"] or 0),
                "d_tone": (a["tone_max_abs_delta_db"] or 0) - (b["tone_max_abs_delta_db"] or 0),
                "d_ms": (a["side_mid_db"] or 0) - (b["side_mid_db"] or 0),
                "d_limmax": a["limiter_max_gr_db"] - b["limiter_max_gr_db"], "d_limmean": a["limiter_mean_gr_db"] - b["limiter_mean_gr_db"],
                "secs": a["render_secs"], "secs_P": b["render_secs"],
            })
        if not deltas:
            continue
        n = len(deltas)
        p(f"  {pol} @ {tgt:+.0f}: n={n}; target hit {sum(x['hit'] for x in deltas)}/{n} (P {sum(x['hit_P'] for x in deltas)}/{n}); exact TP <= -0.9 {sum(x['tp_exact_ok'] for x in deltas)}/{n} (P {sum(x['tp_exact_ok_P'] for x in deltas)}/{n}); "
          f"crest +{np.mean([x['d_crest'] for x in deltas]):.2f} (min {min(x['d_crest'] for x in deltas):+.2f}, max {max(x['d_crest'] for x in deltas):+.2f}); LRA {np.mean([x['d_lra'] for x in deltas]):+.2f}; ST spread {np.mean([x['d_st'] for x in deltas]):+.2f}; "
          f"contrast {np.mean([x['d_contrast'] for x in deltas]):+.2f} (min {min(x['d_contrast'] for x in deltas):+.2f}); attack/body {np.mean([x['d_attack'] for x in deltas]):+.2f}; tone max dev {np.mean([x['d_tone'] for x in deltas]):+.2f}; M/S {np.mean([x['d_ms'] for x in deltas]):+.2f} (max |{max(abs(x['d_ms']) for x in deltas):.2f}|); "
          f"lim max GR {np.mean([x['d_limmax'] for x in deltas]):+.2f} mean GR {np.mean([x['d_limmean'] for x in deltas]):+.2f}; further from target: {sum(x['d_lufs_err'] > 0.05 for x in deltas)}, closer: {sum(x['d_lufs_err'] < -0.05 for x in deltas)}")

p("\n== input-level consistency: crest spread across 0/-6/-12 dB copies at the same target (max - min)")
for src in ("coat", "piano", "rich", "imaginal"):
    for tgt in (-14.0, -9.0):
        line = f"  {src:9s} t{int(-tgt):2d}:"
        for pol in ("P", "A", "A2", "C"):
            vals = [by[(src, l, tgt)][pol]["crest_db"] for l in (0, -6, -12) if (src, l, tgt) in by and pol in by[(src, l, tgt)]]
            lufs = [by[(src, l, tgt)][pol]["receipt_lufs"] for l in (0, -6, -12) if (src, l, tgt) in by and pol in by[(src, l, tgt)]]
            if len(vals) == 3:
                line += f"  {pol}: crest {max(vals)-min(vals):.3f} dB, LUFS {min(lufs):.2f}..{max(lufs):.2f}"
        p(line)

rt = d["runtime"]
p(f"\n== runtime: grid wall {rt['grid_wall_s']:.0f} s for {sum(len(g['points']) for g in d['grid'].values())} chain+SRC+measure points ({np.mean([pt['secs'] for g in d['grid'].values() for pt in g['points']]):.1f} s each, single-threaded); finalists wall {rt['finalists_wall_s']:.0f} s")
p("   per condition: P = 1 render; A = 1 chain+SRC+measure + 1 render (A2: +1 render when iterated); C = 13 chain+SRC+measure + 1 render")
for k, s in d["selection"].items():
    pass
open(os.path.join(RECON, "results", "policy_summary.txt"), "w").write("\n".join(out) + "\n")
