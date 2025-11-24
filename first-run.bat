@echo off
setlocal enabledelayedexpansion

echo Iron Coder installation wizard
echo.

REM variables
set "RUSTUP_HOME=%USERPROFILE%\.rustup"
set "CARGO_HOME=%USERPROFILE%\.cargo"
set "TEMP_DIR=%TEMP%\rust_setup_temp"
set "VSWHERE=%ProgramFiles(x86)%\Microsoft Visual Studio\Installer\vswhere.exe"
set "BUILD_TOOLS_URL=https://aka.ms/vs/17/release/vs_BuildTools.exe"
set "BUILD_TOOLS_INSTALLER=%TEMP_DIR%\vs_BuildTools.exe"

if not exist "%TEMP_DIR%" mkdir "%TEMP_DIR%"

echo [1/4] Checking for Visual Studio 2022 Build Tools

REM detect if MSVC toolchain is available
if not exist "%VSWHERE%" (
    echo Visual Studio/Build Tools not found.
    goto :InstallVS
)

for /f "usebackq tokens=*" %%i in (`"%VSWHERE%" -products * -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -format value -property installationPath`) do (
    if exist "%%i\VC\Tools\MSVC" (
        echo   Found MSVC toolchain: %%i
        goto :VSFound
    )
)

echo No MSVC build tools found.

:InstallVS
echo.
echo Visual Studio Build Tools (C++) are required for Rust on Windows (msvc target).
echo We can download and install them automatically.
echo.
choice /c YN /n /m "Download and install Visual Studio 2022 Build Tools now? [Y/N]"
if errorlevel 2 goto :SkipVS
if errorlevel 1 goto :DoInstallVS

:DoInstallVS
echo.
echo Downloading Visual Studio Build Tools installer
powershell -Command "Invoke-WebRequest -Uri '%BUILD_TOOLS_URL%' -OutFile '%BUILD_TOOLS_INSTALLER%' -UseBasicParsing"
if errorlevel 1 (
    echo Failed to download Visual Studio Build Tools.
    echo Please download manually from:
    echo https://visualstudio.microsoft.com/downloads/ ^(Community ^> 'Build Tools'^)
    pause
    exit /b 1
)

echo Installing Visual Studio Build Tools
echo Please click through the installer, ensure these are selected:
echo   - "C++ build tools"
start "" /wait "%BUILD_TOOLS_INSTALLER%" --quiet --norestart --add Microsoft.VisualStudio.Component.VC.Tools.x86.x64 --add Microsoft.VisualStudio.Component.Windows10SDK

if errorlevel 1 (
    echo Installation may have failed or been cancelled.
    pause
    exit / occupant 1
)

echo Visual Studio Build Tools installed successfully!
echo.
echo Please restart this script to continue.
pause
exit /b 0

:SkipVS
echo Skipping Visual Studio installation. Rust may fail to compile.
echo.
timeout /t 3 >nul

:VSFound
echo Visual C++ build tools are available.
echo.

REM === Install/Update Rust ===
echo [2/4] Installing or updating Rust (stable-msvc)
if not exist "%CARGO_HOME%\bin\rustup.exe" (
    echo Downloading rustup-init.exe
    powershell -Command "Invoke-WebRequest -Uri https://win.rustup.rs/x86_64 -OutFile \"%TEMP_DIR%\rustup-init.exe\" -UseBasicParsing"
    echo Installing Rust
    "%TEMP_DIR%\rustup-init.exe" -y --default-toolchain stable-msvc
    if errorlevel 1 (
        echo Rust installation failed!
        pause
        exit /b 1
    )
    echo Rust installed successfully!
) else (
    echo Updating Rust toolchains
    call "%CARGO_HOME%\bin\rustup.exe" update stable-msvc
    call "%CARGO_HOME%\bin\rustup.exe" self update
)

REM Add Cargo to PATH for this session
set "PATH=%CARGO_HOME%\bin;%PATH%"

echo.
echo [3/4] Installing useful Cargo tools
rustup target add thumbv6m-none-eabi
rustup target add riscv32imac-unknown-none-elf
cargo install cargo-generate espflash ravedude probe-rs-tools || echo Some tools failed to install

echo.
echo [4/4] Building your project
cd /d "%~dp0"

echo Checking project
cargo check --release || pause

echo Building release binary
cargo build --release
if errorlevel 1 (
    echo Build failed!
    pause
    exit /b 1
)

echo.
echo   SUCCESS! Everything is set up.
echo   cargo run --release will open the IDE
echo.
pause
