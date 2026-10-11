@echo off
setlocal
cd /d "%~dp0"
title Remember Me? - Windows setup and build

where winget >nul 2>&1
if errorlevel 1 (
  echo winget not found. Install "App Installer" from the Microsoft Store, then run this again.
  pause
  exit /b 1
)

rem Make freshly installed tools visible in this window
set "PATH=%PATH%;%ProgramFiles%\nodejs;%USERPROFILE%\.cargo\bin"

echo.
echo [1/6] Node.js
where node >nul 2>&1
if errorlevel 1 winget install -e --id OpenJS.NodeJS.LTS --accept-source-agreements --accept-package-agreements

echo.
echo [2/6] Rust
where rustup >nul 2>&1
if errorlevel 1 winget install -e --id Rustlang.Rustup --accept-source-agreements --accept-package-agreements
set "PATH=%PATH%;%USERPROFILE%\.cargo\bin"

echo.
echo [3/6] Visual Studio Build Tools (C++)  - big download, accept the admin prompt
set "VSW=%ProgramFiles(x86)%\Microsoft Visual Studio\Installer\vswhere.exe"
set "HAVEVS="
if exist "%VSW%" for /f "usebackq delims=" %%i in (`"%VSW%" -products * -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath`) do set "HAVEVS=%%i"
if not defined HAVEVS winget install -e --id Microsoft.VisualStudio.2022.BuildTools --accept-source-agreements --accept-package-agreements --override "--wait --passive --norestart --add Microsoft.VisualStudio.Workload.VCTools --includeRecommended"

echo.
echo [4/6] WebView2 runtime
winget install -e --id Microsoft.EdgeWebView2Runtime --accept-source-agreements --accept-package-agreements

echo.
echo [5/6] Rust toolchain and npm packages
call rustup default stable-x86_64-pc-windows-msvc
if errorlevel 1 goto fail
call npm install
if errorlevel 1 goto fail
if not exist "src-tauri\icons\icon.ico" call npx tauri icon app-icon.png
if errorlevel 1 goto fail

echo.
echo [6/6] Building Remember Me?  (first build takes 10-20 minutes)
call npm run tauri build
if errorlevel 1 goto fail

set "FOUND="
for %%f in ("src-tauri\target\release\bundle\nsis\*-setup.exe") do (
  copy /y "%%f" "%~dp0Remember-Me-Setup.exe" >nul
  set "FOUND=1"
)
if not defined FOUND goto fail

echo.
echo DONE. Installer: %~dp0Remember-Me-Setup.exe
explorer /select,"%~dp0Remember-Me-Setup.exe"
pause
exit /b 0

:fail
echo.
echo Something failed. Scroll up for the first error, or copy it to Claude.
echo If tools were just installed, close this window and run the file again.
pause
exit /b 1
