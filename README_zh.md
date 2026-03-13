<div align="right">
  <a href="README.md">🇺🇸 English</a>
</div>

<div align="center">
  <h1>Listory Plus</h1>
  <p><b>一款由 Tauri 和 Rust 驱动的 Windows 极速文件搜索引擎。</b></p>

  <p>
    <img src="https://img.shields.io/badge/Tauri-2.0-24C8D8?logo=tauri&logoColor=white" alt="Tauri">
    <img src="https://img.shields.io/badge/Rust-1.75+-000000?logo=rust&logoColor=white" alt="Rust">
    <img src="https://img.shields.io/badge/React-18-61DAFB?logo=react&logoColor=black" alt="React">
    <img src="https://img.shields.io/badge/License-MIT-blue.svg" alt="License">
  </p>
</div>

![Demo](https://via.placeholder.com/800x450?text=App+Screenshot+Here)

## ✨ 核心特性

- 🚀 **并发 MFT 扫描**：在几秒钟内快速索引整个 NTFS 驱动器。
- 🔄 **实时 USN 监控**：使搜索索引与文件系统更改保持即时同步。
- 🔍 **Spotlight 风格 UI**：简洁、以键盘为中心的界面，不干扰您的工作流。
- 🪶 **极低内存占用**：高度优化的 Rust 后端确保极低的资源消耗。

## 📦 安装指南

1. 访问 [Releases](../../releases) 页面。
2. 下载最新的 `.msi` 或 `.exe` 安装程序。
3. 运行安装程序并按照提示操作。

## 🛠️ 本地开发指南

### 前置要求

- [Rust](https://rustup.rs/) (1.75 或更高版本)
- [Node.js](https://nodejs.org/) (18 或更高版本)
- Windows SDK 和 MSVC (通过 Visual Studio Installer 安装)

### 快速开始

1. 克隆仓库：
   ```bash
   git clone https://github.com/yourusername/listory-plus.git
   cd listory-plus
   ```

2. 初始化开发环境（生成必要的密钥等）：
   ```powershell
   .\scripts\init.ps1
   ```

3. 安装前端依赖：
   ```bash
   npm install
   ```

4. 启动 Tauri 开发服务器：
   ```bash
   npm run tauri dev
   ```

## 📄 许可证

本项目采用 MIT 许可证。
