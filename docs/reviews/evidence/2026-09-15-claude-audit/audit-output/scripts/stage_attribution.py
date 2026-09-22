"""Q5 — what each stage contributes and what the chain costs.

Runs the production chain (harness `chain` subcommand, native rate, float
output, no SRC) with coefficient overrides that switch stages off, on several
sources, and measures each output independently. Two series:

* cumulative: input gain only -> +HPF -> +EQ -> +comp -> +transient -> +width
  -> +sat -> +limiter (= full chain), then + landing
* ablation: full chain minus one stage at a time

Outputs results/stage_attribution.json and a text table.
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

ALL = ["no_hpf", "no_eq", "no_comp", "no_transient", "no_width", "no_sat", "no_limiter"]
CUMULATIVE = [
    ("00_gain_only", ALL),
    ("01_+hpf", [o for o in ALL if o != "no_hpf"]),
    ("02_+eq", [o for o in ALL if o not in ("no_hpf", "no_eq")]),
    ("03_+comp", [o for o in ALL if o not in ("no_hpf", "no_eq", "no_comp")]),
    ("04_+transient", [o for o in ALL if o not in ("no_hpf", "no_eq", "no_comp", "no_transient")]),
    ("05_+width", ["no_sat", "no_limiter"]),
    ("06_+sat", ["no_limiter"]),
    ("07_full", []),
]
ABLATION = [(f"abl_{o}", [o]) for o in ALL] + [("abl_no_input_gain", ["no_input_gain"])]

SOURCES = {
    "input01": "fixtures/inputs/input01.wav",
    "input02": "fixtures/inputs/input02.wav",
    "input04": "fixtures/inputs/input04.wav",
    "input06": "fixtures/inputs/input06.wav",
    "drums": "audit-output/synthetic/drums_60s.wav",
}
SPEC = {"preset": "universal", "intensity": 0.5, "delivery_profile": "streaming-universal"}


def run(args):
    import measure
    src_id, src, name, overrides, land = args
    inp = os.path.join(ROOT, src)
    odir = os.path.join(AUDIT, "renders", "stages")
    os.makedirs(odir, exist_ok=True)
    tag = f"{src_id}_{name}{'_land' if land else ''}"
    out = os.path.join(odir, f"{tag}.wav")
    cmd = [HARNESS, "chain", inp, out, "--spec-json", json.dumps(SPEC), "--json", out + ".json"]
    for o in overrides:
        cmd += ["--override", o]
    if land:
        cmd.append("--land")
    p = subprocess.run(cmd, capture_output=True, text=True)
    if p.returncode != 0:
        return {"id": tag, "error": p.stderr[-1000:]}
    rep = json.load(open(out + ".json"))
    m = measure.full(out, inp, light=False)
    return {
        "id": tag, "source": src_id, "stage": name, "overrides": overrides, "landed": land,
        "pre_lufs": rep["pre_landing"]["lufs"], "pre_tp": rep["pre_landing"]["true_peak_dbtp"],
        "pre_lra": rep["pre_landing"]["lra"], "landing_db": rep["landing_db"],
        "lufs": m["lufs_integrated"], "tp": m["true_peak_dbtp"], "lra": m["lra"],
        "crest_db": m["crest_db"], "psr_db": m["psr_db"], "p95_p10_db": m["p95_p10_db"],
        "corr": m["stereo"]["correlation"], "side_mid_db": m["stereo"]["side_mid_db"],
        "lag_samples": m.get("lag_samples"), "length_delta": m.get("length_delta_frames"),
        "third_octave_delta_db": m.get("third_octave_delta_db"),
        "band_shares_6": m.get("band_shares_6"),
    }


def main():
    jobs = []
    for sid, src in SOURCES.items():
        for name, ov in CUMULATIVE:
            jobs.append((sid, src, name, ov, False))
        jobs.append((sid, src, "07_full", [], True))
        for name, ov in ABLATION:
            jobs.append((sid, src, name, ov, False))
    rows = []
    with ProcessPoolExecutor(max_workers=10) as ex:
        for r in ex.map(run, jobs):
            rows.append(r)
            print(r.get("id"), "ERR" if "error" in r else f"lufs={r['lufs']:.2f} tp={r['tp']:.2f} crest={r['crest_db']:.2f} lra={r['lra']:.2f} lag={r['lag_samples']}", flush=True)
    rows.sort(key=lambda r: r["id"])
    json.dump(rows, open(os.path.join(AUDIT, "results", "stage_attribution.json"), "w"), indent=1)


if __name__ == "__main__":
    main()
