"""Check retained alternating native FIR trials and summarize their measured cost."""
import argparse
import hashlib
import json
from pathlib import Path
import statistics

import numpy as np


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def read(path):
    return json.loads(path.read_text(encoding="utf-8"))


def keyed(report):
    assert report["status"] == "complete"
    rows = {(row["filter"], row["signal"]): row for row in report["rows"]}
    assert len(rows) == len(report["rows"]) == 8
    return rows


def median_time(row):
    return statistics.median(trial["processing_s"] for trial in row["trials"])


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--root", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    assert not args.output.exists()
    root = args.root
    job_path = root / "e-native-opt-job-v1.json"
    job = read(job_path)
    original_job = read(root / "e-native-inputs-v1/job.json")
    for key in ("rate", "amount", "channels", "filters", "signals"):
        assert job[key] == original_job[key]
    for entry in job["filters"] + job["signals"]:
        assert sha(Path(entry["path"])) == entry["sha256"]

    anchors = {"base": keyed(read(root / "e-native-v1/report.json")),
               "opt": keyed(read(root / "e-native-opt-p1/report.json"))}
    references = {}
    for variant, name, report_name in (
        ("base", "e-native-comparison-v1.json", "e-native-v1/report.json"),
        ("opt", "e-native-opt-comparison-v1.json", "e-native-opt-p1/report.json"),
    ):
        reference = read(root / name)
        assert reference["status"] == "complete"
        assert reference["native_sha256"] == sha(root / report_name)
        assert len(reference["rows"]) == 8
        assert all(row["pass_error"] and row["maximum_error"] <= 1e-10
                   for row in reference["rows"])
        references[variant] = {"path": name, "sha256": sha(root / name),
                               "maximum_error": max(row["maximum_error"] for row in reference["rows"])}

    order = ["base-p1", "opt-p1", "opt-p2", "base-p2", "base-p3", "opt-p3"]
    reports = {}
    files = {}
    for name in order:
        path = root / f"e-native-{name}/report.json"
        report = read(path)
        assert report["job_sha256"] == sha(job_path)
        rows = keyed(report)
        anchor = anchors[name.split("-")[0]]
        assert rows.keys() == anchor.keys()
        for key, row in rows.items():
            assert sha(Path(row["path"])) == row["sha256"] == anchor[key]["sha256"]
            for field in ("rate", "channels", "frames_in", "frames_out", "delay_frames",
                          "state_bytes", "kernel_bytes"):
                assert row[field] == anchor[key][field]
            assert [trial["partition"] for trial in row["trials"]] == [1, 257, 1024]
        reports[name] = rows
        files[name] = sha(path)

    summary = {}
    for filter_id in ("filter128", "filter512"):
        pairs = []
        for pair in (1, 2, 3):
            for key, base in reports[f"base-p{pair}"].items():
                if key[0] != filter_id:
                    continue
                opt = reports[f"opt-p{pair}"][key]
                pairs.append({"pair": pair, "signal": key[1],
                              "base_processing_ms": median_time(base) * 1000,
                              "opt_processing_ms": median_time(opt) * 1000,
                              "opt_over_base": median_time(opt) / median_time(base)})
        variants = {}
        for variant in ("base", "opt"):
            rows = [row for name, report in reports.items() if name.startswith(variant)
                    for key, row in report.items() if key[0] == filter_id]
            blocks = [ns / 1e6 for row in rows for trial in row["trials"]
                      for ns in trial["block_256_ns"]]
            interval_ms = 256 / job["rate"] * 1000
            variants[variant] = {
                "median_processing_ms": statistics.median(map(median_time, rows)) * 1000,
                "median_table_preparation_ms": statistics.median(row["preparation_s"] for row in rows) * 1000,
                "median_state_construction_ms": statistics.median(trial["construction_s"] for row in rows for trial in row["trials"]) * 1000,
                "block_ms_median": statistics.median(blocks),
                "block_ms_p95": float(np.percentile(blocks, 95)), "block_ms_max": max(blocks),
                "block_interval_ms": interval_ms, "blocks": len(blocks),
                "interval_exceedances": sum(value > interval_ms for value in blocks),
            }
        summary[filter_id] = {"pairs": pairs, "variants": variants,
                              "median_opt_over_base": statistics.median(row["opt_over_base"] for row in pairs)}
    result = {"status": "complete", "repeat_hashes_pass": 48, "order": order,
              "job_sha256": sha(job_path), "reports": files, "references": references,
              "script_sha256": sha(Path(__file__)), "filters": summary,
              "executables": {name: sha(root / name) for name in ("e-native-v1.exe", "e-native-opt-v1.exe")},
              "scope": "Alternating loaded elapsed native prototype costs; no isolated CPU, production or actual-device deadline claim"}
    args.output.write_text(json.dumps(result, indent=2, allow_nan=False) + "\n", encoding="utf-8")
    print(json.dumps(summary, indent=2))


if __name__ == "__main__":
    main()
