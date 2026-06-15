import numpy as np, wave, os, shutil

sr, dur = 44100, 20
n = sr * dur

def pink(n):
    w = np.random.randn(n)
    f = np.fft.rfftfreq(n)
    f[0] = f[1]                      # avoid div-by-zero at DC
    x = np.fft.irfft(np.fft.rfft(w) / np.sqrt(f), n=n)
    return x / np.max(np.abs(x))

# Two independent channels, peak ~ -12 dBFS to leave the chain headroom.
st = np.stack([pink(n) * 0.25, pink(n) * 0.25], axis=1)
i16 = (np.clip(st, -1, 1) * 32767).astype("<i2")

outdir = "test-output/noise-src"
os.makedirs(outdir, exist_ok=True)
src = os.path.join(outdir, "noise-original-test.wav")
with wave.open(src, "wb") as w:
    w.setnchannels(2); w.setsampwidth(2); w.setframerate(sr)
    w.writeframes(i16.tobytes())

# The reference runner needs a per-preset reference file to exist; we only care
# about the YES renders it produces, so copy the source as dummy references.
for p in ["universal", "clarity", "oomph", "tape"]:
    shutil.copy(src, os.path.join(outdir, f"noise-{p}-test.wav"))

print("wrote pink-noise source + 4 dummy refs in", outdir)
