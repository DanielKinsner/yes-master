import wave, numpy as np, sys, os, glob
from scipy import signal

# 6 bands matching the app's spectral_balance_6band split.
BANDS = [("sub", 20, 80), ("low", 80, 250), ("lowmid", 250, 800),
         ("mid", 800, 2500), ("presence", 2500, 6500), ("air", 6500, 16000)]

def db(x):
    return 20.0 * np.log10(max(float(x), 1e-12))

def read_wav(path):
    with wave.open(path, "rb") as w:
        nch, sw, sr, n = w.getnchannels(), w.getsampwidth(), w.getframerate(), w.getnframes()
        raw = w.readframes(n)
    if sw == 2:
        a = np.frombuffer(raw, dtype="<i2").astype(np.float64) / 32768.0
    elif sw == 3:
        b = np.frombuffer(raw, dtype=np.uint8).reshape(-1, 3).astype(np.int32)
        a = b[:, 0] | (b[:, 1] << 8) | (b[:, 2] << 16)
        a = np.where(a & 0x800000, a - 0x1000000, a).astype(np.float64) / 8388608.0
    elif sw == 4:
        a = np.frombuffer(raw, dtype="<i4").astype(np.float64) / 2147483648.0
    else:
        raise ValueError(f"unsupported sample width {sw}")
    return a.reshape(-1, nch), sr

def analyze(path):
    a, sr = read_wav(path)
    peak = float(np.max(np.abs(a)))
    up = signal.resample_poly(a, 4, 1, axis=0)            # 4x oversample for true peak
    tp = float(np.max(np.abs(up)))
    mono = a.mean(axis=1)
    f, pxx = signal.welch(mono, sr, nperseg=16384)
    tot = float(np.sum(pxx)) or 1.0
    bands = {n: 100.0 * float(np.sum(pxx[(f >= lo) & (f < hi)])) / tot for n, lo, hi in BANDS}
    return db(peak), db(tp), bands

folder = sys.argv[1]
files = sorted(glob.glob(os.path.join(folder, "*.wav")))
hdr = f"{'file':<40}{'pk dBFS':>9}{'truePk':>8}   " + "".join(f"{n:>9}" for n, _, _ in BANDS)
print(hdr); print("-" * len(hdr))
for fp in files:
    pk, tp, b = analyze(fp)
    flag = "  <-- CLIPS" if tp > -0.1 else ""
    print(f"{os.path.basename(fp):<40}{pk:>9.2f}{tp:>8.2f}   "
          + "".join(f"{b[n]:>8.1f}%" for n, _, _ in BANDS) + flag)
