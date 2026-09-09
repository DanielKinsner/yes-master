param(
    [string]$MsysRoot = 'C:\msys64',
    [string]$OutputDirectory = 'test-output/audio-encoders/x86_64-pc-windows-msvc'
)
$ErrorActionPreference = 'Stop'
$repo = Split-Path $PSScriptRoot -Parent
$bash = Join-Path $MsysRoot 'usr/bin/bash.exe'
if (!(Test-Path -LiteralPath $bash) -or !(Test-Path -LiteralPath (Join-Path $MsysRoot 'ucrt64/bin/gcc.exe'))) {
    throw 'Encoder build needs MSYS2 UCRT64 gcc, make and pkgconf. See docs/third-party/audio-encoders.md.'
}
Push-Location $repo
try {
    # Script/arguments are separate native arguments, never assembled shell code.
    & $bash scripts/build-audio-encoders.sh x86_64-pc-windows-msvc $OutputDirectory
    if ($LASTEXITCODE -ne 0) { throw "Encoder build failed ($LASTEXITCODE); see $OutputDirectory/build logs." }
} finally { Pop-Location }
