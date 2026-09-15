"""Independent audio measurements for the YES Master audit.

Everything here is implemented from the public standards (ITU-R BS.1770-4,
EBU Tech 3342) or plain signal statistics, without reference to the engine's
code, so it can arbitrate between the engine's meter and FFmpeg.

CLI:  python measure.py <wav> [--ref <source.wav>] [--json out.json]
"""
from __future__ import annotations

import argparse
import hashlib
import json
import sys

import numpy as np
import soundfile as sf
from scipy import signal


# ---------------------------------------------------------------------------
# I/O
# ---------------------------------------------------------------------------

def read(path):
    x, sr = sf.read(path, dtype="float64", always_2d=True)
    info = sf.info(path)
    return x, sr, info.subtype


def sha256(path):
    h = hashlib.sha256()
    with open(path, "rb") as f:
        for chunk in iter(lambda: f.read(1 << 20), b""):
            h.update(chunk)
    return h.hexdigest()


# ---------------------------------------------------------------------------
# BS.1770-4 K-weighting (coefficients derived per the standard's method for
# arbitrary sample rates, matching the tabulated 48 kHz values).
# ---------------------------------------------------------------------------

def _k_weighting(sr):
    # Stage 1: high shelf. Parameters from BS.1770-4 (f0=1681.97 Hz, G=+3.99 dB,
    # Q=0.7071752) via the RBJ shelf design used by the reference filter.
    f0 = 1681.974450955533
    G = 3.999843853973347
    Q = 0.7071752369554196
    K = np.tan(np.pi * f0 / sr)
    Vh = 10 ** (G / 20)
    Vb = Vh ** 0.4996667741545416
    a0 = 1 + K / Q + K * K
    b0 = (Vh + Vb * K / Q + K * K) / a0
    b1 = 2 * (K * K - Vh) / a0
    b2 = (Vh - Vb * K / Q + K * K) / a0
    a1 = 2 * (K * K - 1) / a0
    a2 = (1 - K / Q + K * K) / a0
    shelf = (np.array([b0, b1, b2]), np.array([1, a1, a2]))
    # Stage 2: RLB high-pass (f0=38.135 Hz, Q=0.5003).
    f0 = 38.13547087602444
    Q = 0.5003270373238773
    K = np.tan(np.pi * f0 / sr)
    a0 = 1 + K / Q + K * K
    a1 = 2 * (K * K - 1) / a0
    a2 = (1 - K / Q + K * K) / a0
    hp = (np.array([1.0, -2.0, 1.0]), np.array([1, a1, a2]))
    return shelf, hp


def _k_filter(x, sr):
    shelf, hp = _k_weighting(sr)
    y = signal.lfilter(shelf[0], shelf[1], x, axis=0)
    y = signal.lfilter(hp[0], hp[1], y, axis=0)
    return y


def _block_loudness(x, sr, block_s, hop_s):
    """Return per-block loudness (LKFS) for a K-weighted multichannel signal."""
    n = int(round(block_s * sr))
    hop = int(round(hop_s * sr))
    if x.shape[0] < n:
        return np.array([])
    starts = np.arange(0, x.shape[0] - n + 1, hop)
    # channel weights: 1.0 for L/R (and C); surround +1.5 dB not needed here
    e = np.empty(len(starts))
    sq = x * x
    csum = np.concatenate([np.zeros((1, x.shape[1])), np.cumsum(sq, axis=0)])
    for i, s in enumerate(starts):
        ms = (csum[s + n] - csum[s]) / n
        e[i] = ms.sum()
    with np.errstate(divide="ignore"):
        return -0.691 + 10 * np.log10(e)


def integrated_lufs(x, sr):
    y = _k_filter(x, sr)
    l = _block_loudness(y, sr, 0.400, 0.100)
    if l.size == 0:
        return float("-inf")
    gated = l[l > -70.0]
    if gated.size == 0:
        return float("-inf")
    # relative gate from the absolute-gated mean of block energies
    e = 10 ** ((gated + 0.691) / 10)
    rel = -0.691 + 10 * np.log10(e.mean()) - 10.0
    gated2 = gated[gated > rel]
    if gated2.size == 0:
        return float("-inf")
    e2 = 10 ** ((gated2 + 0.691) / 10)
    return float(-0.691 + 10 * np.log10(e2.mean()))


def loudness_range(x, sr):
    """EBU Tech 3342 LRA: 3 s short-term blocks, 1 s hop... (the spec uses a
    hop of at least 1 s; libebur128/FFmpeg use 100 ms). Use 100 ms hop to
    match the common implementations."""
    y = _k_filter(x, sr)
    l = _block_loudness(y, sr, 3.0, 0.100)
    if l.size == 0:
        return 0.0
    g = l[l > -70.0]
    if g.size == 0:
        return 0.0
    e = 10 ** ((g + 0.691) / 10)
    rel = -0.691 + 10 * np.log10(e.mean()) - 20.0
    g2 = np.sort(g[g > rel])
    if g2.size < 2:
        return 0.0
    lo = g2[int(np.floor(0.10 * (g2.size - 1)))]
    hi = g2[int(np.floor(0.95 * (g2.size - 1)))]
    return float(hi - lo)


def short_term_series(x, sr):
    y = _k_filter(x, sr)
    return _block_loudness(y, sr, 3.0, 0.100)


def momentary_series(x, sr):
    y = _k_filter(x, sr)
    return _block_loudness(y, sr, 0.400, 0.100)


def true_peak_dbtp(x, sr):
    """BS.1770-4 Annex 2 style: oversample by 4 (<=48k) or 2 (>48k) with a
    long windowed-sinc, take max |y|."""
    factor = 4 if sr <= 48000 else 2
    peak = 0.0
    for ch in range(x.shape[1]):
        # resample_poly uses a Kaiser-windowed sinc; use a long filter for
        # accurate reconstruction near Nyquist.
        y = signal.resample_poly(x[:, ch], factor, 1, window=("kaiser", 12.0), padtype="line")
        peak = max(peak, float(np.max(np.abs(y))) if y.size else 0.0)
    return 20 * np.log10(peak) if peak > 0 else -120.0


def sample_peak_dbfs(x):
    p = float(np.max(np.abs(x))) if x.size else 0.0
    return 20 * np.log10(p) if p > 0 else -120.0


def rms_dbfs(x):
    r = float(np.sqrt(np.mean(x * x))) if x.size else 0.0
    return 20 * np.log10(r) if r > 0 else -120.0


def dc_offset(x):
    return [float(v) for v in x.mean(axis=0)]


def p95_p10_db(x, sr):
    """P95 − P10 of 100 ms RMS blocks (50 ms hop), mono-summed."""
    m = x.mean(axis=1)
    n = int(0.100 * sr)
    hop = int(0.050 * sr)
    if m.size < n:
        return 0.0
    starts = np.arange(0, m.size - n + 1, hop)
    r = np.array([np.sqrt(np.mean(m[s:s + n] ** 2)) for s in starts])
    r = r[r > 1e-6]
    if r.size < 2:
        return 0.0
    db = 20 * np.log10(r)
    return float(np.percentile(db, 95) - np.percentile(db, 10))


def stereo_stats(x):
    if x.shape[1] < 2:
        return {"correlation": None, "side_mid_db": None}
    l, r = x[:, 0], x[:, 1]
    if np.std(l) < 1e-9 or np.std(r) < 1e-9:
        corr = None
    else:
        corr = float(np.corrcoef(l, r)[0, 1])
    mid = 0.5 * (l + r)
    side = 0.5 * (l - r)
    em = float(np.mean(mid * mid))
    es = float(np.mean(side * side))
    smdb = 10 * np.log10(es / em) if em > 0 and es > 0 else None
    return {"correlation": corr, "side_mid_db": smdb}


THIRD_OCT = [20, 25, 31.5, 40, 50, 63, 80, 100, 125, 160, 200, 250, 315, 400, 500,
             630, 800, 1000, 1250, 1600, 2000, 2500, 3150, 4000, 5000, 6300, 8000,
             10000, 12500, 16000, 20000]


def third_octave_spectrum_db(x, sr, nfft=8192):
    """Long-term band levels (dB, power per band) of the mono sum via Welch."""
    m = x.mean(axis=1)
    if m.size < nfft:
        nfft = max(256, 1 << int(np.log2(max(m.size, 256))))
    f, p = signal.welch(m, fs=sr, window="hann", nperseg=nfft, noverlap=nfft // 2,
                        scaling="spectrum")
    out = {}
    for fc in THIRD_OCT:
        lo, hi = fc / 2 ** (1 / 6), fc * 2 ** (1 / 6)
        if hi > sr / 2:
            out[str(fc)] = None
            continue
        sel = (f >= lo) & (f < hi)
        e = float(p[sel].sum()) if sel.any() else 0.0
        out[str(fc)] = 10 * np.log10(e) if e > 0 else None
    return out


def band_shares_6(x, sr):
    """Energy shares in the engine's six analysis bands (for R9 checks)."""
    m = x.mean(axis=1)
    f, p = signal.welch(m, fs=sr, window="hann", nperseg=8192, scaling="spectrum")
    edges = [(20, 80), (80, 250), (250, 800), (800, 2500), (2500, 6500), (6500, min(sr / 2, 16000))]
    e = [float(p[(f >= lo) & (f < hi)].sum()) for lo, hi in edges]
    t = sum(e)
    return [v / t if t > 0 else 0.0 for v in e]


def align_lag(a, b, sr, max_lag_s=0.05):
    """Lag (samples) of b relative to a using mono cross-correlation on a
    central 10 s excerpt. Positive = b is delayed."""
    am, bm = a.mean(axis=1), b.mean(axis=1)
    n = min(am.size, bm.size)
    seg = min(n, int(10 * sr))
    s0 = (n - seg) // 2
    aa = am[s0:s0 + seg]
    bb = bm[s0:s0 + seg]
    maxlag = int(max_lag_s * sr)
    aa = aa - aa.mean()
    bb = bb - bb.mean()
    c = signal.correlate(bb, aa, mode="full", method="fft")
    mid = aa.size - 1
    win = c[mid - maxlag: mid + maxlag + 1]
    return int(np.argmax(win) - maxlag)


def thd_db(x, sr, f0, nharm=10):
    """THD of a sine (mono sum): harmonic power 2..n over fundamental."""
    m = x.mean(axis=1)
    n = m.size
    w = np.hanning(n)
    X = np.abs(np.fft.rfft(m * w))
    freqs = np.fft.rfftfreq(n, 1 / sr)
    def bin_power(fc):
        i = int(round(fc / (sr / n)))
        lo, hi = max(0, i - 3), min(X.size, i + 4)
        return float(np.sum(X[lo:hi] ** 2))
    fund = bin_power(f0)
    harm = sum(bin_power(k * f0) for k in range(2, nharm + 1) if k * f0 < sr / 2)
    return 10 * np.log10(harm / fund) if fund > 0 and harm > 0 else -140.0


def spur_floor_db(x, sr, exclude_hz, exclude_bw=30.0):
    """Largest spectral component (dB re fundamental) outside the listed
    frequencies — aliasing / IMD probe."""
    m = x.mean(axis=1)
    n = m.size
    w = np.blackman(n)
    X = np.abs(np.fft.rfft(m * w)) ** 2
    freqs = np.fft.rfftfreq(n, 1 / sr)
    mask = np.ones_like(X, dtype=bool)
    for f in exclude_hz:
        mask &= ~((freqs > f - exclude_bw) & (freqs < f + exclude_bw))
    mask &= freqs > 20
    ref = max(X[~mask].max() if (~mask).any() else 1.0, 1e-30)
    spur = X[mask].max() if mask.any() else 1e-30
    idx = int(np.argmax(np.where(mask, X, 0)))
    return {"spur_db_re_fund": 10 * np.log10(spur / ref), "spur_freq_hz": float(freqs[idx])}


def clipped_run_fraction(x, thresh_db=-0.05, min_run=3):
    """Fraction of samples in flat runs at or above the threshold (flat-topping)."""
    t = 10 ** (thresh_db / 20)
    total = 0
    for ch in range(x.shape[1]):
        a = np.abs(x[:, ch]) >= t
        # count runs >= min_run
        d = np.diff(np.concatenate([[0], a.astype(int), [0]]))
        starts, ends = np.where(d == 1)[0], np.where(d == -1)[0]
        for s, e in zip(starts, ends):
            if e - s >= min_run:
                total += e - s
    return total / x.size if x.size else 0.0


def full(path, ref=None, light=False):
    x, sr, subtype = read(path)
    out = {
        "file": path,
        "sha256": sha256(path),
        "sample_rate": sr,
        "channels": x.shape[1],
        "frames": x.shape[0],
        "subtype": subtype,
        "finite": bool(np.isfinite(x).all()),
        "sample_peak_dbfs": sample_peak_dbfs(x),
        "true_peak_dbtp": true_peak_dbtp(x, sr),
        "rms_dbfs": rms_dbfs(x),
        "dc_offset": dc_offset(x),
        "lufs_integrated": integrated_lufs(x, sr),
        "lra": loudness_range(x, sr),
        "p95_p10_db": p95_p10_db(x, sr),
        "stereo": stereo_stats(x),
        "clipped_run_fraction": clipped_run_fraction(x),
    }
    out["crest_db"] = out["sample_peak_dbfs"] - out["rms_dbfs"]
    st = short_term_series(x, sr)
    out["lufs_short_term_max"] = float(st.max()) if st.size else None
    out["psr_db"] = (out["true_peak_dbtp"] - out["lufs_short_term_max"]) if st.size else None
    if not light:
        out["third_octave_db"] = third_octave_spectrum_db(x, sr)
        out["band_shares_6"] = band_shares_6(x, sr)
    if ref is not None:
        r, rsr, _ = read(ref)
        out["ref"] = {"file": ref, "frames": r.shape[0], "sample_rate": rsr}
        expected = int(round(r.shape[0] * sr / rsr))
        out["length_delta_frames"] = x.shape[0] - expected
        if rsr == sr:
            out["lag_samples"] = align_lag(r, x, sr)
            n = min(r.shape[0], x.shape[0])
            d = x[:n] - r[:n]
            out["max_abs_diff_vs_ref"] = float(np.max(np.abs(d))) if n else None
            out["lufs_delta_vs_ref"] = out["lufs_integrated"] - integrated_lufs(r, rsr)
        if not light:
            rs = third_octave_spectrum_db(r, rsr)
            out["third_octave_delta_db"] = {
                k: (out["third_octave_db"][k] - rs[k]) if (out["third_octave_db"].get(k) is not None and rs.get(k) is not None) else None
                for k in rs
            }
    return out


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("wav")
    ap.add_argument("--ref")
    ap.add_argument("--json")
    ap.add_argument("--light", action="store_true")
    a = ap.parse_args()
    res = full(a.wav, a.ref, a.light)
    text = json.dumps(res, indent=2)
    if a.json:
        with open(a.json, "w") as f:
            f.write(text)
    print(text)


if __name__ == "__main__":
    main()
