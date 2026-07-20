param(
    [ValidateSet("all", "frontend", "rust", "iphone", "android")]
    [string]$Lane = "all",

    [switch]$AllowAndroidSkip
)

$ErrorActionPreference = "Stop"

$repoRoot = Resolve-Path (Join-Path $PSScriptRoot "..")
$originalLocation = Get-Location
$skippedLanes = [System.Collections.Generic.List[string]]::new()

function Invoke-Step {
    param(
        [Parameter(Mandatory = $true)]
        [string]$Label,

        [Parameter(Mandatory = $true)]
        [scriptblock]$Command
    )

    Write-Host ""
    Write-Host "==> $Label" -ForegroundColor Cyan
    $global:LASTEXITCODE = 0
    & $Command
    if ($LASTEXITCODE -ne 0) {
        throw "$Label failed with exit code $LASTEXITCODE."
    }
}

function Invoke-FrontendLane {
    Invoke-Step "frontend tests" {
        Set-Location $repoRoot
        npm test
    }

    Invoke-Step "frontend build" {
        Set-Location $repoRoot
        npm run build
    }

    Invoke-Step "windows package build" {
        Set-Location $repoRoot
        npm run build:windows
    }
}

function Invoke-RustLane {
    Invoke-Step "rust format check" {
        Set-Location (Join-Path $repoRoot "src-tauri")
        cargo fmt --check
    }

    Invoke-Step "rust clippy" {
        Set-Location (Join-Path $repoRoot "src-tauri")
        cargo clippy --target-dir target\codex-rc --all-targets -- -D warnings
    }

    Invoke-Step "rust lib tests" {
        Set-Location (Join-Path $repoRoot "src-tauri")
        cargo test --lib --target-dir target\codex-rc
    }

    Invoke-Step "rust integration tests" {
        Set-Location (Join-Path $repoRoot "src-tauri")
        cargo test --target-dir target\codex-rc
    }
}

function Invoke-IphoneLane {
    Invoke-Step "iphone rust bridge check" {
        Set-Location (Join-Path $repoRoot "apps\iphone-native\rust")
        cargo check --all-targets
    }

    Invoke-Step "iphone rust bridge tests" {
        Set-Location (Join-Path $repoRoot "apps\iphone-native\rust")
        cargo test
    }
}

function Get-AndroidLaneMissingPrerequisites {
    $missing = @()
    if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
        $missing += "cargo"
    }
    if (-not (Get-Command cargo-ndk -ErrorAction SilentlyContinue)) {
        $missing += "cargo-ndk"
    }
    if (-not ($env:JAVA_HOME -or (Get-Command java -ErrorAction SilentlyContinue))) {
        $missing += "JDK/JAVA_HOME"
    }
    $localProperties = Join-Path $repoRoot "apps\android-native\local.properties"
    if (-not ($env:ANDROID_HOME -or (Test-Path $localProperties))) {
        $missing += "ANDROID_HOME or apps\android-native\local.properties"
    }
    $targets = if (Get-Command rustup -ErrorAction SilentlyContinue) {
        rustup target list --installed
    } else {
        @()
    }
    if ($targets -notcontains "aarch64-linux-android") {
        $missing += "Rust target aarch64-linux-android"
    }

    return $missing
}

function Invoke-AndroidLane {
    $missing = Get-AndroidLaneMissingPrerequisites

    if ($missing.Count -gt 0) {
        $message = "Android lane unavailable; missing: $($missing -join ', ')."
        if ($AllowAndroidSkip) {
            $skippedLanes.Add("android")
            Write-Host ""
            Write-Host "Skipping Android lane by explicit -AllowAndroidSkip; missing: $($missing -join ', ')." -ForegroundColor Yellow
            return
        }

        Write-Host ""
        throw "$message Pass -AllowAndroidSkip to record an intentional non-green skip."
    }

    Invoke-Step "android rust bridge tests" {
        Set-Location (Join-Path $repoRoot "apps\android-native\rust")
        cargo test
    }

    Invoke-Step "android rust bridge arm64 check" {
        Set-Location (Join-Path $repoRoot "apps\android-native\rust")
        cargo ndk -t arm64-v8a --platform 29 check
    }
}

try {
    Set-Location $repoRoot

    switch ($Lane) {
        "frontend" { Invoke-FrontendLane }
        "rust" { Invoke-RustLane }
        "iphone" { Invoke-IphoneLane }
        "android" { Invoke-AndroidLane }
        default {
            Invoke-FrontendLane
            Invoke-RustLane
            Invoke-IphoneLane
            Invoke-AndroidLane
        }
    }

    Write-Host ""
    $laneLabel = if ($Lane -eq "all") { "All verification lanes" } else { "$Lane lane" }
    if ($skippedLanes.Count -gt 0) {
        Write-Host "$laneLabel completed with skipped lane(s): $($skippedLanes -join ', ')." -ForegroundColor Yellow
    } else {
        Write-Host "$laneLabel passed." -ForegroundColor Green
    }
} finally {
    Set-Location $originalLocation
}
