"""Separately versioned classifier; historical quiet_reference stays frozen."""
import math
import re

VERSION = 'complete-block-availability-v1'
TRACE = re.compile(r'\bt:\s*([-\d.]+)\s+TARGET:-23 LUFS\s+M:\s*([-\d.]+)\s+S:')
SUMMARY = re.compile(r'Integrated loudness:\s+I:\s*([-\d.]+) LUFS\s+Threshold:\s*([-\d.]+) LUFS')


def classify(log, frames, rate=48000, channels=2):
    result = dict(version=VERSION, state='unknown', lufs=None, reason='unsupported_input')
    if rate != 48000 or channels != 2 or not 19200 <= frames <= 28800000:
        return result
    traces = TRACE.findall(log)
    expected = frames // 4800
    result.update(expected_readouts=expected, readouts=len(traces), reason='incomplete_trace')
    if len(traces) != expected or len(re.findall(r'\bt:\s*', log)) != expected:
        return result
    times, moments = zip(*[(float(t), float(m)) for t, m in traces])
    if not all(math.isfinite(t) and math.isfinite(m) for t, m in zip(times, moments)):
        result['reason'] = 'nonfinite_trace'
        return result
    if any(abs(t-((i+1)*4800-1)/48000) > .0006 for i, t in enumerate(times)):
        result['reason'] = 'nonconsecutive_trace'
        return result
    result.update(maximum_displayed_momentary_lufs=max(moments), reason='missing_or_duplicate_summary')
    if log.count('Summary:') != 1:
        return result
    summary = SUMMARY.fullmatch(log.rsplit('Summary:', 1)[1].split('Loudness range:', 1)[0].strip())
    if summary is None:
        return result
    value, threshold = map(float, summary.groups())
    result.update(integrated_display=value, integrated_threshold=threshold, reason='ambiguous_gate_boundary')
    if not math.isfinite(value) or not math.isfinite(threshold):
        result['reason'] = 'nonfinite_summary'
    elif value == -70. and threshold == 0. and max(moments) <= -70.1:
        result.update(state='unavailable', reason='complete_trace_below_absolute_gate')
    elif threshold != 0. and value >= -70. and max(moments) >= -70.:
        result.update(state='finite', lufs=value, reason='integrated_blocks_present')
    return result


def agreement(native_state, native_lufs, independent):
    if native_state == 'unavailable' and native_lufs is None:
        return independent['state'] == 'unavailable'
    if native_state != 'finite' or native_lufs is None or not math.isfinite(native_lufs):
        return False
    return independent['state'] == 'finite' and abs(native_lufs-independent['lufs']) <= .11
