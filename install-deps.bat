@echo off
setlocal
rem ============================================================
rem  Clade — install/refresh the Linux-side dependencies only.
rem
rem  Runs tools/setup.sh inside WSL2 (packages, Rust, QEMU,
rem  Buildroot + dev-loop verification). Assumes WSL2/Ubuntu is
rem  already installed - if it is not, run setup.bat instead:
rem  it bootstraps WSL itself, then does everything this does.
rem ============================================================

set "DISTRO=Ubuntu-24.04"

wsl -d %DISTRO% -- true >nul 2>&1
if errorlevel 1 (
    echo   [!] WSL2/%DISTRO% is not installed yet.
    echo       Run setup.bat first ^(as Administrator^) - it installs WSL2,
    echo       then runs this same dependency installer automatically.
    goto :end
)

echo   [*] Installing Clade's Linux dependencies (tools/setup.sh in WSL) ...
echo.
rem --cd starts bash inside THIS folder via WSL's automount - no path
rem conversion, no command substitution, no escaped quotes to mangle.
rem Prefer the fast WSL-side repo copy; create it from this checkout if
rem missing, then run the installer there.
wsl -d %DISTRO% --cd "%~dp0" -- bash -lc "set -e; if [ ! -f $HOME/clade/Custom-OS/tools/setup.sh ]; then echo '[install-deps] copying repo into WSL filesystem...'; mkdir -p $HOME/clade/Custom-OS; cp -a . $HOME/clade/Custom-OS/; fi; cd $HOME/clade/Custom-OS; exec bash tools/setup.sh"

if errorlevel 1 (
    echo.
    echo   [!] setup.sh reported a problem - scroll up for the failing step.
    echo       Re-running is safe; completed steps are skipped.
) else (
    echo.
    echo   [*] Dependencies ready. Start Clade with run.bat
)

:end
echo.
pause
endlocal
