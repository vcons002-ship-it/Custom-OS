@echo off
setlocal
rem ============================================================
rem  Clade - build the bootable OS image (Buildroot).
rem
rem  First build: ~30-60 minutes (downloads + compiles the kernel
rem  and toolchain). After that it is cached and rebuilds of Clade
rem  changes take a few minutes. Safe to interrupt: re-running
rem  resumes where it left off.
rem
rem  When it finishes, start Clade with run.bat - it will find the
rem  image and boot it instead of the dev harness.
rem ============================================================

set "DISTRO=Ubuntu-24.04"

wsl -d %DISTRO% -- true >nul 2>&1
if errorlevel 1 (
    echo   [!] WSL2/%DISTRO% is not installed. Run setup.bat first.
    goto :end
)

echo   [*] Building the Clade OS image. First build takes 30-60 minutes;
echo       leave this window open. Re-running resumes if interrupted.
echo.

wsl -d %DISTRO% --cd "%~dp0." -- bash -lc "set -e; if [ -e $HOME/clade ] && [ ! -O $HOME/clade ]; then echo '[fix] taking ownership of ~/clade'; chown -R $USER: $HOME/clade 2>/dev/null || sudo chown -R $USER: $HOME/clade; fi; if [ -f ./Cargo.toml ]; then mkdir -p $HOME/clade/Custom-OS; cp -a . $HOME/clade/Custom-OS/; fi; cd $HOME/clade/Custom-OS; sed -i 's/\r$//' tools/*.sh kernel/buildroot-external/board/clade/*.sh; cd $HOME/clade/buildroot 2>/dev/null || { echo '  [!] Buildroot not found - run install-deps.bat first'; exit 1; }; make BR2_EXTERNAL=$HOME/clade/Custom-OS/kernel/buildroot-external clade_x86_64_defconfig; exec make"

if errorlevel 1 (
    echo.
    echo   [!] The image build stopped with an error - scroll up for the
    echo       first failing line and paste it back to Claude.
    goto :end
)

echo.
echo   ============================================================
echo    Image built. Boot your OS:  run.bat
echo    Artifacts: ~/clade/buildroot/output/images (inside WSL)
echo   ============================================================

:end
echo.
pause
endlocal
