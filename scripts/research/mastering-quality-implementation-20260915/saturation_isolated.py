"""Frozen E mechanism comparisons; synthetic signals only, no production mutation."""
import argparse
import hashlib
import json
from pathlib import Path
import platform
import time

import numpy as np
import scipy
from scipy.signal import firwin, resample_poly


def curve(x, amount, knee=None):
    if amount == 0:
        return x.copy()
    drive = 1 + 2 * amount
    wet = np.tanh(x * drive) / np.tanh(drive)
    if knee is None:
        return wet
    t = np.clip(amount / knee, 0, 1)
    blend = t * t * (3 - 2 * t)
    return x + blend * (wet - x)


def db(value):
    return float(20 * np.log10(max(float(value), 1e-30)))


def analytic_reference(amplitude, amount, knee, n, tone_bin):
    max_harmonic = (n // 2 - 1) // tone_bin
    transforms = []
    for size in (131072, 262144):
        phase = np.arange(size) * (2 * np.pi / size)
        transforms.append(np.fft.rfft(curve(amplitude * np.sin(phase), amount, knee)) / size)
    difference = float(np.max(np.abs(transforms[0][:max_harmonic + 1] - transforms[1][:max_harmonic + 1])))
    assert difference <= 1e-10, difference
    expected = np.zeros(n // 2 + 1, dtype=np.complex128)
    for harmonic in range(1, max_harmonic + 1):
        expected[harmonic * tone_bin] = transforms[1][harmonic] * n
    return np.fft.irfft(expected, n), difference


def metrics(actual, reference, amplitude, tone_bin):
    n = len(actual)
    transformed = np.fft.rfft(actual) / n
    expected = np.fft.rfft(reference) / n
    bins = np.arange(1, (n // 2 - 1) // tone_bin + 1) * tone_bin
    legitimate = np.zeros(len(transformed), dtype=bool)
    legitimate[bins] = True
    fundamental = abs(transformed[tone_bin])
    assert fundamental > 0
    harmonics = float(np.sqrt(np.sum(np.abs(transformed[bins[1:]]) ** 2)))
    aliases = float(np.sqrt(np.sum(np.abs(transformed[~legitimate]) ** 2)))
    match = abs(expected[tone_bin]) / fundamental
    residual = actual * match - reference
    return dict(fundamental_gain_db=db(fundamental * 2 / amplitude),
                harmonics_dbc=db(harmonics / fundamental), nonharmonic_dbc=db(aliases / fundamental),
                fundamental_vs_ideal_db=db(fundamental / abs(expected[tone_bin])),
                matched_error_dbc=db(np.sqrt(np.mean(residual ** 2)) / np.sqrt(np.mean(reference ** 2))),
                matched_maximum_error=float(np.max(np.abs(residual))), peak=float(np.max(np.abs(actual))))


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--taps-per-factor', type=int, default=32)
    parser.add_argument('--cutoff', type=float, default=.92)
    parser.add_argument('--alias-only', action='store_true')
    parser.add_argument('--protocol', type=Path)
    args = parser.parse_args()
    assert not args.output.exists(), 'retain prior evidence'
    args.output.mkdir(parents=True)
    root = Path(__file__).resolve().parents[3]
    protocol = args.protocol or root / 'docs/reviews/2026-09-15-saturation-calibration-protocol.md'
    sha = lambda path: hashlib.sha256(path.read_bytes()).hexdigest()
    report = dict(status='running', protocol_sha256=sha(protocol), script_sha256=sha(Path(__file__)),
                  python=platform.python_version(), numpy=np.__version__, scipy=scipy.__version__,
                  scope='float64 periodic isolated mechanisms; no native timing, whole-chain or sonic-adoption claim',
                  kernels={}, rows=[], continuity=[])
    report['filter_design'] = dict(taps_per_factor=args.taps_per_factor, cutoff=args.cutoff, beta=10.)
    (args.output/'saturation_isolated.py').write_bytes(Path(__file__).read_bytes())
    (args.output/'protocol.md').write_bytes(protocol.read_bytes())
    started = time.perf_counter()
    kernels = {}
    for factor in (2, 4, 8):
        kernel = firwin(args.taps_per_factor * factor + 1, args.cutoff / factor, window=('kaiser', 10.))
        path = args.output / f'fir-{factor}x.f64le'
        path.write_bytes(kernel.astype('<f8').tobytes())
        report['kernels'][factor] = dict(taps=len(kernel), sha256=sha(path), sum=float(kernel.sum()),
                                       causal_up_down_delay_source_frames=(len(kernel)-1)/factor)
        kernels[factor] = kernel
    report['kernel_prepare_s'] = time.perf_counter() - started
    amounts = (0., 1e-8, .001, .005, .02, .03575, .0715, .13, .25)
    knees = (None, .005, .02)
    if args.alias_only:
        amounts = (.03575, .0715, .13)
        knees = (None,)
    # Continuity over the whole transfer curve, not just a small-signal slope.
    sweep = np.linspace(-2., 2., 65537)
    for knee in (() if args.alias_only else knees):
        for amount in amounts:
            out = curve(sweep, amount, knee)
            slope = 1. if amount == 0 else (1 + 2 * amount) / np.tanh(1 + 2 * amount)
            if knee is not None:
                t = min(amount / knee, 1)
                slope = 1 + t*t*(3-2*t) * (slope-1)
            report['continuity'].append(dict(knee=knee, amount=amount, small_signal_gain_db=db(slope),
                maximum_vs_identity=float(np.max(np.abs(out-sweep))),
                maximum_vs_positive_control=float(np.max(np.abs(out-curve(sweep, amount))))))
    n = 32768
    for rate in (44100, 48000, 96000):
        for frequency in (997, 7001, 11003, 19001):
            tone_bin = round(frequency * n / rate)
            tone_bin += 1 - tone_bin % 2
            for amplitude in (10**(-24/20), 10**(-6/20), 1.4):
                signal = amplitude * np.sin(np.arange(n) * (2 * np.pi * tone_bin / n))
                for amount in amounts:
                    for knee in knees:
                        reference, reference_error = analytic_reference(amplitude, amount, knee, n, tone_bin)
                        start = time.perf_counter()
                        actual = curve(signal, amount, knee)
                        seconds = time.perf_counter() - start
                        report['rows'].append(dict(rate=rate, frequency=tone_bin*rate/n, tone_bin=tone_bin,
                            amplitude=amplitude, amount=amount, knee=knee, factor=1, processing_s=seconds,
                            reference_convergence=reference_error, **metrics(actual, reference, amplitude, tone_bin)))
                        if knee is None and amount in (.03575, .0715, .13):
                            for factor, kernel in kernels.items():
                                start = time.perf_counter()
                                high = resample_poly(signal, factor, 1, window=kernel, padtype='wrap')
                                shaped = curve(high, amount)
                                actual = resample_poly(shaped, 1, factor, window=kernel, padtype='wrap')
                                seconds = time.perf_counter() - start
                                assert len(actual) == n and np.isfinite(actual).all()
                                report['rows'].append(dict(rate=rate, frequency=tone_bin*rate/n, tone_bin=tone_bin,
                                    amplitude=amplitude, amount=amount, knee=None, factor=factor, processing_s=seconds,
                                    reference_convergence=reference_error, **metrics(actual, reference, amplitude, tone_bin)))
            report['elapsed_s'] = time.perf_counter() - started
            (args.output/'report.json').write_text(json.dumps(report, indent=2)+'\n')
            print(rate, frequency, 'complete', flush=True)
    report['status'] = 'complete'
    report['elapsed_s'] = time.perf_counter() - started
    (args.output/'report.json').write_text(json.dumps(report, indent=2)+'\n')


if __name__ == '__main__':
    main()
