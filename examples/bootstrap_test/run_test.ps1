# BMB Bootstrap End-to-End Test Script
# Run from Developer PowerShell for VS 2022

param(
    [switch]$Clean,
    [switch]$SkipInterpreter
)

$ErrorActionPreference = "Stop"
$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$RuntimeDir = Join-Path (Split-Path -Parent (Split-Path -Parent $ScriptDir)) "runtime"
Push-Location $ScriptDir

Write-Host "=== BMB Bootstrap End-to-End Test ===" -ForegroundColor Cyan
Write-Host ""

# Clean up
if ($Clean) {
    Write-Host "Cleaning..." -ForegroundColor Yellow
    Remove-Item -Force *.obj, *.exe 2>$null
}

# Find clang
$Clang = "C:\Program Files\LLVM\bin\clang.exe"
if (-not (Test-Path $Clang)) {
    Write-Host "ERROR: clang not found at $Clang" -ForegroundColor Red
    exit 1
}

# Check for cl.exe (needed for linking on Windows)
$HasCL = $null -ne (Get-Command cl -ErrorAction SilentlyContinue)
if (-not $HasCL) {
    Write-Host "WARNING: cl.exe not found. Run from Developer PowerShell." -ForegroundColor Yellow
    Write-Host ""
}

# Step 1: Run with interpreter (baseline)
if (-not $SkipInterpreter) {
    Write-Host "[1/5] Running with BMB interpreter..." -ForegroundColor Green
    Push-Location (Split-Path -Parent (Split-Path -Parent $ScriptDir))
    $InterpreterResult = & cargo run --release --bin bmb -- run examples/bootstrap_test/fibonacci.bmb 2>&1 | Select-String "^\d+$"
    Pop-Location
    Write-Host "  Interpreter result: $InterpreterResult" -ForegroundColor Gray
} else {
    $InterpreterResult = "55"  # Known correct value
    Write-Host "[1/5] Skipping interpreter (using known value: 55)" -ForegroundColor Yellow
}

# Step 2: Compile LLVM IR to object file
Write-Host "[2/5] Compiling LLVM IR..." -ForegroundColor Green
& $Clang -c fibonacci.ll -o fibonacci.obj 2>&1 | Where-Object { $_ -notmatch "warning:" }
if (-not (Test-Path fibonacci.obj)) {
    Write-Host "  ERROR: Failed to compile LLVM IR" -ForegroundColor Red
    exit 1
}
Write-Host "  Created fibonacci.obj" -ForegroundColor Gray

# Step 3: Compile runtime
Write-Host "[3/5] Compiling runtime..." -ForegroundColor Green
if ($HasCL) {
    & cl /c /nologo "$RuntimeDir\runtime.c" /Foruntime.obj 2>&1 | Out-Null
    if (-not (Test-Path runtime.obj)) {
        Write-Host "  ERROR: Failed to compile runtime" -ForegroundColor Red
        exit 1
    }
    Write-Host "  Created runtime.obj" -ForegroundColor Gray
} else {
    Write-Host "  SKIPPED: No C compiler available" -ForegroundColor Yellow
    Write-Host ""
    Write-Host "To complete the test, run from Developer PowerShell:" -ForegroundColor Cyan
    Write-Host "  & '$ScriptDir\run_test.ps1'" -ForegroundColor White
    Pop-Location
    exit 0
}

# Step 4: Link
Write-Host "[4/5] Linking..." -ForegroundColor Green
& cl /nologo fibonacci.obj runtime.obj /Fe:fibonacci.exe 2>&1 | Out-Null
if (-not (Test-Path fibonacci.exe)) {
    Write-Host "  ERROR: Failed to link" -ForegroundColor Red
    exit 1
}
$Size = (Get-Item fibonacci.exe).Length
Write-Host "  Created fibonacci.exe ($Size bytes)" -ForegroundColor Gray

# Step 5: Run and compare
Write-Host "[5/5] Running native executable..." -ForegroundColor Green
$NativeResult = & .\fibonacci.exe 2>&1 | Select-String "^\d+$"
Write-Host "  Native result: $NativeResult" -ForegroundColor Gray

Write-Host ""
Write-Host "=== Results ===" -ForegroundColor Cyan
Write-Host "Interpreter: $InterpreterResult"
Write-Host "Native:      $NativeResult"

if ($InterpreterResult -eq $NativeResult) {
    Write-Host ""
    Write-Host "SUCCESS: Results match!" -ForegroundColor Green
    $ExitCode = 0
} else {
    Write-Host ""
    Write-Host "FAILURE: Results do not match!" -ForegroundColor Red
    $ExitCode = 1
}

Pop-Location
exit $ExitCode
