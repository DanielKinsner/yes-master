"""Join verified whole-chain E evidence and locate gain-removed PCM differences."""
import argparse
import json
from pathlib import Path

import numpy as np
import soundfile as sf

from evaluate_drive import sha


def read(path):
    return json.loads(path.read_text(encoding="utf-8"))


def differences(control, candidate):
    assert control["gain"] > 0 and candidate["gain"] > 0
    maximum = first = last = energy = reference_energy = 0.0
    location = channel = offset = 0
    with sf.SoundFile(control["path"]) as ref, sf.SoundFile(candidate["path"]) as actual:
        assert (len(ref), ref.samplerate, ref.channels) == (len(actual), actual.samplerate, actual.channels)
        edge = ref.samplerate // 100
        for chunk in ref.blocks(blocksize=65536, dtype="float64", always_2d=True):
            value = actual.read(len(chunk), dtype="float64", always_2d=True)
            chunk /= control["gain"]
            value /= candidate["gain"]
            delta = value - chunk
            flat = np.abs(delta)
            frame, ch = np.unravel_index(flat.argmax(), flat.shape)
            if flat[frame, ch] > maximum:
                maximum, location, channel = float(flat[frame, ch]), offset + int(frame), int(ch)
            if offset < edge:
                first = max(first, float(flat[:edge - offset].max()))
            if offset + len(chunk) > len(ref) - edge:
                last = max(last, float(flat[max(0, len(ref) - edge - offset):].max()))
            energy += float(np.sum(delta * delta))
            reference_energy += float(np.sum(chunk * chunk))
            offset += len(chunk)
        assert offset == len(ref) and actual.tell() == len(actual)
        return dict(maximum=maximum, frame=location, seconds=location / ref.samplerate,
                    channel=channel, first_10ms_maximum=first, last_10ms_maximum=last,
                    error_rms_relative_db=float(10 * np.log10(energy / reference_energy)) if energy else None)


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--root", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    assert not args.output.exists()
    paths = {"native": args.root / "e-whole-v1/report.json",
             "independent": args.root / "e-whole-independent-v1/comparison.json",
             "metrics": args.root / "e-whole-metrics-v1.json"}
    reports = {name: read(path) for name, path in paths.items()}
    assert all(report["status"] == "complete" and len(report["rows"]) == 16 for report in reports.values())
    assert reports["native"]["f64_isolation_pass"]
    assert reports["metrics"]["native_sha256"] == sha(paths["native"])
    independent = {row["path"]: row for row in reports["independent"]["rows"]}
    metrics = {row["path"]: row for row in reports["metrics"]["rows"]}
    native = reports["native"]["rows"]
    assert len(independent) == len(metrics) == len({row["path"] for row in native}) == 16
    controls = {row["case"]: row for row in native if row["variant"] == "current"}
    assert len(controls) == 4
    rows = []
    for row in native:
        checked, measured = independent[row["path"]], metrics[row["path"]]
        assert row["sha256"] == sha(Path(row["path"])) == checked["sha256"] == measured["sha256"]
        assert checked["peak_pass"] and checked["lufs_pass"] and checked["fullscale_samples"] == 0
        assert row["peak"] <= row["ceiling"]
        if row["variant"] == "current":
            assert row["raw_difference_from_current"] == 0
        elif row["variant"] == "wide":
            assert row["raw_difference_from_current"] <= 1e-5
        rows.append(dict(case=row["case"], variant=row["variant"], sha256=row["sha256"],
                         delivered_lufs=row["lufs"], raw_lufs=row["raw_lufs"], native_peak=row["peak"],
                         independent_peak=checked["independent_peak"], gain=row["gain"],
                         target_error_lu=measured["target_error_lu"], render=row["render"],
                         src_s=row["src_s"], finalize_s=row["finalize_s"],
                         raw_difference_from_current=row["raw_difference_from_current"],
                         source_delta=measured["source_delta"], matched_current_delta=measured["matched_current_delta"],
                         c1_proposal_failures=measured["c1_proposal_failures"],
                         gain_removed_delivery_difference=(differences(controls[row["case"]], row)
                                                           if row["variant"] in ("short", "long") else None)))
    result = dict(status="complete", reports={name: sha(path) for name, path in paths.items()},
                  script_sha256=sha(Path(__file__)), independent_passes=16,
                  kernel_read_and_prepare_s=reports["native"]["kernel_read_and_prepare_s"], rows=rows,
                  scope="Whole fixed-duration development songs; source/matched metrics and change locations do not certify audibility, alias-only attribution, selected C or actual-device performance")
    args.output.write_text(json.dumps(result, indent=2, allow_nan=False) + "\n", encoding="utf-8")
    print("Verified 16 whole files, four exact controls, four f64 isolation checks and eight difference scans")


if __name__ == "__main__":
    main()
