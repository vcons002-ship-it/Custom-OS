@echo off
setlocal
rem ============================================================
rem  Clade - WSL diagnostics. Run this when the other .bat files
rem  (or wsl itself) sit silent. Nothing here changes your setup
rem  except restarting the WSL VM (step 5), which is safe.
rem  Paste the full output back into the Claude session.
rem ============================================================

echo === Clade doctor: WSL diagnostics ===
echo.

echo --- 1. WSL version ---
wsl --version
echo (exit code: %errorlevel%)
echo.

echo --- 2. WSL status ---
wsl --status
echo (exit code: %errorlevel%)
echo.

echo --- 3. Installed distros and their state ---
wsl --list --verbose
echo (exit code: %errorlevel%)
echo.

echo --- 4. .wslconfig contents ---
if exist "%USERPROFILE%\.wslconfig" (
    type "%USERPROFILE%\.wslconfig"
) else (
    echo (no .wslconfig file)
)
echo.

echo --- 5. Restarting the WSL VM (safe; takes ~10s) ---
wsl --shutdown
echo (exit code: %errorlevel%) waiting 10 seconds ...
timeout /t 10 /nobreak >nul
echo.

echo --- 6. Launching Ubuntu. If this line is the last thing you see
echo ---    for more than a minute, it is hung: press Ctrl+C, and that
echo ---    fact is itself the key diagnostic.
wsl -d Ubuntu-24.04 -- echo ok-from-ubuntu
echo (exit code: %errorlevel%)
echo.

echo === end of diagnostics - paste everything above back ===
echo.
pause
endlocal
