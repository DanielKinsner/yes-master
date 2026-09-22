"""Claim 1: analyse the SRC probe outputs against an independently computed
reference (analytic sine at the output rate), for the production-verbatim
loop, the independently written corrected loop and Codex's prototype.

For every case: expected length (integer ceil), actual length, and for cases
with >= 100 output frames the per-sample residual against the analytic sine
(least-squares amplitude/phase fit over the interior, 20 ms guard at each
end) -> RMS residual in dBc, max |error|, index of the max error. For short
cases: output RMS vs reference RMS (a near-silent output means the start-up
delay was not removed).
"""
import json
import os
import sys

import numpy as np
import soundfile as sf

HERE = os.path.dirname(os.path.abspath(__file__))
RECON = os.path.dirname(HERE)
P = os.path.join(RECON, "results", "src-probe")

probe = json.load(open(os.path.join(P, "probe.json")))
rows = []
for c in probe["cases"]:
    x, sr = sf.read(os.path.join(P, c["file"]), dtype="float64", always_2d=True)
    assert sr == c["to"]
    y = x[:, 0]
    n = len(y)
    r = dict(c)
    r["length_ok"] = (n == c["expected_frames"])
    r["finite"] = bool(np.isfinite(x).all())
    r["chan_identical"] = bool(np.array_equal(x[:, 0], x[:, 1]))
    if n >= 4 * int(0.02 * sr) + 64:
        # 2 ms guard: keeps the delay index (514/955) inside the fitted range
        g = int(0.002 * sr)
        t = np.arange(n) / sr
        keep = slice(g, n - g)
        A = np.column_stack([np.sin(2 * np.pi * c["hz"] * t[keep]), np.cos(2 * np.pi * c["hz"] * t[keep])])
        coef = np.linalg.lstsq(A, y[keep], rcond=None)[0]
        fit = A @ coef
        res = y[keep] - fit
        r["fit_amplitude"] = float(np.hypot(*coef))
        # phase of the fit relative to the nominal grid -> time offset in output samples
        ph = np.arctan2(coef[1], coef[0])
        r["time_offset_samples"] = float(-ph / (2 * np.pi * c["hz"]) * sr)
        r["residual_dbc"] = float(10 * np.log10(np.mean(res ** 2) / max(np.mean(fit ** 2), 1e-30)))
        k = int(np.argmax(np.abs(res)))
        r["max_error"] = float(abs(res[k]))
        r["max_error_index"] = int(k + g)
        # second-largest error to show the defect is a single sample
        res2 = np.abs(res).copy()
        res2[k] = 0
        r["second_error"] = float(res2.max())
    else:
        ref = 0.5 * np.sin(2 * np.pi * c["hz"] * np.arange(n) / sr)
        r["rms"] = float(np.sqrt(np.mean(y ** 2))) if n else 0.0
        r["reference_rms"] = float(np.sqrt(np.mean(ref ** 2))) if n else 0.0
    rows.append(r)

json.dump(rows, open(os.path.join(RECON, "results", "src_probe_analysis.json"), "w"), indent=1)

# ---- summary tables ----------------------------------------------------------------
out = []
def p(s=""):
    out.append(s); print(s)

p("== rubato Fft parameters per rate pair (chunk 2048 request, FixedSync::Both)")
for q in probe["params"]:
    useful = q["output_frames_next"] - q["output_delay"]
    p(f"  {q['from']:>6}->{q['to']:<6} in_next={q['input_frames_next']:5d} out_next={q['output_frames_next']:5d} delay={q['output_delay']:5d} useful_after_first_block={useful:5d} "
      f"{'-> STALE SAMPLE at index '+str(q['output_delay']) if useful > q['output_delay'] else '-> copy covers useful suffix'}")

p("\n== length correctness (actual == ceil(frames*to/from)) by mode")
for mode in ("prod", "recon", "codex"):
    sub = [r for r in rows if r["mode"] == mode]
    bad = [r for r in sub if not r["length_ok"]]
    p(f"  {mode:6s}: {len(sub)-len(bad)}/{len(sub)} exact; wrong-length cases: {len(bad)}")
    for r in bad[:8]:
        p(f"     {r['from']}->{r['to']} frames_in={r['frames_in']} hz={r['hz']}: got {r['frames_out']} expected {r['expected_frames']}")
    if len(bad) > 8:
        p(f"     ... {len(bad)-8} more")

p("\n== long cases: worst residual per (pair, mode)")
pairs = sorted({(r["from"], r["to"]) for r in rows})
for (f, t) in pairs:
    for mode in ("prod", "recon", "codex"):
        sub = [r for r in rows if r["from"] == f and r["to"] == t and r["mode"] == mode and "residual_dbc" in r]
        if not sub:
            continue
        w = max(sub, key=lambda r: r["max_error"])
        p(f"  {f:>6}->{t:<6} {mode:6s} n={len(sub):3d} worst max_err={w['max_error']:.4f} at idx {w['max_error_index']} (2nd {w['second_error']:.2e}) resid={w['residual_dbc']:7.1f} dBc  [frames_in={w['frames_in']} hz={w['hz']}]; best resid={min(r['residual_dbc'] for r in sub):7.1f}")

p("\n== short cases (< ~100 ms): output RMS / reference RMS by mode (mean over cases)")
for (f, t) in pairs:
    for mode in ("prod", "recon", "codex"):
        sub = [r for r in rows if r["from"] == f and r["to"] == t and r["mode"] == mode and "rms" in r and r["reference_rms"] > 0]
        if not sub:
            continue
        ratios = [r["rms"] / r["reference_rms"] for r in sub]
        silent = sum(1 for q in ratios if q < 0.1)
        p(f"  {f:>6}->{t:<6} {mode:6s} n={len(sub):2d} ratio min={min(ratios):.3f} mean={np.mean(ratios):.3f}; near-silent(<0.1): {silent}")

p("\n== stale-sample check: prod max_error index == delay?")
for (f, t) in pairs:
    sub = [r for r in rows if r["from"] == f and r["to"] == t and r["mode"] == "prod" and "residual_dbc" in r]
    d = sub[0]["delay"] if sub else None
    at_delay = sum(1 for r in sub if r["max_error_index"] == d and r["max_error"] > 1e-3)
    p(f"  {f:>6}->{t:<6}: {at_delay}/{len(sub)} long cases have a >1e-3 error exactly at index {d}")

open(os.path.join(RECON, "results", "src_probe_summary.txt"), "w").write("\n".join(out) + "\n")
