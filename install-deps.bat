@echo off
setlocal
rem ============================================================
rem  Clade - install/refresh the Linux-side dependencies only.
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
rem The checkout is synced into the fast WSL-side copy on every run (so a
rem git pull on Windows is always picked up), then the installer runs there.
wsl -d %DISTRO% --cd "%~dp0." -- bash -lc "set -e; if [ -e $HOME/clade ] && [ ! -O $HOME/clade ]; then echo '[fix] ~/clade was created by root in an earlier run - taking ownership'; chown -R $USER: $HOME/clade 2>/dev/null || sudo chown -R $USER: $HOME/clade; fi; if [ -e $HOME/clade/Custom-OS ] && [ ! -O $HOME/clade/Custom-OS ]; then echo '[fix] taking ownership of the WSL repo copy'; chown -R $USER: $HOME/clade 2>/dev/null || sudo chown -R $USER: $HOME/clade; fi; echo '[install-deps] syncing repo into WSL filesystem...'; mkdir -p $HOME/clade/Custom-OS; cp -af . $HOME/clade/Custom-OS/; cd $HOME/clade/Custom-OS; sed -i 's/\r$//' tools/*.sh kernel/buildroot-external/board/clade/*.sh; exec bash tools/setup.sh"

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
