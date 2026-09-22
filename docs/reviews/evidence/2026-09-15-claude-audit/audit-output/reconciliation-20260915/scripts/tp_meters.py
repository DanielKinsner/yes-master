"""Claim 2: true-peak meters on byte-identical files.

Meters:
  engine   : the crate's ebur128 meter via the harness `measure` command (what
             the limiter/landing/receipt use; 4x for 48 kHz)
  bs1770   : ITU-R BS.1770-4 Annex 2 reference: the published 48-tap, 4-phase
             FIR (coefficients transcribed from the Recommendation), 4x for
             48 kHz / 44.1 kHz, 2x for 96 kHz (own NumPy implementation)
  ffmpeg   : ffmpeg ebur128=peak=true (reports 0.1 dB precision)
  soxr16   : ffmpeg aresample to 16x with soxr precision 33, astats peak
             (Codex's documented reconstruction reference)
  fftexact : chunked FFT zero-padding 16x interpolation (ideal band-limited
             reconstruction of the sampled signal; 2^18 chunks, 8192 overlap)
  sample   : sample peak

Usage: tp_meters.py <out.json> <wav> [<wav> ...]
"""
import json
import os
import re
import subprocess
import sys

import numpy as np
import soundfile as sf

HERE = os.path.dirname(os.path.abspath(__file__))
RECON = os.path.dirname(HERE)
HARNESS = os.path.join(RECON, "build", "target", "release", "yes-recon-harness.exe")

# ITU-R BS.1770-4, Annex 2, "One set of filter coefficients (for the order 48,
# 4-phase, FIR interpolating)". Rows = taps 0..11, columns = phase 0..3.
BS1770_PHASES = np.array([
    [0.0017089843750, -0.0291748046875, -0.0189208984375, -0.0083007812500],
    [0.0109863281250, 0.0292968750000, 0.0330810546875, 0.0148925781250],
    [-0.0196533203125, -0.0517578125000, -0.0582275390625, -0.0266113281250],
    [0.0332031250000, 0.0891113281250, 0.1015625000000, 0.0476074218750],
    [-0.0594482421875, -0.1665039062500, -0.2003173828125, -0.1022949218750],
    [0.1373291015625, 0.4650878906250, 0.7797851562500, 0.9721679687500],
    [0.9721679687500, 0.7797851562500, 0.4650878906250, 0.1373291015625],
    [-0.1022949218750, -0.2003173828125, -0.1665039062500, -0.0594482421875],
    [0.0476074218750, 0.1015625000000, 0.0891113281250, 0.0332031250000],
    [-0.0266113281250, -0.0582275390625, -0.0517578125000, -0.0196533203125],
    [0.0148925781250, 0.0330810546875, 0.0292968750000, 0.0109863281250],
    [-0.0083007812500, -0.0189208984375, -0.0291748046875, 0.0017089843750],
])


def db(v):
    return float(20 * np.log10(max(float(v), 1e-12)))


def bs1770_tp(x, sr):
    """4x (48k/44.1k) polyphase FIR from the Recommendation; 2x for 96 kHz
    uses phases 0 and 2 of the same bank (the Recommendation says 2x is
    sufficient at 96 kHz but gives no separate table; flagged as approximate)."""
    phases = [0, 1, 2, 3] if sr <= 48000 else [0, 2]
    peak = 0.0
    for ch in range(x.shape[1]):
        s = x[:, ch]
        for ph in phases:
            y = np.convolve(s, BS1770_PHASES[:, ph], mode="full")
            peak = max(peak, float(np.abs(y).max()))
    return db(peak)


def fft_exact_tp(x, sr, factor=16, chunk=1 << 18, overlap=8192):
    peak = 0.0
    n = len(x)
    for ch in range(x.shape[1]):
        s = x[:, ch]
        start = 0
        while start < n:
            a = max(0, start - overlap)
            b = min(n, start + chunk + overlap)
            seg = s[a:b]
            m = len(seg)
            S = np.fft.rfft(seg)
            up = np.fft.irfft(S, n=m * factor) * factor
            lo = (start - a) * factor
            hi = lo + min(chunk, n - start) * factor
            peak = max(peak, float(np.abs(up[lo:hi]).max()))
            start += chunk
    return db(peak)


def ffmpeg_tp(path):
    r = subprocess.run(["ffmpeg", "-hide_banner", "-nostats", "-i", path, "-af", "ebur128=peak=true", "-f", "null", "-"],
                       capture_output=True, text=True)
    summ = r.stderr.rsplit("Summary:", 1)[-1]
    tp = float(re.search(r"Peak:\s*([-\d.]+) dBFS", summ)[1])
    lufs = float(re.search(r"I:\s*([-\d.]+) LUFS", summ)[1])
    return tp, lufs


def soxr_tp(path, sr, factor=16):
    r = subprocess.run(["ffmpeg", "-hide_banner", "-nostats", "-i", path, "-af",
                        f"aresample={sr*factor}:resampler=soxr:precision=33,astats=metadata=0:reset=0", "-f", "null", "-"],
                       capture_output=True, text=True)
    return float(re.findall(r"Peak level dB: ([-\d.]+)", r.stderr)[-1])


def engine_tp(path):
    r = subprocess.run([HARNESS, "measure", path], capture_output=True, text=True, check=True)
    j = json.loads(r.stdout)
    return j["true_peak_dbtp"], j["lufs"]


def measure_file(path):
    x, sr = sf.read(path, dtype="float64", always_2d=True)
    eng_tp, eng_lufs = engine_tp(path)
    ff_tp, ff_lufs = ffmpeg_tp(path)
    return {
        "file": path, "sample_rate": sr, "frames": len(x), "channels": x.shape[1],
        "sample_peak_dbfs": db(np.abs(x).max()),
        "engine_tp": eng_tp, "engine_lufs": eng_lufs,
        "bs1770_tp": bs1770_tp(x, sr),
        "ffmpeg_tp": ff_tp, "ffmpeg_lufs": ff_lufs,
        "soxr16_tp": soxr_tp(path, sr, 16),
        "fftexact16_tp": fft_exact_tp(x, sr, 16),
    }


if __name__ == "__main__":
    out = sys.argv[1]
    rows = []
    for p in sys.argv[2:]:
        r = measure_file(p)
        rows.append(r)
        print(f"{os.path.basename(p):48s} sr={r['sample_rate']} sample={r['sample_peak_dbfs']:6.2f} engine={r['engine_tp']:6.3f} bs1770={r['bs1770_tp']:6.3f} ffmpeg={r['ffmpeg_tp']:5.1f} soxr16={r['soxr16_tp']:6.3f} fftexact16={r['fftexact16_tp']:6.3f}", flush=True)
    json.dump(rows, open(out, "w"), indent=1)
