"""C1 source-anchored measurements and explicit development constraints.

Methods match the fixed C1 protocol. No composite quality score or production
policy. Source anchors are prepared once, never reselected on quiet gain copies.
"""
import math
import numpy as np
import soundfile as sf
from scipy.signal import find_peaks, resample_poly, welch

BANDS = [(20, 60), (60, 120), (120, 250), (250, 500), (500, 1000),
         (1000, 2000), (2000, 4000), (4000, 8000), (8000, 16000)]
LIMITS = dict(section_loss_db=.75, attack_median_loss_db=2., attack_p10_loss_db=3.5,
              tone_control_slack_db=.25, tone_control_floor_db=.5,
              side_mid_control_slack_db=.25, correlation_control_slack=.02,
              target_error_lu=.2)


def db(power):
    return float(10 * np.log10(max(float(power), 1e-30)))


def measure(path, anchors=None):
    x, rate = sf.read(path, dtype='float64', always_2d=True)
    assert len(x) > 0 and np.isfinite(x).all()
    facts = dict(rate=rate, frames=len(x), channels=x.shape[1],
                 sample_peak_dbfs=db(np.max(np.abs(x)) ** 2),
                 channel_rms_db=[db(v) for v in np.mean(x * x, axis=0)])
    if rate != 48000:
        divisor = math.gcd(rate, 48000)
        x = resample_poly(x, 48000 // divisor, rate // divisor, axis=0)
    rate = 48000
    channels = x.shape[1]
    rms = db(np.mean(x * x))
    section_frames = rate * 10
    sections = x[:len(x) // section_frames * section_frames].reshape(-1, section_frames, channels)
    section_levels = [db(v) for v in np.mean(sections * sections, axis=(1, 2))]
    if anchors is None:
        eligible = [i for i, value in enumerate(section_levels) if value > -40]
        pair = [min(eligible, key=lambda i: section_levels[i]),
                max(eligible, key=lambda i: section_levels[i])] if len(eligible) >= 2 else None
        blocks = x[:len(x) // 480 * 480].reshape(-1, 480, channels)
        envelope = np.sqrt(np.mean(blocks * blocks, axis=(1, 2)))
        novelty = np.maximum(np.diff(envelope, prepend=envelope[0] if len(envelope) else 0), 0)
        peaks, _ = find_peaks(novelty, distance=20)
        peaks = sorted(peaks, key=lambda k: novelty[k], reverse=True)[:40]
        times = sorted(float(k * .01) for k in peaks if .05 < k * .01 < len(x) / rate - .2)
        anchors = dict(section_indices=pair, transient_times=times,
                       active_section_indices=eligible, method='c1-fixed-source-v1')
    ratios = []
    for t in anchors['transient_times']:
        attack = x[int((t - .02) * rate):int((t + .03) * rate)]
        body = x[int((t + .03) * rate):int((t + .15) * rate)]
        ratios.append(db(np.max(np.abs(attack)) ** 2 / max(np.mean(body * body), 1e-30)))
    frequency, power = welch(x, fs=rate, nperseg=8192, noverlap=4096, axis=0)
    power = power.mean(axis=1)
    df = frequency[1] - frequency[0]
    bands = [db(power[(frequency >= low) & (frequency < high)].sum() * df)
             for low, high in BANDS]
    stereo = None
    if channels == 2:
        mid = x.mean(axis=1)
        side = (x[:, 0] - x[:, 1]) / 2
        correlation = float(np.corrcoef(x.T)[0, 1])
        stereo = dict(side_mid_db=db(np.mean(side * side) / max(np.mean(mid * mid), 1e-30)),
                      correlation=correlation if np.isfinite(correlation) else None)
    return dict(**facts, spectral_rms_db=rms, sections10_rms_db=section_levels,
                transient_attack_body_db=ratios, bands_db=bands, stereo=stereo, anchors=anchors)


def compare(source, candidate):
    gain = candidate['spectral_rms_db'] - source['spectral_rms_db']
    sections = candidate['sections10_rms_db']
    pair = source['anchors']['section_indices']
    section_delta = None if pair is None else ((sections[pair[1]] - sections[pair[0]]) -
                    (source['sections10_rms_db'][pair[1]] - source['sections10_rms_db'][pair[0]]))
    attacks = np.asarray(candidate['transient_attack_body_db']) - source['transient_attack_body_db']
    result = dict(section_contrast_delta_db=section_delta,
                  section_delta_db=[a - b - gain for a, b in zip(sections, source['sections10_rms_db'], strict=True)],
                  paired_attack_delta_db=attacks.tolist(),
                  attack_median_delta_db=float(np.median(attacks)) if len(attacks) else None,
                  attack_p10_delta_db=float(np.percentile(attacks, 10)) if len(attacks) else None,
                  band_delta_db=[a - b - gain for a, b in zip(candidate['bands_db'], source['bands_db'], strict=True)])
    result['stereo_delta'] = None
    if source['stereo'] is not None and candidate['stereo'] is not None:
        result['stereo_delta'] = {
            key: candidate['stereo'][key] - source['stereo'][key]
            if source['stereo'][key] is not None and candidate['stereo'][key] is not None else None
            for key in ('side_mid_db', 'correlation')}
    return result


def constraints(source, delta, control, limits=LIMITS):
    failures = []
    for metric, limit in [('section_contrast_delta_db', 'section_loss_db'),
                          ('attack_median_delta_db', 'attack_median_loss_db'),
                          ('attack_p10_delta_db', 'attack_p10_loss_db')]:
        value = delta[metric]
        if value is None or not np.isfinite(value):
            failures.append(metric + ':unavailable')
        elif value < -limits[limit]:
            failures.append(metric)
    for index, (value, baseline) in enumerate(zip(delta['band_delta_db'], control['band_delta_db'], strict=True)):
        bound = max(abs(baseline), limits['tone_control_floor_db']) + limits['tone_control_slack_db']
        if not np.isfinite(value) or abs(value) > bound:
            failures.append(f'tone_band_{index}')
    if source['channels'] == 2:
        for key, slack in [('side_mid_db', limits['side_mid_control_slack_db']),
                           ('correlation', limits['correlation_control_slack'])]:
            value = delta['stereo_delta'][key] if delta['stereo_delta'] is not None else None
            baseline = control['stereo_delta'][key] if control['stereo_delta'] is not None else None
            if value is None or baseline is None or not np.isfinite(value):
                failures.append(key + ':unavailable')
            elif abs(value) > abs(baseline) + slack:
                failures.append(key)
    return failures
