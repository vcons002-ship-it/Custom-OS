@echo off
setlocal EnableDelayedExpansion
rem ============================================================
rem  Clade - Windows 11 setup
rem
rem  Bootstraps WSL2 (Ubuntu 24.04), enables nested virtualization
rem  so QEMU gets KVM, clones the repo INSIDE the WSL filesystem
rem  (building on /mnt/c is painfully slow), and runs tools/setup.sh
rem  there to install every dependency and verify the dev loop.
rem
rem  Safe to re-run at any point; each step skips itself when done.
rem ============================================================

set "DISTRO=Ubuntu-24.04"

echo.
echo   C L A D E  --  setup (Windows 11 host)
echo.

rem ---- 1. Administrator check (wsl --install needs it) --------
net session >nul 2>&1
if errorlevel 1 (
    echo   [!] This script needs Administrator rights the first time
    echo       ^(installing WSL requires it^).
    echo       Right-click setup.bat and choose "Run as administrator".
    goto :end
)

rem ---- 2. WSL2 present? ----------------------------------------
wsl --status >nul 2>&1
if errorlevel 1 (
    echo   [*] WSL is not installed. Installing WSL2 + %DISTRO% ...
    wsl --install -d %DISTRO% --no-launch
    if errorlevel 1 (
        echo   [!] wsl --install failed. Enable virtualization in your
        echo       BIOS ^(AMD: SVM Mode^) and try again.
        goto :end
    )
    echo.
    echo   [*] WSL installed. Windows needs a REBOOT to finish.
    echo       After rebooting, run setup.bat again - it resumes here.
    goto :end
)

rem Make sure the distro exists (wsl can be present without Ubuntu)
wsl -d %DISTRO% -- true >nul 2>&1
if errorlevel 1 (
    echo   [*] Installing %DISTRO% ...
    wsl --install -d %DISTRO% --no-launch
    if errorlevel 1 (
        echo   [!] Could not install %DISTRO%. Run: wsl --install -d %DISTRO%
        goto :end
    )
    echo   [*] If Ubuntu asks for a username/password on first launch,
    echo       pick anything - then run setup.bat again.
)

rem ---- 3. .wslconfig: nested virtualization (KVM) + resources --
set "WSLCONF=%USERPROFILE%\.wslconfig"
set "NEED_RESTART=0"

if not exist "%WSLCONF%" (
    echo   [*] Writing %WSLCONF% ...
    (
        echo [wsl2]
        echo nestedVirtualization=true
        echo memory=20GB
        echo processors=12
    ) > "%WSLCONF%"
    set "NEED_RESTART=1"
) else (
    findstr /i /c:"nestedVirtualization=true" "%WSLCONF%" >nul 2>&1
    if errorlevel 1 (
        echo   [*] Enabling nested virtualization in existing %WSLCONF%
        echo       ^(backup at .wslconfig.clade-backup^) ...
        copy /y "%WSLCONF%" "%WSLCONF%.clade-backup" >nul
        findstr /i /c:"[wsl2]" "%WSLCONF%" >nul 2>&1
        if errorlevel 1 (
            (
                echo [wsl2]
                echo nestedVirtualization=true
            ) >> "%WSLCONF%"
        ) else (
            rem Append the key; last value under [wsl2] wins in WSL's parser
            echo nestedVirtualization=true>> "%WSLCONF%"
        )
        set "NEED_RESTART=1"
    )
)

if "!NEED_RESTART!"=="1" (
    echo   [*] Restarting WSL so the config applies ...
    wsl --shutdown
)

rem ---- 4. Hand off to Linux ------------------------------------
rem The repo this .bat lives in is copied INTO the WSL filesystem
rem (~/clade/Custom-OS) - builds on /mnt/c are painfully slow, and
rem copying needs no GitHub credentials. --cd starts bash inside
rem this folder via WSL's automount: no path conversion, no command
rem substitution, no escaped quotes for wsl.exe to mangle.
echo   [*] Copying the repo into WSL and running tools/setup.sh
echo       ^(installs packages, Rust, QEMU, Buildroot + verifies the
echo        dev loop - a few minutes on first run^) ...
echo.

wsl -d %DISTRO% --cd "%~dp0." -- bash -lc "set -e; if [ -e $HOME/clade ] && [ ! -O $HOME/clade ]; then echo '[fix] ~/clade was created by root in an earlier run - taking ownership'; chown -R $USER: $HOME/clade 2>/dev/null || sudo chown -R $USER: $HOME/clade; fi; if [ -e $HOME/clade/Custom-OS ] && [ ! -O $HOME/clade/Custom-OS ]; then echo '[fix] taking ownership of the WSL repo copy'; chown -R $USER: $HOME/clade 2>/dev/null || sudo chown -R $USER: $HOME/clade; fi; mkdir -p $HOME/clade/Custom-OS; cp -af . $HOME/clade/Custom-OS/; cd $HOME/clade/Custom-OS; sed -i 's/\r$//' tools/*.sh kernel/buildroot-external/board/clade/*.sh; exec bash tools/setup.sh"

if errorlevel 1 (
    echo.
    echo   [!] setup.sh reported a problem - scroll up for the failing step.
    echo       Re-running setup.bat is safe; completed steps are skipped.
    goto :end
)

echo.
echo   ============================================================
echo    Clade setup complete.
echo.
echo    Work in WSL from now on:
echo       wsl -d %DISTRO%
echo       cd ~/clade/Custom-OS
echo.
echo    Fast dev loop:     tools/dev-run.sh
echo    Build the OS image ^(~30-60 min, once^):  see kernel/README.md
echo    Boot it:           tools/qemu-run.sh ../buildroot/output/images
echo   ============================================================

:end
echo.
pause
endlocal
