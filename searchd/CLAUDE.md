# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is `searchd` - a Windows-only file search engine daemon written in Rust. It's an MVP implementation that provides fast local file search using NTFS MFT (Master File Table) indexing with USN Journal incremental updates. The daemon exposes a Named Pipe IPC interface using length-prefixed Protobuf messages for communication with Qt-based GUI clients.

**Key Design Principle**: This is an MVP focused on core functionality. Avoid over-engineering - no persistence layer, no complex abstractions, no configuration system.

## Build and Development Commands

### Building
```bash
cargo build          # Debug build
cargo build --release # Release build
```

### Running
```bash
cargo run            # Run the daemon (debug mode)
cargo run --release  # Run optimized version
```

### Testing
```bash
cargo test           # Run all tests
cargo test --lib     # Run library tests only
cargo test <test_name> # Run specific test
```

### Protobuf Generation
Protobuf files are automatically compiled during build via `build.rs`. The proto definition is at `../proto/search.proto` and generates code into `OUT_DIR/search.ipc.rs`.

## Architecture

### Process Model
- **searchd.exe** (this codebase): Rust daemon that handles indexing, search, and IPC server
- **qt_gui.exe** (separate): C++/Qt client that connects via Named Pipe IPC

### Core Components

#### 1. IPC Layer (`src/ipc/`)
- **Named Pipe Server**: Windows Named Pipe at `\\.\pipe\listory_plus_search`
- **Protocol**: Length-prefixed Protobuf (4-byte little-endian length + protobuf payload)
- **Request/Response**: Synchronous, single-connection, serial processing
- **Messages**: Ping, BuildIndex, Search (defined in `../proto/search.proto`)

#### 2. Index Layer (`src/index/`)
- **MemoryIndex** (`memory.rs`): Everything-style Vec-based architecture
  - **Main table**: `Vec<FileEntry>` for sequential scanning (SIMD-ready)
  - **Fast lookup**: `HashMap<u64, usize>` mapping file_ref → index
  - **Parent-child**: `HashMap<u64, Vec<usize>>` mapping parent_ref → children indices
  - **Path reconstruction**: `get_full_path()` traverses parent references to build full paths
  - No persistence - rebuilt on restart
- **MFT Reader** (`mft.rs`): NTFS MFT enumeration using USN Journal API
  - Root drives (C:\, D:\): Uses `FSCTL_ENUM_USN_DATA` for fast MFT enumeration (~10,000 files/sec)
  - Subdirectories: Falls back to `std::fs::read_dir` filesystem walk (~1,000 files/sec)
  - Extracts `file_reference_number` and `parent_file_reference_number` from MFT records
- **USN Monitor** (`usn.rs`): Handles incremental updates via USN Journal (FILE_CREATE, FILE_DELETE, RENAME_NEW_NAME)

#### 3. Search Layer (`src/search/`)
- **Filename Search** (`filename.rs`): Case-insensitive substring matching on filenames
- **Content Search** (`content.rs`): Delegates to ripgrep CLI (not indexed)

#### 4. Data Model (`src/model/`)
- **FileEntry**: Everything-style structure optimized for memory and performance
  - `file_ref: u64` - MFT File Reference Number (unique identifier)
  - `parent_ref: u64` - Parent directory MFT Reference
  - `name: Arc<str>` - Filename only (shared string for memory efficiency)
  - `size: u64` - File size in bytes
  - `attributes: u32` - File attributes (directory, hidden, system, etc.)
  - Backward compatibility: `from_path_filename()` for non-MFT sources

### Main Loop Flow
1. Start Named Pipe server
2. Wait for client connection (`accept()`)
3. Read length-prefixed messages
4. Decode Protobuf and dispatch to handler
5. Execute request (Ping/BuildIndex/Search)
6. Encode response and send back
7. On disconnect, loop back to accept

### IPC Protocol Details

**Current Protocol Format (v2):**
```
[1 byte message type][4 bytes u32 length][protobuf payload]
```

**Message Types:**
- `0` = Ping
- `1` = BuildIndex
- `2` = Search

**Protocol Evolution:**
- Initial version used only length-prefix, causing message type ambiguity
- Current version adds 1-byte type field for explicit message identification
- Endianness: Little-endian for length field
- Error Handling: Returns empty results on error, never panics

**Implementation Notes:**
- Empty messages (length = 0) are valid (e.g., PingReq)
- Server reads: type byte → length (4 bytes) → payload (if length > 0)
- Client must send all three parts in correct order
- Response format: `[4 bytes length][protobuf payload]` (no type byte in response)

## Important Constraints

### What This MVP Does NOT Include
- No index persistence (SQLite/RocksDB)
- No full-text inverted index
- No multi-threading/async for search operations
- No configuration files
- No Trie/FST data structures
- No directory permissions/timestamps in index
- No USN checkpoint persistence

### Search Limitations
- Filename search: substring match only, returns first N results
- Content search: synchronous ripgrep call, no indexing
- No ranking/scoring algorithm

### Windows-Specific
- Uses Windows API directly via `windows` crate
- NTFS-specific features (MFT, USN Journal)
- Named Pipes for IPC

## Development Guidelines

### When Modifying Index Logic
- The index structure is intentionally simple: `HashMap<String, Vec<FileEntry>>`
- Do not introduce complex data structures unless explicitly required
- Keep search as simple substring matching

### When Modifying IPC
- Maintain backward compatibility with the Protobuf schema
- Keep the length-prefix protocol unchanged (4-byte little-endian)
- All IPC operations are synchronous - do not add async complexity

### When Adding Features
- Verify it aligns with MVP scope (see 功能清单.md for original requirements)
- Avoid adding configuration, persistence, or advanced search features
- Keep the single-threaded serial processing model

### Error Handling
- Log errors but don't crash the daemon
- Return empty results on search failures
- Skip inaccessible directories during indexing

## Testing

### Unit Tests
- Unit tests exist in each module (marked with `#[cfg(test)]`)
- Tests use simple assertions on core functionality
- Run with: `cargo test`

### IPC Integration Testing
A Python test client is provided for end-to-end IPC testing:

```bash
# Test all IPC operations (Ping, BuildIndex, Search)
python test_ipc_full.py

# Simple connection test
python test_simple.py
```

**Test Client Requirements:**
- Python 3.x
- `pywin32` package: `pip install pywin32`

**What the tests verify:**
1. **Ping Test**: Connection and version retrieval
2. **BuildIndex Test**: Directory scanning and index building
3. **Search Test**: Filename search with substring matching

**Expected Results:**
- Ping returns version "0.1.0"
- BuildIndex successfully indexes files from specified directories
- Search returns matching files with full paths

**Test Output Example:**
```
✓ Ping test PASSED - Version: 0.1.0
✓ BuildIndex test PASSED - Success: True, Indexed files: 14
✓ Search test PASSED - Found 4 results for 'mod'
```

## Dependencies
- **windows**: Windows API bindings
- **prost**: Protobuf runtime
- **tokio**: Async runtime (used for Named Pipe server)
- **anyhow/thiserror**: Error handling
- **log/env_logger**: Logging

## Logging
Set `RUST_LOG` environment variable to control log level:

**PowerShell:**
```powershell
$env:RUST_LOG="debug"
cargo run
```

**Bash:**
```bash
RUST_LOG=debug cargo run
RUST_LOG=info cargo run
```

**Log Levels:**
- `error` - Only errors
- `warn` - Warnings and errors
- `info` - General information (default)
- `debug` - Detailed debugging information
- `trace` - Very verbose tracing

**Useful Debug Commands:**
```powershell
# Run with debug logging to troubleshoot IPC issues
$env:RUST_LOG="debug"
cargo run

# Check if service is running
tasklist | findstr searchd

# Test IPC connection
python test_simple.py
```

## Troubleshooting

### Build Issues

**Problem: `protoc` not found**
```
Error: Could not find `protoc`
```
**Solution:** Install Protocol Buffers compiler:
- Download from: https://github.com/protocolbuffers/protobuf/releases
- Or use package manager: `choco install protoc` / `scoop install protobuf`

**Problem: Build fails with "access denied"**
```
error: failed to remove file `target\debug\searchd.exe`
Caused by: 拒绝访问。 (os error 5)
```
**Solution:** Stop the running searchd.exe process:
```powershell
taskkill /F /IM searchd.exe
```

### IPC Issues

**Problem: Client can't connect - "All pipe instances are busy"**
```
pywintypes.error: (231, 'CreateFile', '所有的管道范例都在使用中。')
```
**Solution:**
- Only one client can connect at a time (MVP limitation)
- Wait for previous client to disconnect
- Or restart the searchd service

**Problem: Client hangs waiting for response**
**Solution:**
- Check server logs with `RUST_LOG=debug`
- Verify message type byte is correct (0=Ping, 1=BuildIndex, 2=Search)
- Ensure payload length matches actual payload size

**Problem: "Failed to read from pipe" error**
**Solution:**
- This usually means the client disconnected unexpectedly
- Check client-side error messages
- Verify protocol format: `[1 byte type][4 bytes length][payload]`

## Quick Start Guide

1. **Build the project:**
   ```bash
   cargo build
   ```

2. **Run the daemon:**
   ```powershell
   $env:RUST_LOG="info"
   cargo run
   ```

3. **In another terminal, test the IPC:**
   ```bash
   python test_ipc_full.py
   ```

4. **Expected output:**
   - Server: `Named Pipe server started, waiting for connections...`
   - Client: `✓ ALL TESTS PASSED!`

## Next Steps for Development

### Implementing Qt Client
The Qt client should implement the same IPC protocol:
1. Connect to `\\.\pipe\listory_plus_search`
2. Send messages with format: `[1 byte type][4 bytes length][payload]`
3. Read responses with format: `[4 bytes length][payload]`
4. Use Qt's Protobuf support or manual encoding

### Performance Optimization
- Replace `std::fs::read_dir` with direct MFT reading for faster indexing
- Implement proper USN Journal monitoring for real-time updates
- Add multi-threading for concurrent search requests (requires protocol changes)

### Feature Additions
- Content search indexing (currently uses ripgrep)
- Search result ranking/scoring
- Filter by file type, size, date
- Regular expression support


🧱 架构总览

┌─────────────┐
│  USN Journal│ ← 增量变化
└──────┬──────┘
       │
┌──────▼────────┐
│   索引维护层   │
│ (增删改 MFT记录)│
└──────┬────────┘
       │
┌──────▼────────┐
│  内存索引结构  │ ← 搜索只访问这里
│  (Trie / Vec)  │
└──────┬────────┘
       │
┌──────▼────────┐
│   Rust 查询引擎│
└───────────────┘
完整流程：
程序启动
  ↓
一次性扫描 MFT（全盘）
  ↓
构建内存索引（Vec / Trie / Hash）
  ↓
记录当前 USN 起点
  ↓
─────────────── 程序运行中 ───────────────
  ↓
持续读取 USN Journal
  ↓
只处理“发生变化的那几个文件”
  ↓
增量更新内存索引

那 MFT 什么时候“再扫一次”？
1️⃣ 接收到IPC的建立索引请求

建立完整索引

获取 file_id → path 映射

2️⃣ USN Journal 断档

例如：

程序关机太久

USN Journal 被清空

USN 起点号 < Journal First USN

👉 增量不可用，只能全量重建





📊 当前状态：

  项目进度： MVP 核心功能已完成 🎉
  - ✅ Named Pipe IPC 服务器
  - ✅ Protobuf 消息编解码
  - ✅ 文件索引构建（MFT + 文件系统遍历）
  - ✅ 文件名搜索（不区分大小写）
  - ✅ 完整的测试套件
  - ✅ MFT 优化性能（已实现，路径过滤已修复）
  - ⏳ 架构优化（Everything 风格索引） <- 当前进度
  - ⏳ USN Journal 增量更新（代码已存在，待集成）
  - ⏳ Qt 客户端（待开发）

## 架构升级计划

### 当前架构（V1）
```rust
HashMap<String, Vec<FileEntry>>  // 按文件名分组

struct FileEntry {
    path: String,      // 完整路径
    filename: String,  // 文件名
}
```

### 目标架构（V2）- Everything 风格
```rust
Vec<FileEntry>                    // 主表（顺序扫描）
HashMap<u64, usize>               // file_ref → index
HashMap<u64, Vec<usize>>          // parent_ref → children

struct FileEntry {
    file_ref: u64,      // MFT Reference
    parent_ref: u64,    // 父目录引用
    name: Arc<str>,     // 仅文件名（共享字符串）
    size: u64,
    attributes: u32,
}
```

**优势：**
- ✅ 支持 MFT Reference，可增量更新
- ✅ 支持路径重建
- ✅ Vec 顺序扫描，SIMD 优化潜力
- ✅ 内存效率提升 ~47%

**实施计划：**
详见 `MIGRATION_PLAN.md` 文档

**新文件：**
- `src/model/file_entry_v2.rs` - 新索引架构实现
- `MIGRATION_PLAN.md` - 详细迁移计划
- `PROGRESS.md` - 工作进度报告
