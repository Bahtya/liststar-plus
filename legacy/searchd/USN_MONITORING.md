# USN Journal 实时监控功能

## 📋 功能概述

USN (Update Sequence Number) Journal 是 NTFS 文件系统的变更日志，记录所有文件系统操作。searchd 现在支持通过 USN Journal 实现实时文件变化监控，无需重新扫描整个文件系统。

## ✨ 支持的操作

- ✅ **文件创建** (FILE_CREATE) - 自动添加到索引
- ✅ **文件删除** (FILE_DELETE) - 自动从索引移除
- ✅ **文件重命名** (RENAME_NEW_NAME) - 自动更新索引

## 🚀 启用 USN 监控

### 方法 1: 修改 main.rs（推荐用于开发测试）

在 `src/main.rs` 中取消注释以下代码：

```rust
// Start USN monitoring task (optional, can be enabled after initial indexing)
// Uncomment to enable real-time file monitoring:
let index_clone = index.clone();
tokio::spawn(async move {
    if let Err(e) = start_usn_monitoring(index_clone, 'C').await {
        log::error!("USN monitoring failed: {}", e);
    }
});
```

修改为：

```rust
// Start USN monitoring task for C: drive
let index_clone = index.clone();
tokio::spawn(async move {
    if let Err(e) = start_usn_monitoring(index_clone, 'C').await {
        log::error!("USN monitoring failed: {}", e);
    }
});
```

### 方法 2: 通过 IPC 协议启动（未来实现）

可以添加新的 IPC 消息类型来动态启动/停止 USN 监控：

```protobuf
message StartMonitoringReq {
    string drive_letter = 1;
}

message StartMonitoringResp {
    bool success = 1;
}
```

## 🧪 测试步骤

### 1. 启用 USN 监控

编辑 `src/main.rs`，取消注释 USN 监控代码。

### 2. 编译并运行

```bash
cargo build
cargo run
```

### 3. 建立初始索引

使用 Python 测试客户端建立索引：

```python
python test_ipc_full.py
```

或者手动发送 BuildIndex 请求：

```python
import win32file
import struct

# Connect to pipe
handle = win32file.CreateFile(
    r'\\.\pipe\listory_plus_search',
    win32file.GENERIC_READ | win32file.GENERIC_WRITE,
    0, None,
    win32file.OPEN_EXISTING,
    0, None
)

# Build index for C:\
msg_type = 1  # BuildIndex
payload = b'\n\x03C:\\'  # roots: ["C:\\"]
length = len(payload)
message = struct.pack('<B', msg_type) + struct.pack('<I', length) + payload

win32file.WriteFile(handle, message)
```

### 4. 测试文件创建

在 C:\ 驱动器上创建一个新文件：

```bash
echo "test" > C:\test_usn_create.txt
```

查看 searchd 日志，应该看到：

```
[DEBUG] File created: test_usn_create.txt (ref: 12345678)
```

### 5. 测试文件删除

删除刚创建的文件：

```bash
del C:\test_usn_create.txt
```

查看日志：

```
[DEBUG] File deleted: test_usn_create.txt (ref: 12345678)
```

### 6. 测试文件重命名

```bash
echo "test" > C:\test_usn_old.txt
ren C:\test_usn_old.txt test_usn_new.txt
```

查看日志：

```
[DEBUG] File renamed: test_usn_new.txt (ref: 12345678)
```

### 7. 验证索引更新

使用搜索功能验证索引已更新：

```python
# Search for the new file
msg_type = 2  # Search
payload = b'\n\x0ftest_usn_new\x10\x05'  # keyword: "test_usn_new", limit: 5
length = len(payload)
message = struct.pack('<B', msg_type) + struct.pack('<I', length) + payload

win32file.WriteFile(handle, message)
# Read response...
```

## 📊 性能特性

### 优势

1. **实时性**: 文件变化立即反映到索引中（延迟 < 100ms）
2. **低开销**: 不需要定期扫描文件系统
3. **精确性**: 只处理实际发生的变化

### 限制

1. **仅支持 NTFS**: USN Journal 是 NTFS 特有功能
2. **需要管理员权限**: 读取 USN Journal 需要提升权限
3. **驱动器级别**: 每个驱动器需要单独监控

## 🔧 配置选项

### 监控间隔

在 `src/index/usn.rs` 中修改：

```rust
// Sleep briefly to avoid busy-waiting
tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
```

- 默认: 100ms
- 建议范围: 50ms - 1000ms
- 更短的间隔 = 更快的响应，但 CPU 使用率更高

### 过滤规则

当前过滤规则（在 `handle_usn_record` 中）：

```rust
// Skip directories and system files
let is_directory = (record.file_attributes & 0x10) != 0;
if is_directory || filename.starts_with('$') {
    return Ok(());
}
```

可以添加更多过滤条件：

```rust
// Skip temporary files
if filename.ends_with(".tmp") || filename.ends_with("~") {
    return Ok(());
}

// Skip hidden files
if (record.file_attributes & 0x02) != 0 {
    return Ok(());
}
```

## 🐛 故障排除

### 问题 1: "Failed to query USN journal"

**原因**: 驱动器不支持 USN Journal 或未启用

**解决方案**:
```bash
# 启用 USN Journal
fsutil usn createjournal m=1000 a=100 C:

# 查询 USN Journal 状态
fsutil usn queryjournal C:
```

### 问题 2: "Access denied"

**原因**: 需要管理员权限

**解决方案**: 以管理员身份运行 searchd.exe

### 问题 3: 监控未启动

**原因**: 代码被注释掉了

**解决方案**: 检查 `src/main.rs` 中的 USN 监控代码是否已取消注释

### 问题 4: 日志中看不到文件变化

**原因**:
1. 日志级别设置为 INFO，DEBUG 消息被过滤
2. 文件变化发生在未监控的驱动器上

**解决方案**:
```bash
# 设置日志级别为 DEBUG
$env:RUST_LOG="debug"
cargo run
```

## 📝 实现细节

### 架构

```
┌─────────────────┐
│   main.rs       │
│  (IPC Server)   │
└────────┬────────┘
         │
         ├─────────────────┐
         │                 │
         ▼                 ▼
┌─────────────────┐  ┌──────────────────┐
│  RequestHandler │  │  UsnMonitor      │
│  (Sync Requests)│  │  (Async Task)    │
└────────┬────────┘  └────────┬─────────┘
         │                    │
         │                    │ read_usn_changes()
         │                    │ handle_usn_record()
         │                    │
         ▼                    ▼
    ┌────────────────────────────┐
    │      MemoryIndex           │
    │  (Arc<RwLock<...>>)        │
    │                            │
    │  - add_entry()             │
    │  - remove_entry()          │
    │  - update_entry()          │
    └────────────────────────────┘
```

### 数据流

1. **USN Journal 读取**:
   ```
   FSCTL_READ_USN_JOURNAL → buffer → USN_RECORD
   ```

2. **记录处理**:
   ```
   D → extract filename → check reason → update index
   ```

3. **索引更新**:
   ```
   FILE_CREATE  → index.add_entry(FileEntry::new(...))
   FILE_DELETE  → index.remove_entry(file_ref)
   RENAME       → index.update_entry(file_ref, new_entry)
   ```

### 线程安全

- `MemoryIndex` 使用 `Arc<RwLock<...>>` 包装
- USN 监控任务在独立的 tokio 任务中运行
- 写操作使用 `index.write().await` 获取独占锁
- 搜索操作使用 `index.read().await` 获取共享锁

## 🚀 未来改进

### 短期

- [ ] 添加 IPC 消息来动态启动/停止监控
- [ ] 支持监控多个驱动器
- [ ] 添加监控状态查询接口

### 中期

- [ ] 实现更智能的过滤规则（配置文件）
- [ ] 添加监控统计信息（处理的记录数、错误数等）
- [ ] 支持暂停/恢复监控

### 长期

- [ ] 持久化 USN 位置，重启后继续监控
- [ ] 实现 USN Journal 回放（处理离线期间的变化）
- [ ] 支持网络驱动器监控（如果可能）

## 📚 参考资料

- [MSDN: Change Journals](https://docs.microsoft.com/en-us/windows/win32/fileio/change-journals)
- [MSDN: FSCTL_READ_USN_JOURNAL](https://docs.microsoft.com/en-us/windows/win32/api/winioctl/ni-winioctl-fsctl_read_usn_journal)
- [Everything 源码分析](https://www.voidtools.com/)

---

**创建时间**: 2026-01-15
**状态**: 功能已实现，待测试
**优先级**: 高
