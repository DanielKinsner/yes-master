param(
    [ValidateSet("all", "frontend", "rust", "iphone")]
    [string]$Lane = "all"
)

$ErrorActionPreference = "Stop"

$repoRoot = Resolve-Path (Join-Path $PSScriptRoot "..")
$originalLocation = Get-Location

function Invoke-Step {
    param(
        [Parameter(Mandatory = $true)]
        [string]$Label,

        [Parameter(Mandatory = $true)]
        [scriptblock]$Command
    )

    Write-Host ""
    Write-Host "==> $Label" -ForegroundColor Cyan
    & $Command
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

try {
    Set-Location $repoRoot

    switch ($Lane) {
        "frontend" { Invoke-FrontendLane }
        "rust" { Invoke-RustLane }
        "iphone" { Invoke-IphoneLane }
        default {
            Invoke-FrontendLane
            Invoke-RustLane
            Invoke-IphoneLane
        }
    }

    Write-Host ""
    $laneLabel = if ($Lane -eq "all") { "Fast verification lane" } else { "$Lane lane" }
    Write-Host "$laneLabel passed." -ForegroundColor Green
} finally {
    Set-Location $originalLocation
}
