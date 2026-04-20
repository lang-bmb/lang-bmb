# BMB Development Environment Doctor
# Checks, diagnoses, and (where possible) auto-installs required tools.
#
# Usage:
#   .\scripts\doctor.ps1              # Check only
#   .\scripts\doctor.ps1 -Fix         # Check + auto-install missing tools
#   .\scripts\doctor.ps1 -Verbose     # Show detailed output
#
# Exit codes:
#   0 = All checks passed
#   1 = Critical issues found (cannot build)
#   2 = Warnings only (can build, some features limited)

param(
    [switch]$Fix,
    [switch]$Verbose
)

$ErrorActionPreference = "Continue"

$ProjectRoot = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)

# ─────────────────────────────────────────────────────────────────────────────
# Helpers
# ─────────────────────────────────────────────────────────────────────────────
$script:Errors      = [System.Collections.ArrayList]@()
$script:Warnings    = [System.Collections.ArrayList]@()
$script:Fixed       = [System.Collections.ArrayList]@()
$script:NeedRestart = [System.Collections.ArrayList]@()
$script:Guides      = [System.Collections.ArrayList]@()

function Add-Error($msg)       { [void]$script:Errors.Add($msg) }
function Add-Warning($msg)     { [void]$script:Warnings.Add($msg) }
function Add-Fixed($msg)       { [void]$script:Fixed.Add($msg) }
function Add-NeedRestart($msg) { [void]$script:NeedRestart.Add($msg) }

function Remove-Error($pattern) {
    $toRemove = @($script:Errors | Where-Object { $_ -like $pattern })
    foreach ($item in $toRemove) { [void]$script:Errors.Remove($item) }
}
function Remove-Warning($pattern) {
    $toRemove = @($script:Warnings | Where-Object { $_ -like $pattern })
    foreach ($item in $toRemove) { [void]$script:Warnings.Remove($item) }
}

function Write-Check($msg) {
    Write-Host "  Checking " -NoNewline -ForegroundColor DarkGray
    Write-Host "$msg... " -NoNewline
}

function Write-OK($msg) {
    Write-Host "OK" -ForegroundColor Green -NoNewline
    if ($msg) { Write-Host " ($msg)" -ForegroundColor DarkGray } else { Write-Host "" }
}

function Write-Fail($msg) {
    Write-Host "MISSING" -ForegroundColor Red -NoNewline
    if ($msg) { Write-Host " ($msg)" -ForegroundColor DarkGray } else { Write-Host "" }
}

function Write-Warn($msg) {
    Write-Host "WARNING" -ForegroundColor Yellow -NoNewline
    if ($msg) { Write-Host " ($msg)" -ForegroundColor DarkGray } else { Write-Host "" }
}

function Write-FixAttempt($msg) {
    Write-Host "  -> " -NoNewline -ForegroundColor Cyan
    Write-Host "$msg" -ForegroundColor Cyan
}

function Write-Guide {
    param([string]$Tool, [string[]]$Lines)
    [void]$script:Guides.Add(@{ Tool = $Tool; Lines = $Lines })
}

function Test-Command($cmd) {
    $null = Get-Command $cmd -ErrorAction SilentlyContinue
    return $?
}

function Test-Msys2 {
    return Test-Path "C:\msys64\usr\bin\pacman.exe"
}

function Invoke-Pacman {
    param([string[]]$Packages)
    $pacman = "C:\msys64\usr\bin\pacman.exe"
    if (-not (Test-Path $pacman)) { return $false }
    & $pacman -S --noconfirm --needed @Packages 2>&1 | Out-Null
    return $LASTEXITCODE -eq 0
}

function Test-Ucrt64InPath {
    return $env:PATH -like "*msys64*ucrt64*"
}

# Refresh PATH from registry (pick up changes made during this session)
function Update-SessionPath {
    $machinePath = [Environment]::GetEnvironmentVariable("PATH", "Machine")
    $userPath    = [Environment]::GetEnvironmentVariable("PATH", "User")
    $env:PATH    = "$userPath;$machinePath"
}

# ─────────────────────────────────────────────────────────────────────────────
# Banner
# ─────────────────────────────────────────────────────────────────────────────
Write-Host ""
Write-Host "  BMB Development Environment Doctor" -ForegroundColor Cyan
Write-Host "  ===================================" -ForegroundColor DarkCyan
Write-Host ""

if ($Fix) {
    Write-Host "  Mode: Check + Auto-Fix" -ForegroundColor Yellow
} else {
    Write-Host "  Mode: Check Only (use -Fix to auto-install)" -ForegroundColor DarkGray
}
Write-Host ""

# ─────────────────────────────────────────────────────────────────────────────
# 1. Git
# ─────────────────────────────────────────────────────────────────────────────
Write-Host "[1/8] Git" -ForegroundColor White

Write-Check "git"
if (Test-Command "git") {
    $v = (git --version 2>&1) -replace 'git version ', ''
    Write-OK $v
} else {
    Write-Fail
    Add-Error "Git is not installed"

    if ($Fix) {
        Write-FixAttempt "Installing Git via winget..."
        winget install --id Git.Git -e --accept-source-agreements --accept-package-agreements 2>&1 | Out-Null
        if ($LASTEXITCODE -eq 0) {
            Remove-Error "*Git*"
            Add-Fixed "Git installed"
            Add-NeedRestart "Git"
        } else {
            Write-Guide -Tool "Git" -Lines @(
                "Install from: https://git-scm.com/download/win",
                "Or: winget install --id Git.Git -e"
            )
        }
    } else {
        Write-Guide -Tool "Git" -Lines @(
            "Install from: https://git-scm.com/download/win",
            "Or run: winget install --id Git.Git -e"
        )
    }
}

# ─────────────────────────────────────────────────────────────────────────────
# 2. MSYS2
# ─────────────────────────────────────────────────────────────────────────────
Write-Host ""
Write-Host "[2/8] MSYS2 (UCRT64 Toolchain)" -ForegroundColor White

Write-Check "MSYS2 installation"
$HasMsys2 = Test-Msys2
if ($HasMsys2) {
    Write-OK "C:\msys64"
} else {
    Write-Fail
    Add-Error "MSYS2 is not installed"

    if ($Fix) {
        Write-FixAttempt "Installing MSYS2 via winget..."
        winget install --id MSYS2.MSYS2 -e --accept-source-agreements --accept-package-agreements 2>&1 | Out-Null
        if ($LASTEXITCODE -eq 0) {
            $HasMsys2 = Test-Msys2
            Remove-Error "*MSYS2*"
            Add-Fixed "MSYS2 installed"
            if (-not $HasMsys2) {
                Add-NeedRestart "MSYS2 (then run: C:\msys64\msys2_shell.cmd -ucrt64 -c 'pacman -Syu')"
            }
        } else {
            Write-Guide -Tool "MSYS2" -Lines @(
                "Download and install from: https://www.msys2.org",
                "Or: winget install --id MSYS2.MSYS2 -e",
                "After install, open UCRT64 terminal and run: pacman -Syu"
            )
        }
    } else {
        Write-Guide -Tool "MSYS2" -Lines @(
            "Download and install from: https://www.msys2.org",
            "Or run: winget install --id MSYS2.MSYS2 -e",
            "After install, open UCRT64 terminal and run: pacman -Syu"
        )
    }
}

Write-Check "UCRT64 bin in PATH"
if (Test-Ucrt64InPath) {
    Write-OK
} else {
    if ($HasMsys2) {
        Write-Warn "C:\msys64\ucrt64\bin not in PATH"
        Add-Warning "UCRT64 bin directory not in system PATH"

        if ($Fix) {
            Write-FixAttempt "Adding C:\msys64\ucrt64\bin to user PATH..."
            $currentPath = [Environment]::GetEnvironmentVariable("PATH", "User")
            if ($currentPath -notlike "*msys64\ucrt64\bin*") {
                [Environment]::SetEnvironmentVariable("PATH", "C:\msys64\ucrt64\bin;$currentPath", "User")
                $env:PATH = "C:\msys64\ucrt64\bin;$env:PATH"
                Remove-Warning "*UCRT64*"
                Add-Fixed "Added C:\msys64\ucrt64\bin to user PATH"
            }
        } else {
            Write-Guide -Tool "PATH" -Lines @(
                "Add to system PATH: C:\msys64\ucrt64\bin",
                "PowerShell (permanent):",
                '  [Environment]::SetEnvironmentVariable("PATH", "C:\msys64\ucrt64\bin;" + [Environment]::GetEnvironmentVariable("PATH", "User"), "User")',
                "Or add via: Settings > System > About > Advanced > Environment Variables"
            )
        }
    } else {
        Write-Fail "MSYS2 not installed"
    }
}

# ─────────────────────────────────────────────────────────────────────────────
# 3. Rust Toolchain
# ─────────────────────────────────────────────────────────────────────────────
Write-Host ""
Write-Host "[3/8] Rust Toolchain" -ForegroundColor White

Write-Check "rustc"
# Check both rustc and cargo — on Windows, cargo can exist without rustc in PATH
$HasRustc = Test-Command "rustc"
$HasCargo = Test-Command "cargo"
$HasRust = $HasRustc -or $HasCargo

if ($HasRustc) {
    $rustVersion = (rustc --version 2>&1) -replace 'rustc ', ''
    Write-OK $rustVersion

    if ($rustVersion -match '^(\d+)\.(\d+)') {
        $major = [int]$Matches[1]
        $minor = [int]$Matches[2]
        if ($major -lt 1 -or ($major -eq 1 -and $minor -lt 85)) {
            Add-Warning "Rust $rustVersion detected, 1.85+ recommended (edition 2024 support)"
        }
    }
} else {
    Write-Fail
    Add-Error "rustc is not in PATH"

    if ($Fix) {
        if ($HasCargo) {
            # cargo exists but rustc doesn't — likely PATH issue after install
            Write-FixAttempt "rustc not found but cargo exists. Trying rustup to fix..."
            if (Test-Command "rustup") {
                rustup default stable 2>&1 | Out-Null
                Update-SessionPath
                if (Test-Command "rustc") {
                    $HasRustc = $true
                    Remove-Error "*rustc*"
                    Add-Fixed "rustc restored via rustup default stable"
                } else {
                    Add-NeedRestart "Rust (rustc)"
                }
            }
        } else {
            Write-FixAttempt "Installing Rust via winget (rustup)..."
            winget install --id Rustlang.Rustup -e --accept-source-agreements --accept-package-agreements 2>&1 | Out-Null
            if ($LASTEXITCODE -eq 0) {
                Remove-Error "*rustc*"
                Add-Fixed "Rust (rustup) installed"
                Add-NeedRestart "Rust (restart terminal, then: rustup default stable)"
            } else {
                Write-Guide -Tool "Rust" -Lines @(
                    "Install from: https://rustup.rs",
                    "Or: winget install --id Rustlang.Rustup -e",
                    "After install: rustup default stable"
                )
            }
        }
    } else {
        if ($HasCargo) {
            Write-Guide -Tool "Rust" -Lines @(
                "cargo found but rustc missing. Run: rustup default stable",
                "Then restart terminal."
            )
        } else {
            Write-Guide -Tool "Rust" -Lines @(
                "Install from: https://rustup.rs",
                "Or run: winget install --id Rustlang.Rustup -e",
                "After install: rustup default stable"
            )
        }
    }
}

Write-Check "cargo"
if ($HasCargo) {
    $cargoV = (cargo --version 2>&1) -replace 'cargo ', ''
    Write-OK $cargoV
} else {
    Write-Fail
    if (-not $HasRustc) {
        # Already reported with rustc
    } else {
        Add-Error "cargo not found (Rust installation may be broken)"
    }
}

Write-Check "x86_64-pc-windows-gnu target"
if ($HasRustc -and (Test-Command "rustup")) {
    $targets = rustup target list --installed 2>&1 | Out-String
    if ($targets -like "*x86_64-pc-windows-gnu*") {
        Write-OK
    } else {
        Write-Warn "not installed"
        Add-Warning "GNU target not installed (needed for LLVM build)"

        if ($Fix) {
            Write-FixAttempt "Adding GNU target..."
            rustup target add x86_64-pc-windows-gnu 2>&1 | Out-Null
            if ($LASTEXITCODE -eq 0) {
                Remove-Warning "*GNU target*"
                Add-Fixed "Added x86_64-pc-windows-gnu target"
            }
        } else {
            Write-Guide -Tool "Rust GNU Target" -Lines @(
                "Run: rustup target add x86_64-pc-windows-gnu"
            )
        }
    }
} elseif ($HasCargo -and -not $HasRustc) {
    Write-Warn "skipped (rustc not in PATH)"
} else {
    Write-Fail "Rust not installed"
}

# Check default host — MSVC host without Visual Studio causes link.exe failures
Write-Check "default host toolchain"
$HasRustup = Test-Command "rustup"
if ($HasRustup) {
    $defaultHost = (rustup show 2>&1 | Select-String "Default host") -replace '.*:\s*', ''
    $defaultHost = $defaultHost.Trim()

    if ($defaultHost -like "*msvc*") {
        # Check if MSVC linker is actually available
        $hasLinkExe = Test-Command "link"
        $hasCl = Test-Command "cl"

        if (-not $hasLinkExe -and -not $hasCl) {
            Write-Warn "msvc host but no MSVC Build Tools"
            Add-Error "Rust host is '$defaultHost' but MSVC (link.exe) is not installed. Build scripts will fail."

            if ($Fix) {
                Write-FixAttempt "Switching default host to x86_64-pc-windows-gnu..."
                rustup set default-host x86_64-pc-windows-gnu 2>&1 | Out-Null
                rustup default stable-x86_64-pc-windows-gnu 2>&1 | Out-Null
                if ($LASTEXITCODE -eq 0) {
                    Remove-Error "*Rust host*"
                    Add-Fixed "Switched Rust default host to x86_64-pc-windows-gnu"
                    Add-NeedRestart "Rust host change (restart terminal)"
                } else {
                    Write-Guide -Tool "Rust Host" -Lines @(
                        "Your Rust default host is '$defaultHost' but MSVC is not installed.",
                        "Switch to GNU host (recommended for this project):",
                        "  rustup set default-host x86_64-pc-windows-gnu",
                        "  rustup default stable-x86_64-pc-windows-gnu",
                        "",
                        "Or install Visual Studio Build Tools:",
                        "  winget install Microsoft.VisualStudio.2022.BuildTools",
                        "  (select 'C++ build tools' workload during install)"
                    )
                }
            } else {
                Write-Guide -Tool "Rust Host" -Lines @(
                    "Your Rust default host is '$defaultHost' but MSVC (link.exe) is not installed.",
                    "This causes all build scripts to fail with 'linking with link.exe failed'.",
                    "",
                    "Option A - Switch to GNU host (recommended for this project):",
                    "  rustup set default-host x86_64-pc-windows-gnu",
                    "  rustup default stable-x86_64-pc-windows-gnu",
                    "",
                    "Option B - Install MSVC Build Tools:",
                    "  winget install Microsoft.VisualStudio.2022.BuildTools",
                    "  (select 'C++ build tools' workload during install)"
                )
            }
        } else {
            Write-OK "$defaultHost (MSVC available)"
        }
    } elseif ($defaultHost -like "*gnu*") {
        Write-OK "$defaultHost"
    } else {
        Write-OK "$defaultHost"
    }
} elseif ($HasRustc) {
    # No rustup, check host from rustc
    $hostInfo = (rustc -vV 2>&1 | Select-String "host:") -replace '.*:\s*', ''
    Write-OK $hostInfo.Trim()
} else {
    Write-Warn "skipped (Rust not installed)"
}

# ─────────────────────────────────────────────────────────────────────────────
# 4. LLVM Toolchain
# ─────────────────────────────────────────────────────────────────────────────
Write-Host ""
Write-Host "[4/8] LLVM 21 Toolchain" -ForegroundColor White

$LlvmTools = @("opt", "llc", "clang")
$LlvmOk = $true
$LlvmMissing = @()

foreach ($tool in $LlvmTools) {
    Write-Check $tool
    if (Test-Command $tool) {
        $ver = & $tool --version 2>&1 | Select-String -Pattern '\d+\.\d+' | Select-Object -First 1 | ForEach-Object { $_.Matches[0].Value }
        if ($ver) { Write-OK "v$ver" } else { Write-OK }
    } else {
        Write-Fail
        $LlvmOk = $false
        $LlvmMissing += $tool
    }
}

if (-not $LlvmOk) {
    Add-Error "LLVM tools missing ($($LlvmMissing -join ', '))"

    if ($Fix -and $HasMsys2) {
        Write-FixAttempt "Installing LLVM via MSYS2 pacman..."
        $installed = Invoke-Pacman -Packages @(
            "mingw-w64-ucrt-x86_64-llvm",
            "mingw-w64-ucrt-x86_64-clang"
        )
        if ($installed) {
            # Verify tools are now available
            $allFound = $true
            foreach ($tool in $LlvmMissing) {
                if (-not (Test-Command $tool)) { $allFound = $false }
            }
            if ($allFound) {
                $LlvmOk = $true
                Remove-Error "*LLVM tools*"
                Add-Fixed "LLVM installed via MSYS2"
            } else {
                Remove-Error "*LLVM tools*"
                Add-Fixed "LLVM packages installed via MSYS2"
                Add-NeedRestart "LLVM tools (restart terminal for PATH update)"
            }
        } else {
            Write-Host "    pacman install failed" -ForegroundColor Red
        }
    } elseif (-not $HasMsys2) {
        Write-Guide -Tool "LLVM" -Lines @(
            "Requires MSYS2 first. Install MSYS2, then:",
            "  pacman -S mingw-w64-ucrt-x86_64-llvm mingw-w64-ucrt-x86_64-clang"
        )
    } else {
        Write-Guide -Tool "LLVM" -Lines @(
            "Open MSYS2 UCRT64 terminal and run:",
            "  pacman -S mingw-w64-ucrt-x86_64-llvm mingw-w64-ucrt-x86_64-clang",
            "Or use -Fix flag to auto-install"
        )
    }
}

# ─────────────────────────────────────────────────────────────────────────────
# 5. GCC (Linker)
# ─────────────────────────────────────────────────────────────────────────────
Write-Host ""
Write-Host "[5/8] GCC (Linker)" -ForegroundColor White

Write-Check "gcc"
$HasGcc = Test-Command "gcc"
if ($HasGcc) {
    $gccVer = (gcc --version 2>&1 | Select-Object -First 1) -replace '.*\) ', ''
    Write-OK $gccVer
} else {
    Write-Fail
    Add-Error "GCC is not installed (required for linking)"

    if ($Fix -and $HasMsys2) {
        Write-FixAttempt "Installing GCC via MSYS2 pacman..."
        $installed = Invoke-Pacman -Packages @("mingw-w64-ucrt-x86_64-gcc")
        if ($installed) {
            $HasGcc = Test-Command "gcc"
            Remove-Error "*GCC*"
            Add-Fixed "GCC installed via MSYS2"
            if (-not $HasGcc) {
                Add-NeedRestart "GCC"
            }
        }
    } elseif (-not $HasMsys2) {
        Write-Guide -Tool "GCC" -Lines @(
            "Requires MSYS2 first. Install MSYS2, then:",
            "  pacman -S mingw-w64-ucrt-x86_64-gcc"
        )
    } else {
        Write-Guide -Tool "GCC" -Lines @(
            "Open MSYS2 UCRT64 terminal and run:",
            "  pacman -S mingw-w64-ucrt-x86_64-gcc"
        )
    }
}

Write-Check "ar (archiver)"
if (Test-Command "ar") {
    Write-OK
} else {
    Write-Warn "not found (needed for runtime library)"
    Add-Warning "ar not found (installed with GCC)"
}

# Check expected linker path from .cargo/config.toml
Write-Check "linker at C:\msys64\ucrt64\bin\gcc.exe"
if (Test-Path "C:\msys64\ucrt64\bin\gcc.exe") {
    Write-OK
} else {
    if ($HasGcc) {
        Write-Warn "gcc exists but not at expected path"
        Add-Warning ".cargo/config.toml expects gcc at C:\msys64\ucrt64\bin\gcc.exe"
    } else {
        Write-Fail
    }
}

# ─────────────────────────────────────────────────────────────────────────────
# 6. Python
# ─────────────────────────────────────────────────────────────────────────────
Write-Host ""
Write-Host "[6/8] Python (Benchmarks)" -ForegroundColor White

Write-Check "python3 / python"
$HasPython = $false
$pythonCmd = $null

# Detect real Python (not Windows Store stub which outputs just "Python" with no version)
foreach ($candidate in @("python3", "python")) {
    if (Test-Command $candidate) {
        $pyOutput = (& $candidate --version 2>&1 | Out-String).Trim()
        # Windows Store stub outputs "Python" with no version number — skip it
        if ($pyOutput -match 'Python (\d+\.\d+\.\d+)') {
            $pyFullVer = $Matches[1]
            if ($pyFullVer -like "3.*") {
                $pythonCmd = $candidate
                $HasPython = $true
                break
            }
        }
    }
}

if ($HasPython) {
    Write-OK "Python $pyFullVer"
} else {
    # Check if Windows Store stub is present
    $stubPath = "$env:LOCALAPPDATA\Microsoft\WindowsApps\python.exe"
    if (Test-Path $stubPath) {
        Write-Warn "only Windows Store stub found (not real Python)"
    } else {
        Write-Warn "not found (optional, needed for benchmark scripts)"
    }
    Add-Warning "Python 3 not found (needed for benchmark comparison scripts)"

    if ($Fix) {
        Write-FixAttempt "Installing Python via winget..."
        winget install --id Python.Python.3.12 -e --accept-source-agreements --accept-package-agreements 2>&1 | Out-Null
        if ($LASTEXITCODE -eq 0) {
            Remove-Warning "*Python*"
            Add-Fixed "Python 3.12 installed"
            Add-NeedRestart "Python"
        } else {
            Write-Guide -Tool "Python" -Lines @(
                "Install from: https://www.python.org/downloads/",
                "Or: winget install --id Python.Python.3.12 -e",
                "Check 'Add to PATH' during installation"
            )
        }
    } else {
        Write-Guide -Tool "Python" -Lines @(
            "Install from: https://www.python.org/downloads/",
            "Or run: winget install --id Python.Python.3.12 -e",
            "Check 'Add to PATH' during installation"
        )
    }
}

# ─────────────────────────────────────────────────────────────────────────────
# 7. Environment Variables & Project Files
# ─────────────────────────────────────────────────────────────────────────────
Write-Host ""
Write-Host "[7/8] Environment & Project" -ForegroundColor White

# LLVM_SYS_211_PREFIX
Write-Check "LLVM_SYS_211_PREFIX"
$llvmPrefix = $env:LLVM_SYS_211_PREFIX
if ($llvmPrefix) {
    if (Test-Path "$llvmPrefix\bin") {
        Write-OK $llvmPrefix
    } else {
        Write-Warn "set to '$llvmPrefix' but bin/ not found"
        Add-Warning "LLVM_SYS_211_PREFIX points to non-existent directory"
    }
} else {
    # Check if .cargo/config.toml handles it
    $cargoConfig = "$ProjectRoot\.cargo\config.toml"
    if (Test-Path $cargoConfig) {
        $content = Get-Content $cargoConfig -Raw
        if ($content -match 'LLVM_SYS_211_PREFIX') {
            Write-OK "set in .cargo/config.toml"
        } else {
            Write-Warn "not set"
            Add-Warning "LLVM_SYS_211_PREFIX not set (may be auto-detected)"
        }
    } else {
        Write-Warn "not set"
        Add-Warning "LLVM_SYS_211_PREFIX not set"
    }
}

# BMB_RUNTIME_PATH
Write-Check "BMB_RUNTIME_PATH"
$runtimeEnv = $env:BMB_RUNTIME_PATH
if ($runtimeEnv) {
    if (Test-Path $runtimeEnv) {
        Write-OK $runtimeEnv
    } else {
        Write-Warn "set but path not found: $runtimeEnv"
    }
} else {
    $defaultRuntime = "$ProjectRoot\bmb\runtime"
    if (Test-Path "$defaultRuntime\bmb_runtime.c") {
        Write-OK "not set, but runtime found at default location"
    } else {
        Write-Warn "not set, runtime not found"
        Add-Warning "BMB_RUNTIME_PATH not set and runtime not at default location"
    }
}

# .cargo/config.toml
Write-Check ".cargo/config.toml"
if (Test-Path "$ProjectRoot\.cargo\config.toml") {
    Write-OK
} else {
    Write-Warn "missing (linker and LLVM settings)"
    Add-Warning ".cargo/config.toml not found"
}

# C Runtime files
Write-Check "C runtime sources"
$runtimeC = "$ProjectRoot\bmb\runtime\bmb_runtime.c"
$eventLoopC = "$ProjectRoot\bmb\runtime\bmb_event_loop.c"
if ((Test-Path $runtimeC) -and (Test-Path $eventLoopC)) {
    Write-OK
} else {
    Write-Fail "runtime C sources missing"
    Add-Error "Runtime C sources not found in bmb/runtime/"
}

# Pre-built runtime library
Write-Check "pre-built libbmb_runtime.a"
$runtimeLib = "$ProjectRoot\bmb\runtime\libbmb_runtime.a"
if (Test-Path $runtimeLib) {
    $size = (Get-Item $runtimeLib).Length / 1KB
    Write-OK ("{0:N0} KB" -f $size)
} else {
    Write-Warn "not found (will need to build)"
    Add-Warning "Pre-built runtime library not found"

    if ($Fix -and (Test-Command "clang") -and (Test-Command "ar") -and (Test-Path $runtimeC)) {
        Write-FixAttempt "Building runtime library..."
        Push-Location "$ProjectRoot\bmb\runtime"
        try {
            & clang -c bmb_runtime.c -o bmb_runtime.o -O3 2>&1 | Out-Null
            & clang -c bmb_event_loop.c -o bmb_event_loop.o -O3 2>&1 | Out-Null
            & ar rcs libbmb_runtime.a bmb_runtime.o bmb_event_loop.o 2>&1 | Out-Null
            if (Test-Path "libbmb_runtime.a") {
                Remove-Warning "*Pre-built runtime*"
                Add-Fixed "Built libbmb_runtime.a"
            }
        } catch {
            Write-Host "    Build failed: $_" -ForegroundColor Red
        }
        Pop-Location
    } elseif (-not $Fix) {
        Write-Guide -Tool "Runtime Library" -Lines @(
            "cd bmb\runtime",
            "clang -c bmb_runtime.c -o bmb_runtime.o -O3",
            "clang -c bmb_event_loop.c -o bmb_event_loop.o -O3",
            "ar rcs libbmb_runtime.a bmb_runtime.o bmb_event_loop.o"
        )
    }
}

# Golden binary
Write-Check "golden binary"
$goldenBin = "$ProjectRoot\golden\windows-x64\bmb.exe"
if (Test-Path $goldenBin) {
    $size = (Get-Item $goldenBin).Length / 1KB
    Write-OK ("{0:N0} KB" -f $size)
} else {
    Write-Warn "not found (optional, for golden bootstrap)"
    Add-Warning "Golden binary not found at golden/windows-x64/bmb.exe"
}

# ─────────────────────────────────────────────────────────────────────────────
# 8. Build Verification
# ─────────────────────────────────────────────────────────────────────────────
Write-Host ""
Write-Host "[8/8] Build Verification" -ForegroundColor White

$CanBuild = $HasRustc -and $HasCargo

# Skip build verification if host toolchain issue was already detected
$HostIssueDetected = ($script:Errors | Where-Object { $_ -like "*Rust host*" }).Count -gt 0

Write-Check "cargo check --release"
if (-not $CanBuild) {
    Write-Warn "skipped (Rust toolchain not ready)"
} elseif ($HostIssueDetected) {
    Write-Warn "skipped (fix Rust host toolchain first)"
} else {
    $checkOutput = cargo check --release 2>&1
    $checkExitCode = $LASTEXITCODE
    if ($checkExitCode -eq 0) {
        Write-OK
    } else {
        $outputStr = ($checkOutput | Out-String).Trim()
        $isLinkExeError = $outputStr -match 'link\.exe'

        if ($isLinkExeError) {
            Write-Fail "link.exe not found (MSVC host without MSVC tools)"
            Add-Error "cargo check failed: Rust host uses MSVC but link.exe is missing. Run: rustup set default-host x86_64-pc-windows-gnu && rustup default stable-x86_64-pc-windows-gnu"
        } else {
            Write-Fail "cargo check failed"
            Add-Error "cargo check --release failed"
        }

        # Show last error lines for diagnosis
        $errorLines = $outputStr -split "`n" | Where-Object { $_ -match '(error[:\[]|linking with)' } | Select-Object -Last 5
        if ($errorLines) {
            Write-Host ""
            foreach ($line in $errorLines) {
                Write-Host "    $($line.Trim())" -ForegroundColor DarkRed
            }
        } elseif ($Verbose) {
            Write-Host $outputStr -ForegroundColor DarkGray
        }
    }
}

Write-Check "cargo check --features llvm --target x86_64-pc-windows-gnu"
if (-not $CanBuild) {
    Write-Warn "skipped (Rust toolchain not ready)"
} elseif ($HostIssueDetected) {
    Write-Warn "skipped (fix Rust host toolchain first)"
} elseif (-not $LlvmOk) {
    Write-Warn "skipped (LLVM tools missing)"
} else {
    $checkOutput = cargo check --release --features llvm --target x86_64-pc-windows-gnu 2>&1
    $checkExitCode = $LASTEXITCODE
    if ($checkExitCode -eq 0) {
        Write-OK
    } else {
        $outputStr = ($checkOutput | Out-String).Trim()
        $isLinkExeError = $outputStr -match 'link\.exe'

        if ($isLinkExeError) {
            Write-Warn "link.exe not found (same host toolchain issue)"
        } else {
            Write-Warn "LLVM build check failed"
            Add-Warning "cargo check with LLVM feature failed (interpreter-only build still works)"
        }

        $errorLines = $outputStr -split "`n" | Where-Object { $_ -match '(error[:\[]|linking with)' } | Select-Object -Last 5
        if ($errorLines) {
            Write-Host ""
            foreach ($line in $errorLines) {
                Write-Host "    $($line.Trim())" -ForegroundColor DarkYellow
            }
        } elseif ($Verbose) {
            Write-Host $outputStr -ForegroundColor DarkGray
        }
    }
}

# ─────────────────────────────────────────────────────────────────────────────
# Summary
# ─────────────────────────────────────────────────────────────────────────────
Write-Host ""
Write-Host "  ==========================================" -ForegroundColor DarkCyan
Write-Host "  Summary" -ForegroundColor Cyan
Write-Host "  ==========================================" -ForegroundColor DarkCyan
Write-Host ""

# Fixed items
if ($script:Fixed.Count -gt 0) {
    Write-Host "  Fixed:" -ForegroundColor Green
    foreach ($f in $script:Fixed) {
        Write-Host "    [+] $f" -ForegroundColor Green
    }
    Write-Host ""
}

# Need restart
if ($script:NeedRestart.Count -gt 0) {
    Write-Host "  Restart Required:" -ForegroundColor Magenta
    foreach ($r in $script:NeedRestart) {
        Write-Host "    [~] $r" -ForegroundColor Magenta
    }
    Write-Host ""
}

# Errors
if ($script:Errors.Count -gt 0) {
    Write-Host "  Errors ($($script:Errors.Count)):" -ForegroundColor Red
    foreach ($e in $script:Errors) {
        Write-Host "    [x] $e" -ForegroundColor Red
    }
    Write-Host ""
}

# Warnings
if ($script:Warnings.Count -gt 0) {
    Write-Host "  Warnings ($($script:Warnings.Count)):" -ForegroundColor Yellow
    foreach ($w in $script:Warnings) {
        Write-Host "    [!] $w" -ForegroundColor Yellow
    }
    Write-Host ""
}

# Installation guides
if ($script:Guides.Count -gt 0) {
    Write-Host "  ==========================================" -ForegroundColor DarkCyan
    Write-Host "  Installation Guide" -ForegroundColor Cyan
    Write-Host "  ==========================================" -ForegroundColor DarkCyan
    Write-Host ""

    foreach ($guide in $script:Guides) {
        Write-Host "  $($guide.Tool):" -ForegroundColor White
        foreach ($line in $guide.Lines) {
            Write-Host "    $line" -ForegroundColor Gray
        }
        Write-Host ""
    }
}

# Final verdict
$hasUnresolved = $script:Errors.Count -gt 0
$hasNeedRestart = $script:NeedRestart.Count -gt 0

if (-not $hasUnresolved -and -not $hasNeedRestart -and $script:Warnings.Count -eq 0) {
    Write-Host "  All checks passed. Ready to build!" -ForegroundColor Green
    Write-Host ""
    Write-Host "  Quick start:" -ForegroundColor DarkGray
    Write-Host "    cargo build --release                                                 # interpreter only" -ForegroundColor DarkGray
    Write-Host "    cargo build --release --features llvm --target x86_64-pc-windows-gnu  # full build" -ForegroundColor DarkGray
    Write-Host "    cargo test --release                                                  # run tests" -ForegroundColor DarkGray
    Write-Host ""
    exit 0
} elseif (-not $hasUnresolved -and $hasNeedRestart) {
    Write-Host "  All tools installed. Restart terminal and run doctor.ps1 again to verify." -ForegroundColor Magenta
    Write-Host ""
    exit 2
} elseif (-not $hasUnresolved) {
    Write-Host "  Environment OK with warnings. Core build should work." -ForegroundColor Yellow
    Write-Host ""
    exit 2
} else {
    Write-Host "  Environment has issues. Fix errors above before building." -ForegroundColor Red
    if (-not $Fix) {
        Write-Host "  Tip: Run with -Fix to auto-install missing tools." -ForegroundColor DarkGray
    } else {
        Write-Host "  Tip: Restart terminal and run doctor.ps1 again after fixes." -ForegroundColor DarkGray
    }
    Write-Host ""
    exit 1
}
