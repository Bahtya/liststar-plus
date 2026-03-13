# Listory Plus v2 发布指南

本指南详细说明了如何使用 GitHub Actions 自动化发布 Listory Plus v2。

## 发布流程

### Step 1: 准备签名密钥

为了启用自动更新和安装包签名，你需要生成一对 Tauri 签名密钥。

1.  在项目根目录下运行以下 PowerShell 脚本：
    ```powershell
    .\scripts\generate-keys.ps1
    ```
2.  脚本会调用 `npx tauri signer generate` 并引导你生成密钥。
3.  **保存输出**：记录生成的 `Private key` 和 `Public key`。
4.  **更新配置**：将 `Public key` 填入 `src-tauri/tauri.conf.json` 中的 `plugins.updater.pubkey` 字段。

### Step 2: 配置 GitHub Secrets

在 GitHub 仓库的 **Settings > Secrets and variables > Actions** 页面中，点击 **New repository secret** 添加以下 3 个密钥：

| Secret 名称 | 说明 | 来源 |
| :--- | :--- | :--- |
| `TAURI_SIGNING_PRIVATE_KEY` | 用于签署安装包的私钥 | `generate-keys.ps1` 输出的 Private key |
| `TAURI_SIGNING_PASSWORD` | 生成私钥时设置的密码 | 你在生成密钥时输入的密码（若无则留空或填任意值） |
| `GITHUB_TOKEN` | 用于创建 Release 和上传资源 | GitHub 自动提供（确保 Workflow 权限为 Read and write） |

> **注意**：`GITHUB_TOKEN` 通常由 GitHub Actions 自动注入，但请确保在仓库设置中开启了 `Read and write permissions` (**Settings > Actions > General > Workflow permissions**)。

### Step 3: 触发自动化发布

发布新版本只需推送一个新的 Git Tag：

1.  **更新版本号**：确保 `package.json` 和 `src-tauri/tauri.conf.json` 中的版本号一致。
2.  **提交更改**：
    ```bash
    git add .
    git commit -m "chore: bump version to v0.1.0"
    git push origin main
    ```
3.  **推送 Tag**：
    ```bash
    git tag v0.1.0
    git push origin v0.1.0
    ```
    或者一次性推送所有 Tag：`git push --tags`

### Step 4: 手动发布 Release

1.  推送 Tag 后，GitHub Actions 会自动启动 `Release` 工作流。
2.  等待工作流运行完成（通常需要 5-10 分钟）。
3.  前往仓库的 **Releases** 页面，你会看到一个名为 `Listory Plus v0.1.0` 的 **Draft**。
4.  点击 **Edit**，完善发布说明，然后点击 **Publish release** 正式发布。

---

## 快速排查表 (Troubleshooting)

| 问题现象 | 可能原因 | 解决方法 |
| :--- | :--- | :--- |
| **Workflow 报错: Permission denied** | `GITHUB_TOKEN` 权限不足 | 在仓库设置中将 Workflow permissions 改为 `Read and write` |
| **安装包签名失败** | Secret 配置错误 | 检查 `TAURI_SIGNING_PRIVATE_KEY` 是否包含完整字符串 |
| **版本号不匹配** | 配置文件未同步 | 确保 `package.json` 和 `tauri.conf.json` 版本号完全一致 |
| **未触发 Workflow** | Tag 格式不正确 | 确保 Tag 以 `v` 开头（例如 `v1.0.0`），匹配 `release.yml` 的配置 |
| **自动更新失效** | 公钥不匹配 | 检查 `tauri.conf.json` 中的 `pubkey` 是否与 GitHub 使用的私钥对应 |

---
*文档版本: 1.0.0 | 关联源: .github/workflows/release.yml, scripts/generate-keys.ps1*
