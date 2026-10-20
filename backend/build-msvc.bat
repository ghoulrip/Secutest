@echo off
REM Build helper: sets up the Visual Studio 2019 Build Tools environment then
REM builds the Secutest backend for x86_64-pc-windows-msvc.
REM This must be run from cmd.exe so vcvars64.bat can export PATH/link.exe into
REM the current process environment.

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
cargo build --target x86_64-pc-windows-msvc %*
exit /b %errorlevel%
