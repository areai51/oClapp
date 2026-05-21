#!/usr/bin/env pwsh
Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

Push-Location $PSScriptRoot\..

try {
    Write-Host "=== Building oClapp for Windows ===" -ForegroundColor Cyan

    # Install frontend dependencies
    Set-Location frontend
    npm install
    Set-Location ..

    # Build
    cargo tauri build

    Write-Host "=== Windows build complete ===" -ForegroundColor Green
    Write-Host "Artifacts:"
    Get-ChildItem target\release\bundle\msi\*.msi -ErrorAction SilentlyContinue | ForEach-Object { Write-Host $_.FullName }
    Get-ChildItem target\release\bundle\nsis\*.exe -ErrorAction SilentlyContinue | ForEach-Object { Write-Host $_.FullName }
} finally {
    Pop-Location
}
