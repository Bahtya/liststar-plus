# Listory Plus v2 UI/UX Redesign Design Document (uTools/Spotlight Style)

## 1. Problem Statement
The current frontend UI of Listory Plus is a basic prototype. To evolve it into a high-frequency productivity tool comparable to macOS Spotlight or uTools, we need to completely overhaul its interaction logic and visual presentation. The standard desktop window format and plain-text lists are inefficient for quick searching and launching.

## 2. Requirements

### Functional Requirements
- **Global Shortcut**: Toggle window visibility via a global hotkey (e.g., `Alt+Space`). Auto-hide when the window loses focus.
- **Frameless Floating Window**: Remove OS title bars and borders. The window should appear centered, floating, and transparent with rounded corners.
- **Rich Media List**: Search results must display a composite layout: "File Icon (Left) + File Name (Top) + File Path (Bottom)".
- **Keyboard-First Interaction**: Permanent focus on the search input; use Up/Down arrows to navigate results; `Enter` to open.

### Non-Functional Requirements
- **Performance (`ui-ux-pro-max` strictness)**: Zero input latency. Avoid complex CSS animations that could block the rendering of large result lists.

### Constraints
- Refactor the React frontend using Tailwind CSS for rapid and consistent styling.
- Integrate Tauri v2's System Tray and Global Shortcut plugins.

## 3. Approach & Architecture
We will implement **Approach 1: Tauri Native Launcher**.

### Tauri Backend (Rust)
1. **Window Config (`tauri.conf.json`)**: `decorations: false`, `transparent: true`, `alwaysOnTop: true`, `center: true`, `resizable: false`.
2. **Lifecycle Management (`lib.rs`)**: Intercept the close event to hide the window instead of quitting. Keep the app running in the background.
3. **Plugins**: Integrate `tauri-plugin-global-shortcut` and `tauri-plugin-tray-icon`.
4. **Icon Extraction (New)**: Expose a new command `get_file_icon(path)` using Windows APIs to extract the associated executable or shell icon as Base64.

### React Frontend (UI)
1. **Styling Engine**: Install and configure Tailwind CSS.
2. **Search Component**: A large, seamless, borderless input box.
3. **List Component**: A fixed-height scrollable list displaying the Base64 icon, title, and path.
4. **Event Listeners**: Listen for `blur` events (on the window) to trigger Tauri's `hide()` API.

## 4. Agent Team
- **devops_engineer**: Setup Tailwind CSS, install Tauri plugins.
- **coder (Rust)**: Update window properties, implement tray/shortcut logic, and build the Windows Icon extraction API.
- **coder (React)**: Rewrite `App.tsx` and `App.css` using Tailwind to achieve the Spotlight aesthetic.
- **tester**: Verify shortcut responsiveness, UI layout correctness, and icon rendering.

## 5. Risk Assessment & Mitigation
- **Risk**: Extracting file icons via Windows API can be slow and block the search response.
  - **Mitigation**: Fetch icons asynchronously on the frontend via lazy loading, or cache them.
- **Risk**: Borderless transparent windows lack native OS drop shadows.
  - **Mitigation**: Apply Tailwind's `shadow-2xl` and `rounded-xl` to the outermost React container wrapper.

## 6. Success Criteria
- The app starts in the system tray.
- Pressing `Alt+Space` instantly summons a centered, shadow-casted, rounded search bar.
- Search results display native system icons alongside filenames and paths.
- Clicking outside the window immediately hides it.