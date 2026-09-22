"""Isolated, rotating before/after trials of copied native peak probes.

Run after all other task-owned builds/measurements finish. Process cost includes
decode, hash and the library comparison; candidate/FIR times are reported apart.
"""
import argparse
import ctypes
from ctypes import wintypes
import datetime
import json
import os
from pathlib import Path
import platform
import shutil
import statistics
import subprocess
import time
from qualify_fir import ref


class MemoryCounters(ctypes.Structure):
    # Same Windows layout as the frozen run_bench helper; no drive-runner import.
    _fields_ = [('cb', wintypes.DWORD), ('PageFaultCount', wintypes.DWORD)] + [
        (name, ctypes.c_size_t) for name in ['PeakWorkingSetSize', 'WorkingSetSize',
        'QuotaPeakPagedPoolUsage', 'QuotaPagedPoolUsage', 'QuotaPeakNonPagedPoolUsage',
        'QuotaNonPagedPoolUsage', 'PagefileUsage', 'PeakPagefileUsage']]


def main():
    p = argparse.ArgumentParser()
    for name in ['before', 'after', 'inputs', 'kernel', 'output']:
        p.add_argument('--' + name, type=Path, required=True)
    p.add_argument('--trials', type=int, default=3)
    a = p.parse_args()
    assert os.name == 'nt' and a.trials >= 3 and not a.output.exists()
    a.output.mkdir(parents=True)
    memory = ctypes.WinDLL('psapi').GetProcessMemoryInfo
    memory.argtypes = [wintypes.HANDLE, ctypes.POINTER(MemoryCounters), wintypes.DWORD]
    memory.restype = wintypes.BOOL
    times = ctypes.WinDLL('kernel32').GetProcessTimes
    times.argtypes = [wintypes.HANDLE] + [ctypes.POINTER(ctypes.c_uint64)] * 4
    times.restype = wintypes.BOOL
    binaries = {}
    for name in ['before', 'after']:
        source = getattr(a, name)
        copied = a.output / (name + '.exe')
        shutil.copy2(source, copied)
        binaries[name] = {'path': str(copied.resolve()), 'sha256': ref.sha(copied)}
    entries = json.loads(a.inputs.read_text())
    report = {'version': 'peak-cost-1', 'platform': platform.platform(), 'cpus': os.cpu_count(),
              'started_utc': datetime.datetime.now(datetime.timezone.utc).isoformat(),
              'binaries': binaries, 'inputs_sha256': ref.sha(a.inputs),
              'kernel_sha256': ref.sha(a.kernel), 'runs': []}
    for trial in range(a.trials):
        for name in (['before', 'after'] if trial % 2 == 0 else ['after', 'before']):
            output = a.output / f'{trial}-{name}.json'
            args = [binaries[name]['path'], str(a.inputs.resolve()), str(output.resolve()),
                    str(a.kernel.resolve())]
            start = time.perf_counter()
            peak_bytes = 0
            cpu = 0.
            with output.with_suffix('.log').open('w') as log:
                child = subprocess.Popen(args, stdout=log, stderr=subprocess.STDOUT)
                handle = wintypes.HANDLE(int(child._handle))
                while True:
                    counters = MemoryCounters(); counters.cb = ctypes.sizeof(counters)
                    if memory(handle, ctypes.byref(counters), counters.cb):
                        peak_bytes = max(peak_bytes, counters.PeakWorkingSetSize)
                    values = [ctypes.c_uint64() for _ in range(4)]
                    if times(handle, *[ctypes.byref(x) for x in values]):
                        cpu = (values[2].value + values[3].value)/1e7
                    status = child.poll()
                    if status is not None:
                        break
                    time.sleep(.2)
            assert status == 0, (args, status)
            results = json.loads(output.read_text())
            assert [r['id'] for r in results['rows']] == [e['id'] for e in entries]
            run = {'trial': trial, 'name': name, 'wall_s': time.perf_counter()-start,
                   'cpu_s': cpu, 'peak_working_set_bytes': peak_bytes, 'rows': results['rows']}
            report['runs'].append(run)
            (a.output/'report.json').write_text(json.dumps(report, indent=2)+'\n')
            print(name, trial, run['wall_s'], 's', peak_bytes, 'bytes', flush=True)
    summary = {}
    for entry in entries:
        summary[entry['id']] = {}
        for name in binaries:
            rows = [row for run in report['runs'] if run['name'] == name
                    for row in run['rows'] if row['id'] == entry['id']]
            summary[entry['id']][name] = {key: {'median': statistics.median(r[key] for r in rows),
                                               'min': min(r[key] for r in rows),
                                               'max': max(r[key] for r in rows)}
                                           for key in ['candidate_seconds', 'fir_seconds', 'fir_preparation_s']}
    report['summary'] = summary
    (a.output/'report.json').write_text(json.dumps(report, indent=2)+'\n')


if __name__ == '__main__':
    main()
