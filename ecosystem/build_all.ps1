# Build all BMB binding libraries (Windows PowerShell)
# Usage: .\ecosystem\build_all.ps1

$ErrorActionPreference = "Stop"
$BMB = ".\target\release\bmb.exe"

if (-not (Test-Path $BMB)) {
    Write-Host "ERROR: BMB compiler not found at $BMB" -ForegroundColor Red
    Write-Host "Run: cargo build --release"
    exit 1
}

Write-Host "=== Building BMB Binding Libraries ===" -ForegroundColor Cyan
Write-Host ""

$libs = @(
    @{name="bmb-algo"; src="ecosystem/bmb-algo/src/lib.bmb"; out="ecosystem/bmb-algo/bmb_algo.dll"; funcs=41}
    @{name="bmb-compute"; src="ecosystem/bmb-compute/src/lib.bmb"; out="ecosystem/bmb-compute/bmb_compute.dll"; funcs=25}
    @{name="bmb-crypto"; src="ecosystem/bmb-crypto/src/lib.bmb"; out="ecosystem/bmb-crypto/bmb_crypto.dll"; funcs=11}
    @{name="bmb-text"; src="ecosystem/bmb-text/src/lib.bmb"; out="ecosystem/bmb-text/bmb_text.dll"; funcs=20}
    @{name="bmb-json"; src="ecosystem/bmb-json/src/lib.bmb"; out="ecosystem/bmb-json/bmb_json.dll"; funcs=8}
)

$i = 0
foreach ($lib in $libs) {
    $i++
    Write-Host "[$i/$($libs.Count)] Building $($lib.name) ($($lib.funcs) functions)..." -ForegroundColor Yellow
    & $BMB build $lib.src --shared -o $lib.out 2>&1 | Select-String "build_success|error"
    $pyDir = "$(Split-Path $lib.out)\bindings\python"
    if (Test-Path $pyDir) {
        Copy-Item $lib.out $pyDir -Force
    }
}

Write-Host ""
Write-Host "=== All 5 libraries built ===" -ForegroundColor Green
$total = ($libs | Measure-Object -Property funcs -Sum).Sum
Write-Host "  Total: $total @export functions"
