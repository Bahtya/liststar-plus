# Listory Plus v2 自动化打包与 Release 设计文档

## 1. 问题陈述
Listory Plus 目前已完成 Tauri v2 重构，但尚未建立标准化的发布流程。手动打包效率低下且易出错。为了对齐 `clash-verge-rev` 的工程标准，我们需要一套基于 GitHub Actions 的 CI/CD 系统，实现推送 Tag 自动完成 Windows 平台的打包、签名及 GitHub Release 发布。

## 2. 需求说明

### 功能性需求
- **自动化构建**：推送 `v*` 标签时自动触发。
- **多格式支持**：同时提供 `.msi` (Wix) 和 `.exe` (NSIS) 安装包。
- **自动更新**：支持 Tauri 内置的自动更新机制。
- **数字签名**：集成 Tauri 签名私钥。
- **Release 管理**：自动创建 GitHub Release Draft 并上传所有二进制资源。

### 非功能性需求
- **环境一致性**：使用 `windows-latest` 干净环境。
- **安全性**：敏感密钥通过 GitHub Secrets 管理。
- **稳定性**：错误时自动重试。

### 约束
- **平台**：仅限 Windows (x64)。
- **技术栈**：GitHub Actions, Tauri 2.0, Rust, Node.js 20。

## 3. 架构方案 (Approach 1: Windows-Native Automation)
我们将构建一个专用的 Windows 打包流程：

- **CI 环境**：GitHub Actions `windows-latest`。
- **核心工具**：`tauri-apps/tauri-action@v0`。
- **触发器**：`on: push: tags: - 'v*'`。
- **签名机制**：集成 `TAURI_SIGNING_PRIVATE_KEY`。

## 4. 协作流程与 GitHub Secrets
- **GITHUB_TOKEN**：用于操作 Release (自动分配)。
- **TAURI_SIGNING_PRIVATE_KEY**：用于包签名。
- **TAURI_SIGNING_PASSWORD**：签名密码。

## 5. 风险评估
- **构建时长**：Windows 环境初始化较慢，计划通过 `cache` 优化。
- **密钥泄露风险**：严格通过 GitHub Secrets 管理。
- **版本冲突**：强制要求推送 Tag 与 `tauri.conf.json` 版本一致。

## 6. 成功标准
- 推送 `v1.0.0` 标签后，自动在 GitHub 创建对应的 Release Draft。
- Release 中包含 `.msi` 和 `.exe` 文件。
- 生成的包能够通过 `init.ps1` 环境验证。
- 自动更新元数据文件已生成且内容正确。
