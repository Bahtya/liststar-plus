# Listory Plus v2 README Design Document

## 1. Problem Statement
The current documentation for Listory Plus does not reflect its new, high-performance architecture (Tauri + Rust + React). It lacks the visual appeal and clear feature presentation needed to attract open-source contributors and users.

## 2. Requirements
- **Style**: Modern product showcase style, comparable to popular tools like `clash-verge-rev` or `Everything`. Includes badges, clear feature lists, and placeholders for screenshots.
- **Selling Points**: Highlight the "Hexagon Warrior" traits: Tauri-based lightweight architecture, concurrent MFT scanning, real-time USN monitoring, and a minimalist Spotlight-style UI.
- **Localization**: Bilingual support using separate files (`README.md` for English, `README_zh.md` for Chinese).

## 3. Approach: Independent Bilingual Files (Approach 1)
We will create two separate README files. Both will feature a navigation header to switch between languages easily. This provides a clean reading experience for international users while catering to the domestic audience.

## 4. Architecture (Document Structure)
Both files will follow this structure:
1. **Header**: Logo (placeholder), Badges (License, Platform, Version), Language Switcher.
2. **Introduction**: A concise 1-2 sentence pitch.
3. **Features**: Bullet points highlighting core selling points.
4. **Installation**: Download links (from GitHub Releases) and requirements.
5. **Development Setup**: Instructions for building from source (referencing `init.ps1`).
6. **Tech Stack**: Brief mention of Tauri, Rust, React, and Windows API.
7. **License**: MIT (or current project license).

## 5. Agent Team
- **technical_writer**: Drafts and refines the markdown files in both English and Chinese.
- **devops_engineer**: Commits the changes and pushes them to the repository.

## 6. Risk Assessment
- **Risk**: Missing graphical assets.
- **Mitigation**: Use standardized markdown placeholders (e.g., `![Demo](url)`) that the user can replace later.

## 7. Success Criteria
- Both `README.md` and `README_zh.md` are created and properly linked.
- The content accurately reflects the v2 architecture.
- The changes are successfully pushed to the remote repository.