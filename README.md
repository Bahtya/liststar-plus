<div align="right">
  <a href="README_zh.md">🇨🇳 简体中文</a>
</div>

<div align="center">

# 🔍 Listory Plus

**A lightning-fast file search engine for Windows**

*Find any file on your NTFS drive in milliseconds — not seconds.*

[![Tauri](https://img.shields.io/badge/Tauri-2.0-24C8D8?logo=tauri&logoColor=white)](https://tauri.app)
[![Rust](https://img.shields.io/badge/Rust-1.75+-000000?logo=rust&logoColor=white)](https://www.rust-lang.org)
[![React](https://img.shields.io/badge/React-18-61DAFB?logo=react&logoColor=black)](https://react.dev)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Downloads](https://img.shields.io/github/downloads/Bahtya/liststar-plus/total)](https://github.com/Bahtya/liststar-plus/releases)

</div>

---

## ⚡ Why Listory Plus?

| Feature | Listory Plus | Windows Search | Everything |
|---------|-------------|----------------|------------|
| Cold scan (1M files) | ~3s | Minutes | ~2s |
| Real-time updates | ✅ USN Journal | ✅ | ✅ |
| Memory usage | < 30MB | High | < 10MB |
| Modern UI | ✅ Native | ❌ | ❌ |
| Content search | ✅ | ✅ | ❌ |

> 🚀 **Concurrent MFT Scanning** — Reads the NTFS Master File Table directly, indexing millions of files in seconds.
> 
> 🔄 **Real-time USN Monitoring** — Watches file system changes via the USN Change Journal. No polling, no delays.
> 
> 🔍 **Content Search** — Search inside files, not just filenames.
> 
> 🪶 **Minimal Footprint** — Built with Rust + Tauri. No Electron, no bloat.

## 📸 Screenshots

> 📝 *Screenshots coming soon — UI is under active development!*

## 📦 Installation

### Download

Grab the latest installer from the [**Releases page**](https://github.com/Bahtya/liststar-plus/releases/latest):

- **`.msi`** — Recommended for most users (auto-updates)
- **`.exe`** — Portable installer

### Winget (coming soon)

```bash
winget install Bahtya.ListoryPlus
```

### Scoop (coming soon)

```bash
scoop install listory-plus
```

## 🛠️ Build from Source

### Prerequisites

- [Rust](https://rustup.rs/) 1.75+
- [Node.js](https://nodejs.org/) 18+
- Windows SDK + MSVC (via Visual Studio Installer)

### Quick Start

```bash
git clone https://github.com/Bahtya/liststar-plus.git
cd liststar-plus

# Initialize dev environment
.\scripts\init.ps1

# Install dependencies
npm install

# Run in dev mode
npm run tauri dev
```

## 🗺️ Roadmap

- [x] MFT-based file indexing
- [x] USN Journal real-time monitoring
- [x] Spotlight-style search UI
- [x] Content search
- [ ] Plugin system for custom search providers
- [ ] Scoop / Winget package support
- [ ] Regex search mode
- [ ] File preview panel
- [ ] Multi-language support

## 🤝 Contributing

Contributions are welcome! Feel free to:

- 🐛 [Report bugs](https://github.com/Bahtya/liststar-plus/issues/new?template=bug_report.md)
- 💡 [Request features](https://github.com/Bahtya/liststar-plus/issues/new?template=feature_request.md)
- 🔀 Submit pull requests

## 📄 License

This project is licensed under the [MIT License](LICENSE).

---

<div align="center">

**If you find this project useful, please consider giving it a ⭐!**

</div>
