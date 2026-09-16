"""Measure new low-drive files with unchanged C1 anchors and retained exact controls."""
import argparse
import json
from pathlib import Path
import time

import soundfile as sf

from drive_metrics import LIMITS, compare, constraints, measure
from evaluate_drive import sha


def read(path):
    return json.loads(path.read_text(encoding="utf-8"))


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--report", type=Path, required=True)
    parser.add_argument("--metrics-map", type=Path, required=True)
    parser.add_argument("--baseline-metrics", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--lower-grid", action="store_true",
                        help="Separate fixed three-case 0/-3/-6/-12 experiment; no convergence claim")
    args = parser.parse_args()
    assert not args.output.exists()
    mapping = read(args.metrics_map)
    baseline = read(args.baseline_metrics)
    assert baseline["status"] == "complete"
    facts = {}
    for case, path in mapping.items():
        source = read(Path(path))
        assert source["status"] == "complete"
        facts[case] = source["source"]
    controls = {row["case"]: row for row in baseline["rows"] if row["variant"] == "current"}
    assert len(controls) == 4 and controls.keys() == facts.keys()
    result = dict(status="running", rows=[], limits=LIMITS,
                  baseline_metrics_sha256=sha(args.baseline_metrics), metrics_map_sha256=sha(args.metrics_map),
                  script_sha256=sha(Path(__file__)), source_metrics={case: sha(Path(path)) for case, path in mapping.items()},
                  scope="Fixed source and matched-current C1 metrics; existing limits descriptive; no small-signal convergence or sonic adoption inferred solely from limiter inactivity")
    if args.lower_grid:
        result.update(experiment="bounded-lower-drive-v1",
                      scope="Fixed corrected-chain lower-drive grid with original C1 anchors and limits; no adoption")
    checked, lower = {}, {}
    while True:
        try:
            native = read(args.report)
        except (FileNotFoundError, json.JSONDecodeError):
            time.sleep(5)
            continue
        for row in native["rows"]:
            if args.lower_grid:
                assert native["experiment"] == "bounded-lower-drive-v1"
                assert row["case"] in {"coat-single-t14", "piano-single-t14", "imaginal-single-t14"}
                assert row["offset_db"] in {0., -3., -6., -12.}
            path = Path(row["path"])
            if str(path) in checked:
                assert checked[str(path)] == row["sha256"]
                continue
            assert sha(path) == row["sha256"]
            source, control = facts[row["case"]], controls[row["case"]]
            if row["offset_db"] == 0:
                assert row["reused_exact_control"] and control["sha256"] == row["sha256"]
                metrics = control["metrics"]
            else:
                metrics = measure(path, source["anchors"])
            info = sf.info(row["source"])
            assert metrics["frames"] == (info.frames * 48000 + info.samplerate - 1) // info.samplerate
            assert metrics["rate"] == 48000 and metrics["channels"] == info.channels == 2
            delta = compare(source, metrics)
            prior = lower.get(row["case"])
            if row["offset_db"] == -24:
                lower[row["case"]] = metrics
            result["rows"].append(dict(case=row["case"], offset_db=row["offset_db"], path=str(path), sha256=row["sha256"],
                render=row["render"], metrics=metrics, source_delta=delta,
                matched_current_delta=compare(control["metrics"], metrics),
                matched_m24_delta=compare(prior, metrics) if row["offset_db"] == -48 else None,
                descriptive_character_failures=constraints(source, delta, control["source_delta"]),
                lufs=row["lufs"], peak=row["peak"], gain=row["gain"]))
            checked[str(path)] = row["sha256"]
            args.output.write_text(json.dumps(result, indent=2, allow_nan=False) + "\n", encoding="utf-8")
            print("Measured", row["case"], row["offset_db"], flush=True)
        if native["status"] == "complete":
            assert len(result["rows"]) == 12
            assert len(native["convergence"]) == (0 if args.lower_grid else 4)
            if args.lower_grid:
                assert len({(row["case"], row["offset_db"]) for row in result["rows"]}) == 12
            result.update(status="complete", native_sha256=sha(args.report), convergence=native["convergence"])
            args.output.write_text(json.dumps(result, indent=2, allow_nan=False) + "\n", encoding="utf-8")
            return
        assert native["status"] == "running"
        time.sleep(5)


if __name__ == "__main__":
    main()
