"""Claim 4b: does the gentle continuous curve tanh(2a*x)/(2a) shift work into
the limiter at matched -9 delivery on It's a coat?

Emulation (no engine change): the production chain is split at the
saturator using coefficient overrides:
    pre  = chain(no_sat, no_limiter)            -> everything up to width
    sat  = NumPy curve applied to `pre`
    post = limiter-only chain on `sat`          -> production limiter
    then verbatim production SRC + production landing rule (gain-only).
Validation: with the CURRENT curve the emulation must reproduce the full
chain output to float rounding (reported as max abs diff).
Limiter GR = |limited| / |unlimited| per frame (bypass ratio), as in `stats`.
Trim search for the continuous curve: input trims -3..+6 dB in 1.5 dB steps
through the same emulation; pick the least trim reaching -9 within 0.2 LU.
"""
import json
import os
import re
import subprocess

import numpy as np
import soundfile as sf

RECON = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
ROOT = os.path.dirname(os.path.dirname(RECON))
HARNESS = os.path.join(RECON, "build", "target", "release", "yes-recon-harness.exe")
R = os.path.join(RECON, "renders", "sat_interaction")
os.makedirs(R, exist_ok=True)
SRC = os.path.join(ROOT, "fixtures", "inputs", "input01.wav")
BASE = {"preset": "universal", "intensity": 0.75, "delivery_profile": "custom", "ceiling_dbtp": -1.0,
        "sample_rate": 48000, "bit_depth": 24, "compression_density": 0.5, "adapt_strength": 0.5, "lufs_target": -9.0}
A = 0.0715
PRE_OVR = ["no_sat", "no_limiter"]
LIM_OVR = ["no_hpf", "no_eq", "no_comp", "no_transient", "no_width", "no_sat", "no_input_gain"]


def run(cmd):
    r = subprocess.run(cmd, capture_output=True, text=True)
    if r.returncode != 0:
        raise RuntimeError(r.stderr[-800:])
    return r.stdout


def chain(inp, out, trim, overrides):
    cmd = [HARNESS, "chain", inp, out, "--spec-json", json.dumps(dict(BASE, input_gain_db=trim)), "--analysis-from", SRC]
    for o in overrides:
        cmd += ["--override", o]
    run(cmd)
    return sf.read(out, dtype="float32", always_2d=True)


def write(path, x, sr):
    sf.write(path, x, sr, subtype="FLOAT")


def current_curve(x):
    d = 1 + 2 * A
    return np.tanh(x * d) / np.tanh(d)


def continuous_curve(x):
    k = 2 * A
    return np.tanh(k * x) / k


def ffmpeg_lra(path):
    r = subprocess.run(["ffmpeg", "-hide_banner", "-nostats", "-i", path, "-af", "ebur128=peak=true", "-f", "null", "-"], capture_output=True, text=True).stderr
    s = r.rsplit("Summary:", 1)[-1]
    return float(re.search(r"I:\s*([-\d.]+) LUFS", s)[1]), float(re.search(r"LRA:\s*([-\d.]+) LU", s)[1]), float(re.search(r"Peak:\s*([-\d.]+) dBFS", s)[1])


def emulate(trim, curve, tag):
    pre_path = os.path.join(R, f"{tag}_pre.f32.wav")
    pre, sr = chain(SRC, pre_path, trim, PRE_OVR)
    sat = curve(pre.astype(np.float64)).astype(np.float32)
    sat_path = os.path.join(R, f"{tag}_sat.f32.wav")
    write(sat_path, sat, sr)
    lim_path = os.path.join(R, f"{tag}_lim.f32.wav")
    lim, _ = chain(sat_path, lim_path, 0.0, LIM_OVR)
    # limiter GR from the bypass ratio (sat == unlimited input to the limiter; chain output is delay compensated)
    n = min(len(lim), len(sat))
    mag = np.abs(sat[:n]).max(axis=1)
    idx = np.argmax(np.abs(sat[:n]), axis=1)
    full = np.abs(lim[:n][np.arange(n), idx])
    ok = mag > 1e-3
    gr = np.zeros(n)
    gr[ok] = np.maximum(0.0, -20 * np.log10(np.maximum(full[ok] / mag[ok], 1e-9)))
    src_path = os.path.join(R, f"{tag}_src48.f32.wav")
    run([HARNESS, "src-file", lim_path, src_path, "--to", "48000", "--mode", "prod"])
    m = json.loads(run([HARNESS, "measure", src_path]))
    g = min(-9.0 - m["lufs"], -1.0 - m["true_peak_dbtp"])
    y, sr48 = sf.read(src_path, dtype="float32", always_2d=True)
    y = y * (10 ** (g / 20))
    out = os.path.join(R, f"{tag}_delivered.f32.wav")
    write(out, y, sr48)
    lufs, lra, tp = ffmpeg_lra(out)
    crest = 20 * np.log10(np.abs(y).max()) - 10 * np.log10(np.mean(y.astype(np.float64) ** 2))
    row = {"tag": tag, "trim": trim, "pre_landing_lufs": m["lufs"], "pre_landing_tp": m["true_peak_dbtp"], "landing_db": g,
           "delivered_lufs": lufs, "lra": lra, "tp_ffmpeg": tp, "crest_db": float(crest),
           "limiter_max_gr_db": float(gr.max()), "limiter_mean_gr_db": float(gr.mean()), "limiter_active_fraction": float((gr > 0.01).mean()),
           "sat_peak_in_dbfs": float(20 * np.log10(np.abs(pre).max())), "sat_peak_out_dbfs": float(20 * np.log10(np.abs(sat).max()))}
    for p in (pre_path, sat_path, src_path, out):
        os.remove(p)
    return row, lim


rows = []
# validation: current curve emulation vs full production chain at trim 0
full_path = os.path.join(R, "full_chain.f32.wav")
full, sr = chain(SRC, full_path, 0.0, [])
row, lim = emulate(0.0, current_curve, "control_emulated")
n = min(len(full), len(lim))
row["emulation_max_abs_diff_vs_full_chain"] = float(np.abs(full[:n] - lim[:n]).max())
os.remove(full_path)
os.remove(os.path.join(R, "control_emulated_lim.f32.wav"))
rows.append(row)
print(json.dumps(row), flush=True)
# continuous curve: trim search
for trim in [0.0, 1.5, 3.0, 4.5, 6.0]:
    row, _ = emulate(trim, continuous_curve, f"continuous_trim{trim:+.1f}")
    os.remove(os.path.join(R, f"continuous_trim{trim:+.1f}_lim.f32.wav"))
    rows.append(row)
    print(json.dumps(row), flush=True)
json.dump(rows, open(os.path.join(RECON, "results", "sat_interaction_coat_t9.json"), "w"), indent=1)
