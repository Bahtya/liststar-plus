<div align="right">
  <a href="README.md">🇺🇸 English</a>
</div>

<div align="center">

# 🔍 Listory Plus

**Windows 极速文件搜索引擎**

*毫秒级定位 NTFS 磁盘上的任何文件*

[![Tauri](https://img.shields.io/badge/Tauri-2.0-24C8D8?logo=tauri&logoColor=white)](https://tauri.app)
[![Rust](https://img.shields.io/badge/Rust-1.75+-000000?logo=rust&logoColor=white)](https://www.rust-lang.org)
[![React](https://img.shields.io/badge/React-18-61DAFB?logo=react&logoColor=black)](https://react.dev)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Downloads](https://img.shields.io/github/downloads/Bahtya/liststar-plus/total)](https://github.com/Bahtya/liststar-plus/releases)

</div>

---

## ⚡ 为什么选择 Listory Plus？

| 特性 | Listory Plus | Windows 搜索 | Everything |
|------|-------------|-------------|------------|
| 冷启动扫描 (100万文件) | ~3秒 | 数分钟 | ~2秒 |
| 实时更新 | ✅ USN 日志 | ✅ | ✅ |
| 内存占用 | < 30MB | 高 | < 10MB |
| 现代界面 | ✅ 原生体验 | ❌ | ❌ |
| 文件内容搜索 | ✅ | ✅ | ❌ |

> 🚀 **并发 MFT 扫描** — 直接读取 NTFS 主文件表，秒级索引百万文件。
> 
> 🔄 **实时 USN 监控** — 通过 USN 变更日志实时跟踪文件变动，零延迟。
> 
> 🔍 **内容搜索** — 不仅搜文件名，还能搜索文件内容。
> 
> 🪶 **极低资源占用** — Rust + Tauri 构建，无 Electron 负担。

## 📦 安装

前往 [**Releases 页面**](https://github.com/Bahtya/liststar-plus/releases/latest) 下载最新版本：

- **`.msi`** — 推荐大多数用户使用（支持自动更新）
- **`.exe`** — 便携安装包

## 🛠️ 从源码构建

### 前置要求

- [Rust](https://rustup.rs/) 1.75+
- [Node.js](https://nodejs.org/) 18+
- Windows SDK + MSVC（通过 Visual Studio Installer 安装）

### 快速开始

```bash
git clone https://github.com/Bahtya/liststar-plus.git
cd liststar-plus

# 初始化开发环境
.\scripts\init.ps1

# 安装依赖
npm install

# 启动开发服务器
npm run tauri dev
```

## 🗺️ 路线图

- [x] 基于 MFT 的文件索引
- [x] USN 日志实时监控
- [x] Spotlight 风格搜索界面
- [x] 文件内容搜索
- [ ] 插件系统
- [ ] Scoop / Winget 包支持
- [ ] 正则搜索模式
- [ ] 文件预览面板
- [ ] 多语言支持

## 🤝 参与贡献

欢迎贡献！你可以：

- 🐛 [报告 Bug](https://github.com/Bahtya/liststar-plus/issues/new?template=bug_report.md)
- 💡 [提出功能建议](https://github.com/Bahtya/liststar-plus/issues/new?template=feature_request.md)
- 🔀 提交 Pull Request

## 📄 许可证

本项目基于 [MIT 许可证](LICENSE) 开源。

---

<div align="center">

**觉得有用的话，给个 ⭐ 吧！**

</div>
