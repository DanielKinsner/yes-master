"""Q6 candidate: target-aware pre-gain, implemented ONLY through the existing
user-facing Input Gain setting (no engine change).

Pass 1: run the production chain at unity input gain (harness `chain`,
native rate, no landing) and read the chain's own integrated loudness L1.
Pass 2: render normally with input_gain_db = g where
    A (attenuate-only):  g = min(0, target - L1)
    B (symmetric, capped): g = clamp(target - L1, -24, +6)
The production landing then trims the remainder exactly as today.

Compared against the existing main-matrix render (g = 0) on the same
source with independent measurements. Held-out sources are listed separately
and only evaluated after the candidate is fixed.
"""
import json
import os
import subprocess
import sys
from concurrent.futures import ProcessPoolExecutor

ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", ".."))
AUDIT = os.path.join(ROOT, "audit-output")
HARNESS = os.path.join(AUDIT, "build", "target", "release", "yes-audit-harness.exe")
sys.path.insert(0, os.path.join(AUDIT, "scripts"))

DEV = {
    "input01": "fixtures/inputs/input01.wav",
    "input02": "fixtures/inputs/input02.wav",
    "input03": "fixtures/inputs/input03.wav",
    "input04": "fixtures/inputs/input04.wav",
    "input06": "fixtures/inputs/input06.wav",
    "input08": "fixtures/inputs/input08.wav",
    "drums": "audit-output/synthetic/drums_60s.wav",
    "input02_-6dB": "audit-output/variants/input02_-6dB.wav",
    "input04_-12dB": "audit-output/variants/input04_-12dB.wav",
}
HELDOUT = {
    "input05": "fixtures/inputs/input05.wav",
    "input07": "fixtures/inputs/input07.wav",
    "heldout_drums": "audit-output/variants/heldout_drums_-6dB_96k.wav",
}
PRESETS = ["universal", "loud"]
TARGET = -14.0


def run(args):
    import measure
    sid, src, preset, variant = args
    inp = os.path.join(ROOT, src)
    odir = os.path.join(AUDIT, "renders", "candidate")
    os.makedirs(odir, exist_ok=True)
    spec = {"preset": preset, "intensity": 0.5, "delivery_profile": "streaming-universal"}
    # pass 1
    p1 = os.path.join(odir, f"{sid}_{preset}_pass1.f32.wav")
    r = subprocess.run([HARNESS, "chain", inp, p1, "--spec-json", json.dumps(spec), "--json", p1 + ".json"],
                       capture_output=True, text=True)
    if r.returncode != 0:
        return {"id": f"{sid}_{preset}_{variant}", "error": r.stderr[-800:]}
    L1 = json.load(open(p1 + ".json"))["pre_landing"]["lufs"]
    g = TARGET - L1
    if variant == "A":
        g = min(0.0, g)
    elif variant == "B":
        g = max(-24.0, min(6.0, g))
    else:
        g = 0.0
    spec2 = dict(spec, input_gain_db=round(g, 3))
    out = os.path.join(odir, f"{sid}_{preset}_{variant}.wav")
    if os.path.exists(out):
        os.remove(out)
    r = subprocess.run([HARNESS, "render", inp, out, "--spec-json", json.dumps(spec2), "--json", out + ".json"],
                       capture_output=True, text=True)
    if r.returncode != 0:
        return {"id": f"{sid}_{preset}_{variant}", "error": r.stderr[-800:]}
    rep = json.load(open(out + ".json"))
    m = measure.full(rep["output"], inp, light=False)
    d = m.get("third_octave_delta_db") or {}
    ref = d.get("1000") or 0.0
    # spectral deviation from source: RMS over bands 63..8k of (delta - delta@1k)
    bands = ["63", "125", "250", "500", "1000", "2000", "4000", "8000"]
    dev = [((d[b] - ref)) for b in bands if d.get(b) is not None]
    spec_dev = (sum(v * v for v in dev) / len(dev)) ** 0.5 if dev else None
    return {
        "id": f"{sid}_{preset}_{variant}", "source": sid, "preset": preset, "variant": variant,
        "L1_chain_lufs": L1, "pregain_db": g,
        "lufs": m["lufs_integrated"], "tp": m["true_peak_dbtp"], "lra": m["lra"],
        "crest_db": m["crest_db"], "psr_db": m["psr_db"], "p95_p10_db": m["p95_p10_db"],
        "spec_dev_db": spec_dev, "checks": [c["code"] for c in rep["checks"]],
        "receipt_lufs": rep["measurements"]["lufs_integrated"], "receipt_tp": rep["measurements"]["true_peak_dbtp"],
        "src_lufs": rep["analysis"]["lufs_integrated"], "src_lra": rep["analysis"]["dynamic_range_lu"],
    }


def main():
    which = sys.argv[1] if len(sys.argv) > 1 else "dev"
    srcs = DEV if which == "dev" else HELDOUT
    only = sys.argv[2:]
    if only:
        srcs = {k: v for k, v in srcs.items() if k in only}
    jobs = [(sid, src, p, v) for sid, src in srcs.items() for p in PRESETS for v in ("0", "A", "B")]
    rows = []
    with ProcessPoolExecutor(max_workers=10) as ex:
        for r in ex.map(run, jobs):
            rows.append(r)
            print(r["id"], "ERR" if "error" in r else f"g={r['pregain_db']:+.2f} lufs={r['lufs']:.2f} tp={r['tp']:.2f} crest={r['crest_db']:.2f} lra={r['lra']:.2f} psr={r['psr_db']:.2f} specdev={r['spec_dev_db']:.2f}", flush=True)
    out_path = os.path.join(AUDIT, "results", f"candidate_{which}.json")
    if os.path.exists(out_path):
        prev = {r["id"]: r for r in json.load(open(out_path))}
        for r in rows:
            prev[r["id"]] = r
        rows = list(prev.values())
    rows.sort(key=lambda r: r["id"])
    json.dump(rows, open(out_path, "w"), indent=1)
    # pass-1 float intermediates are large; hash them into the deleted-renders record and remove
    subprocess.run([sys.executable, os.path.join(AUDIT, "scripts", "prune_renders.py"), os.path.join(AUDIT, "renders", "candidate", "*_pass1.f32.wav")], cwd=AUDIT)


if __name__ == "__main__":
    main()
