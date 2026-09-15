"""Case runner: renders through the harness, measures independently, and
writes a ledger.

Usage: python run_matrix.py cases.json ledger_name [--workers N]

cases.json: [{"id": ..., "input": ..., "spec": {...}, "group": ...,
              "measure": "light"|"full", "repeat": 1}]
Outputs: audit-output/renders/<ledger>/<id>.wav (+ .json harness report)
         audit-output/results/<ledger>.json and .csv
"""
from __future__ import annotations

import csv
import json
import os
import subprocess
import sys
import time
from concurrent.futures import ProcessPoolExecutor, as_completed

ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", ".."))
AUDIT = os.path.join(ROOT, "audit-output")
HARNESS = os.path.join(AUDIT, "build", "target", "release", "yes-audit-harness.exe")
MEASURE = os.path.join(AUDIT, "scripts", "measure.py")
sys.path.insert(0, os.path.join(AUDIT, "scripts"))


def ffmpeg_ebur128(path):
    p = subprocess.run(
        ["ffmpeg", "-nostats", "-hide_banner", "-i", path, "-af", "ebur128=peak=true",
         "-f", "null", "-"],
        capture_output=True, text=True)
    txt = p.stderr
    out = {}
    tail = txt[txt.rfind("Summary:"):] if "Summary:" in txt else ""
    for line in tail.splitlines():
        s = line.strip()
        if s.startswith("I:"):
            out["I"] = float(s.split()[1])
        elif s.startswith("LRA:"):
            out["LRA"] = float(s.split()[1])
        elif s.startswith("Peak:"):
            out["Peak"] = float(s.split()[1])
    return out


def run_case(case, ledger):
    import measure  # noqa

    cid = case["id"]
    inp = case["input"] if os.path.isabs(case["input"]) else os.path.join(ROOT, case["input"])
    rdir = os.path.join(AUDIT, "renders", ledger)
    os.makedirs(rdir, exist_ok=True)
    out_wav = os.path.join(rdir, f"{cid}.wav")
    out_json = os.path.join(rdir, f"{cid}.json")
    spec_json = json.dumps(case["spec"])
    t0 = time.time()
    rows = []
    repeats = case.get("repeat", 1)
    hashes = []
    for k in range(repeats):
        target = out_wav if k == 0 else out_wav.replace(".wav", f"_rep{k}.wav")
        p = subprocess.run(
            [HARNESS, "render", inp, target, "--spec-json", spec_json, "--json",
             out_json if k == 0 else out_json.replace(".json", f"_rep{k}.json")],
            capture_output=True, text=True)
        if p.returncode != 0:
            return {"id": cid, "group": case.get("group"), "error": p.stderr[-2000:],
                    "input": case["input"], "spec": case["spec"]}
        hashes.append(measure.sha256(target))
    with open(out_json) as f:
        rep = json.load(f)
    actual_out = rep["output"]
    light = case.get("measure", "light") == "light"
    m = measure.full(actual_out, inp, light=light)
    ff = ffmpeg_ebur128(actual_out)
    meas = rep["measurements"]
    target_lufs = rep["effective_target_lufs"]
    ceiling = rep["effective_ceiling_dbtp"]
    row = {
        "id": cid,
        "group": case.get("group"),
        "input": case["input"],
        "spec": case["spec"],
        "elapsed_s": round(time.time() - t0, 2),
        "output": actual_out,
        "sha256": hashes[0],
        "repeat_identical": (len(set(hashes)) == 1) if repeats > 1 else None,
        "target_lufs": target_lufs,
        "ceiling_dbtp": ceiling,
        "receipt_lufs": meas["lufs_integrated"],
        "receipt_tp": meas["true_peak_dbtp"],
        "receipt_lra": meas["dynamic_range_lu"],
        "receipt_sr": meas["sample_rate"],
        "receipt_bits": meas["bit_depth"],
        "receipt_profile_digest": meas.get("source_profile_digest"),
        "checks": [c["code"] for c in rep["checks"]],
        "ind_lufs": m["lufs_integrated"],
        "ind_tp": m["true_peak_dbtp"],
        "ind_lra": m["lra"],
        "ind_sample_peak": m["sample_peak_dbfs"],
        "ind_sr": m["sample_rate"],
        "ind_subtype": m["subtype"],
        "ind_finite": m["finite"],
        "ind_frames": m["frames"],
        "length_delta_frames": m.get("length_delta_frames"),
        "lag_samples": m.get("lag_samples"),
        "max_abs_diff_vs_ref": m.get("max_abs_diff_vs_ref"),
        "crest_db": m["crest_db"],
        "psr_db": m["psr_db"],
        "p95_p10_db": m["p95_p10_db"],
        "corr": m["stereo"]["correlation"],
        "side_mid_db": m["stereo"]["side_mid_db"],
        "dc": m["dc_offset"],
        "clipped_run_fraction": m["clipped_run_fraction"],
        "ff_I": ff.get("I"),
        "ff_LRA": ff.get("LRA"),
        "ff_Peak": ff.get("Peak"),
        "src_lufs": rep["analysis"]["lufs_integrated"] if rep.get("analysis") else None,
        "src_tp": rep["analysis"]["true_peak_dbtp"] if rep.get("analysis") else None,
        "src_lra": rep["analysis"]["dynamic_range_lu"] if rep.get("analysis") else None,
        "src_profile_digest": rep["analysis"]["source_profile_digest"] if rep.get("analysis") else None,
    }
    if not light:
        row["third_octave_delta_db"] = m.get("third_octave_delta_db")
        row["band_shares_6"] = m.get("band_shares_6")
    # derived pass/fail per protocol §4
    row["A1_tp_excess_db"] = row["ind_tp"] - ceiling if ceiling is not None else None
    if target_lufs is not None and row["ind_lufs"] is not None and row["ind_lufs"] > -70:
        row["A2_lufs_err"] = row["ind_lufs"] - target_lufs
    else:
        row["A2_lufs_err"] = None
    row["A3_lufs_diff"] = row["ind_lufs"] - row["receipt_lufs"] if row["ind_lufs"] > -70 else None
    row["A3_tp_diff"] = row["ind_tp"] - row["receipt_tp"]
    row["A3_lra_diff"] = row["ind_lra"] - row["receipt_lra"]
    return row


def main():
    cases_path = sys.argv[1]
    ledger = sys.argv[2]
    workers = int(sys.argv[sys.argv.index("--workers") + 1]) if "--workers" in sys.argv else max(1, (os.cpu_count() or 4) - 2)
    with open(cases_path) as f:
        cases = json.load(f)
    rows = []
    t0 = time.time()
    with ProcessPoolExecutor(max_workers=workers) as ex:
        futs = {ex.submit(run_case, c, ledger): c["id"] for c in cases}
        for fut in as_completed(futs):
            r = fut.result()
            rows.append(r)
            tag = "ERR " if "error" in r else "ok  "
            extra = "" if "error" in r else f"tp_ex={r['A1_tp_excess_db']:+.2f} lufs_err={r['A2_lufs_err'] if r['A2_lufs_err'] is None else round(r['A2_lufs_err'],2)} checks={r['checks']}"
            print(f"{tag}{r['id']:40s} {extra}", flush=True)
    rows.sort(key=lambda r: r["id"])
    os.makedirs(os.path.join(AUDIT, "results"), exist_ok=True)
    with open(os.path.join(AUDIT, "results", f"{ledger}.json"), "w") as f:
        json.dump(rows, f, indent=1)
    keys = ["id", "group", "input", "target_lufs", "ceiling_dbtp", "receipt_lufs", "ind_lufs", "ff_I",
            "receipt_tp", "ind_tp", "ff_Peak", "receipt_lra", "ind_lra", "A1_tp_excess_db", "A2_lufs_err",
            "A3_lufs_diff", "A3_tp_diff", "A3_lra_diff", "crest_db", "psr_db", "p95_p10_db", "corr",
            "side_mid_db", "length_delta_frames", "lag_samples", "max_abs_diff_vs_ref", "ind_sr",
            "ind_subtype", "ind_finite", "clipped_run_fraction", "repeat_identical", "checks", "src_lufs",
            "src_tp", "src_lra", "sha256", "error"]
    with open(os.path.join(AUDIT, "results", f"{ledger}.csv"), "w", newline="") as f:
        w = csv.DictWriter(f, fieldnames=keys, extrasaction="ignore")
        w.writeheader()
        for r in rows:
            w.writerow({k: r.get(k) for k in keys})
    print(f"done {len(rows)} cases in {time.time()-t0:.0f}s -> results/{ledger}.json")


if __name__ == "__main__":
    main()
