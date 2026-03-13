#!/bin/bash

# Listory Plus v2 Environment Initialization Script
# This script ensures the development environment is correctly set up.

set -e

echo "Checking environment for Listory Plus v2..."

# 1. Verify Administrator Privileges
echo "Checking for administrator privileges..."
if ! net session > /dev/null 2>&1; then
    echo "ERROR: This script must be run with administrator privileges."
    exit 1
fi
echo "Administrator privileges verified."

# 2. Check Rust Toolchain
echo "Checking Rust toolchain..."
if ! command -v rustc >/dev/null 2>&1 || ! command -v cargo >/dev/null 2>&1; then
    echo "Rust toolchain not found in PATH. Attempting to add default cargo bin path..."
    export PATH="$PATH:$HOME/.cargo/bin"
    if ! command -v rustc >/dev/null 2>&1 || ! command -v cargo >/dev/null 2>&1; then
        echo "ERROR: Rust toolchain (rustc, cargo) is not installed or not in PATH."
        echo "Please install Rust from https://rustup.rs/"
        exit 1
    fi
fi
echo "Rust toolchain found: $(rustc --version)"

# 3. Check Node.js and npm
echo "Checking Node.js and npm..."
if ! command -v node >/dev/null 2>&1 || ! command -v npm >/dev/null 2>&1; then
    echo "ERROR: Node.js or npm is not installed."
    exit 1
fi
echo "Node.js found: $(node --version)"
echo "npm found: $(npm --version)"

# 4. Check Tauri CLI
echo "Checking Tauri CLI..."
if ! cargo tauri --version >/dev/null 2>&1; then
    echo "Tauri CLI not found. Installing cargo-tauri..."
    cargo install tauri-cli
fi
echo "Tauri CLI found: $(cargo tauri --version)"

# 5. Check for .gemini/feature_list.json
if [ ! -f ".gemini/feature_list.json" ]; then
    echo "WARNING: .gemini/feature_list.json not found. Please ensure it is created."
fi

echo "Environment check passed successfully!"
