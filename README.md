<div align="right">
  <a href="README_zh.md">🇨🇳 简体中文</a>
</div>

<div align="center">
  <h1>Listory Plus</h1>
  <p><b>A lightning-fast file search engine for Windows, powered by Tauri and Rust.</b></p>

  <p>
    <img src="https://img.shields.io/badge/Tauri-2.0-24C8D8?logo=tauri&logoColor=white" alt="Tauri">
    <img src="https://img.shields.io/badge/Rust-1.75+-000000?logo=rust&logoColor=white" alt="Rust">
    <img src="https://img.shields.io/badge/React-18-61DAFB?logo=react&logoColor=black" alt="React">
    <img src="https://img.shields.io/badge/License-MIT-blue.svg" alt="License">
  </p>
</div>

![Demo](https://via.placeholder.com/800x450?text=App+Screenshot+Here)

## ✨ Features

- 🚀 **Concurrent MFT Scanning**: Rapidly indexes your entire NTFS drive in seconds.
- 🔄 **Real-time USN Monitoring**: Keeps the search index instantly up-to-date with file system changes.
- 🔍 **Spotlight-style UI**: A clean, keyboard-centric interface that stays out of your way.
- 🪶 **Minimal memory footprint**: Highly optimized Rust backend ensures low resource consumption.

## 📦 Installation

1. Go to the [Releases](../../releases) page.
2. Download the latest `.msi` or `.exe` installer.
3. Run the installer and follow the prompts.

## 🛠️ Development Setup

### Prerequisites

- [Rust](https://rustup.rs/) (1.75 or later)
- [Node.js](https://nodejs.org/) (18 or later)
- Windows SDK and MSVC (via Visual Studio Installer)

### Getting Started

1. Clone the repository:
   ```bash
   git clone https://github.com/yourusername/listory-plus.git
   cd listory-plus
   ```

2. Initialize the development environment (generates necessary keys):
   ```powershell
   .\scripts\init.ps1
   ```

3. Install frontend dependencies:
   ```bash
   npm install
   ```

4. Start the development server with Tauri:
   ```bash
   npm run tauri dev
   ```

## 📄 License

This project is licensed under the MIT License.
