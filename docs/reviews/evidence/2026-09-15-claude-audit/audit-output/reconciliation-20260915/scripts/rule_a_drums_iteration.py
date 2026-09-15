"""Verify the blind report's synthetic regression for rule A (Loud preset on
the synthetic drum train at -14: A under-shot the target by ~0.9 LU) and test
one bounded iteration (A2) on the production export path."""
import json, os, subprocess, sys
RECON = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
AUDIT = os.path.dirname(RECON)
sys.path.insert(0, os.path.join(RECON, "scripts")); from prune import sha
H = os.path.join(RECON, "build", "target", "release", "yes-recon-harness.exe")
rows = []
for src, preset in [("drums_60s.wav", "loud"), ("drums_60s.wav", "universal"), ("pink_-20_60s.wav", "loud")]:
    inp = os.path.join(AUDIT, "synthetic", src)
    base = {"preset": preset, "intensity": 0.5, "delivery_profile": "streaming-universal"}
    def render(g, tag):
        out = os.path.join(RECON, "renders", f"ruleA_{src[:-4]}_{preset}_{tag}.wav")
        j = json.loads(subprocess.run([H, "render", inp, out, "--spec-json", json.dumps(dict(base, input_gain_db=round(g, 3)))], capture_output=True, text=True, check=True).stdout)
        m = j["measurements"]; h = sha(out); os.remove(out)
        return {"trim": g, "lufs": m["lufs_integrated"], "tp": m["true_peak_dbtp"], "lra": m["dynamic_range_lu"], "checks": [c["code"] for c in j["checks"]], "sha256": h}
    chain_out = os.path.join(RECON, "renders", "tmp_chain.f32.wav"); src48 = os.path.join(RECON, "renders", "tmp_src48.f32.wav")
    subprocess.run([H, "chain", inp, chain_out, "--spec-json", json.dumps(base)], capture_output=True, text=True, check=True)
    subprocess.run([H, "src-file", chain_out, src48, "--to", "48000", "--mode", "prod"], capture_output=True, text=True, check=True)
    L1 = json.loads(subprocess.run([H, "measure", src48], capture_output=True, text=True, check=True).stdout)["lufs"]
    os.remove(chain_out); os.remove(src48)
    P = render(0.0, "P")
    gA = min(0.0, -14.0 - L1)
    A = render(gA, "A")
    short = -14.0 - A["lufs"]
    A2 = None
    if short > 0.2 and gA < 0:
        A2 = render(min(0.0, gA + short), "A2")
        short2 = -14.0 - A2["lufs"]
        A3 = render(min(0.0, A2["trim"] + short2), "A3") if short2 > 0.2 and A2["trim"] < 0 else None
    else:
        A3 = None
    row = {"source": src, "preset": preset, "L1": L1, "P": P, "A": A, "A2": A2, "A3": A3}
    rows.append(row)
    print(f"{src} {preset}: L1={L1:.2f}  P: LUFS {P['lufs']:.2f} TP {P['tp']:.2f} | A: g={gA:+.2f} LUFS {A['lufs']:.2f} TP {A['tp']:.2f} LRA {A['lra']:.2f}" + (f" | A2: g={A2['trim']:+.2f} LUFS {A2['lufs']:.2f} TP {A2['tp']:.2f}" if A2 else " | A2: not needed") + (f" | A3: g={A3['trim']:+.2f} LUFS {A3['lufs']:.2f}" if A3 else ""), flush=True)
json.dump(rows, open(os.path.join(RECON, "results", "rule_a_drums_iteration.json"), "w"), indent=1)
