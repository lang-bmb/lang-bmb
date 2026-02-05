# BMB Installation Script for Windows
# Installs BMB compiler and runtime
#
# Usage:
#   .\install.ps1 [-Prefix PATH] [-User] [-Uninstall]
#
# Options:
#   -Prefix PATH    Installation prefix (default: C:\Program Files\BMB)
#   -User           Install to user directory instead
#   -Uninstall      Remove BMB installation

param(
    [string]$Prefix = "",
    [switch]$User,
    [switch]$Uninstall
)

$ErrorActionPreference = "Stop"

# Set default prefix
if ($Prefix -eq "") {
    if ($User) {
        $Prefix = "$env:LOCALAPPDATA\BMB"
    } else {
        $Prefix = "$env:ProgramFiles\BMB"
    }
}

# Detect script directory
$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$ProjectRoot = Split-Path -Parent $ScriptDir
if (-not (Test-Path "$ProjectRoot\golden")) {
    $ProjectRoot = $ScriptDir
}

function Write-Status($msg) {
    Write-Host $msg -ForegroundColor Yellow
}

function Write-Success($msg) {
    Write-Host $msg -ForegroundColor Green
}

function Write-Error($msg) {
    Write-Host "Error: $msg" -ForegroundColor Red
    exit 1
}

# Find BMB binary
function Find-BmbBinary {
    $paths = @(
        "$ProjectRoot\golden\windows-x64\bmb.exe",
        "$ProjectRoot\bmb.exe",
        "$ProjectRoot\target\golden-bootstrap\bmb-stage1.exe"
    )

    foreach ($p in $paths) {
        if (Test-Path $p) {
            return $p
        }
    }

    Write-Error "BMB binary not found. Run .\scripts\golden-bootstrap.sh first."
}

# Uninstall
function Do-Uninstall {
    Write-Status "Uninstalling BMB from $Prefix..."

    if (Test-Path "$Prefix\bin\bmb.exe") {
        Remove-Item "$Prefix\bin\bmb.exe" -Force
    }
    if (Test-Path "$Prefix\lib") {
        Remove-Item "$Prefix\lib" -Recurse -Force
    }
    if (Test-Path "$Prefix\share") {
        Remove-Item "$Prefix\share" -Recurse -Force
    }

    # Remove from PATH
    $currentPath = [Environment]::GetEnvironmentVariable("PATH", "User")
    if ($currentPath -like "*$Prefix\bin*") {
        $newPath = ($currentPath -split ';' | Where-Object { $_ -ne "$Prefix\bin" }) -join ';'
        [Environment]::SetEnvironmentVariable("PATH", $newPath, "User")
        Write-Host "Removed $Prefix\bin from PATH"
    }

    Write-Success "BMB uninstalled successfully"
}

# Install
function Do-Install {
    $BmbBinary = Find-BmbBinary

    Write-Status "Installing BMB to $Prefix..."
    Write-Host "Binary: $BmbBinary"

    # Create directories
    New-Item -ItemType Directory -Force -Path "$Prefix\bin" | Out-Null
    New-Item -ItemType Directory -Force -Path "$Prefix\lib\bmb\runtime" | Out-Null
    New-Item -ItemType Directory -Force -Path "$Prefix\share\bmb\bootstrap" | Out-Null
    New-Item -ItemType Directory -Force -Path "$Prefix\share\bmb\stdlib" | Out-Null

    # Install binary
    Copy-Item $BmbBinary "$Prefix\bin\bmb.exe" -Force

    # Install runtime
    $runtimePath = "$ProjectRoot\bmb\runtime"
    if (-not (Test-Path $runtimePath)) {
        $runtimePath = "$ProjectRoot\runtime"
    }
    if (Test-Path $runtimePath) {
        Copy-Item "$runtimePath\*" "$Prefix\lib\bmb\runtime\" -Recurse -Force
    }

    # Install bootstrap sources
    if (Test-Path "$ProjectRoot\bootstrap") {
        Copy-Item "$ProjectRoot\bootstrap\*.bmb" "$Prefix\share\bmb\bootstrap\" -Force -ErrorAction SilentlyContinue
    }

    # Install stdlib
    if (Test-Path "$ProjectRoot\stdlib") {
        Copy-Item "$ProjectRoot\stdlib\*" "$Prefix\share\bmb\stdlib\" -Recurse -Force -ErrorAction SilentlyContinue
    }

    # Add to PATH
    $currentPath = [Environment]::GetEnvironmentVariable("PATH", "User")
    if ($currentPath -notlike "*$Prefix\bin*") {
        [Environment]::SetEnvironmentVariable("PATH", "$Prefix\bin;$currentPath", "User")
        Write-Host "Added $Prefix\bin to PATH (restart terminal to take effect)"
    }

    Write-Host ""
    Write-Success "BMB installed successfully!"
    Write-Host ""
    Write-Host "Installation summary:"
    Write-Host "  Binary:    $Prefix\bin\bmb.exe"
    Write-Host "  Runtime:   $Prefix\lib\bmb\runtime\"
    Write-Host "  Bootstrap: $Prefix\share\bmb\bootstrap\"
    Write-Host "  Stdlib:    $Prefix\share\bmb\stdlib\"
    Write-Host ""
    Write-Host "Set runtime path for compilation:"
    Write-Host "  `$env:BMB_RUNTIME_PATH = `"$Prefix\lib\bmb\runtime`""
    Write-Host ""
    Write-Host "Restart your terminal, then run:"
    Write-Host "  bmb --help"
}

# Main
if ($Uninstall) {
    Do-Uninstall
} else {
    Do-Install
}
