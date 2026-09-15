"""Run the actual preview-preparation test worker, with process cost recording.

Uses an isolated copy of the built test executable. The Rust probe checks source
hashes, rotates existing/reuse strategies, retains scalar-cache hits in both,
and requires identical returned gain/LUFS. Run after other builds finish.
"""
import argparse
import ctypes
from ctypes import wintypes
import hashlib
import json
import os
from pathlib import Path
import shutil
import statistics
import subprocess
import time
from run_bench import MemoryCounters


def main():
    p = argparse.ArgumentParser()
    for name in ['binary', 'inputs', 'output']:
        p.add_argument('--' + name, type=Path, required=True)
    a = p.parse_args()
    assert os.name == 'nt' and not a.output.exists()
    a.output.mkdir(parents=True)
    binary = a.output / 'preview-preparation-test.exe'
    shutil.copy2(a.binary, binary)
    native = a.output / 'native.json'
    env = {**os.environ, 'YES_MASTER_PREPARATION_INPUTS': str(a.inputs.resolve()),
           'YES_MASTER_PREPARATION_REPORT': str(native.resolve())}
    command = [str(binary.resolve()), 'mastering_quality_preparation_cost_bench',
               '--ignored', '--nocapture', '--test-threads=1']
    memory = ctypes.WinDLL('psapi').GetProcessMemoryInfo
    memory.argtypes = [wintypes.HANDLE, ctypes.POINTER(MemoryCounters), wintypes.DWORD]
    memory.restype = wintypes.BOOL
    times = ctypes.WinDLL('kernel32').GetProcessTimes
    times.argtypes = [wintypes.HANDLE] + [ctypes.POINTER(ctypes.c_uint64)] * 4
    times.restype = wintypes.BOOL
    report = {'binary_sha256': hashlib.file_digest(binary.open('rb'), 'sha256').hexdigest(),
              'inputs_sha256': hashlib.file_digest(a.inputs.open('rb'), 'sha256').hexdigest(),
              'scope': 'only launched child; memory/CPU sampled at 200 ms, CPU includes decode/hash',
              'samples': []}
    start = time.perf_counter()
    with (a.output / 'native.log').open('w', encoding='utf-8') as log:
        child = subprocess.Popen(command, env=env, stdout=log, stderr=subprocess.STDOUT)
        handle = wintypes.HANDLE(int(child._handle))
        while True:
            counters = MemoryCounters(); counters.cb = ctypes.sizeof(counters)
            stamps = [ctypes.c_uint64() for _ in range(4)]
            if memory(handle, ctypes.byref(counters), counters.cb) and times(handle, *[ctypes.byref(v) for v in stamps]):
                report['samples'].append({'unix_s': time.time(), 'working_set_bytes': counters.WorkingSetSize,
                    'peak_working_set_bytes': counters.PeakWorkingSetSize, 'cpu_s': (stamps[2].value + stamps[3].value) / 1e7})
            status = child.poll()
            if status is not None: break
            time.sleep(.2)
    report.update(exit_code=status, wall_s=time.perf_counter()-start)
    (a.output / 'process.json').write_text(json.dumps(report, indent=2), encoding='utf-8')
    if status: raise SystemExit(status)
    summarize(a.output, report)


def summarize(output, report):
    rows = json.loads((output / 'native.json').read_text(encoding='utf-8'))['rows']
    summary = []
    for source in sorted({r['source'] for r in rows}):
        for strategy in ['existing', 'reuse']:
            subset = [r for r in rows if r['source'] == source and r['strategy'] == strategy]
            events = {}
            for event in subset[0]['events']:
                values = [next(e['seconds'] for e in r['events'] if e['event'] == event['event']) for r in subset]
                events[event['event']] = {'median_s': statistics.median(values), 'worst_s': max(values)}
            cpu_costs = []
            for row in subset:
                samples = [s for s in report['samples'] if row['started_unix_s'] <= s['unix_s'] <= row['ended_unix_s']]
                if len(samples) >= 2:
                    cpu_costs.append(samples[-1]['cpu_s'] - samples[0]['cpu_s'])
            memory_samples = [s['working_set_bytes'] for s in report['samples'] if any(r['started_unix_s'] <= s['unix_s'] <= r['ended_unix_s'] for r in subset)]
            summary.append({'source': source, 'strategy': strategy, 'trials': len(subset), 'events': events,
                'median_session_s': statistics.median(r['session_s'] for r in subset),
                'median_sampled_cpu_s': statistics.median(cpu_costs) if cpu_costs else None,
                'cpu_scope': 'interior samples omit up to 200 ms at each boundary; process CPU across all cores',
                'maximum_sampled_working_set': max(memory_samples, default=None),
                'maximum_cache_bytes': max(e['cache_bytes'] for r in subset for e in r['events'])})
    (output / 'summary.json').write_text(json.dumps(summary, indent=2), encoding='utf-8')
    print(json.dumps(summary, indent=2), flush=True)


if __name__ == '__main__': main()
