# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is the Qt 6 GUI frontend for Listory Search, a Windows file search application. The GUI is a **client-only** application that communicates with a Rust-based search engine (`searchd.exe`) via Named Pipes using Protobuf messages.

**Critical Constraint**: This codebase is UI-only. Do NOT implement any indexing, NTFS/MFT/USN parsing, or search logic here. All business logic resides in the Rust backend (`../searchd`).

## Build Commands

### Prerequisites
- Qt 6.x (6.5 or higher recommended) with Qt Quick and Qt Concurrent
- CMake 3.16+
- Protobuf compiler (`protoc.exe`) - path configured in CMakeLists.txt line 12
- Visual Studio 2019/2022 or MinGW
- vcpkg (recommended for Protobuf) or manual Protobuf installation

### Build with CMake (Command Line)

```bash
# From qt_gui directory
mkdir build
cd build

# Configure (adjust paths to your Qt installation)
cmake .. -DCMAKE_PREFIX_PATH="C:/Qt/6.5.0/msvc2019_64" -DCMAKE_TOOLCHAIN_FILE="C:/vcpkg/scripts/buildsystems/vcpkg.cmake" -G "Visual Studio 17 2022" -A x64

# Build
cmake --build . --config Release

# Run
Release\listory_search.exe
```

### Build with Qt Creator (Recommended)
1. Open `CMakeLists.txt` in Qt Creator
2. Configure with Qt 6 Kit
3. Build and run directly from IDE

### Deploy
```bash
# After building Release version
cd build/Release
C:\Qt\6.5.0\msvc2019_64\bin\windeployqt.exe listory_search.exe

# Copy the Rust search engine
copy ..\..\searchd\target\release\searchd.exe .
```

## Architecture

### High-Level Design

```
┌─────────────────────────────────────────────────────────────┐
│                    Qt GUI (this repo)                       │
│  ┌──────────────┐    ┌──────────────┐    ┌──────────────┐ │
│  │  QML UI      │───▶│SearchBackend │───▶│ PipeClient   │ │
│  │  (main.qml)  │    │(QAbstractList│    │ (Win32 API)  │ │
│  └──────────────┘    │    Model)    │    └──────┬───────┘ │
│                      └──────────────┘           │          │
└──────────────────────────────────────────────────┼──────────┘
                                                   │
                                    Named Pipe: \\.\pipe\listory_search
                                                   │
┌──────────────────────────────────────────────────┼──────────┐
│                 Rust Backend (searchd.exe)       │          │
│                                                  ▼          │
│  ┌──────────────────────────────────────────────────────┐  │
│  │  IPC Handler (Named Pipe Server)                     │  │
│  └──────────────────────────────────────────────────────┘  │
│  ┌──────────────────────────────────────────────────────┐  │
│  │  Search Engine (MFT/USN indexing, search logic)      │  │
│  └──────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
```

### Component Responsibilities

**QML UI (`qml/main.qml`)**
- Single-window interface with search input, results ListView, and status bar
- Delegates all logic to SearchBackend
- Uses Qt Quick Controls for minimal, tool-like UI (similar to Everything/Spotlight)

**SearchBackend (`src/search_backend.{h,cpp}`)**
- QAbstractListModel exposing search results to QML
- Manages PipeClient lifecycle
- Runs IPC operations in background threads via QtConcurrent to prevent UI blocking
- Marshals Protobuf messages to/from the Rust backend
- Handles file opening via ShellExecuteW

**PipeClient (`src/ipc/pipe_client.{h,cpp}`)**
- Windows Named Pipe client using Win32 API (CreateFileW, ReadFile, WriteFile)
- Auto-starts `searchd.exe` if pipe connection fails
- Synchronous request/response pattern (no streaming)
- Pipe name: `\\.\pipe\listory_search`

**IPC Codec (`src/ipc/ipc_codec.h`)**
- Header-only utility for encoding/decoding messages
- Wire format: `[4 bytes length (little-endian uint32)][protobuf payload]`
- No type byte prefix (unlike some IPC protocols)

### IPC Protocol

Defined in `proto/search.proto`:

**Messages:**
- `PingReq/PingResp` - Health check, returns backend version
- `BuildIndexReq/BuildIndexResp` - indexing (not currently used by UI)
- `SearchReq/SearchResp` - File search with keyword and limit
- `SearchResult` - Contains `filename` and `path` fields

**Wire Format:**
```
[4 bytes: payload length (uint32 LE)] [N bytes: protobuf binary]
```

The Qt client sends requests and blocks waiting for responses. All IPC calls happen in background threads to keep the UI responsive.

### Threading Model

- **UI Thread**: QML rendering, user input, model updates
- **Background Threads** (QtConcurrent::run): All PipeClient operations
  - `performPing()` - Engine health check
  - `performSearch()` - Search requests
- Results are marshaled back to UI thread via `QMetaObject::invokeMethod(..., Qt::QueuedConnection)`

**Important**: Never call PipeClient methods directly from the UI thread.

### Startup Flow

1. Qt app launches, loads QML
2. SearchBackend constructor creates PipeClient
3. `connectToEngine()` called from main.cpp
4. Background thread attempts pipe connection
5. If connection fails, PipeClient starts `searchd.exe` via CreateProcessW
6. Retries connection up to 3 times with 500ms delays
7. Sends PingReq to verify connection and get version
8. Updates UI status bar with connection stn### Search Flow

1. User types in search box and presses Enter or clicks "查找"
2. QML calls `searchBackend.search(keyword)`
3. SearchBackend spawns background thread
4. Thread creates SearchReq protobuf, serializes it
5. PipeClient sends via Named Pipe with length prefix
6. Waits for response (blocking in background thread)
7. Deserializes SearchResp protobuf
8. Marshals results to UI thread
9. Updates QAbstractListModel, triggering QML ListView refresh

## Key Constraints and Design Decisions

### What This Codebase Does NOT Do
- ❌ File indexing (MFT/USN parsing)
- ❌ Search algorithms or ranking
- ❌ File system monitoring
- ❌ Protocol design changes (proto file is shared with Rust backend)
- ❌ Multi-window UI, settings pages, themes, animations, i18n

### What This Codebase DOES Do
- ✅ Minimal search UI (input box, results list, status bar)
- ✅ Named Pipe client communication
- ✅ Protobuf message serialization
- ✅ Background threading for IPC
- ✅ Auto-starting the Rust backend
- ✅ Opening files via ShellExecuteW

### Protobuf Configuration

The CMakeLists.txt hardcodes the protoc path (line 12):
```cmake
set(Protobuf_PROTOC_EXECUTABLE "C/protoc-33.4-win64/bin/protoc

Adjust this path for your environment or use vcpkg's protoc.

Generated files (`search.pb.h`, `search.pb.cc`) are created in the build directory and automatically linked.

## Common Issues

### "Could not find Qt6"
Set CMAKE_PREFIX_PATH to your Qt installation:
```bash
cmake .. -DCMAKE_PREFIX_PATH="C:/Qt/6.5.0/msvc2019_64"
```

### "Could not find Protobuf"
Use vcpkg toolchain file:
```bash
cmake .. -DCMAKE_TOOLCHAIN_FILE="C:/vcpkg/scripts/buildsystems/vcpkg.cmake"
```

### "Engine: Disconnected" in UI
- Ensure `searchd.exe` is in the same directory as `listory_search.exe`
- Check that the pipe name matches: `\\.\pipe\listory_search`
- Verify the Rust backend is running and listening on the correct pipe

### Runtime DLL errors
Run windeployqt to copy Qt dependencies:
```bash
C:\Qt\6.5.0\msvc2019_64\bin\windeployqt.exe listory_search.exe
```

## Modifying the UI

The QML file (`qml/main.qml`) defines the entire UI. Key elements:
- **Line 22-40**: Search input TextField and Button
- **Line 49-144**: Results ListView with custom delegate
- **Line 147-182**: Status bar showing engine status and file count

The UI follows a strict vertical layout with no tabs, dialogs, or secondary windows.

## Modifying IPC

If you need to change the protocol:
1. **DO NOT** modify `proto/search.proto` without coordinating with the Rust backend
2. The proto file is shared between Qt and Rust codebases
3. Both sides must be rebuilt after proto changes
4. The wire format (length prefix) is fixed and must not change

## File Opening

Double-clicking a result calls `SearchBackend::openFile()` which uses Windows ShellExecuteW to open the file with its default application. This is intentionally simple - no custom file viewers or editors.
