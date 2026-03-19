# MultiSerial

[![Rust](https://img.shields.io/badge/language-Rust-orange.svg)](https://www.rust-lang.org/)
[![Version](https://img.shields.io/badge/version-0.1.0-brightgreen.svg)](https://github.com/your-username/multi_serial/releases)
[![egui](https://img.shields.io/badge/ui-egui-blue.svg)](https://github.com/emilk/egui)
[![License](https://img.shields.io/badge/license-MIT-green.svg)](LICENSE)

**MultiSerial** 是一款基于 Rust 和 egui 开发的高性能多串口监控与调试工具。它旨在为嵌入式开发者提供一个现代、简洁且高效的串口工作环境，支持同时监控多个串口、自动重连、以及多种数据格式化功能。

**MultiSerial** is a high-performance multi-port serial monitoring and debugging tool built with Rust and egui. It provides embedded developers with a modern, clean, and efficient workspace, supporting simultaneous monitoring of multiple ports, auto-reconnection, and various data formatting features.

---

## ✨ 主要特性 | Key Features

- **多串口并发监控 (Multi-Port Monitoring)**: 允许同时打开并独立控制多个串口，通过标签页或侧边栏快速切换。
- **自动重连 (Auto-Reconnection)**: 智能识别设备拔插，物理连接恢复后自动尝试重新打开串口。
- **高级日志显示 (Advanced Log Display)**:
  - **HEX 视图 (Hex View)**: 一键切换查看原始二进制数据。
  - **JSON 格式化 (JSON Formatting)**: 自动检测输出中的 JSON 字符串并进行格式化美化打印。
  - **实时时间戳 (Timestamps)**: 为每条接收到的消息提供精确的时间戳。
  - **ANSI 过滤 (ANSI Filtering)**: 自动过滤 ANSI 转义代码，保持日志整洁。
- **多编码支持 (Multiple Encodings)**: 支持 UTF-8, GBK, ASCII 以及 ISO-8859-1 (Latin1)，解决乱码困扰。
- **全局搜索 (Global Search)**: 内置搜索功能 (Ctrl+F)，支持高亮显示与结果跳转。
- **灵活的数据发送 (Data Sending)**: 支持发送文本、HEX 字节流，可自定义换行符 (CRLF, LF, CR, None)。
- **持久化配置 (Persistent Settings)**: 自动保存用户喜好的字体、字号、配色及界面布局。
- **护眼主题 (Modern Theme)**: 采用 Catppuccin-Mocha 风格的暗色主题，长时间使用不疲劳。
- **日志导出 (Log Export)**: 支持将监控历史一键导出为本地文件。

---

## 📸 界面预览 | Screenshots
<img width="975" height="639" alt="image" src="https://github.com/user-attachments/assets/77c4dbc6-aeb7-445b-9493-501069b96dc7" />


## 🛠️ 编译与运行 | Build & Run

### 前置条件 | Prerequisites

- 已安装 [Rust](https://rustup.rs/) (建议使用最新的稳定版 Stable channel)。
- Installed [Rust](https://rustup.rs/) (Stable channel recommended).

### 1. Windows

在 Windows 环境下，直接通过 Cargo 编译即可：

```powershell
# 克隆仓库 | Clone repository
git clone https://github.com/your-username/multi_serial.git
cd multi_serial

# 开发模式运行 | Run in dev mode
cargo run

# 构建发布版本 | Build release version
cargo build --release
```
编译产物位于: `target/release/multi_serial.exe`

### 2. Linux (Ubuntu/Debian)

在 Linux 上，您需要安装 `libudev` 开发库以及一些 UI 相关的依赖：

```bash
# 安装依赖 | Install dependencies
sudo apt update
sudo apt install libudev-dev pkg-config libx11-dev libxcursor-dev libxinerama-dev libxrandr-dev libxi-dev libgl1-mesa-dev

# 编译 | Build
cargo build --release
```
编译产物位于: `target/release/multi_serial`

### 3. macOS

macOS 通常不需要额外的系统库：

```bash
# 源码编译 | Build from source
cargo build --release
```
编译产物位于: `target/release/multi_serial`

---

## 🚀 快速上手 | Quick Start

1. **刷新串口**: 点击左侧面板顶部的刷新按钮 (⟳) 获取当前可用串口列表。
2. **勾选与打开**: 在列表中勾选您想连接的串口，点击 "▶ Open" 按钮一键打开。
3. **配置参数**: 点击串口名旁边的齿轮 (⚙) 按钮，可实时修改波特率、校验位等。
4. **切换视图**: 通过顶部菜单 "View" 或工具栏，快速开启 HEX、JSON 格式化或时间戳。
5. **数据交互**: 在底部的发送栏输入数据（或 HEX），点击发送（支持历史记录回溯）。

---

## ⚙️ 配置文件 | Configuration

程序在启动时会读取/创建根目录下的 `config.json` 文件，用于保存用户的偏好设置，例如：
- 默认使用的字符编码 (Charset)
- 界面字体及大小
- 是否开启自动滚动
- 换行符偏好

---

## 🤝 贡献 | Contributing

欢迎提交 Issue 汇报问题或提交 Pull Request 改进代码。
Feel free to open issues or submit pull requests to improve the tool.

---

## 📄 开源协议 | License

本项目采用 [MIT License](LICENSE) 协议开源。

Designed with ❤️ using Rust & egui.
