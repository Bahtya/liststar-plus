# Listory Search UI 构建指南

## 前置要求

### 1. 安装 Qt 6
下载并安装 Qt 6.x (推荐 6.5 或更高版本)：
- 官网：https://www.qt.io/download
- 选择 MSVC 2019 或 2022 版本
- 确保安装以下组件：
  - Qt Quick
  - Qt Concurrent
  - CMake 支持

### 2. 安装 Protobuf
有两种方式安装 Protobuf：

#### 方式 A：使用 vcpkg（推荐）
```bash
# 安装 vcpkg
git clone https://github.com/Microsoft/vcpkg.git
cd vcpkg
.\bootstrap-vcpkg.bat

# 安装 protobuf
.\vcpkg install protobuf:x64-windows
```

#### 方式 B：手动下载
- 下载预编译版本：https://github.com/protocolbuffers/protobuf/releases
- 解压到指定目录（如 `C:\protobuf`）

### 3. 安装 CMake
- 下载：https://cmake.org/download/
- 版本要求：3.16 或更高

### 4. 安装 Visual Studio
- Visual Studio 2019 或 2022
- 安装 "使用 C++ 的桌面开发" 工作负载

## 构建步骤

### 方法 1：使用 CMake 命令行

```bash
# 进入 qt_gui 目录
cd D:\Project\listory-plus\qt_gui

# 创建构建目录
mkdir build
cd build

# 配置项目（根据实际路径修改）
cmake .. ^
  -DCMAKE_PREFIX_PATH="C:/Qt/6.5.0/msvc2019_64" ^
  -DCMAKE_TOOLCHAIN_FILE="C:/vcpkg/scripts/buildsystems/vcpkg.cmake" ^
  -G "Visual Studio 17 2022" ^
  -A x64

# 构建项目
cmake --build . --config Release

# 运行程序
Release\listory_search.exe
```

### 方法 2：使用 Qt Creator（推荐新手）

1. 打开 Qt Creator
2. 文件 → 打开文件或项目
3. 选择 `D:\Project\listory-plus\qt_gui\CMakeLists.txt`
4. 配置项目（选择 Qt 6 Kit）
5. 点击左下角的 "构建" 按钮（锤子图标）
6. 点击 "运行" 按钮（绿色三角形）

### 方法 3：使用 Visual Studio

```bash
# 生成 Visual Studio 解决方案
cd D:\Project\listory-plus\qt_gui
mkdir build
cd build

cmake .. ^
  -DCMAKE_PREFIX_PATH="C:/Qt/6.5.0/msvc2019_64" ^
  -G "Visual Studio 17 2022"

# 打开生成的 .sln 文件
start listory_search.sln
```

然后在 Visual Studio 中：
1. 右键点击 `listory_search` 项目
2. 选择 "设为启动项目"
3. 按 F5 运行

## 常见问题

### 问题 1：找不到 Qt
**错误信息**：`Could not find a package configuration file provided by "Qt6"`

**解决方案**：
```bash
# 设置 CMAKE_PREFIX_PATH 指向 Qt 安装目录
cmake .. -DCMAKE_PREFIX_PATH="C:/Qt/6.5.0/msvc2019_64"
```

### 问题 2：找不到 Protobuf
**错误信息**：`Could not find a package configuration file provided by "Protobuf"`

**解决方案**：
```bash
# 使用 vcpkg
cmake .. -DCMAKE_TOOLCHAIN_FILE="C:/vcpkg/scripts/buildsystems/vcpkg.cmake"

# 或手动指定 Protobuf 路径
cmake .. -DProtobuf_DIR="C:/protobuf/cmake"
```

### 问题 3：编译错误 "QByteArray"
**解决方案**：确保在 pipe_client.cpp 中有正确的 include：
```cpp
#include <QByteArray>
```

### 问题 4：运行时找不到 DLL
**解决方案**：
```bash
# 使用 windeployqt 复制必要的 Qt DLL
cd build\Release
C:\Qt\6.5.0\msvc2019_64\bin\windeployqt.exe listory_search.exe
```

## 部署

### 创建可分发版本

```bash
# 1. 构建 Release 版本
cmake --build . --config Release

# 2. 创建部署目录
mkdir deploy
copy Release\listory_search.exe deploy\

# 3. 复制 Qt 依赖
cd deploy
C:\Qt\6.5.0\msvc2019_64\bin\windeployqt.exe listory_search.exe

# 4. 复制 searchd.exe（Rust 搜索引擎）
copy ..\..\searchd\target\release\searchd.exe .

# 5. 复制 VC++ 运行时（如果需要）
# 从 Visual Studio 安装目录复制 vcruntime140.dll 等
```

## 验证构建

运行程序后应该看到：
1. 窗口标题：Listory Search
2. 搜索框和查找按钮
3. 结果列表（初始为空）
4. 状态栏显示：`Engine: Connected (vX.X.X)` 或 `Engine: Disconnected`

如果状态栏显示 "Disconnected"：
- 确保 `searchd.exe` 在同一目录
- 检查 Named Pipe 名称是否匹配：`\\.\pipe\listory_search`

## 开发模式

### 启用调试输出
在 `main.cpp` 中添加：
```cpp
#include <QLoggingCategory>

int main(int argc, char *argv[]) {
    QLoggingCategory::setFilterRules("*.debug=true");
    // ...
}
```

### 热重载 QML
修改 QML 文件后，程序会自动重新加载（Qt 6 特性）。

## 下一步

1. 确保 Rust 搜索引擎 `searchd.exe` 已构建
2. 将 `searchd.exe` 放在与 `listory_search.exe` 相同的目录
3. 运行 UI 程序会自动启动搜索引擎
4. 输入关键词测试搜索功能
