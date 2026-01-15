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
- **MemoryIndex** (`memory.rs`): In-memory HashMap<String, Vec<FileEntry>>
  - Key: lowercase filename for case-insensitive search
  - Value: list of FileEntry (path + filename)
  - No persistence - rebuilt on restart
- **MFT Reader** (`mft.rs`): Simplified filesystem walker (MVP uses `std::fs::read_dir` instead of raw MFT parsing)
- **USN Monitor** (`usn.rs`): Handles incremental updates via USN Journal (FILE_CREATE, FILE_DELETE, RENAME_NEW_NAME)

#### 3. Search Layer (`src/search/`)
- **Filename Search** (`filename.rs`): Case-insensitive substring matching on filenames
- **Content Search** (`content.rs`): Delegates to ripgrep CLI (not indexed)

#### 4. Data Model (`src/model/`)
- **FileEntry**: Simple struct with `path` and `filename` fields

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
