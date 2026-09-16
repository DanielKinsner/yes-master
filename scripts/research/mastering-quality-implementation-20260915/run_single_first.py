"""Run the frozen copied probe and measure only that child process on Windows."""
import argparse
import ctypes
from ctypes import wintypes
import json
import os
from pathlib import Path
import shutil
import subprocess
import time
from peak_cost import MemoryCounters
from verification_common import sha


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--job', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    assert os.name == 'nt' and not args.output.exists()
    job = json.loads(args.job.read_text(encoding='utf-8'))
    binary = Path(job['binary'])
    assert sha(binary) == job['binary_sha256']
    assert sha(Path(job['specification'])) == job['specification_sha256']
    free = shutil.disk_usage(args.output.parent).free
    assert free >= job['minimum_free_reserve_bytes'] + job['estimated_new_wav_bytes']
    log = args.output.with_suffix('.process.log')
    report_path = args.output.with_suffix('.process.json')
    assert not log.exists() and not report_path.exists()
    memory = ctypes.WinDLL('psapi').GetProcessMemoryInfo
    memory.argtypes = [wintypes.HANDLE,ctypes.POINTER(MemoryCounters),wintypes.DWORD]
    memory.restype = wintypes.BOOL
    times = ctypes.WinDLL('kernel32').GetProcessTimes
    times.argtypes = [wintypes.HANDLE] + [ctypes.POINTER(ctypes.c_uint64)]*4
    times.restype = wintypes.BOOL
    started = time.perf_counter()
    peak, cpu = 0, 0.
    with log.open('x',encoding='utf-8') as stream:
        child = subprocess.Popen([str(binary),str(args.job.resolve()),str(args.output.resolve())],stdout=stream,stderr=subprocess.STDOUT)
        handle = wintypes.HANDLE(int(child._handle))
        while True:
            counters = MemoryCounters()
            counters.cb = ctypes.sizeof(counters)
            if memory(handle,ctypes.byref(counters),counters.cb):
                peak = max(peak,counters.PeakWorkingSetSize)
            values = [ctypes.c_uint64() for _ in range(4)]
            if times(handle,*[ctypes.byref(value) for value in values]):
                cpu = (values[2].value+values[3].value)/1e7
            code = child.poll()
            if code is not None:
                break
            time.sleep(.1)
    report = dict(status='complete',exit_code=code,wall_s=time.perf_counter()-started,
                  cpu_s=cpu,peak_working_set_bytes=peak,job_sha256=sha(args.job),
                  binary_sha256=sha(binary),free_bytes_at_launch=free,
                  scope='this offline child only, including file I/O/hash/decode/render; excludes separate metric/reference processes and OS/driver work')
    report_path.write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8')
    print(json.dumps(report,indent=2))
    assert code == 0, f'Native probe failed; retained log: {log}'


if __name__ == '__main__':
    main()
