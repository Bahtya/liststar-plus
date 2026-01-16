@echo off
REM Qt GUI Configuration Script
REM This script configures the build with vcpkg

echo Configuring Qt GUI with vcpkg...

REM Create build directory if it doesn't exist
if not exist build-manual mkdir build-manual
cd build-manual

REM Configure with CMake
C:\Qt\Tools\CMake_64\bin\cmake .. ^
  -DCMAKE_TOOLCHAIN_FILE=D:/Project/vcpkg/scripts/buildsystems/vcpkg.cmake ^
  -G "MinGW Makefiles"

if %ERRORLEVEL% EQU 0 (
    echo.
    echo Configuration successful!
    echo To build, run: cmake --build .
    echo Or open the project in Qt Creator
) else (
    echo.
    echo Configuration failed! Check the error messages above.
)

cd ..
pause
