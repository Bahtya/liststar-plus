# CLAUDE.md

本文件为 Claude Code (claude.ai/code) 在此代码库中工作时提供指导。

## 项目概述

这是 Listory Search 的 Qt 6 GUI 前端，一个 Windows 文件搜索应用程序。该 GUI 是一个**纯客户端**应用程序，通过 Named Pipes 使用 Protobuf 消息与基于 Rust 的搜索引擎（`searchd.exe`）通信。

**关键约束**：此代码库仅负责 UI。不要在这里实现任何索引、NTFS/MFT/USN 解析或搜索逻辑。所有业务逻辑都在 Rust 后端（`../searchd`）中。

## 构建命令

### 前置要求
- **Qt 6.10.1 MSVC 2022 64-bit**（必需 - MinGW 与 vcpkg 的 MSVC 库不兼容）
- **CMake 3.16+**（Qt 自带：`C:\Qt\Tools\CMake_64\bin\cmake.exe`）
- **Visual Studio 2022** 带 C++ 桌面开发工作负载
- **vcpkg** 已为 x64-windows triplet 安装 protobuf 和 abseil
- Protobuf 通过 vcpkg 工具链自动查找

### 快速构建（推荐）

使用提供的构建脚本：
```bash
# 在 qt_gui 目录下
build-msvc.bat
```

此脚本将：
1. 使用 MSVC 工具链配置 CMake
2. 构建 Release 版本
3. 使用 windeployqt 部署 Qt 依赖

### 使用 CMake 手动构建

```bash
# 在 qt_gui 目录下
mkdir build-msvc
cd build-msvc

# 配置
C:\Qt\Tools\CMake_64\bin\cmake.exe .. ^
  -DCMAKE_PREFIX_PATH=C:/Qt/6.10.1/msvc2022_64 ^
  -DCMAKE_TOOLCHAIN_FILE=D:/Project/vcpkg/scripts/buildsystems/vcpkg.cmake ^
  -G "Visual Studio 17 2022" ^
  -A x64

# 构建
C:\Qt\Tools\CMake_64\bin\cmake.exe --build . --config Release

# 部署 Qt 依赖
cd Release
C:\Qt\6.10.1\msvc2022_64\bin\windeployqt.exe listory_search.exe --release
```

### 使用 Qt Creator 构建

1. 在 Qt Creator 中打开 `CMakeLists.txt`
2. 选择 **Desktop Qt 6.10.1 MSVC2022 64bit** Kit（不要选 MinGW）
3. 在项目设置 → 构建 → CMake 中添加：
   - 键：`CMAKE_TOOLCHAIN_FILE`
   - 值：`D:/Project/vcpkg/scripts/buildsystems/vcpkg.cmake`
4. 运行 CMake 并构建

### 运行应用程序

```bash
# 在 qt_gui 目录下
run.bat

# 或手动运行
cd build-msvc\Release
listory_search.exe
```

## 架构

### 高层设计

```
┌─────────────────────────────────────────────────────────────┐
│                    Qt GUI (本仓库)                          │
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
│                 Rust 后端 (searchd.exe)          │          │
│                                                  ▼          │
│  ┌──────────────────────────────────────────────────────┐  │
│  │  IPC 处理器 (Named Pipe 服务器)                      │  │
│  └──────────────────────────────────────────────────────┘  │
│  ┌──────────────────────────────────────────────────────┐  │
│  │  搜索引擎 (MFT/USN 索引、搜索逻辑)                   │  │
│  └──────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
```

### 组件职责

**QML UI (`qml/main.qml`)**
- 单窗口界面，包含搜索输入框、结果 ListView 和状态栏
- 将所有逻辑委托给 SearchBackend
- 使用 Qt Quick Controls 实现简洁的工具型 UI（类似 Everything/Spotlight）

**SearchBackend (`src/search_backend.{h,cpp}`)**
- QAbstractListModel，向 QML 暴露搜索结果
- 管理 PipeClient 生命周期
- 通过 QtConcurrent 在后台线程运行 IPC 操作以防止 UI 阻塞
- 与 Rust 后端之间编组 Protobuf 消息
- 通过 ShellExecuteW 处理文件打开

**PipeClient (`src/ipc/pipe_client.{h,cpp}`)**
- 使用 Win32 API 的 Windows Named Pipe 客户端（CreateFileW、ReadFile、WriteFile）
- 如果管道连接失败，自动启动 `searchd.exe`
- 同步请求/响应模式（无流式传输）
- 管道名称：`\\.\pipe\listory_search`

**IPC Codec (`src/ipc/ipc_codec.h`)**
- 仅头文件的消息编码/解码工具
- 传输格式：`[4 字节长度 (小端 uint32)][protobuf 载荷]`
- 无类型字节前缀（与某些 IPC 协议不同）

### IPC 协议

在 `proto/search.proto` 中定义：

**消息：**
- `PingReq/PingResp` - 健康检查，返回后端版本
- `BuildIndexReq/BuildIndexResp` - 索引构建（UI 当前未使用）
- `SearchReq/SearchResp` - 文件搜索，带关键词和限制
- `SearchResult` - 包含 `filename` 和 `path` 字段

**传输格式：**
```
[4 字节：载荷长度 (uint32 小端)] [N 字节：protobuf 二进制]
```

Qt 客户端发送请求并阻塞等待响应。所有 IP台线程中进行，以保持 UI 响应。

### 线程模型

- **UI 线程**：QML 渲染、用户输入、模型更新
- **后台线程**（QtConcurrent::run）：所有 PipeClient 操作
  - `performPing()` - 引擎健康检查
  - `performSearch()` - 搜索请求
- 结果通过 `QMetaObject::invokeMethod(..., Qt::QueuedConnection)` 编组回 UI 线程

**重要**：永远不要从 UI 线程直接调用 PipeClient 方法。

### 启动流程

1. Qt 应用启动，加载 QML
2. SearchBackend 构造函数创建 PipeClient
3. 从 main.cpp 调用 `connectToEngine()`
4. 后台线程尝试管道连接
5. 如果连接失败，PipeClient 通过 CreateProcessW 启动 `searchd.exe`
6. 重试连接最多 3 次，每次延迟 500ms
7. 发送 PingReq 验证连接并获取版本
8. 更新 UI 状态栏显示连接状态

### 搜索流程

1. 用户在搜索框输入并按 Enter 或点击"查找"
2. QML 调用 `searchBackend.search(keyword)`
3. SearchBa后台线程
4. 线程创建 SearchReq protobuf，序列化
5. PipeClient 通过 Named Pipe 发送（带长度前缀）
6. 等待响应（在后台线程中阻塞）
7. 反序列化 SearchResp protobuf
8. 将结果编组到 UI 线程
9. 更新 QAbstractListModel，触发 QML ListView 刷新

## 关键约束和设计决策

### 此代码库不做的事情
- ❌ 文件索引（MFT/USN 解析）
- ❌ 搜索算法或排名
- ❌ 文件系统监控
- ❌ 协议设计更改（proto 文件与 Rust 后端共享）
- ❌ 多窗口 UI、设置页面、主题、动画、国际化

### 此代码库做的事情
- ✅ 最小化搜索 UI（输入框、结果列表、状态栏）
- ✅ Named Pipe 客户端通信
- ✅ Protobuf 消息序列化
- ✅ IPC 的后台线程处理
- ✅ 自动启动 Rust 后端
- ✅ 通过 ShellExecuteW 打开文件

### Protobuf 配置

CMakeLists.txt 通过 vcpkg 自动查找 protobuf。

生成的文件（`search.pb.h`、`search.pb.cc`）在构建目录中创建并自动链接。

## 常见问题

### "找不到 Qt6"
设置 CMAKE_PREFIX_PATH 到你的 Qt 安装目录：
```bash
cmake .. -DCMAKE_PREFIX_PATH="C:/Qt/6.10.1/msvc2022_64"
```

### "找不到 Protobuf"
使用 vcpkg 工具链文件：
```bash
cmake .. -DCMAKE_TOOLCHAIN_FILE="D:/Project/vcpkg/scripts/buildsystems/vcpkg.cmake"
```

### UI 中显示 "Engine: Disconnected"
- 确保 `searchd.exe` 与 `listory_search.exe` 在同一目录
- 检查管道名称是否匹配：`\\.\pipe\listory_search`
- 验证 Rust 后端正在运行并监听正确的管道

### 运行时 DLL 错误
运行 windeployqt 复制 Qt 依赖：
```bash
C:\Qt\6.10.1\msvc2022_64\bin\windeployqt.exe listory_search.exe
```

## 修改 UI

QML 文件（`qml/main.qml`）定义了整个 UI。关键元素：
- **第 22-40 行**：搜索输入 TextField 和 Button
- **第 49-144 行**：结果 ListView 和自定义委托
- **第 147-182 行**：状态栏显示引擎状态和文件计数

UI 遵循严格的垂直布局，没有选项卡、对话框或辅助窗口。

## 修改 IPC

如果需要更改协议：
1. **不要**在未与 Rust 后端协调的情况下修改 `proto/search.proto`
2. proto 文件在 Qt 和 Rust 代码库之间共享
3. proto 更改后必须重新构建两端
4. 传输格式（长度前缀）是固定的，不能更改

## 文件打开

双击结果调用 `SearchBackend::openFile()`，它使用 Windows ShellExecuteW 用默认应用程序打开文件。这是有意保持简单的 - 没有自定义文件查看器或编辑器。
