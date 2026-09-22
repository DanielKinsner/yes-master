"""Claim 3: matched comparison of three automatic-drive policies on the
production chain, Universal 75 / density 0.5 / Adapt 0.5 / -1 dBTP / 48 kHz
PCM24, targets -9 and -14 LUFS (Codex's control conditions).

  P  production: input trim 0 dB, ceiling-bounded scalar landing.
  A  attenuate-only (blind report): g = min(0, target - L1), L1 = engine
     loudness of the unity-trim chain output after production SRC.
  A2 A with one bounded iteration: if the delivered result misses the target
     by > 0.2 LU and g < 0, add the shortfall (still <= 0) and re-render.
  C  Codex centred search: centre = -18 - source LUFS, offsets -12..+6 dB in
     1.5 dB steps, trims clamped to +/-24 dB; pick the least drive reaching
     the target within 0.2 LU, else the smallest error (tie within 0.05 LU
     -> lower drive).

Grid evaluation (for A's L1 and C's 13 points) uses: production chain at
source rate (harness `chain`) -> verbatim production SRC (`src-file --mode
prod`, proven bit-identical to the export path) -> engine meter -> the
production landing rule (gain = min(target - L, ceiling - TP)). This is the
export path minus dither, so predicted delivery equals the receipt to within
the 24-bit quantisation. Finalists are then rendered through the real
production export (`render`) and measured independently.

Sources: coat/piano/rich/imaginal at 0/-6/-12 dB (gain-only copies) and
funk/metal/aphelion/baby at 0 dB. Renders are hashed and deleted after
measurement (disk); results/policy_compare.json keeps everything.
"""
import json
import os
import re
import subprocess
import sys
import time
from concurrent.futures import ProcessPoolExecutor

import numpy as np
import soundfile as sf

HERE = os.path.dirname(os.path.abspath(__file__))
RECON = os.path.dirname(HERE)
AUDIT = os.path.dirname(RECON)
ROOT = os.path.dirname(AUDIT)
HARNESS = os.path.join(RECON, "build", "target", "release", "yes-recon-harness.exe")
sys.path.insert(0, os.path.join(AUDIT, "scripts"))  # sealed blind measure.py (read-only import)
sys.path.insert(0, HERE)
import measure  # noqa: E402
import tp_meters  # noqa: E402
from prune import sha  # noqa: E402

PIN = os.path.join(RECON, "renders", "policy_inputs")
POUT = os.path.join(RECON, "renders", "policy")
os.makedirs(PIN, exist_ok=True)
os.makedirs(POUT, exist_ok=True)

SOURCES = {"coat": "input01", "piano": "input02", "rich": "input07", "imaginal": "input08",
           "funk": "input04", "metal": "input05", "aphelion": "input03", "baby": "input06"}
LEVELS = {"coat": [0, -6, -12], "piano": [0, -6, -12], "rich": [0, -6, -12], "imaginal": [0, -6, -12],
          "funk": [0], "metal": [0], "aphelion": [0], "baby": [0]}
TARGETS = [-9.0, -14.0]
CEIL = -1.0
OFFSETS = [round(-12 + 1.5 * i, 1) for i in range(13)]
BASE = {"preset": "universal", "intensity": 0.75, "delivery_profile": "custom", "ceiling_dbtp": CEIL,
        "sample_rate": 48000, "bit_depth": 24, "compression_density": 0.5, "adapt_strength": 0.5}


def run(cmd):
    r = subprocess.run(cmd, capture_output=True, text=True)
    if r.returncode != 0:
        raise RuntimeError(f"{cmd[:3]} failed: {r.stderr[-1000:]}")
    return r.stdout


def prepare_input(name, level):
    src = os.path.join(ROOT, "fixtures", "inputs", SOURCES[name] + ".wav")
    if level == 0:
        return src
    dst = os.path.join(PIN, f"{name}_{level}dB.wav")
    if not os.path.exists(dst):
        x, sr = sf.read(src, dtype="float32", always_2d=True)
        sf.write(dst, x * (10 ** (level / 20)), sr, subtype="FLOAT")
    return dst


def landing(L, TP, target):
    g = min(target - L, CEIL - TP)
    return (0.0 if abs(g) <= 1e-4 else g)


def grid_point(args):
    """chain at trim -> prod SRC -> engine meter. Returns (trim, L, TP, secs)."""
    name, level, inp, trim = args
    tag = f"{name}_{level}dB_trim{trim:+.2f}"
    chain_out = os.path.join(POUT, tag + "_chain.f32.wav")
    src_out = os.path.join(POUT, tag + "_src48.f32.wav")
    spec = dict(BASE, input_gain_db=trim, lufs_target=-14.0)
    t0 = time.time()
    run([HARNESS, "chain", inp, chain_out, "--spec-json", json.dumps(spec)])
    run([HARNESS, "src-file", chain_out, src_out, "--to", "48000", "--mode", "prod"])
    m = json.loads(run([HARNESS, "measure", src_out]))
    secs = time.time() - t0
    os.remove(chain_out)
    os.remove(src_out)
    return {"trim": trim, "L": m["lufs"], "TP": m["true_peak_dbtp"], "LRA": m["lra"], "secs": secs}


def select_codex(points, target):
    ok = [p for p in points if abs(p["L"] + landing(p["L"], p["TP"], target) - target) <= 0.2]
    if ok:
        return min(ok, key=lambda p: p["trim"]), "reached"
    err = [(abs(p["L"] + landing(p["L"], p["TP"], target) - target), p) for p in points]
    best = min(err, key=lambda e: e[0])
    ties = [p for e, p in err if e <= best[0] + 0.05]
    return min(ties, key=lambda p: p["trim"]), "min_error"


def ffmpeg_shortterm(path):
    r = subprocess.run(["ffmpeg", "-hide_banner", "-nostats", "-i", path, "-af", "ebur128=peak=true", "-f", "null", "-"],
                       capture_output=True, text=True).stderr
    summ = r.rsplit("Summary:", 1)[-1]
    lufs = float(re.search(r"I:\s*([-\d.]+) LUFS", summ)[1])
    lra = float(re.search(r"LRA:\s*([-\d.]+) LU", summ)[1])
    tp = float(re.search(r"Peak:\s*([-\d.]+) dBFS", summ)[1])
    short = []
    for line in r.splitlines():
        m = re.search(r"t:\s*([\d.]+).*?M:\s*([-\d.]+)\s+S:\s*([-\d.]+)", line)
        if m and float(m[1]) >= 3:
            short.append(float(m[3]))
    short = np.array(short)
    active = short[short > -40]
    spread = float(np.percentile(active, 95) - np.percentile(active, 10)) if len(active) else None
    return lufs, lra, tp, spread


def source_anchors(x, sr):
    """Codex's definitions: 10 s section RMS (loudest/quietest active) and
    transient anchors (10 ms RMS-envelope novelty peaks, top 40)."""
    n = len(x) // 480000 * 480000 if sr == 48000 else len(x) // (10 * sr) * (10 * sr)
    sec = x[:n].reshape(-1, 10 * sr, x.shape[1])
    srms = 10 * np.log10(np.mean(sec ** 2, axis=(1, 2)) + 1e-30)
    active = [i for i, v in enumerate(srms) if v > -40]
    lo = min(active, key=lambda i: srms[i])
    hi = max(active, key=lambda i: srms[i])
    b = int(0.01 * sr)
    blk = x[:len(x) // b * b].reshape(-1, b, x.shape[1])
    env = np.sqrt(np.mean(blk ** 2, axis=(1, 2)))
    nov = np.maximum(np.diff(env, prepend=env[0]), 0)
    from scipy.signal import find_peaks
    peaks, _ = find_peaks(nov, distance=20)
    peaks = sorted(peaks, key=lambda k: nov[k], reverse=True)[:40]
    anchors = sorted(float(k * 0.01) for k in peaks if 0.05 < k * 0.01 < len(x) / sr - 0.2)
    return (lo, hi, float(srms[hi] - srms[lo])), anchors


def section_and_attack(y, sr, lo, hi, anchors):
    n = len(y) // (10 * sr) * (10 * sr)
    sec = y[:n].reshape(-1, 10 * sr, y.shape[1])
    srms = 10 * np.log10(np.mean(sec ** 2, axis=(1, 2)) + 1e-30)
    contrast = float(srms[hi] - srms[lo]) if hi < len(srms) and lo < len(srms) else None
    ratios = []
    for t in anchors:
        a = y[int((t - .02) * sr):int((t + .03) * sr)]
        b = y[int((t + .03) * sr):int((t + .15) * sr)]
        if len(a) and len(b):
            ratios.append(10 * np.log10(np.abs(a).max() ** 2 / max(np.mean(b * b), 1e-30)))
    return contrast, (float(np.median(ratios)) if ratios else None)


def measure_finalist(path, src_path, anchors_cache):
    x, sr = sf.read(path, dtype="float64", always_2d=True)
    m = measure.full(path, src_path, light=False)
    lufs, lra, tp_ff, spread = ffmpeg_shortterm(path)
    (lo, hi, src_contrast), anchors = anchors_cache
    # source-anchored windows are defined on a common 48 kHz grid; sources at
    # 44.1 kHz map by time (anchors in seconds; section index unchanged)
    contrast, attack = section_and_attack(x, sr, lo, hi, anchors)
    d = m.get("third_octave_delta_db") or {}
    ref = d.get("1000") or 0.0
    bands = ["63", "125", "250", "500", "1000", "2000", "4000", "8000"]
    dev = [d[b] - ref for b in bands if d.get(b) is not None]
    return {
        "lufs_own": m["lufs_integrated"], "lufs_ffmpeg": lufs, "lra_ffmpeg": lra, "tp_ffmpeg": tp_ff,
        "tp_engine": json.loads(run([HARNESS, "measure", path]))["true_peak_dbtp"],
        "tp_fftexact16": tp_meters.fft_exact_tp(x, sr, 16),
        "sample_peak_dbfs": m["sample_peak_dbfs"], "crest_db": m["crest_db"], "psr_db": m["psr_db"],
        "shortterm_p95_p10": spread, "section_contrast_db": contrast, "source_section_contrast_db": src_contrast,
        "attack_body_median_db": attack, "tone_max_abs_delta_db": (max(abs(v) for v in dev) if dev else None),
        "tone_rms_delta_db": (float(np.sqrt(np.mean(np.square(dev)))) if dev else None),
        "side_mid_db": m.get("side_mid_db"), "corr": m.get("corr"),
    }


def stats(inp, trim, analysis_from):
    j = json.loads(run([HARNESS, "stats", inp, "--spec-json", json.dumps(dict(BASE, input_gain_db=trim, lufs_target=-14.0)), "--analysis-from", analysis_from]))
    return {"comp_max_gr_db": j["comp_max_gr_db"], "limiter_max_gr_db": j["limiter"]["max_gr_db"],
            "limiter_mean_gr_db": j["limiter"]["mean_gr_db_over_all_frames"], "limiter_active_fraction": j["limiter"]["active_fraction"],
            "sat_in_peak_dbfs": j["saturation"]["in_peak_dbfs"], "sat_out_peak_dbfs": j["saturation"]["out_peak_dbfs"],
            "sat_rms_gain_db": j["saturation"]["out_rms_dbfs"] - j["saturation"]["in_rms_dbfs"],
            "pre_landing_lufs": j["pre_landing"]["lufs"], "pre_landing_tp": j["pre_landing"]["true_peak_dbtp"]}


def render_finalist(args):
    name, level, inp, target, policy, trim, anchors_cache, src_path = args
    tag = f"{name}_{level}dB_t{int(-target)}_{policy}"
    out = os.path.join(POUT, tag + ".wav")
    spec = dict(BASE, input_gain_db=round(trim, 3), lufs_target=target)
    t0 = time.time()
    rep = json.loads(run([HARNESS, "render", inp, out, "--spec-json", json.dumps(spec)]))
    secs = time.time() - t0
    ms = rep["measurements"]
    row = {"key": tag, "source": name, "level_db": level, "target": target, "policy": policy, "trim_db": trim,
           "receipt_lufs": ms["lufs_integrated"], "receipt_tp": ms["true_peak_dbtp"], "receipt_lra": ms["dynamic_range_lu"],
           "render_secs": secs, "checks": [c["code"] for c in rep["checks"]], "sha256": sha(out), "bytes": os.path.getsize(out)}
    row.update(measure_finalist(out, src_path, anchors_cache))
    row.update(stats(inp, trim, inp))
    os.remove(out)
    return row


def main():
    only = sys.argv[1:]
    t_all = time.time()
    results = {"grid": {}, "finalists": [], "selection": {}, "runtime": {}}
    jobs = []
    for name, levels in LEVELS.items():
        if only and name not in only:
            continue
        for level in levels:
            inp = prepare_input(name, level)
            a = json.loads(run([HARNESS, "analyze", inp]))
            centre = -18.0 - a["lufs_integrated"]
            trims = sorted({0.0} | {max(-24.0, min(24.0, round(centre + o, 3))) for o in OFFSETS})
            results["grid"][f"{name}_{level}"] = {"input": inp, "source_lufs": a["lufs_integrated"], "source_tp": a["true_peak_dbtp"],
                                                  "source_lra": a["dynamic_range_lu"], "centre": centre, "trims": trims, "points": []}
            jobs += [(name, level, inp, t) for t in trims]
    print(f"grid: {len(jobs)} chain+SRC+measure jobs", flush=True)
    t0 = time.time()
    with ProcessPoolExecutor(max_workers=8) as ex:
        for (name, level, inp, trim), pt in zip(jobs, ex.map(grid_point, jobs)):
            results["grid"][f"{name}_{level}"]["points"].append(pt)
            print(f"  {name}_{level} trim {trim:+6.2f}: L={pt['L']:7.2f} TP={pt['TP']:6.2f} ({pt['secs']:.1f}s)", flush=True)
    results["runtime"]["grid_wall_s"] = time.time() - t0

    # selection
    fjobs = []
    for key, g in results["grid"].items():
        name, level = key.rsplit("_", 1)
        level = int(level)
        pts = g["points"]
        p0 = next(p for p in pts if p["trim"] == 0.0)
        centred = [p for p in pts if p["trim"] != 0.0 or any(abs(g["centre"] + o) < 1e-6 for o in OFFSETS)]
        src_path = g["input"]
        x, sr = sf.read(src_path, dtype="float64", always_2d=True)
        anchors_cache = source_anchors(x, sr)
        for target in TARGETS:
            gA = min(0.0, target - p0["L"])
            sel, how = select_codex(centred, target)
            results["selection"][f"{key}_t{int(-target)}"] = {
                "P": {"trim": 0.0, "pred_lufs": p0["L"] + landing(p0["L"], p0["TP"], target), "landing_db": landing(p0["L"], p0["TP"], target)},
                "A": {"trim": gA, "L1": p0["L"]},
                "C": {"trim": sel["trim"], "how": how, "pred_lufs": sel["L"] + landing(sel["L"], sel["TP"], target), "grid_secs_total": sum(p["secs"] for p in centred)},
            }
            fjobs.append((name, level, src_path, target, "P", 0.0, anchors_cache, src_path))
            fjobs.append((name, level, src_path, target, "A", gA, anchors_cache, src_path))
            fjobs.append((name, level, src_path, target, "C", sel["trim"], anchors_cache, src_path))
    print(f"finalists: {len(fjobs)} production renders", flush=True)
    t0 = time.time()
    with ProcessPoolExecutor(max_workers=6) as ex:
        for row in ex.map(render_finalist, fjobs):
            results["finalists"].append(row)
            print(f"  {row['key']:32s} trim {row['trim_db']:+6.2f} LUFS {row['receipt_lufs']:7.2f} TPeng {row['receipt_tp']:6.2f} TPexact {row['tp_fftexact16']:6.2f} crest {row['crest_db']:5.2f} LRA {row['lra_ffmpeg']:4.1f} lim {row['limiter_max_gr_db']:4.2f}/{row['limiter_mean_gr_db']:4.2f}", flush=True)
    # A2: one bounded iteration where A missed the target
    ijobs = []
    for row in [r for r in results["finalists"] if r["policy"] == "A"]:
        short = row["target"] - row["receipt_lufs"]
        if short > 0.2 and row["trim_db"] < 0:
            g2 = min(0.0, row["trim_db"] + short)
            key = f"{row['source']}_{row['level_db']}"
            g = results["grid"][key]
            x, sr = sf.read(g["input"], dtype="float64", always_2d=True)
            ijobs.append((row["source"], row["level_db"], g["input"], row["target"], "A2", g2, source_anchors(x, sr), g["input"]))
    print(f"A2 iterations: {len(ijobs)}", flush=True)
    with ProcessPoolExecutor(max_workers=6) as ex:
        for row in ex.map(render_finalist, ijobs):
            results["finalists"].append(row)
            print(f"  {row['key']:32s} trim {row['trim_db']:+6.2f} LUFS {row['receipt_lufs']:7.2f} crest {row['crest_db']:5.2f}", flush=True)
    results["runtime"]["finalists_wall_s"] = time.time() - t0
    results["runtime"]["total_wall_s"] = time.time() - t_all
    json.dump(results, open(os.path.join(RECON, "results", "policy_compare.json"), "w"), indent=1)
    print("done", results["runtime"])


if __name__ == "__main__":
    main()
