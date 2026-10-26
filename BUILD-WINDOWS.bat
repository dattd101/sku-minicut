@echo off
setlocal
cd /d "%~dp0"
echo Building MiniCut for Windows 11...
where cargo >nul 2>nul || (echo Rust/Cargo not found. Install from https://rustup.rs/ & pause & exit /b 1)
cargo build --release
if errorlevel 1 (
  echo Build failed. Check error messages above.
  pause
  exit /b 1
)
echo Build complete: target\release\minicut.exe
pause
