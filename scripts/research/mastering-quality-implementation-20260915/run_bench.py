"""Run a copied native probe and retain Windows process cost with its results.

No third-party process-monitor dependency. Only inspect this launched child.
Run controlled timing trials after competing build/measurement work finishes.
"""
import argparse
import ctypes
from ctypes import wintypes
import datetime
import hashlib
import json
import os
from pathlib import Path
import platform
import shutil
import subprocess
import time


class MemoryCounters(ctypes.Structure):
    _fields_=[('cb',wintypes.DWORD),('PageFaultCount',wintypes.DWORD)]+[
        (name,ctypes.c_size_t) for name in ['PeakWorkingSetSize','WorkingSetSize',
        'QuotaPeakPagedPoolUsage','QuotaPagedPoolUsage','QuotaPeakNonPagedPoolUsage',
        'QuotaNonPagedPoolUsage','PagefileUsage','PeakPagefileUsage']]


def main():
    p=argparse.ArgumentParser()
    p.add_argument('--source',type=Path,required=True)
    p.add_argument('--output',type=Path,required=True)
    p.add_argument('--binary',type=Path,required=True)
    p.add_argument('--trials',type=int,default=3)
    p.add_argument('--candidates-only',action='store_true')
    p.add_argument('--timing-only',action='store_true')
    a=p.parse_args()
    assert os.name=='nt' and not a.output.exists()
    assert not (a.candidates_only and a.timing_only)
    sha=lambda path:hashlib.file_digest(path.open('rb'),'sha256').hexdigest()
    binary_hash=sha(a.binary)
    copied=a.output.parent/(a.binary.stem+'-'+binary_hash[:12]+'.exe')
    a.output.parent.mkdir(parents=True,exist_ok=True)
    if copied.exists(): assert sha(copied)==binary_hash
    else: shutil.copy2(a.binary,copied)
    meta=a.output.with_suffix('.process.json');log=a.output.with_suffix('.log')
    assert not meta.exists() and not log.exists()
    args=[str(copied.resolve()),str(a.source.resolve()),str(a.output.resolve()),'--trials',str(a.trials)]
    if a.candidates_only: args+=['--candidates-only']
    if a.timing_only: args+=['--timing-only']
    memory=ctypes.WinDLL('psapi').GetProcessMemoryInfo
    memory.argtypes=[wintypes.HANDLE,ctypes.POINTER(MemoryCounters),wintypes.DWORD]
    memory.restype=wintypes.BOOL
    times=ctypes.WinDLL('kernel32').GetProcessTimes
    times.argtypes=[wintypes.HANDLE]+[ctypes.POINTER(ctypes.c_uint64)]*4
    times.restype=wintypes.BOOL
    report={'binary_sha256':binary_hash,'source_sha256':sha(a.source),
            'started_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),
            'platform':platform.platform(),'logical_cpus':os.cpu_count(),
            'monitor_scope':'this child only; peak working set from Windows, CPU from process times',
            'samples':[]}
    start=time.perf_counter()
    with log.open('w') as logfile:
        child=subprocess.Popen(args,stdout=logfile,stderr=subprocess.STDOUT)
        handle=wintypes.HANDLE(int(child._handle))
        while True:
            counters=MemoryCounters();counters.cb=ctypes.sizeof(counters)
            ct,et,kt,ut=[ctypes.c_uint64() for _ in range(4)]
            if memory(handle,ctypes.byref(counters),counters.cb) and times(handle,*[ctypes.byref(x) for x in [ct,et,kt,ut]]):
                report['samples'].append({'wall_s':time.perf_counter()-start,
                    'working_set_bytes':counters.WorkingSetSize,
                    'peak_working_set_bytes':counters.PeakWorkingSetSize,
                    'cpu_s':(kt.value+ut.value)/1e7})
            status=child.poll()
            if status is not None: break
            time.sleep(.2)
        report['exit_code']=status;report['wall_s']=time.perf_counter()-start
    meta.write_text(json.dumps(report,indent=2)+'\n')
    print(meta,'exit',status,'wall',report['wall_s'],
          'peak bytes',max(s['peak_working_set_bytes'] for s in report['samples']),flush=True)
    if status: raise SystemExit(status)


if __name__=='__main__':main()
