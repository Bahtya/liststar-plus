---
name: ui-ux-pro-max
description: Use this skill when designing or implementing UI/UX components to ensure they follow the minimalist, tool-oriented style (like Everything or Spotlight).
---

# UI/UX Pro Max Skill

This skill enforces a high-performance, minimalist UI/UX design philosophy for the `listory-plus` project, drawing from the standards of tools like Everything and Spotlight.

## Core Philosophy
- **Minimalist**: Strip away all non-essential visual elements.
- **Tool-oriented**: Prioritize function, speed, and efficiency over aesthetics.
- **System-native**: Use default system styles and controls whenever possible.

## Design Guidelines
- **Layout**: 
  - Search input box at the top (with immediate focus on launch).
  - Results list (table or list view) taking up the main area.
  - Status bar at the bottom showing engine status (Connected/Disconnected) and file counts.
- **Styling**:
  - No custom themes or complex decorations.
  - Use system-default fonts and colors.
  - No animations or visual effects that could impede performance.
- **Interaction**:
  - Focus on keyboard-driven workflows (Enter to search/open, Arrow keys to navigate results).
  - Double-click or Enter to open the selected file/folder.
  - Right-click for a simple context menu (Open, Open Path, Copy Path).

## Constraints (MVP Phase)
- **No Multi-windows**: Single window interface only.
- **No Settings Page**: Configuration should be handled via config files or simple defaults.
- **No Decorations**: No splash screens, fancy borders, or custom icons where system ones suffice.
- **No Internationalization**: Focus on the core locale first.

## Output Standards
When generating UI code (React/Tauri or Qt), ensure it adheres to these minimalist principles. Avoid adding "just-in-case" features or decorative CSS/styles.
