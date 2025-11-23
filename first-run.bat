@echo off
setlocal enabledelayedexpansion

echo ========================================
echo Rust Development Environment Setup
echo ========================================
echo.

REM === Setup paths ===
set RUSTUP_INIT_URL=https://win.rustup.rs/x86_64
set RUSTUP_INIT_EXE=rustup-init.exe
set CARGO_HOME=%USERPROFILE%\.cargo
set RUSTUP_HOME=%USERPROFILE%\.rustup
set TEMP_DIR=%TEMP%\rust_install_temp

REM === Create temp directory ===
if not exist "%TEMP_DIR%" mkdir "%TEMP_DIR%"
cd /d "%TEMP_DIR%"

REM === Check if MSVC is available ===
echo Checking for MSVC (link.exe)...
where link.exe >nul 2>&1
if errorlevel 1 (
    echo ERROR: MSVC link.exe not found!
    echo Please install Visual Studio Build Tools or Visual Studio with C++ support.
    echo Download from: https://visualstudio.microsoft.com/downloads/
    pause
    exit /b 1
)
echo MSVC found!
echo.

REM === Install Rust with MSVC toolchain ===
if not exist "%CARGO_HOME%\bin\cargo.exe" (
    echo Downloading rustup-init...
    powershell -Command "Invoke-WebRequest -Uri '%RUSTUP_INIT_URL%' -OutFile '%RUSTUP_INIT_EXE%' -UseBasicParsing"
    if errorlevel 1 (
        echo Failed to download rustup-init
        pause
        exit /b 1
    )

    echo Installing Rust (MSVC toolchain)...
    echo This will add Cargo to your PATH automatically.
    %RUSTUP_INIT_EXE% -y --default-toolchain stable-msvc --default-host x86_64-pc-windows-msvc
    if errorlevel 1 (
        echo Failed to install Rust
        pause
        exit /b 1
    )

    echo.
    echo Rust installed successfully!
    echo Please close and reopen this terminal for PATH changes to take effect.
    echo Then run this script again to continue.
    pause
    exit /b 0
) else (
    echo Rust already installed, updating...
    call "%CARGO_HOME%\bin\rustup.exe" self update
    call "%CARGO_HOME%\bin\rustup.exe" update stable-msvc
    if errorlevel 1 (
        echo Warning: Update failed, but continuing...
    )
)

REM === Verify Cargo is accessible ===
call "%CARGO_HOME%\bin\cargo.exe" --version >nul 2>&1
if errorlevel 1 (
    echo ERROR: Cargo not found in PATH!
    echo Please close and reopen this terminal, then run this script again.
    pause
    exit /b 1
)

REM === Build project ===
cd /d "%~dp0"
echo.
echo Installing Rust tools (this may take a while)...
call "%CARGO_HOME%\bin\cargo.exe" install cargo-generate espflash ravedude
if errorlevel 1 (
    echo Warning: Some tools may have failed to install, but continuing...
)

echo.
echo Checking project...
call "%CARGO_HOME%\bin\cargo.exe" check --release
if errorlevel 1 (
    echo Project check failed
    pause
    exit /b 1
)

echo.
echo Building project...
call "%CARGO_HOME%\bin\cargo.exe" build --release
if errorlevel 1 (
    echo Build failed
    pause
    exit /b 1
)

echo.
echo Running project...
call "%CARGO_HOME%\bin\cargo.exe" run --release
if errorlevel 1 (
    echo Run failed
    pause
    exit /b 1
)

echo.
echo ========================================
echo Setup Complete!
echo ========================================
pause
