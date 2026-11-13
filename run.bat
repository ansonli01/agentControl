@echo off
setlocal
cd /d "%~dp0."

set "APP_NAME=AgentControl"
set "TAURI_DIR=%~dp0src-tauri"

cls
echo ============================================================
echo   AgentControl - Multi-Agent Controller
echo   Build / Run / Test helper
echo ============================================================
echo.

where node >nul 2>nul
if not errorlevel 1 goto menu
echo   [ERROR] node not found in PATH, please install Node.js 20+
pause
exit /b 1

:menu
echo.
echo   [1] Dev mode            - npm run tauri:dev
echo   [2] Build frontend      - npm run build
echo   [3] Bundle installer    - npm run tauri:build
echo   [4] Run backend tests   - cargo test
echo   [5] Detect local agents - cargo run --example detect
echo   [6] Regenerate icons    - scripts\generate-icons.ps1
echo   [7] Release build + run - build + cargo build --release + start
echo   [0] Quit
echo.
set "choice="
set /p choice=Choose an option and press Enter:
if "%choice%"=="" set "choice=0"
if "%choice%"=="1" goto dev
if "%choice%"=="2" goto build_web
if "%choice%"=="3" goto bundle
if "%choice%"=="4" goto test
if "%choice%"=="5" goto detect
if "%choice%"=="6" goto icons
if "%choice%"=="7" goto release
if "%choice%"=="0" goto end
echo   Invalid input, please try again.
goto menu

:dev
echo.
echo   Starting dev mode, close the app window to stop...
call npm run tauri:dev
goto menu

:build_web
echo.
echo   Building frontend with vue-tsc and vite...
call npm run build
goto pauseback

:bundle
echo.
echo   Bundling Windows installer, needs network for wix / NSIS...
call npm run tauri:build
goto pauseback

:test
echo.
echo   Running backend tests...
cd /d "%TAURI_DIR%"
call cargo test
cd /d "%~dp0."
goto pauseback

:detect
echo.
echo   Detecting agents installed on this machine...
cd /d "%TAURI_DIR%"
call cargo run --example detect
cd /d "%~dp0."
goto pauseback

:icons
echo.
echo   Regenerating application icons...
powershell -ExecutionPolicy Bypass -File "scripts\generate-icons.ps1"
goto pauseback

:release
echo.
echo   Step 1/3 build frontend...
call npm run build
if errorlevel 1 goto pauseback
echo   Step 2/3 compile release...
cd /d "%TAURI_DIR%"
call cargo build --release --features custom-protocol
cd /d "%~dp0."
if errorlevel 1 goto pauseback
echo   Step 3/3 launch application...
start "AgentControl" "%TAURI_DIR%\target\release\agentcontrol.exe"
goto pauseback

:pauseback
echo.
pause
goto menu

:end
echo.
echo   Bye.
endlocal