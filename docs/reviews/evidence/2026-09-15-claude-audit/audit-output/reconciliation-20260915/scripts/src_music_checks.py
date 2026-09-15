"""Persist the music-file SRC checks (claim 1 on real material):

  A. production-path equivalence: chain(48 kHz) -> verbatim prod_convert must
     equal a production 32-bit-float Custom/no-target 44.1 kHz export bit for
     bit (proves the harness SRC copy is the shipped path).
  B. the stale sample at frame 955 in that export and in a hot-start 48 kHz
     clip exported to the CD profile (16-bit, -14 LUFS): delivered file vs
     corrected SRC scaled by the production landing gain.

Reads the renders produced earlier by the harness; writes
results/src_music_checks.json.
"""
import json
import os

import numpy as np
import soundfile as sf

RECON = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
R = os.path.join(RECON, "renders")
out = {}

# ---- A. equivalence (input08, Universal 0.75, Custom, no target, 44.1 kHz, float) ----
a, sa = sf.read(os.path.join(R, "eq_input08_render44_f32.wav"), dtype="float32", always_2d=True)
b, sb = sf.read(os.path.join(R, "eq_input08_prodsrc44.f32.wav"), dtype="float32", always_2d=True)
c, sc = sf.read(os.path.join(R, "eq_input08_reconsrc44.f32.wav"), dtype="float32", always_2d=True)
rep = json.load(open(os.path.join(R, "eq_input08_render44.json")))
m = min(len(a), len(b))
dd = np.abs(a[:m] - c[:m]).max(axis=1)
k = int(np.argmax(dd))
out["equivalence_input08"] = {
    "render_frames": len(a), "prod_convert_frames": len(b), "corrected_frames": len(c),
    "render_bit_identical_to_prod_convert": bool(len(a) == len(b) and np.array_equal(a, b)),
    "render_vs_corrected_max_abs_diff": float(dd.max()), "diff_frame": k,
    "frames_differing_gt_1e-6": int((dd > 1e-6).sum()),
    "delivered_frame955": a[955].tolist(), "corrected_frame955": c[955].tolist(),
    "receipt": {"sample_rate": rep["measurements"]["sample_rate"], "bit_depth": rep["measurements"]["bit_depth"],
                "true_peak_dbtp": rep["measurements"]["true_peak_dbtp"], "effective_target_lufs": rep["effective_target_lufs"]},
    "delivered_sample_peak_dbfs": float(20 * np.log10(np.abs(a).max())),
}

# ---- B. hot-start clip to CD profile ----
src = "hotstart_input04_48k_60-120s"
d, sr = sf.read(os.path.join(R, src + "_cd.wav"), dtype="float64", always_2d=True)
p, _ = sf.read(os.path.join(R, src + "_prod44.f32.wav"), dtype="float64", always_2d=True)
cc, _ = sf.read(os.path.join(R, src + "_recon44.f32.wav"), dtype="float64", always_2d=True)
rep = json.load(open(os.path.join(R, src + "_cd.json")))
n = min(len(d), len(p))
mask = np.ones(n, bool)
mask[955] = False
g = float(np.sum(d[:n][mask] * p[:n][mask]) / np.sum(p[:n][mask] ** 2))
resid = d[:n] - g * p[:n]
click = d[955] - g * cc[955]
out["hotstart_cd_click"] = {
    "clip": src + ".wav (input04 resampled to 48 kHz by soxr in the blind phase, 60-120 s)",
    "delivery": {"sample_rate": rep["measurements"]["sample_rate"], "bit_depth": rep["measurements"]["bit_depth"],
                 "lufs": rep["measurements"]["lufs_integrated"], "true_peak_dbtp": rep["measurements"]["true_peak_dbtp"]},
    "estimated_landing_gain_db": 20 * np.log10(g),
    "max_abs_resid_delivered_vs_g_prod": float(np.abs(resid).max()),
    "delivered_frame955": d[955].tolist(), "corrected_scaled_frame955": (g * cc[955]).tolist(),
    "click_error": click.tolist(), "click_dbfs": float(20 * np.log10(np.abs(click).max())),
    "context_L_951_959": d[951:960, 0].round(4).tolist(),
}
json.dump(out, open(os.path.join(RECON, "results", "src_music_checks.json"), "w"), indent=1)
print(json.dumps(out, indent=1))
