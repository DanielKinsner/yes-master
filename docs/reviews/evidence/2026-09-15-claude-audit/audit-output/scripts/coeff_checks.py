"""R9 / R10 / UI-1 invariants from coefficient dumps (no audio rendering).

* A9: guardrail multipliers within caps; strength 0 == no profile; reduce-only
* A10: intensity monotone for every preset scalar; preset directions
* UI-1: UI compressor readout formula == engine thresholds/ratios per density
"""
import json
import os
import subprocess

ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", ".."))
AUDIT = os.path.join(ROOT, "audit-output")
HARNESS = os.path.join(AUDIT, "build", "target", "release", "yes-audit-harness.exe")
SYN = lambda n: os.path.join(AUDIT, "synthetic", n)


def coeffs(spec, sr=48000, analysis=None):
    cmd = [HARNESS, "coeffs", "--sr", str(sr), "--spec-json", json.dumps(spec)]
    if analysis:
        cmd += ["--analysis", analysis]
    p = subprocess.run(cmd, capture_output=True, text=True, check=True)
    return json.loads(p.stdout)


PRESETS = ["universal", "clarity", "tape", "spatial", "oomph", "warmth", "punch", "loud", "custom"]
out = {"intensity": {}, "adapt": {}, "ui": {}, "presets": {}}

# ---- A10 intensity monotonicity & preset table ---------------------------------
keys = ["input_gain_db", "saturation_amount", "saturation_small_signal_gain_db", "transient_amount",
        "eq_sub_80_db", "eq_low_200_db", "eq_low_mid_400_db", "eq_mid_1500_db", "eq_high_mid_3500_db",
        "eq_high_6000_db", "eq_sparkle_12000_db", "width_side_scale", "comp_low_threshold_db",
        "comp_low_ratio", "comp_low_makeup_db"]
print("== preset table at intensity 0.5 (no profile)")
print(f"{'preset':10s} " + " ".join(f"{k[:11]:>11s}" for k in keys))
for p in PRESETS:
    c = coeffs({"preset": p, "intensity": 0.5, "resolve_adaptive": False})
    out["presets"][p] = {k: c[k] for k in keys}
    print(f"{p:10s} " + " ".join(f"{c[k]:11.3f}" for k in keys))

print("\n== intensity sweep (Universal): monotone check")
mono_ok = True
for p in PRESETS:
    series = []
    for i in (0.0, 0.25, 0.5, 0.75, 1.0):
        c = coeffs({"preset": p, "intensity": i, "resolve_adaptive": False})
        series.append({k: c[k] for k in keys})
    out["intensity"][p] = series
    for k in keys:
        vals = [s[k] for s in series]
        diffs = [b - a for a, b in zip(vals, vals[1:])]
        if not (all(d >= -1e-6 for d in diffs) or all(d <= 1e-6 for d in diffs)):
            mono_ok = False
            print(f"  NON-MONOTONE {p} {k}: {vals}")
    if p == "universal":
        for s, i in zip(series, (0.0, 0.25, 0.5, 0.75, 1.0)):
            print(f"  int={i:4.2f} gain={s['input_gain_db']:.2f} sat={s['saturation_amount']:.4f} (+{s['saturation_small_signal_gain_db']:.2f} dB small-signal) air6k={s['eq_high_6000_db']:.2f} low200={s['eq_low_200_db']:.2f} trans={s['transient_amount']:.3f} thr={s['comp_low_threshold_db']:.2f} ratio={s['comp_low_ratio']:.2f} makeup={s['comp_low_makeup_db']:.2f}")
print("  monotone for all presets/keys:", mono_ok)

# ---- A9 adaptive guardrails ------------------------------------------------------
print("\n== adaptive guardrails (Universal, intensity 0.5)")
base = coeffs({"preset": "universal", "intensity": 0.5, "resolve_adaptive": False})
for src in ("pink_-20_60s.wav", "pink_bright.wav", "pink_boomy.wav", "pink_wide_corr0.wav", "drums_60s.wav"):
    row = {}
    for strength in (0.0, 0.5, 1.0):
        c = coeffs({"preset": "universal", "intensity": 0.5, "adapt_strength": strength}, analysis=SYN(src))
        row[strength] = c
        print(f"  {src:22s} s={strength}: air6k={c['eq_high_6000_db']:.3f} (base {base['eq_high_6000_db']:.3f}) low200={c['eq_low_200_db']:.3f} (base {base['eq_low_200_db']:.3f}) width={c['width_side_scale']:.3f} (base {base['width_side_scale']:.3f}) thr={c['comp_low_threshold_db']:.2f} (base {base['comp_low_threshold_db']:.2f}) ratio={c['comp_low_ratio']:.3f}")
    out["adapt"][src] = row
    # reduce-only + caps + strength-0 identity
    c0 = row[0.0]
    same = all(abs(c0[k] - base[k]) < 1e-6 for k in keys)
    print(f"      strength 0 identical to no-profile: {same}")
    for s in (0.5, 1.0):
        c = row[s]
        assert c["eq_high_6000_db"] <= base["eq_high_6000_db"] + 1e-6
        assert c["eq_low_200_db"] <= base["eq_low_200_db"] + 1e-6
        assert c["width_side_scale"] <= base["width_side_scale"] + 1e-6
        assert c["comp_low_threshold_db"] >= base["comp_low_threshold_db"] - 1e-6
        # caps: EQ trim <= 50 % of boost (floor 0.5 dB), width <= 70 %, density <= 60 %
        # dump reads a shelf at its corner (= half the shelf gain); full shelf gain = 2x corner.
        full_base = 2 * base["eq_high_6000_db"]; full_c = 2 * c["eq_high_6000_db"]
        if full_base > 0.5:
            assert full_c >= max(0.5, 0.5 * full_base) - 1e-3, (full_base, full_c)
        assert c["width_side_scale"] >= 1.0 + 0.3 * (base["width_side_scale"] - 1.0) - 1e-6
print("  reduce-only / caps assertions passed")

# ---- UI-1 compressor readout formula ------------------------------------------------
print("\n== UI-1: engine thresholds vs UI formula per density")
PRESET_COMPRESSOR = {
    "universal": (-12.5, 1.45), "clarity": (-12.5, 1.45), "tape": (-16, 1.65), "spatial": (-16, 1.8),
    "oomph": (-15.5, 1.7), "warmth": (-19, 2.0), "punch": (-20, 2.8), "loud": (-23, 4.09),
    "custom": (-16, 1.8)}
ui_ok = True
for p in PRESETS:
    for d in (0.0, 0.25, 0.5, 0.75, 1.0):
        c = coeffs({"preset": p, "intensity": 0.5, "compression_density": d, "resolve_adaptive": False})
        thr0, r0 = PRESET_COMPRESSOR[p]
        eng = min(d * 2, 1)
        od = max(d * 2 - 1, 0)
        thr = thr0 * eng + (-3) * od
        ratio = max(1, 1 + (r0 - 1) * eng + 0.5 * od)
        ok = abs(c["comp_low_threshold_db"] - thr) < 1e-3 and abs(c["comp_low_ratio"] - ratio) < 1e-3
        ui_ok &= ok
        out["ui"][f"{p}_{d}"] = {"engine_thr": c["comp_low_threshold_db"], "ui_thr": thr, "engine_ratio": c["comp_low_ratio"], "ui_ratio": ratio, "active": c["compression_active"]}
        if not ok:
            print(f"  MISMATCH {p} d={d}: engine thr {c['comp_low_threshold_db']:.2f} ratio {c['comp_low_ratio']:.2f} vs UI {thr:.2f}/{ratio:.2f}")
print("  UI formula matches engine for all presets/densities:", ui_ok)
print("  note: intensity does not enter the compressor formula (engine or UI)")

# sample-rate consistency of static EQ response (44.1 / 48 / 96)
print("\n== static EQ response vs sample rate (Universal 0.5, dB at probe freqs)")
resp = {}
for sr in (44100, 48000, 96000):
    c = coeffs({"preset": "universal", "intensity": 0.5, "resolve_adaptive": False}, sr=sr)
    resp[sr] = dict((str(int(f)), v) for f, v in c["static_response_db"])
for fq in ("20", "40", "80", "200", "1000", "4000", "8000", "12500", "16000", "20000"):
    print(f"  {fq:>6s} Hz: " + "  ".join(f"{sr}: {resp[sr].get(fq, float('nan')):6.2f}" for sr in resp))
out["eq_response_by_rate"] = resp

json.dump(out, open(os.path.join(AUDIT, "results", "coeff_checks.json"), "w"), indent=1)
