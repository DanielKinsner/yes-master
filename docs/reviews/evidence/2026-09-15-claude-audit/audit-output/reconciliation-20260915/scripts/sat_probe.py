"""Claim 4: saturation mapping, zero-continuity, aliasing and the reference
oversampling filters.

Native: the production chain with every other stage disabled (overrides
no_hpf no_eq no_comp no_transient no_width no_limiter no_input_gain) so only
`if amount > 0 { tanh(x*(1+2a)) / tanh(1+2a) }` acts; the amount is set with
the harness `sat=` override (0 = production bypass branch).
Analytic: the same formula in NumPy (float64).
Spectra: coherent FFT over integer seconds; harmonic power = bins at k*f0
(k>=2) below Nyquist; folded (nonharmonic) power = everything else outside
+/-2 bins of the fundamental/harmonics, above 20 Hz.
Oversampled reference: analytic curve applied at 4x/8x with (a) SciPy
resample_poly defaults (Kaiser beta 5, as Codex used) and (b) a long
Kaiser beta 14 filter, to show the reference's own filter dependence.
"""
import json
import os
import subprocess

import numpy as np
import soundfile as sf
from scipy.signal import resample_poly, firwin

RECON = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
HARNESS = os.path.join(RECON, "build", "target", "release", "yes-recon-harness.exe")
TONES = os.path.join(RECON, "renders", "sat_synth")
OUT = os.path.join(RECON, "renders", "sat_out")
os.makedirs(OUT, exist_ok=True)
OVR = ["no_hpf", "no_eq", "no_comp", "no_transient", "no_width", "no_limiter", "no_input_gain"]
AMOUNTS = {"bypass": 0.0, "epsilon": 1e-8, "half": 0.03575, "current_u75": 0.0715, "u50_amount": None}


def db(p):
    return float(10 * np.log10(max(p, 1e-30)))


def analytic(x, a):
    if a <= 0:
        return x
    d = 1 + 2 * a
    return np.tanh(x * d) / np.tanh(d)


def spectrum_stats(y, sr, f0):
    n = int(sr) * 2  # 2 coherent seconds from the middle
    s = y[len(y) // 2 - n // 2: len(y) // 2 + n // 2]
    S = np.abs(np.fft.rfft(s * np.hanning(n) * 2)) ** 2 / n ** 2
    f = np.fft.rfftfreq(n, 1 / sr)
    df = f[1] - f[0]
    k0 = int(round(f0 / df))
    fund = S[k0 - 2:k0 + 3].sum()
    harm = 0.0
    mask = np.ones_like(S, bool)
    mask[:int(20 / df)] = False
    mask[k0 - 2:k0 + 3] = False
    k = 2
    while k * f0 < sr / 2:
        kk = int(round(k * f0 / df))
        harm += S[kk - 2:kk + 3].sum()
        mask[kk - 2:kk + 3] = False
        k += 1
    folded = S[mask].sum()
    return {"fundamental_power": fund, "harmonic_dbc": db(harm / fund), "nonharmonic_dbc": db(folded / fund)}


def native(tone_path, a, tag):
    out = os.path.join(OUT, f"{tag}.f32.wav")
    spec = {"preset": "universal", "intensity": 0.75, "resolve_adaptive": False}
    cmd = [HARNESS, "chain", tone_path, out, "--spec-json", json.dumps(spec)]
    for o in OVR:
        cmd += ["--override", o]
    if a is not None:
        cmd += ["--override", f"sat={a}"]
    subprocess.run(cmd, capture_output=True, text=True, check=True)
    y, sr = sf.read(out, dtype="float64", always_2d=True)
    os.remove(out)
    return y[:, 0], sr


rows = []
tones = {"tone1k_-24": 1000, "tone1k_-12": 1000, "tone1k_-6": 1000, "tone5k_-12": 5000, "tone11k_0": 11000, "tone11k_-6": 11000, "tone15k_-6": 15000}
for name, f0 in tones.items():
    x, sr = sf.read(os.path.join(TONES, name + "_44k.wav"), dtype="float64", always_2d=True)
    x = x[:, 0]
    base = spectrum_stats(x, sr, f0)
    for mode, a in AMOUNTS.items():
        y, _ = native(os.path.join(TONES, name + "_44k.wav"), a, f"{name}_{mode}")
        # native chain output is delayed by the limiter lookahead ring even in
        # bypass (3 ms) and has the flushed tail; align by cross-correlation
        lag = 0  # harness `chain` output is delay-compensated by flush_render_tail
        ya = y[lag:lag + len(x)]
        m = min(len(ya), len(x))
        ya = ya[:m]
        st = spectrum_stats(ya, sr, f0)
        ana = analytic(x[:m], a if a is not None else 0.0)
        row = {"tone": name, "f0": f0, "mode": mode, "amount": a, "sr": sr,
               "fundamental_gain_db": db(st["fundamental_power"] / base["fundamental_power"]) / 1.0,
               "harmonic_dbc": st["harmonic_dbc"], "nonharmonic_dbc": st["nonharmonic_dbc"],
               "peak_gain_db": db(np.max(np.abs(ya)) ** 2 / np.max(np.abs(x[:m])) ** 2)}
        if a is not None:
            row["native_vs_analytic_max_err"] = float(np.max(np.abs(ya - ana)))
            row["analytic_small_signal_gain_db"] = float(20 * np.log10((1 + 2 * a) / np.tanh(1 + 2 * a))) if a > 0 else 0.0
        rows.append(row)
        print(f"{name:12s} {mode:12s} a={a}: fund gain {row['fundamental_gain_db']:+6.3f} dB, harmonics {row['harmonic_dbc']:7.2f} dBc, folded {row['nonharmonic_dbc']:7.2f} dBc, peak gain {row['peak_gain_db']:+6.3f}" + (f", |native-analytic| {row['native_vs_analytic_max_err']:.2e}" if a is not None else ""), flush=True)

# ---- oversampled analytic references for the 11 kHz full-scale case ----
# (a) SciPy resample_poly default window (Kaiser beta 5, 81 taps at 4x) as
#     Codex used; (b) the same with Kaiser beta 14; (c) exact FFT-domain
#     oversampling (zero-padded spectrum -> tanh -> brick-wall at the original
#     Nyquist -> decimate), whose residual is only the inherent aliasing of
#     the oversampled nonlinearity itself.
x, sr = sf.read(os.path.join(TONES, "tone11k_0_44k.wav"), dtype="float64", always_2d=True)
x = x[:, 0]
ref = []
def fft_os(x, factor, a):
    n = len(x)
    X = np.fft.rfft(x)
    up = np.fft.irfft(X, n=n * factor) * factor
    y = analytic(up, a)
    Y = np.fft.rfft(y)
    Y[len(X):] = 0  # brick-wall at the original Nyquist
    return np.fft.irfft(Y[:len(X)], n=n) / factor
for os_factor in (4, 8, 16):
    for win_name, win in (("resample_poly_kaiser5_default", ("kaiser", 5.0)), ("resample_poly_kaiser14", ("kaiser", 14.0)), ("fft_exact", None)):
        if win is None:
            down = fft_os(x, os_factor, 0.0715)
        else:
            up = resample_poly(x, os_factor, 1, window=win)
            down = resample_poly(analytic(up, 0.0715), 1, os_factor, window=win)
        st = spectrum_stats(down, sr, 11000)
        ref.append({"oversample": os_factor, "method": win_name, "harmonic_dbc": st["harmonic_dbc"], "nonharmonic_dbc": st["nonharmonic_dbc"]})
        print(f"11k 0 dBFS current amount, {os_factor}x {win_name}: harmonics {st['harmonic_dbc']:7.2f} dBc, folded {st['nonharmonic_dbc']:7.2f} dBc", flush=True)
json.dump({"native": rows, "oversampled_reference": ref}, open(os.path.join(RECON, "results", "sat_probe.json"), "w"), indent=1)
