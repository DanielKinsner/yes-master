"""Join qualified whole files and fixed C1 metrics; retain both policy outcomes."""
import argparse
import json
from pathlib import Path

from drive_metrics import LIMITS, compare, constraints
from evaluate_drive import sha


def read(path):
    return json.loads(path.read_text(encoding="utf-8"))


def select(rows, preserving):
    eligible = [row for row in rows if not preserving or not row["character_failures"]]
    fallback = not eligible
    if fallback:
        eligible = [row for row in rows if row["offset_db"] == 0]
    hits = [row for row in eligible if abs(row["target_error_lu"]) <= LIMITS["target_error_lu"]]
    if hits:
        winner = min(hits, key=lambda row: abs(row["offset_db"]))
    else:
        winner = min(eligible, key=lambda row: (abs(row["target_error_lu"]), abs(row["offset_db"])))
    return dict(offset_db=winner["offset_db"], lufs=winner["lufs"],
                target_error_lu=winner["target_error_lu"],
                fallback=fallback, character_failures=winner["character_failures"],
                reason="no_character_qualified_candidate" if fallback else
                       "target_hit" if hits else "candidate_with_target_shortfall")


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--root", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    assert not args.output.exists()
    paths = dict(job=args.root / "c1-lower-drive-job-v1.json",
                 native=args.root / "c1-lower-drive-v1/report.json",
                 metrics=args.root / "c1-lower-drive-metrics-v1.json",
                 independent=args.root / "c1-lower-drive-independent-v1/comparison.json",
                 baseline_metrics=args.root / "e-whole-metrics-v1.json",
                 metrics_map=args.root / "e-whole-metrics-inputs-v1.json")
    data = {key: read(path) for key, path in paths.items()}
    job, native, metrics, independent = [data[key] for key in ("job", "native", "metrics", "independent")]
    assert all(data[key]["status"] == "complete" for key in ("native", "metrics", "independent", "baseline_metrics"))
    assert all(data[key]["experiment"] == "bounded-lower-drive-v1" for key in ("job", "native", "metrics"))
    assert native["job_sha256"] == sha(paths["job"])
    assert independent["all_pass"] and metrics["limits"] == LIMITS
    assert metrics["native_sha256"] == independent["native_sha256"] == sha(paths["native"])
    assert metrics["baseline_metrics_sha256"] == sha(paths["baseline_metrics"])
    assert metrics["metrics_map_sha256"] == sha(paths["metrics_map"])
    assert sha(Path(job["specification"])) == job["specification_sha256"]
    assert sha(Path(job["baseline_report"])) == job["baseline_report_sha256"]
    assert len(native["rows"]) == len(metrics["rows"]) == len(independent["rows"]) == 12
    controls = {row["case"]: row for row in data["baseline_metrics"]["rows"] if row["variant"] == "current"}
    measured = {(row["case"], row["offset_db"]): row for row in metrics["rows"]}
    checked = {str(Path(row["path"]).resolve()): row for row in independent["rows"]}
    assert len(measured) == len(checked) == 12
    result = dict(status="complete", scope="Three known development tracks; fixed lower grid, no holdout or adoption",
                  inputs={key: dict(path=str(path), sha256=sha(path)) for key, path in paths.items()},
                  script_sha256=sha(Path(__file__)), limits=LIMITS, cases=[],
                  reused_independent_controls=0, new_independent_outputs=0)
    for case in job["cases"]:
        name = case["id"]
        source_metrics_path = Path(data["metrics_map"][name])
        assert metrics["source_metrics"][name] == sha(source_metrics_path)
        facts = read(source_metrics_path)["source"]
        rows = []
        for row in native["rows"]:
            if row["case"] != name:
                continue
            metric = measured[(name, row["offset_db"])]
            check = checked[str(Path(row["path"]).resolve())]
            assert row["sha256"] == metric["sha256"] == check["sha256"] == sha(Path(row["path"]))
            assert row["frames"] == check["frames"] == metric["metrics"]["frames"]
            assert row["rate"] == check["rate"] == metric["metrics"]["rate"] == 48000
            assert row["channels"] == check["channels"] == metric["metrics"]["channels"] == 2
            assert check["peak_pass"] and check["lufs_pass"] and check["fullscale_samples"] == 0
            assert row["peak"] <= row["ceiling"] == check["ceiling"] == -1
            assert row["lufs"] == metric["lufs"] == check["native_lufs"]
            settings = row["requested_settings"]
            assert settings["delivery_profile"] == "custom" and settings["album"] is None
            assert settings["advanced"]["lufs_offset_db"] == -14
            assert compare(facts, metric["metrics"]) == metric["source_delta"]
            failures = constraints(facts, metric["source_delta"], controls[name]["source_delta"])
            assert failures == metric["descriptive_character_failures"]
            reused = "verification_reused_from" in check
            assert reused == row["reused_exact_control"] == (row["offset_db"] == 0)
            if reused:
                assert row["sha256"] == controls[name]["sha256"]
            result["reused_independent_controls"] += int(reused)
            result["new_independent_outputs"] += int(not reused)
            cost = dict(chain_s=row["render"]["chain_compensation_and_observation_s"],
                        src_s=row["src_s"], finalize_s=row["finalize_s"])
            rows.append(dict(offset_db=row["offset_db"], lufs=row["lufs"],
                             target_error_lu=row["lufs"] + 14, peak=row["peak"],
                             character_failures=failures, source_delta=metric["source_delta"],
                             limiter_max_db=row["render"]["limiter_max_db"], cost=cost,
                             evaluation_s=sum(cost.values()),
                             fresh_independent_check_s=0 if reused else check["reference_s"],
                             path=row["path"], sha256=row["sha256"]))
        assert {row["offset_db"] for row in rows} == {0., -3., -6., -12.}
        single = next(row for row in rows if row["offset_db"] == 0)
        qualifies = not single["character_failures"] and abs(single["target_error_lu"]) <= LIMITS["target_error_lu"]
        result["cases"].append(dict(case=name, rows=rows,
            preserving=select(rows, True), target_first=select(rows, False),
            single_evaluation_s=single["evaluation_s"],
            all_four_evaluation_s=sum(row["evaluation_s"] for row in rows),
            optional_extra_work_if_single_qualifies_s=sum(row["evaluation_s"] for row in rows if row["offset_db"] != 0) if qualifies else None))
    assert result["reused_independent_controls"] == 3 and result["new_independent_outputs"] == 9
    args.output.write_text(json.dumps(result, indent=2, allow_nan=False) + "\n", encoding="utf-8")
    print(json.dumps([dict(case=case["case"], preserving=case["preserving"],
                          target_first=case["target_first"]) for case in result["cases"]], indent=2))


if __name__ == "__main__":
    main()
