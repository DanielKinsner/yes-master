"""Derived input variants for the dimension sweeps (protocol §2, dims 7-9).

* level: exact scalar gain (-6, -12 dB) in float64, written as float32 WAV
* rate: FFmpeg soxr resample to 48 k / 96 k (documented external SRC)
* channels: mono (L only), dual-mono (L copied to R), swapped (R,L)
* held-out: drums at -6 dB, 96 kHz
"""
import os
import subprocess
import numpy as np
import soundfile as sf

ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", ".."))
OUT = os.path.join(ROOT, "audit-output", "variants")
os.makedirs(OUT, exist_ok=True)


def wpath(name):
    return os.path.join(OUT, name)


def gain_variant(src, name, db):
    x, sr = sf.read(src, dtype="float64", always_2d=True)
    y = (x * 10 ** (db / 20)).astype(np.float32)
    sf.write(wpath(name), y, sr, subtype="FLOAT")
    print(name)


def rate_variant(src, name, rate):
    subprocess.run(["ffmpeg", "-y", "-nostats", "-hide_banner", "-loglevel", "error", "-i", src,
                    "-af", f"aresample=resampler=soxr:precision=28:osr={rate}",
                    "-c:a", "pcm_f32le", wpath(name)], check=True)
    print(name)


def channel_variants(src, stem):
    x, sr = sf.read(src, dtype="float32", always_2d=True)
    sf.write(wpath(f"{stem}_mono.wav"), x[:, 0], sr, subtype="FLOAT")
    sf.write(wpath(f"{stem}_dualmono.wav"), np.stack([x[:, 0], x[:, 0]], 1), sr, subtype="FLOAT")
    sf.write(wpath(f"{stem}_swapped.wav"), x[:, ::-1], sr, subtype="FLOAT")
    # 6-channel: L R C LFE Ls Rs with C = 0.5*(L+R), LFE = low-passed mid, surrounds = 0.3*L/R
    from scipy import signal
    b, a = signal.butter(2, 120 / (sr / 2))
    mid = 0.5 * (x[:, 0] + x[:, 1])
    lfe = signal.lfilter(b, a, mid).astype(np.float32)
    six = np.stack([x[:, 0], x[:, 1], 0.5 * mid, lfe, 0.3 * x[:, 0], 0.3 * x[:, 1]], 1).astype(np.float32)
    sf.write(wpath(f"{stem}_6ch.wav"), six, sr, subtype="FLOAT")
    print(stem, "channel variants")


for i in ("02", "04", "06"):
    src = os.path.join(ROOT, "fixtures", "inputs", f"input{i}.wav")
    gain_variant(src, f"input{i}_-6dB.wav", -6.0)
    gain_variant(src, f"input{i}_-12dB.wav", -12.0)
    rate_variant(src, f"input{i}_48k.wav", 48000)
    rate_variant(src, f"input{i}_96k.wav", 96000)

channel_variants(os.path.join(ROOT, "fixtures", "inputs", "input04.wav"), "input04")

# held-out synthetic: drums -6 dB at 96 k
drums = os.path.join(ROOT, "audit-output", "synthetic", "drums_60s.wav")
gain_variant(drums, "drums_-6dB.wav", -6.0)
rate_variant(wpath("drums_-6dB.wav"), "heldout_drums_-6dB_96k.wav", 96000)
# and a +6 dB hot version of the drums (peaks above 0 dBFS in float) for R12
gain_variant(drums, "drums_+6dB_overfs.wav", 6.0)
