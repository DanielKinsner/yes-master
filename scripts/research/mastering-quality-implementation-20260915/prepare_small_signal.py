"""Freeze low-drive diagnostic inputs from retained E cases, with a disk reserve."""
import argparse
import json
from pathlib import Path
import shutil
import subprocess

import soundfile as sf

from evaluate_drive import sha


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--job", type=Path, required=True)
    parser.add_argument("--baseline", type=Path, required=True)
    parser.add_argument("--specification", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    assert not args.output.exists()
    original = json.loads(args.job.read_text(encoding="utf-8"))
    baseline = json.loads(args.baseline.read_text(encoding="utf-8"))
    assert baseline["status"] == "complete" and baseline["job_sha256"] == sha(args.job)
    assert len(original["cases"]) == 4
    estimated_bytes = 0
    for case in original["cases"]:
        prior_path = Path(case["report"])
        assert sha(prior_path) == case["report_sha256"]
        prior = json.loads(prior_path.read_text(encoding="utf-8"))
        assert prior["status"] == "complete" and sha(Path(prior["source"])) == prior["source_sha256"]
        info = sf.info(prior["source"])
        estimated_bytes += 2 * ((info.frames * 48000 + info.samplerate - 1) // info.samplerate * 8 + 44)
    reserve = 25 * 1024**3
    free = shutil.disk_usage(args.output.parent).free
    assert free >= reserve + estimated_bytes, ("Preserve 25 GiB after the eight new files", free, estimated_bytes)
    job = dict(cases=original["cases"], specification=str(args.specification.resolve()),
               specification_sha256=sha(args.specification), baseline_report=str(args.baseline.resolve()),
               baseline_report_sha256=sha(args.baseline), original_job_sha256=sha(args.job),
               preparation_script_sha256=sha(Path(__file__)),
               local_commit=subprocess.check_output(["git", "rev-parse", "HEAD"], text=True).strip(),
               new_wav_bytes_estimate=estimated_bytes, minimum_free_reserve_bytes=reserve)
    args.output.write_text(json.dumps(job, indent=2, allow_nan=False) + "\n", encoding="utf-8")
    print("Frozen four cases; new WAV bytes:", estimated_bytes, "free bytes:", free)


if __name__ == "__main__":
    main()
