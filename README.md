<p align="center">
  <img src="flutter/assets/icons/tray_icon.png" width="80" alt="Waylex">
</p>

<h1 align="center">Waylex</h1>

<p align="center">
  AI 驱动的 Linux Wayland 桌面翻译工具 | 截图即译 · 多厂商对比 · 全局快捷键
</p>

<p align="center">
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="License"></a>
  <img src="https://img.shields.io/badge/platform-Linux%20Wayland-orange" alt="Platform">
  <img src="https://img.shields.io/badge/language-Rust%20%2B%20Flutter-purple" alt="Language">
</p>

> 因为 Linux Wayland 没有好用的翻译工具，所以我自己开发了一个。<br>
> 目前只在 **CachyOS + Hyprland** 环境跑通，其他桌面/发行版未充分测试。<br>
> 有 Bug 欢迎提 Issue，但不一定会及时修复 😄

---

## 功能特性

- **多厂商翻译** — OpenAI、DeepL、Google、Qwen、DeepSeek、Kimi、GLM、Anthropic、Azure、Custom，10 家厂商即配即用
- **厂商对比** — 同时对比多家翻译结果，展示响应时间与 Token 消耗
- **截图翻译** — 快捷键框选区域 → OCR 识别 → 自动翻译，一键完成
- **提示词模板** — 自定义多套系统提示词，翻译时一键切换
- **全局快捷键** — Hyprland IPC / KDE KGlobalAccel / evdev 三层自适应，快捷键实时录制修改
- **系统托盘** — 最小化到托盘，左键唤醒，右键快捷菜单
- **浮动窗口** — 无边框置顶，拖拽自由

## 截图

<p align="center">
  <i>翻译主界面 / 厂商设置 / 快捷键录制</i>
</p>

## 安装

### 方式一：AppImage（推荐）

从 [Releases](https://github.com/xiyeming/flutter-translate/releases) 下载最新 `Waylex-*.AppImage`：

```bash
chmod +x Waylex-*.AppImage
./Waylex-*.AppImage
```

系统依赖（目标机器需预装）：

| 包 | 用途 |
|----|------|
| `wl-clipboard` | Wayland 剪贴板 |
| `grim` `slurp` | 截图 + 选区 |
| `tesseract` `tesseract-data-eng` `tesseract-data-chi_sim` | OCR 引擎 + 中英文语言包 |

```bash
# Arch
sudo pacman -S wl-clipboard grim slurp tesseract tesseract-data-eng tesseract-data-chi_sim

# Ubuntu/Debian
sudo apt install wl-clipboard grim slurp tesseract-ocr tesseract-ocr-eng tesseract-ocr-chi-sim

# Fedora
sudo dnf install wl-clipboard grim slurp tesseract tesseract-langpack-eng tesseract-langpack-chi_sim
```

### 全局快捷键权限

Hyprland / Sway 上 evdev 快捷键需要 `input` 组权限：

```bash
sudo usermod -aG input $USER
# 重新登录生效
groups $USER | grep input
```

### Hyprland 窗口规则

将以下内容添加到 `~/.config/hypr/hyprland.conf`（或放在单独文件用 `source` 引入）：

```hypr
# Waylex 浮动窗口规则 (Hyprland 0.54+)
windowrule {
    name = xym-ft-float
    match:class = ^(com.xym.ft)$
    float = true
    size = 400 600
    center = true
    pin = true
    animation = popin 80%
}
```

> 项目内 `hyprland/flutter-translate.conf` 即为可用的独立配置文件

### 方式二：从源码构建

| 依赖 | 版本 |
|------|------|
| Rust | 1.95+ |
| Flutter | 3.41+ |
| CMake / gcc | — |

```bash
# 安装 Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 安装 Flutter (Arch)
sudo pacman -S flutter

# 构建
git clone https://github.com/xiyeming/flutter-translate.git
cd flutter-translate
./scripts/build.sh

# 运行
cd flutter/build/linux/x64/release/bundle && ./run.sh

# 打包 AppImage
./scripts/release.sh
```

## 使用

### 配置 API Key

应用内：**设置 → 厂商 → 点击厂商卡片 → 填写 API Key → 保存**

或设置环境变量：

```bash
export OPENAI_API_KEY="sk-..."
export DEEPSEEK_API_KEY="sk-..."
export DASHSCOPE_API_KEY="sk-..."   # Qwen
export ANTHROPIC_API_KEY="sk-..."
export DEEPL_API_KEY="your-key"
```

### 快捷键

| 快捷键 | 功能 |
|--------|------|
| `Super+Alt+F` | 翻译选中文本（先 Ctrl+C 复制，再按快捷键） |
| `Ctrl+Shift+S` | 截图 OCR → 自动翻译 |
| `Ctrl+Shift+F` | 显示/隐藏窗口 |
| `Enter` | 翻译输入框内容（Shift+Enter 换行） |

> 快捷键可在 **设置 → 快捷键** 中实时录制修改

### 提示词模板

1. 翻译主界面 → 点击提示词选择器旁的 **+**
2. 新增模板：输入名称和提示词内容
3. 激活后自动覆盖所有厂商的 system prompt
4. 下拉切换或恢复 "使用厂商默认提示词"

## 技术栈

| 层 | 技术 |
|----|------|
| UI | Flutter 3.41 + Riverpod + go_router |
| FFI | flutter_rust_bridge 2.12 |
| 后端 | Rust (Tokio, reqwest, sqlx, chrono) |
| 存储 | SQLite + keyring (Wayland 回退 SQLite) |
| 热键 | evdev / Hyprland IPC / KDE KGlobalAccel |
| OCR | tesseract CLI + image crate 预处理 |
| 截图 | grim + slurp (Wayland) |
| 剪贴板 | wl-clipboard |
| 托盘 | tray_manager (AppIndicator) |
| 打包 | AppImage |

## 架构

```
Flutter ——FFI——→ flutter_rust_bridge ——→ translate (10 厂商)
                  ├── config (SQLite + keyring)
                  ├── hotkey (evdev / Hyprland IPC / KDE)
                  ├── ocr (grim + tesseract + image preprocessing)
                  └── clipboard (wl-clipboard)
```

## 项目结构

```
flutter-translate/
├── native/src/           # Rust 后端
│   ├── ffi/bridge.rs     # FFI 接口：翻译/配置/热键/OCR/剪贴板
│   ├── config/           # SQLite 配置 + keyring API Key
│   ├── hotkey/           # evdev + Hyprland IPC + KDE 三层热键
│   ├── ocr/              # 截图 + tesseract + 图像预处理
│   ├── translate/        # 10 厂商翻译引擎
│   └── tray/             # 系统托盘
├── flutter/lib/          # Flutter 前端
│   ├── app/              # 路由 + 主题
│   ├── data/             # 模型 + 数据源 + Repository
│   └── presentation/     # 页面 + HotkeyService
├── scripts/              # build.sh / release.sh
└── hyprland/             # Hyprland 窗口规则
```

## 贡献

欢迎提交 Issue 和 Pull Request。

```bash
# 开发流程
cd native && cargo check && cargo test      # Rust
cd flutter && flutter analyze               # Flutter
./scripts/build.sh                           # 完整构建
```

## License

[MIT](LICENSE) © 2026 [xiyeming](mailto:xiyeming@163.com)
