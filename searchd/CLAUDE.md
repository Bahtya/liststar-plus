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
- **Format**: `[4 bytes u32 length][protobuf payload]`
- **Endianness**: Little-endian
- **Message Type Detection**: MVP tries to decode as each message type in order (Ping → BuildIndex → Search)
- **Error Handling**: Returns empty results on error, never panics

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
- Unit tests exist in each module (marked with `#[cfg(test)]`)
- Tests use simple assertions on core functionality
- No integration tests or end-to-end tests in MVP

## Dependencies
- **windows**: Windows API bindings
- **prost**: Protobuf runtime
- **tokio**: Async runtime (used for Named Pipe server)
- **anyhow/thiserror**: Error handling
- **log/env_logger**: Logging

## Logging
Set `RUST_LOG` environment variable to control log level:
```bash
RUST_LOG=debug cargo run
RUST_LOG=info cargo run
```
