param(
    [Parameter(Mandatory = $true)][string]$Exe,
    [Parameter(Mandatory = $true)][string]$Test,
    [Parameter(Mandatory = $true)][string]$Prefix
)
$ErrorActionPreference = 'Stop'
$executable = (Get-Item -LiteralPath $Exe).FullName
$prefixPath = [System.IO.Path]::GetFullPath($Prefix)
$stdout = "$prefixPath.stdout.log"
$stderr = "$prefixPath.stderr.log"
$report = "$prefixPath.resources.json"
foreach ($path in @($stdout, $stderr, $report)) {
    if (Test-Path -LiteralPath $path) { throw "Preserve existing evidence: $path" }
}
$watch = [System.Diagnostics.Stopwatch]::StartNew()
$process = Start-Process -FilePath $executable -ArgumentList @('--exact', $Test, '--ignored', '--nocapture') -PassThru -WindowStyle Hidden -RedirectStandardOutput $stdout -RedirectStandardError $stderr
$samples = [System.Collections.Generic.List[object]]::new()
while (-not $process.HasExited) {
    $process.Refresh()
    if (-not $process.HasExited) {
        $samples.Add([pscustomobject][ordered]@{
            elapsed_s = $watch.Elapsed.TotalSeconds
            working_set_bytes = $process.WorkingSet64
            peak_working_set_bytes = $process.PeakWorkingSet64
            private_bytes = $process.PrivateMemorySize64
            cpu_s = $process.TotalProcessorTime.TotalSeconds
        })
    }
    Start-Sleep -Milliseconds 100
}
$process.WaitForExit()
$watch.Stop()
[ordered]@{
    status = 'complete'
    scope = 'Windows process resource observations at 100 ms; includes test setup, preparation and playback, excludes driver/OS and unrelated processes; no isolated-system comparison'
    executable_sha256 = (Get-FileHash -LiteralPath $executable -Algorithm SHA256).Hash
    test = $Test
    exit_code = $process.ExitCode
    total_wall_s = $watch.Elapsed.TotalSeconds
    peak_working_set_bytes = ($samples | Measure-Object -Property peak_working_set_bytes -Maximum).Maximum
    maximum_observed_private_bytes = ($samples | Measure-Object -Property private_bytes -Maximum).Maximum
    last_observed_cpu_s = ($samples | Select-Object -Last 1).cpu_s
    samples = $samples
} | ConvertTo-Json -Depth 5 | Set-Content -LiteralPath $report -Encoding utf8
exit $process.ExitCode
