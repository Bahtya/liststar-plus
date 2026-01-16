@echo off
REM Build script for Qt GUI with MSVC
REM This script configures and builds the project using Visual Studio 2022

echo ========================================
echo Building Listory Search Qt GUI (MSVC)
echo ========================================
echo.

REM Create build directory if it doesn't exist
if not exist build-msvc mkdir build-msvc
cd build-msvc

echo [1/3] Configuring with CMake...
C:\Qt\Tools\CMake_64\bin\cmake.exe .. ^
  -DCMAKE_PREFIX_PATH=C:/Qt/6.10.1/msvc2022_64 ^
  -DCMAKE_TOOLCHAIN_FILE=D:/Project/vcpkg/scripts/buildsystems/vcpkg.cmake ^
  -G "Visual Studio 17 2022" ^
  -A x64

if %ERRORLEVEL% NEQ 0 (
    echo.
    echo [ERROR] CMake configuration failed!
    cd ..
    pause
    exit /b 1
)

echo.
echo [2/3] Building Release version...
C:\Qt\Tools\CMake_64\bin\cmake.exe --build . --config Release

if %ERRORLEVEL% NEQ 0 (
    echo.
    echo [ERROR] Build failed!
    cd ..
    pause
    exit /b 1
)

echo.
echo [3/3] Deploying Qt dependencies...
cd Release
C:\Qt\6.10.1\msvc2022_64\bin\windeployqt.exe listory_search.exe --release --no-translations

if %ERRORLEVEL% NEQ 0 (
    echo.
    echo [WARNING] windeployqt failed, but build is complete
)

cd ..\..

echo.
echo ========================================
echo Build completed successfully!
echo ========================================
echo.
echo Executable location:
echo   build-msvc\Release\listory_search.exe
echo.
echo To run the application:
echo   cd build-msvc\Release
echo   listory_search.exe
echo.
pause
