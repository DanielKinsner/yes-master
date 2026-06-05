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

try {
    Set-Location $repoRoot

    Invoke-Step "frontend tests" {
        npm test
    }

    Invoke-Step "frontend build" {
        npm run build
    }

    Invoke-Step "windows package build" {
        npm run build:windows
    }

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

    Invoke-Step "iphone rust bridge check" {
        Set-Location (Join-Path $repoRoot "apps\iphone-native\rust")
        cargo check --all-targets
    }

    Invoke-Step "iphone rust bridge tests" {
        Set-Location (Join-Path $repoRoot "apps\iphone-native\rust")
        cargo test
    }

    Write-Host ""
    Write-Host "Fast verification lane passed." -ForegroundColor Green
} finally {
    Set-Location $originalLocation
}
