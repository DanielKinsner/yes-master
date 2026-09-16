"""Bind the separately frozen three-case lower grid without replacing prior jobs."""
import argparse
import json
from pathlib import Path
import shutil
import subprocess

import soundfile as sf

from evaluate_drive import sha


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--prior-job", type=Path, required=True)
    parser.add_argument("--specification", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    assert not args.output.exists()
    prior = json.loads(args.prior_job.read_text(encoding="utf-8"))
    cases = [case for case in prior["cases"] if case["id"].endswith("-single-t14")]
    assert {case["id"] for case in cases} == {
        "coat-single-t14", "piano-single-t14", "imaginal-single-t14"}
    baseline_path = Path(prior["baseline_report"])
    assert sha(baseline_path) == prior["baseline_report_sha256"]
    baseline = json.loads(baseline_path.read_text(encoding="utf-8"))
    assert baseline["status"] == "complete"
    estimated_bytes = 0
    for case in cases:
        path = Path(case["report"])
        assert sha(path) == case["report_sha256"]
        source = json.loads(path.read_text(encoding="utf-8"))
        assert source["status"] == "complete"
        assert sha(Path(source["source"])) == source["source_sha256"]
        control = next(row for row in baseline["rows"]
                       if row["case"] == case["id"] and row["variant"] == "current")
        assert sha(Path(control["path"])) == control["sha256"]
        info = sf.info(source["source"])
        assert info.channels == 2
        estimated_bytes += 3 * (((info.frames * 48000 + info.samplerate - 1)
                                 // info.samplerate) * 8 + 128)
    reserve = 25 * 1024**3
    free = shutil.disk_usage(args.output.parent).free
    assert free >= reserve + estimated_bytes, ("25 GiB reserve", free, estimated_bytes)
    job = dict(experiment="bounded-lower-drive-v1", cases=cases,
               offsets_db=[0., -3., -6., -12.],
               specification=str(args.specification.resolve()),
               specification_sha256=sha(args.specification),
               baseline_report=prior["baseline_report"],
               baseline_report_sha256=prior["baseline_report_sha256"],
               prior_job_sha256=sha(args.prior_job),
               preparation_script_sha256=sha(Path(__file__)),
               local_commit=subprocess.check_output(["git", "rev-parse", "HEAD"], text=True).strip(),
               new_wav_bytes_estimate=estimated_bytes, minimum_free_reserve_bytes=reserve)
    args.output.write_text(json.dumps(job, indent=2, allow_nan=False) + "\n", encoding="utf-8")
    print("Frozen 3 controls and 9 new outputs; bytes:", estimated_bytes, "free:", free)


if __name__ == "__main__":
    main()
