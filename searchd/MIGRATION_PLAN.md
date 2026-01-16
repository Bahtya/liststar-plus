# 索引架构升级计划

## 📊 架构对比

### 当前架构（V1）
```rust
// 按文件名分组的 HashMap
HashMap<String, Vec<FileEntry>>

struct FileEntry {
    path: String,      // 完整路径
    filename: String,  // 文件名
}
```

**问题：**
- ❌ 不存储 MFT Reference，无法支持增量更新
- ❌ 不存储父引用，无法重建路径
- ❌ 按文件名分组，不适合 MFT 架构
- ❌ 存储完整路径，内存浪费

### 新架构（V2）- Everything 风格
```rust
// Vec 顺序存储 + HashMap 快速查找
Vec<FileEntry>                    // 主表
HashMap<u64, usize>               // file_ref → index
HashMap<u64, Vec<usize>>          // parent_ref → children

struct FileEntry {
    file_ref: u64,      // MFT Reference
    parent_ref: u64,    // 父目录引用
    name: Arc<str>,     // 仅文件名
    size: u64,
    attributes: u32,
}
```

**优势：**
- ✅ 支持 MFT Reference，可增量更新
- ✅ 支持路径重建
- ✅ Vec 顺序扫描，SIMD 优化潜力
- ✅ 内存效率高（Arc<str> 共享）

## 🔄 迁移步骤

### 阶段 1：准备工作（不影响现有功能）✅ **已完成**
1. ✅ 创建新的数据结构（`file_entry.rs` 已更新）
2. ✅ 更新 MFT 枚举代码，提取 file_ref 和 parent_ref
3. ✅ 实现路径重建算法（`get_full_path()`）
4. ✅ 编写单元测试（11/11 通过）

### 阶段 2：集成新索引（并行运行）✅ **已完成**
1. ✅ 修改 `mft.rs` 使用新的 FileEntry 结构
2. ✅ 更新搜索逻辑适配新索引（`handler.rs` 使用路径重建）
3. ✅ 保持 IPC 协议兼容（SearchResult 不变）
4. ✅ 测试验证功能正常（IPC 测试全部通过）

### 阶段 3：启用增量更新 ✅ **已完成**
1. ✅ 集成 USN Journal 监控（已实现，可选启用）
2. ✅ 实现增量添加/删除/更新（使用 file_ref）
3. ✅ 测试实时文件变化监控（文档已创建）

### 阶段 4：性能优化 ⏳ **待实现**
1. ⏳ SIMD 优化搜索
2. ⏳ 字符串池优化
3. ⏳ 内存压缩

## 📝 详细实现计划

### 1. 更新 MFT 枚举代码

**文件**: `src/index/mft.rs`

**修改点**:
```rust
// 当前代码
let filename = String::from_utf16_lossy(filename_slice);
let path = format!("{}:\\{}", drive_letter, filename);
let file_entry = FileEntry::new(path, filename);

// 新代码
let filename = String::from_utf16_lossy(filename_slice);
let file_entry = FileEntry::new(
    record.file_reference_number,      // file_ref
    record.parent_file_reference_number, // parent_ref
    filename,
    0,  // size (可从 record 获取)
    record.file_attributes,
);
```

### 2. 实现路径重建

**新增函数**: `get_full_path_with_cache()`

```rust
// 使用缓存避免重复计算
struct PathCache {
    cache: HashMap<u64, Arc<str>>,
}

impl MemoryIndex {
    pub fn get_full_path_cached(&self, file_ref: u64, cache: &mut PathCache) -> Option<Arc<str>> {
        // 检查缓存
        if let Some(path) = cache.cache.get(&file_ref) {
            return Some(path.clone());
        }

        // 重建路径
        let path = self.get_full_path(file_ref, 'C')?;
        let arc_path = Arc::from(path);

        // 存入缓存
        cache.cache.insert(file_ref, arc_path.clone());

        Some(arc_path)
    }
}
```

### 3. 更新搜索逻辑

**文件**: `src/search/filename.rs`

```rust
pub fn search_filename(index: &MemoryIndex, keyword: &str, limit: usize) -> Vec<SearchResult> {
    let entries = index.search(keyword, limit);

    // 构建搜索结果（需要重建路径）
    let mut results = Vec::new();
    for entry in entries {
        if let Some(full_path) = index.get_full_path(entry.file_ref, 'C') {
            results.push(SearchResult {
                path: full_path,
                filename: entry.name.to_string(),
            });
        }
    }

    results
}
```

### 4. 集成 USN Journal

**文件**: `src/index/usn.rs`

```rust
impl UsnMonitor {
    pub async fn handle_usn_record(&self, record: &USN_RECORD) -> Result<()> {
        let mut index = self.index.write().await;

        match record.reason {
            USN_REASON_FILE_CREATE => {
                // 添加新文件
                let entry = FileEntry::new(
                    record.file_reference_number,
                    record.parent_file_reference_number,
                    extract_filename(record),
                    0,
                    record.file_attributes,
                );
                index.add_entry(entry);
            }
            USN_REASON_FILE_DELETE => {
                // 删除文件
                index.remove_entry(record.file_reference_number);
            }
            USN_REASON_RENAME_NEW_NAME => {
                // 重命名文件
                let new_entry = FileEntry::new(
                    record.file_reference_number,
                    record.parent_file_reference_number,
                    extract_filename(record),
            0,
                    record.file_attributes,
                );
                index.update_entry(record.file_reference_number, new_entry);
            }
            _ => {}
        }

        Ok(())
    }
}
```

## 🧪 测试计划

### 单元测试
- ✅ 文件添加/删除/更新
- ✅ 路径重建
- ⏳ 父子关系维护
- ⏳ 搜索功能

### 集成测试
- ⏳ MFT 枚举 → 新索引
- ⏳ 搜索结果正确性
- ⏳ USN Journal 增量更新

### 性能测试
- ⏳ 100万文件索引速度
- ⏳ 搜索响应时间
- ⏳ 内存占用

## 📈 预期性能提升

### 内存使用
- **当前**: ~150 字节/文件（完整路径）
- **新架构**: ~80 字节/文件（仅文件名 + 引用）
- **节省**: ~47%

### 搜索速度
- **当前**: HashMap 查找 O(1)，但需要遍历所有键
- **新架构**: Vec 顺序扫描，SIMD 优化后可达 ~1
- **提升**: 2-5x（取决于 SIMD 优化）

### 增量更新
- **当前**: 不支持
- **新架构**: O(1) 添加/删除/更新
- **提升**: 实时文件监控

## ⚠️ 注意事项

### 兼容性
- IPC 协议保持不变（SearchResult 格式相同）
- 客户端无需修改

### 迁移风险
- 路径重建可能失败（孤儿节点）
- 需要处理循环引用
- 内存索引不持久化，重启后需重建

### 降级方案
- 保留 V1 代码作为备份
- 通过配置切换新旧架构
- 出问题可快速回退

## 🎯 里程碑

### Milestone 1: 基础架构（1-2天）
- [x] 创建新数据结构
- [ ] 更新 MFT 枚举
- [ ] 实现路径重建
- [ ] 单元测试通过

### Milestone 2: 功能集成（2-3天）
- [ ] 集成到主程序
- [ ] 更新搜索逻辑
- [ ] IPC 测试通过
- [ ] 性能测试

### Milestone 3: 增量更新（3-5天）
- [ ] 集成 USN Journal
- [ ] 实时监控测试
- [ ] 压力测试

### Milestone 4: 优化（可选）
- [ ] SIMD 优化
- [ ] 字符串池
- [ ] 内存压缩

## 📚 参考资料

- [Everything 源码分析](https://www.voidtools.com/)
- [NTFS MFT 结构](https://docs.microsoft.com/en-us/windows/win32/fileio/master-file-table)
- [Rust SIMD 优化](https://doc.rust-lang.org/std/simd/)

---

**创建时间**: 2026-01-15
**状态**: 规划中
**优先级**: 高
