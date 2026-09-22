"""M/S ratio and correlation for the P/A/C finalists at -14 on four sources
(re-rendered; the policy run did not capture the nested stereo keys)."""
import json, os, subprocess, sys
import numpy as np, soundfile as sf
RECON = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
sys.path.insert(0, os.path.join(RECON, "scripts")); from prune import sha
H = os.path.join(RECON, "build", "target", "release", "yes-recon-harness.exe")
d = json.load(open(os.path.join(RECON, "results", "policy_compare.json")))
sel = d["selection"]; grid = d["grid"]
BASE = {"preset": "universal", "intensity": 0.75, "delivery_profile": "custom", "ceiling_dbtp": -1.0, "sample_rate": 48000, "bit_depth": 24, "compression_density": 0.5, "adapt_strength": 0.5, "lufs_target": -14.0}
def ms(x):
    m = x.mean(axis=1); s = (x[:, 0] - x[:, 1]) / 2
    return 10 * np.log10(np.mean(s ** 2) / max(np.mean(m ** 2), 1e-30)), float(np.corrcoef(x.T)[0, 1])
rows = []
for src in ["coat", "metal", "rich", "imaginal"]:
    inp = grid[f"{src}_0"]["input"]; x, sr = sf.read(inp, dtype="float64", always_2d=True); sm0, c0 = ms(x)
    for pol in ["P", "A", "C"]:
        trim = sel[f"{src}_0_t14"][pol]["trim"]
        out = os.path.join(RECON, "renders", f"ms_{src}_{pol}.wav")
        subprocess.run([H, "render", inp, out, "--spec-json", json.dumps(dict(BASE, input_gain_db=round(trim, 3)))], capture_output=True, text=True, check=True)
        y, _ = sf.read(out, dtype="float64", always_2d=True); smy, cy = ms(y); h = sha(out); os.remove(out)
        rows.append({"source": src, "policy": pol, "trim": trim, "source_side_mid_db": sm0, "side_mid_db": smy, "delta_side_mid_vs_source_db": smy - sm0, "source_corr": c0, "corr": cy, "sha256": h})
        print(f"{src:9s} {pol} trim {trim:+6.2f}: S/M {smy:6.2f} dB (source {sm0:6.2f}, delta {smy-sm0:+.2f}); corr {cy:.3f} (source {c0:.3f})", flush=True)
json.dump(rows, open(os.path.join(RECON, "results", "policy_ms_check.json"), "w"), indent=1)
