@echo off
setlocal
title Build Trading System
color 0A

echo ============================================
echo  BUILD SYSTEM
echo ============================================
echo.

set ROOT=%CD%
set TOOLS=%ROOT%\.tools
set XWIN=%TOOLS%\xwin
set CLANG=%TOOLS%\clang
set DIST=%ROOT%\dist

:: Check Cargo.toml exists
if not exist "%ROOT%\Cargo.toml" (
    color 0C
    echo ERROR: Cargo.toml not found!
    echo Please create it first with the package configuration.
    pause
    exit /b 1
)

:: Step 1: Check Rust
echo [1/6] Checking Rust...
where rustc >nul 2>&1
if errorlevel 1 (
    echo ERROR: Rust not found
    pause
    exit /b 1
)
echo OK

:: Step 2: Get xwin
echo.
echo [2/6] Checking xwin...
if not exist "%TOOLS%" mkdir "%TOOLS%"
if not exist "%XWIN%" mkdir "%XWIN%"

if exist "%XWIN%\xwin.exe" goto has_xwin

echo Downloading xwin...
curl -L -o "%XWIN%\xwin.tgz" "https://github.com/Jake-Shadle/xwin/releases/download/0.5.0/xwin-0.5.0-x86_64-pc-windows-msvc.tar.gz"
tar -xzf "%XWIN%\xwin.tgz" -C "%XWIN%" --strip-components=1
del "%XWIN%\xwin.tgz"

:has_xwin
echo OK

:: Step 3: Get SDK
echo.
echo [3/6] Checking Windows SDK...
:: Note: --output sdk creates nested sdk\sdk structure, so check for sdk\sdk\lib...
if exist "%XWIN%\sdk\sdk\lib\um\x86_64\kernel32.lib" goto has_sdk

echo Downloading SDK...
cd "%XWIN%"
xwin.exe --accept-license download
xwin.exe --accept-license splat --output sdk
cd "%ROOT%"

:has_sdk
echo OK

:: Step 4: Get portable LLVM/Clang
echo.
echo [4/6] Checking LLVM/Clang...

:: Check if we already have it
if exist "%CLANG%\bin\clang-cl.exe" goto has_clang

if not exist "%CLANG%" mkdir "%CLANG%"

:: Download 7z portable (if not already have it)
if not exist "%TOOLS%\7z.exe" (
    echo Downloading 7z unpacker...
    mkdir "%TOOLS%\7ztemp" 2>nul
    curl -L -o "%TOOLS%\7z-install.exe" "https://www.7-zip.org/a/7z2408-x64.exe"
    :: Silently install 7z to temp location
    "%TOOLS%\7z-install.exe" /S /D=%TOOLS%\7ztemp
    :: Copy just the files we need
    copy "%TOOLS%\7ztemp\7z.exe" "%TOOLS%\7z.exe" >nul
    copy "%TOOLS%\7ztemp\7z.dll" "%TOOLS%\7z.dll" >nul
    :: Clean up
    rmdir /S /Q "%TOOLS%\7ztemp" 2>nul
    del "%TOOLS%\7z-install.exe" 2>nul
)

:: Download LLVM installer (which is a 7z SFX archive)
echo Downloading LLVM 17.0.6...
curl -L -o "%CLANG%\llvm.7z.exe" "https://github.com/llvm/llvm-project/releases/download/llvmorg-17.0.6/LLVM-17.0.6-win64.exe"

:: Check download size
for %%I in ("%CLANG%\llvm.7z.exe") do set size=%%~zI
if %size% LSS 1000000 (
    echo ERROR: LLVM download failed ^(%size% bytes^)
    del "%CLANG%\llvm.7z.exe" 2>nul
    pause
    exit /b 1
)

:: Extract LLVM using 7z (the EXE is a self-extracting 7z archive)
echo Extracting LLVM with 7z...
"%TOOLS%\7z.exe" x "%CLANG%\llvm.7z.exe" -o"%CLANG%" -y >nul

:: Move files from nested dir if needed
for /d %%D in ("%CLANG%\LLVM-*") do (
    xcopy /E /I /Y "%%D\*" "%CLANG%\" >nul
    rmdir /S /Q "%%D" 2>nul
)

:: Clean up
del "%CLANG%\llvm.7z.exe" 2>nul

:: Verify
if not exist "%CLANG%\bin\clang-cl.exe" (
    echo ERROR: Failed to extract LLVM
    pause
    exit /b 1
)

:has_clang
echo OK - using portable clang
echo Adding %CLANG%\bin to PATH
set "PATH=%CLANG%\bin;%PATH%"
set "CC=clang-cl"
set "CXX=clang-cl"

:: Step 5: Setup environment
echo.
echo [5/6] Configuring...

:: Setup cargo to use LLVM linker
if not exist .cargo mkdir .cargo
echo [target.x86_64-pc-windows-msvc] > .cargo\config.toml
echo linker = "lld-link" >> .cargo\config.toml
echo ar = "llvm-ar" >> .cargo\config.toml

:: Setup SDK paths for clang
set INCLUDE=%XWIN%\sdk\include\ucrt;%XWIN%\sdk\include\um;%XWIN%\sdk\include\shared
set LIB=%XWIN%\sdk\lib\ucrt\x64;%XWIN%\sdk\lib\um\x64

:: Fix Python environment for PyO3
set PYTHONHOME=
for /f "usebackq delims=" %%a in (`python -c "import sys; print(sys.executable)"`) do set PYO3_PYTHON=%%a
echo Using Python: %PYO3_PYTHON%

:: Step 6: Build
echo.
echo [6/6] Building...
rustup target add x86_64-pc-windows-msvc

:: Build with Python environment set
cargo build --release

if errorlevel 1 (
    color 0C
    echo.
    echo BUILD FAILED
    echo.
    echo Try installing Visual Studio Build Tools:
    echo https://aka.ms/vs/17/release/vs_BuildTools.exe 
    echo Select: Desktop development with C++
    pause
    exit /b 1
)

:: Package
echo.
echo Packaging...
if not exist "%DIST%" mkdir "%DIST%"
copy "target\release\LocoHFT.exe" "%DIST%\"

:: Get Python DLL
for /f "usebackq delims=" %%a in (`python -c "import sys; print(sys.executable)"`) do set PY=%%a
for %%F in ("%PY%") do set PYDIR=%%~dpF
copy "%PYDIR%python3*.dll" "%DIST%\" 2>nul

echo @echo off> "%DIST%\run.bat"
echo LocoHFT.exe>> "%DIST%\run.bat"
echo pause>> "%DIST%\run.bat"

color 0A
echo.
echo ============================================
echo  BUILD SUCCESS
echo ============================================
echo Output: %DIST%\LocoHFT.exe
pause
