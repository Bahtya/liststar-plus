1️⃣ UI 线的职责边界（必须严格遵守）

你只负责 Qt 前端（C++），你的职责是：

启动 / 连接 Rust 搜索引擎（searchd.exe）

通过 Named Pipe + Length + Protobuf 调用服务

提供最小可用的搜索 UI,QT设计使用QML

展示搜索结果并打开文件

你不允许：

实现任何索引逻辑

解析 NTFS / MFT / USN

做业务判断

修改协议字段

2️⃣ 技术栈锁定

Qt 6.x

C++17

Windows only

Protobuf C++ runtime

Named Pipe（Win32 API）

3️⃣ UI MVP 功能清单（只做这些）
必须实现

单窗口主界面

搜索输入框

搜索按钮（或回车）

搜索结果列表

状态栏（引擎状态 / 文件数）

双击结果打开文件

禁止实现

❌ 多窗口
❌ 设置页
❌ 主题系统
❌ 动画 / 特效
❌ 国际化

4️⃣ UI 布局规范（Claude 友好）
窗口结构（垂直布局）
┌───────────────────────────────┐
│ 🔍 [ 搜索输入框        ] [查找] │
├───────────────────────────────┤
│ 文件名        | 完整路径       │
│--------------------------------│
│ xxx.txt       | C:\...\xxx.txt │
│                               │
│                               │
├───────────────────────────────┤
│ Engine: Connected | Files: 123 │
└───────────────────────────────┘

5️⃣ IPC 协议（只作为 Client 使用）
传输格式（固定）
[4 bytes length][protobuf payload]


length：uint32 little-endian

payload：protobuf binary

Protobuf（只生成 Client 代码）
syntax = "proto3";

package search.ipc;

message PingReq {}
message PingResp {
  string version = 1;
}

message BuildIndexReq {
  repeated string roots = 1;
}

message BuildIndexResp {
  bool success = 1;
  uint64 indexed_files = 2;
}

message SearchReq {
  string keyword = 1;
  uint32 limit = 2;
}

message SearchResp {
  repeated SearchResult results = 1;
}

message SearchResult {
  string path = 1;
  string filename = 2;
}

6️⃣ Qt 工程结构（必须按此拆）
qt_gui/
├── CMakeLists.txt
├── proto/
│   └── search.proto
├── src/
│   ├── main.cpp
│   ├── main_window.h
│   ├── main_window.cpp
│   ├── ipc/
│   │   ├── pipe_client.h
│   │   ├── pipe_client.cpp
│   │   └── ipc_codec.h
│   └── model/
│       └── search_result.h

7️⃣ IPC Client 实现要求（Qt）
PipeClient 行为

使用 CreateFileW 连接 Named Pipe

若连接失败：

尝试启动 searchd.exe

延迟重连

提供同步 API：

bool connect();
QByteArray request(const QByteArray& payload);

Length + Protobuf 编解码
send:
  uint32 length
  protobuf bytes

recv:
  read 4 bytes
  read length bytes


不做 streaming

不做 async pipe

错误直接返回空结果

8️⃣ UI ↔ IPC 调用流程（固定）
启动流程

Qt 启动

尝试连接 searchd

发送 Ping

显示版本号

搜索流程
UI 输入 → SearchReq
→ IPC Client
→ SearchResp
→ 填充 QTableWidget

9️⃣ 多线程规范（避免 UI 卡死）

IPC 请求必须在 QThread / QtConcurrent

UI 线程只负责刷新界面

不允许在 UI 线程读写 Pipe

🔟 打开文件行为（必须实现）

双击结果行

使用 ShellExecuteW

直接打开文件

11️⃣ 输出要求（Claude Code）

请 按文件逐个生成完整可编译代码，顺序如下：

CMakeLists.txt

search.proto

IPC Client（pipe_client）

主窗口 UI

main.cpp

12️⃣ UI/UX 风格说明（供 UI/UX Pro Max 使用）

极简

偏工具型

类 Everything / Spotlight

不做装饰

所有控件默认系统样式

13️⃣ 成功标准（UI 线 DoD）

Qt 程序能独立运行

能自动拉起 searchd.exe

能搜索并显示结果

双击可打开文件

UI 无明显卡顿