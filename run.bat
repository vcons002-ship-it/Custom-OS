@echo off
setlocal
rem ============================================================
rem  Clade - start the system.
rem
rem    run.bat        boots the real Clade OS in QEMU if the image
rem                   is built; otherwise starts the dev harness
rem                   (same services, no VM) and prints the image
rem                   build command.
rem    run.bat dev    force the dev harness.
rem    run.bat headless   boot the OS with serial console only.
rem
rem  The QEMU window appears via WSLg. Clade's mind lives on a
rem  persistent data volume (~/clade/clade-data.img inside WSL) that
rem  survives shutdowns and OS-image rebuilds. Ctrl-C here stops it.
rem ============================================================

set "DISTRO=Ubuntu-24.04"
set "MODE=%~1"

wsl -d %DISTRO% -- true >nul 2>&1
if errorlevel 1 (
    echo   [!] WSL2/%DISTRO% is not installed. Run setup.bat first.
    goto :end
)

rem --cd starts bash inside this checkout; the payload first SYNCS it into
rem the WSL filesystem (so a git pull on Windows is picked up automatically),
rem then runs from there. All paths are $HOME-relative (no spaces possible) -
rem no escaped quotes or path conversion for wsl.exe to mangle.
echo   [*] Starting Clade via WSL ^(%DISTRO%^) - syncing repo, then booting ...
echo.
wsl -d %DISTRO% --cd "%~dp0." -- bash -lc "set -e; if [ -e $HOME/clade ] && [ ! -O $HOME/clade ]; then echo '[fix] ~/clade was created by root in an earlier run - taking ownership'; chown -R $USER: $HOME/clade 2>/dev/null || sudo chown -R $USER: $HOME/clade; fi; if [ -e $HOME/clade/Custom-OS ] && [ ! -O $HOME/clade/Custom-OS ]; then echo '[fix] taking ownership of the WSL repo copy'; chown -R $USER: $HOME/clade 2>/dev/null || sudo chown -R $USER: $HOME/clade; fi; if [ -f ./Cargo.toml ]; then mkdir -p $HOME/clade/Custom-OS; cp -af . $HOME/clade/Custom-OS/; fi; cd $HOME/clade/Custom-OS 2>/dev/null || { echo '  [!] repo not found in WSL - run setup.bat first'; exit 1; }; sed -i 's/\r$//' tools/*.sh kernel/buildroot-external/board/clade/*.sh; IMG=$HOME/clade/buildroot/output/images; MODE='%MODE%'; if [ x$MODE = xdev ]; then exec tools/dev-run.sh; fi; if [ -f $IMG/bzImage ] && [ -f $IMG/rootfs.ext4 ]; then exec tools/qemu-run.sh $IMG $MODE; else echo; echo '  [*] The Clade OS image is not built yet - starting the dev harness'; echo '      (same mind-plane services, no VM). To build the real image once'; echo '      (~30-60 min, then cached):'; echo; echo '        wsl -d Ubuntu-24.04'; echo '        cd ~/clade/buildroot'; echo '        make BR2_EXTERNAL=$HOME/clade/Custom-OS/kernel/buildroot-external clade_x86_64_defconfig'; echo '        make'; echo; exec tools/dev-run.sh; fi"

:end
echo.
pause
endlocal
