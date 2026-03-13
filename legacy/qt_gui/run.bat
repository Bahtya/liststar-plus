@echo off
REM Run Listory Search Qt GUI
REM This script runs the application from the build directory

cd /d "%~dp0build-msvc\Release"

if not exist listory_search.exe (
    echo Error: listory_search.exe not found!
    echo Please build the project first using build-msvc.bat
    pause
    exit /b 1
)

echo Starting Listory Search...
echo.
start listory_search.exe
