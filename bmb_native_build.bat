@echo off
REM BMB Native Build Script (Rust-free)
REM Usage: bmb_native_build.bat <input.bmb> <output.exe>
REM Limitations: Works for files with ~60 functions or fewer

setlocal enabledelayedexpansion

if "%~1"=="" (
    echo Usage: bmb_native_build.bat ^<input.bmb^> [output.exe]
    echo.
    echo This script compiles BMB files without Rust dependency.
    echo Uses BMB Native compiler ^(bmb_unified_fixed.exe^).
    echo.
    echo Limitations:
    echo   - Works for files with ~60 functions or fewer
    echo   - Large files may timeout or crash
    exit /b 1
)

set INPUT=%~1
set OUTPUT=%~2
if "%OUTPUT%"=="" set OUTPUT=%~n1.exe

set SCRIPT_DIR=%~dp0
set BMB_NATIVE=%SCRIPT_DIR%bmb_unified_fixed.exe
set LL_OUTPUT=%~n1.ll

echo === BMB Native Build (Rust-free) ===
echo Input:  %INPUT%
echo Output: %OUTPUT%
echo.

REM Check if BMB Native compiler exists
if not exist "%BMB_NATIVE%" (
    echo ERROR: BMB Native compiler not found: %BMB_NATIVE%
    echo Run 'bmb build bootstrap/bmb_unified_cli.bmb' first.
    exit /b 1
)

REM Initialize Visual Studio environment
call "C:\Program Files\Microsoft Visual Studio\18\Enterprise\VC\Auxiliary\Build\vcvars64.bat" >nul 2>&1
if errorlevel 1 (
    call "C:\Program Files\Microsoft Visual Studio\2022\Enterprise\VC\Auxiliary\Build\vcvars64.bat" >nul 2>&1
)

REM Step 1: Compile BMB to LLVM IR
echo [1/3] Compiling BMB to LLVM IR...
"%BMB_NATIVE%" "%INPUT%" "%LL_OUTPUT%"
if errorlevel 1 (
    echo ERROR: BMB compilation failed
    exit /b 1
)

REM Step 2: Fix LLVM IR format (| to newlines, fix target triple)
echo [2/3] Formatting LLVM IR...
powershell -Command "(Get-Content '%LL_OUTPUT%') -replace '\|', \"`n\" -replace 'x86_64-unknown-linux-gnu', 'x86_64-pc-windows-msvc' | Set-Content '%LL_OUTPUT%'"

REM Step 3: Compile LLVM IR to executable
echo [3/3] Compiling to native executable...
clang -O2 "%LL_OUTPUT%" "%SCRIPT_DIR%runtime\runtime.c" -o "%OUTPUT%"
if errorlevel 1 (
    echo ERROR: Clang compilation failed
    exit /b 1
)

echo.
echo === Build complete ===
echo Executable: %OUTPUT%
dir "%OUTPUT%"
