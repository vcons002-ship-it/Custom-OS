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

for %%A in ("%~dp0.") do set "WINREPO=%%~fA"

echo   [*] Installing Clade's Linux dependencies (tools/setup.sh in WSL) ...
echo.
rem Prefer the WSL-side repo copy (fast filesystem); fall back to running
rem the installer straight from this Windows checkout.
wsl -d %DISTRO% -- bash -lc "if [ -f ~/clade/Custom-OS/tools/setup.sh ]; then bash ~/clade/Custom-OS/tools/setup.sh; else SRC=\"$(wslpath -u '%WINREPO%')\"; bash \"$SRC/tools/setup.sh\"; fi"

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
