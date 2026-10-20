@echo off
REM Build helper: configures the Visual Studio 2019 Build Tools environment and
REM builds the Secutest backend for the x86_64-pc-windows-msvc target.
REM
REM The default Rust host on this machine is x86_64-pc-windows-gnu, but no
REM GCC/libgcc is available there, so the MSVC toolchain is used instead.

setlocal

set "VSDIR=C:\Program Files (x86)\Microsoft Visual Studio\2019\BuildTools"
if not exist "%VSDIR%\VC\Auxiliary\Build\vcvars64.bat" (
    echo ERROR: vcvars64.bat not found under "%VSDIR%".
    exit /b 1
)

call "%VSDIR%\VC\Auxiliary\Build\vcvars64.bat" >nul
if errorlevel 1 (
    echo ERROR: failed to configure the MSVC build environment.
    exit /b 1
)

cd /d "%~dp0"

REM Allow any cargo subcommand: build-msvc.cmd, build-msvc.cmd test,
REM build-msvc.cmd clippy, build-msvc.cmd run, ... Default to "build".
set "CARGO_SUBCOMMAND=build"
if not "%~1"=="" set "CARGO_SUBCOMMAND=%~1"
shift

cargo %CARGO_SUBCOMMAND% --target x86_64-pc-windows-msvc %*
exit /b %errorlevel%
