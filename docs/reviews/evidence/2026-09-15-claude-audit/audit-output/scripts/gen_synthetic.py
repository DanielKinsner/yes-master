"""Deterministic synthetic inputs (fixed seeds) for the audit.

Writes 32-bit float WAVs into audit-output/synthetic/. Each signal has a
known expectation documented in the protocol. Run once; hashes are recorded
in the evidence manifest.
"""
import os
import numpy as np
import soundfile as sf
from scipy import signal

OUT = os.path.join(os.path.dirname(__file__), "..", "synthetic")
os.makedirs(OUT, exist_ok=True)
SR = 44100


def write(name, x, sr=SR):
    x = np.asarray(x, dtype=np.float32)
    if x.ndim == 1:
        x = np.stack([x, x], axis=1)
    sf.write(os.path.join(OUT, name), x, sr, subtype="FLOAT")
    print(name, x.shape, sr)


def db(v):
    return 10 ** (v / 20)


def pink(n, rng):
    # Voss-free: shape white noise with a 1/f magnitude in the frequency domain
    w = rng.standard_normal(n)
    W = np.fft.rfft(w)
    f = np.fft.rfftfreq(n, 1 / SR)
    f[0] = f[1]
    W = W / np.sqrt(f)
    p = np.fft.irfft(W, n)
    return p / np.sqrt(np.mean(p * p))


def fade(x, ms=10):
    n = int(SR * ms / 1000)
    r = np.linspace(0, 1, n)
    x = x.copy()
    x[:n] *= r
    x[-n:] *= r[::-1]
    return x


rng = np.random.default_rng(20260914)
t60 = np.arange(60 * SR) / SR
t20 = np.arange(20 * SR) / SR

# 1. pink noise, -20 dBFS RMS, 60 s, decorrelated stereo at corr ~0.9
p1 = pink(60 * SR, rng)
p2 = pink(60 * SR, rng)
l = p1
r = 0.9 * p1 + np.sqrt(1 - 0.81) * p2
write("pink_-20_60s.wav", np.stack([l, r], 1) * db(-20))

# 2. sines for THD: 1 kHz at -20, -6, -1 dBFS (20 s)
for lvl in (-20, -6, -1):
    write(f"sine1k_{lvl}dBFS.wav", fade(np.sin(2 * np.pi * 1000 * t20)) * db(lvl))

# 3. SMPTE IMD: 60 Hz + 7 kHz, 4:1, peak -6 dBFS
imd = 4 * np.sin(2 * np.pi * 60 * t20) + np.sin(2 * np.pi * 7000 * t20)
write("imd_smpte_-6.wav", fade(imd / 5) * db(-6))

# 4. HF alias probes: 15 kHz and 19 kHz at -6 dBFS
write("sine15k_-6.wav", fade(np.sin(2 * np.pi * 15000 * t20)) * db(-6))
write("sine19k_-6.wav", fade(np.sin(2 * np.pi * 19000 * t20)) * db(-6))

# 5. drum-like impulse train: kick (60 Hz decaying) + snare (noise burst) at
#    120 BPM, 60 s, peak about -3 dBFS, with a pink bed at -30 dBFS.
d = np.zeros(60 * SR)
beat = SR // 2
for i in range(0, 60 * SR, beat):
    n = min(SR // 4, 60 * SR - i)
    tt = np.arange(n) / SR
    kick = np.sin(2 * np.pi * (60 + 40 * np.exp(-tt * 30)) * tt) * np.exp(-tt * 12)
    d[i:i + n] += kick
    if (i // beat) % 2 == 1:
        sn = rng.standard_normal(n) * np.exp(-tt * 25)
        d[i:i + n] += 0.6 * sn
d = d / np.max(np.abs(d)) * db(-3)
bed = pink(60 * SR, rng) * db(-30)
write("drums_60s.wav", d + bed)

# 6. full-scale square wave 100 Hz, 10 s
write("square100_0dBFS.wav", fade(np.sign(np.sin(2 * np.pi * 100 * t20[:10 * SR]))) * 0.999)

# 7. inter-sample peak probe: alternating +/-1 samples near Nyquist (max ISP)
#    A 0 dBFS sine at fs/4 sampled at 45 degrees gives true peak +3 dB.
tt = np.arange(10 * SR) / SR
isp = np.sin(2 * np.pi * (SR / 4) * tt + np.pi / 4) * 0.999
write("isp_+3dBTP.wav", fade(isp))

# 8. DC offset pink noise (+0.2 DC, -20 dBFS noise)
write("pink_dc0.2.wav", pink(20 * SR, rng) * db(-20) + 0.2)

# 9. silence 10 s, 50 ms clip, 2.5 s clip
write("silence_10s.wav", np.zeros(10 * SR))
write("clip_50ms.wav", (pink(int(0.05 * SR), rng) * db(-12)))
write("clip_2p5s.wav", fade(pink(int(2.5 * SR), rng) * db(-12)))

# 10. -60 dBFS noise floor probe
write("noise_-60.wav", pink(20 * SR, rng) * db(-60))

# 11. decorrelated wide stereo (corr ~0.0) pink at -20
write("pink_wide_corr0.wav", np.stack([pink(20 * SR, rng), pink(20 * SR, rng)], 1) * db(-20))

# 12. mono file and dual-mono, swapped stereo from drums
dd, _ = sf.read(os.path.join(OUT, "drums_60s.wav"), always_2d=True)
sf.write(os.path.join(OUT, "drums_mono.wav"), dd[:, 0].astype(np.float32), SR, subtype="FLOAT")
print("drums_mono.wav")

# 13. bright source (pink tilted +6 dB/oct above 2 kHz) and boomy source
p = pink(60 * SR, rng)
b, a = signal.butter(2, 2000 / (SR / 2), "high")
bright = p + 3.0 * signal.lfilter(b, a, p)
bright = bright / np.sqrt(np.mean(bright ** 2)) * db(-20)
write("pink_bright.wav", bright)
b, a = signal.butter(2, 150 / (SR / 2), "low")
boomy = p + 4.0 * signal.lfilter(b, a, p)
boomy = boomy / np.sqrt(np.mean(boomy ** 2)) * db(-20)
write("pink_boomy.wav", boomy)

# 14. tone burst train for limiter behaviour (1 kHz, 50 ms bursts every 500 ms, 0 dBFS)
tb = np.zeros(20 * SR)
for i in range(0, 20 * SR, SR // 2):
    n = int(0.05 * SR)
    tb[i:i + n] = np.sin(2 * np.pi * 1000 * np.arange(n) / SR) * np.hanning(n)
write("burst1k_0dBFS.wav", tb * 0.999)
