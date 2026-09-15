"""Frequency response of the production rubato Fft resampler from the
impulse probe (src-impulse). Reports passband ripple, gain near the
transition band and the peak of the impulse response (a symmetric brick-wall
sinc with large ringing overshoots limited waveforms on resampling)."""
import json, os, numpy as np, soundfile as sf
RECON = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
P = os.path.join(RECON, "results", "src-impulse")
out = {}
for (f, t) in [(44100, 48000), (48000, 44100), (96000, 48000)]:
    y, sr = sf.read(os.path.join(P, f"impulse_{f}_{t}.wav"), dtype="float64", always_2d=True)
    h = y[:, 0]
    k = int(np.argmax(np.abs(h)))
    seg = h[max(0, k - 4096):k + 4096]
    H = np.fft.rfft(seg, n=1 << 18)
    fr = np.fft.rfftfreq(1 << 18, 1 / t)
    mag = 20 * np.log10(np.abs(H) * (t / f) + 1e-12)  # normalise by rate ratio (impulse energy scaling)
    mag -= mag[(fr > 900) & (fr < 1100)].mean()  # 0 dB at 1 kHz
    nyq = min(f, t) / 2
    band = (fr >= 20) & (fr <= 0.9 * nyq)
    trans = (fr >= 0.9 * nyq) & (fr <= nyq)
    stop = fr >= nyq * 1.02
    row = {
        "impulse_peak": float(np.abs(h).max()), "impulse_index": k,
        "ringing_sum_abs_over_peak": float(np.abs(seg).sum() / np.abs(h).max()),
        "passband_ripple_pp_db_20Hz_to_0.9nyq": float(mag[band].max() - mag[band].min()),
        "passband_max_db": float(mag[band].max()),
        "gain_at_0.95nyq_db": float(np.interp(0.95 * nyq, fr, mag)),
        "gain_at_0.99nyq_db": float(np.interp(0.99 * nyq, fr, mag)),
        "gain_at_nyq_db": float(np.interp(nyq, fr, mag)),
        "stopband_max_db": float(mag[stop].max()) if stop.any() else None,
        "transition_max_db": float(mag[trans].max()),
    }
    out[f"{f}->{t}"] = row
    print(f"{f}->{t}: " + ", ".join(f"{k}={v:.3f}" if isinstance(v, float) else f"{k}={v}" for k, v in row.items()))
json.dump(out, open(os.path.join(RECON, "results", "rubato_response.json"), "w"), indent=1)
