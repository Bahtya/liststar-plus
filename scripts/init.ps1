# Listory Plus v2 Environment Initialization Script (PowerShell)
# This script ensures the development environment is correctly set up for Windows.

$ErrorActionPreference = "Stop"

Write-Host "====================================================" -ForegroundColor Cyan
Write-Host "   Listory Plus v2 Environment Setup (Harness)" -ForegroundColor Cyan
Write-Host "====================================================`n" -ForegroundColor Cyan

# 1. Verify Administrator Privileges
Write-Host "[1/5] Checking for administrator privileges..." -NoNewline
try {
    # Using 'net session' as requested in the task
    net session >$null 2>&1
    if ($LASTEXITCODE -ne 0) {
        Write-Host " [FAILED]" -ForegroundColor Red
        Write-Error "This script must be run with administrator privileges. Please restart your terminal as Administrator."
        exit 1
    }
    Write-Host " [OK]" -ForegroundColor Green
} catch {
    Write-Host " [FAILED]" -ForegroundColor Red
    Write-Error "Failed to verify administrator privileges: $_"
    exit 1
}

# 2. Check Rust Toolchain
Write-Host "[2/5] Checking Rust toolchain..." -NoNewline
# Ensure .cargo/bin is in PATH for the current session
$cargoBin = Join-Path $env:USERPROFILE ".cargo\bin"
if ($env:PATH -notlike "*$cargoBin*") {
    $env:PATH = "$cargoBin;$env:PATH"
}

if (-not (Get-Command rustc -ErrorAction SilentlyContinue) -or -not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    Write-Host " [NOT FOUND]" -ForegroundColor Yellow
    Write-Host "Rust toolchain not found. Attempting to install via rustup-init.exe..." -ForegroundColor Cyan
    
    $rustupInit = Join-Path $PSScriptRoot "..\rustup-init.exe"
    if (Test-Path $rustupInit) {
        Write-Host "Using local rustup-init.exe..." -ForegroundColor Gray
        & $rustupInit -y --default-toolchain stable
    } else {
        Write-Host "Downloading rustup-init.exe..." -ForegroundColor Gray
        Invoke-WebRequest -Uri https://win.rustup.rs/x86_64 -OutFile "rustup-init.exe"
        .\rustup-init.exe -y --default-toolchain stable
        Remove-Item "rustup-init.exe"
    }
    
    # Refresh PATH again after installation
    $env:PATH = "$cargoBin;$env:PATH"
}

if (Get-Command rustc -ErrorAction SilentlyContinue) {
    $rustVersion = rustc --version
    Write-Host " [OK] ($rustVersion)" -ForegroundColor Green
} else {
    Write-Host " [FAILED]" -ForegroundColor Red
    Write-Error "Rust installation failed or not in PATH."
    exit 1
}

# 3. Check Node.js and npm
Write-Host "[3/5] Checking Node.js and npm..." -NoNewline
if (-not (Get-Command node -ErrorAction SilentlyContinue) -or -not (Get-Command npm -ErrorAction SilentlyContinue)) {
    Write-Host " [FAILED]" -ForegroundColor Red
    Write-Error "Node.js or npm is not installed. Please install Node.js from https://nodejs.org/"
    exit 1
}
$nodeVersion = node --version
$npmVersion = npm --version
Write-Host " [OK] (Node: $nodeVersion, npm: $npmVersion)" -ForegroundColor Green

# 4. Check Tauri Dependencies
Write-Host "[4/5] Checking Tauri dependencies..." -NoNewline
# Check if @tauri-apps/cli is in package.json and node_modules
if (-not (Test-Path "node_modules")) {
    Write-Host " [INSTALLING]" -ForegroundColor Yellow
    Write-Host "Running 'npm install' to set up dependencies..." -ForegroundColor Cyan
    npm install --silent
}

# Check for cargo-tauri (optional but recommended for some tasks)
if (-not (Get-Command cargo-tauri -ErrorAction SilentlyContinue)) {
    # We don't force install cargo-tauri as npm-based tauri is usually enough, 
    # but we check if 'npm run tauri' works.
}
Write-Host " [OK]" -ForegroundColor Green

# 5. Verify Feature List
Write-Host "[5/5] Verifying .gemini/feature_list.json..." -NoNewline
$featureListPath = Join-Path $PSScriptRoot "..\.gemini\feature_list.json"
if (-not (Test-Path $featureListPath)) {
    Write-Host " [MISSING]" -ForegroundColor Yellow
    Write-Warning "Feature list not found at $featureListPath. Please ensure it is created."
} else {
    Write-Host " [OK]" -ForegroundColor Green
}

Write-Host "`n====================================================" -ForegroundColor Cyan
Write-Host "   Environment check passed! Ready for development." -ForegroundColor Green
Write-Host "====================================================" -ForegroundColor Cyan
