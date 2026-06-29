# BadNorth 存档修改器 (BadNorthSaveModifier)

[English](#english) | [中文](#中文)

## 中文

### 项目介绍

**BadNorthSaveModifier** 是一个功能强大的 GUI 应用程序，专为 *Bad North* 游戏设计，用于快速、方便地修改游戏存档数据。该工具提供了直观的用户界面，使玩家能够轻松管理英雄、升级、背包物品和其他游戏数据。存档转换功能已内置，无需额外安装外部工具。

### 主要功能

- **英雄管理**
  - 查看所有已招募的英雄信息
  - 编辑英雄属性（等级、经验值等）
  - 管理英雄状态
  - 修改英雄兵种（Class）、装备（Item）、特质（Trait）

- **升级系统**
  - 圣杯升级 (Grail Upgrade)
  - 炸弹升级 (Bomb Upgrade)
  - 地雷升级 (Mine Upgrade)
  - 哲学家之石升级 (Philosopher's Stone Upgrade)
  - 体型升级 (Size Upgrade)
  - 战锤升级 (Warhammer Upgrade)
  - 丰收角升级 (Cornucopia Upgrade)
  - 战号升级 (War Horn Upgrade)

- **背包管理**
  - 查看和编辑背包物品数量
  - 自动背包容量检查（最大 20 个物品）
  - 快速增加/减少物品
  - 自定义物品添加

- **Mod 支持**
  - 魔改版专属装备与特质
  - 融合版专属装备与特质
  - 旧灰复燃的战旗专属特质

- **界面特性**
  - 中文与英文双语支持
  - 三种色彩模式：黑色、彩色、跟随系统
  - 主题切换平滑过渡动画
  - 设置自动保存和恢复
  - 友好的错误提示与操作日志
  - 导出存档为 JSON

### 技术栈

- **语言**: Rust (edition 2021)
- **UI 框架**: egui 0.24 / eframe 0.24
- **序列化**: serde, serde_json
- **其他**:
  - walkdir (文件遍历)
  - rfd (文件选择对话框)
  - anyhow, thiserror (错误处理)
  - log, env_logger (日志记录)
  - paste (宏辅助)

### 项目结构

```
BadNorthSaveModifier/
├── src/
│   ├── main.rs                 # 应用入口和主逻辑 (~1887 行)
│   ├── lib.rs                  # 库入口
│   ├── models.rs               # 数据模型
│   ├── save_manager.rs         # 存档读写管理
│   ├── settings.rs             # 应用设置
│   ├── class_dictionary.rs     # 兵种字典
│   ├── upgrade_dictionary.rs   # 升级字典
│   └── ui/
│       ├── mod.rs              # UI 模块入口
│       ├── styles.rs           # UI 样式定义
│       └── components/
│           └── mod.rs          # UI 组件
├── Cargo.toml                  # 项目配置和依赖
├── Cargo.lock                  # 依赖锁定文件
├── LICENSE                     # 许可证
├── .gitignore                  # Git 忽略规则
└── README.md                   # 项目说明文档
```

### 安装与编译

#### 前置要求
- Rust 1.56+（推荐最新稳定版）
- Cargo

#### 编译步骤

1. **克隆或下载项目**
   ```bash
   git clone https://github.com/ABaLaQiYaShanMaiI/BadNorthSaveModifier.git
   cd BadNorthSaveModifier
   ```

2. **编译项目**
   ```bash
   cargo build --release
   ```

3. **运行应用**
   ```bash
   cargo run --release
   ```

编译后的可执行文件位于 `target/release/BadNorthSaveModifier.exe`

### 使用方法

1. **启动应用**
   - 运行编译后的 `BadNorthSaveModifier.exe`

2. **选择存档文件**
   - 应用启动后，浏览并选择要编辑的 Bad North 游戏存档文件
   - 存档转换由内置转换器自动完成，无需额外工具

3. **编辑存档**
   - 在左侧菜单中选择要修改的功能（设置、指挥官、货币与物品）
   - 在右侧面板中修改英雄、升级、物品等数据
   - 修改实时预览，操作日志即时反馈

4. **保存修改**
   - 点击「备份并替换存档」按钮将修改后的数据写回存档文件
   - 应用会自动备份原存档，防止数据丢失

5. **导出 JSON**
   - 可以将当前存档数据导出为 JSON 文件，便于查看或备份

### 核心模块详解

#### `main.rs` (~1887 行)
应用程序的主入口和核心 UI 逻辑，包含：
- 应用状态管理（选择存档 / 加载存档 / 编辑存档）
- 色彩主题系统（含平滑过渡动画）
- 中英文国际化翻译函数
- Windows/macOS/Linux 系统暗黑模式检测

#### `save_manager.rs`
包含存档文件的读写操作，主要分为以下功能模块：
- 文件 I/O 和序列化 / 反序列化
- 英雄数据查询与修改
- 货币（coinBank）查询与修改
- 圣杯（Grail）查询与修改
- 背包物品的增删改查
- 快捷操作方法（通过宏生成）

#### `ui/` 目录
包含所有用户界面相关代码，使用 egui 框架提供跨平台支持。

### 配置与设置

应用支持以下设置项（通过 `settings.rs` 管理，自动保存为 `settings.json`）：

- **色彩模式** (ColorMode)
  - 黑色 (Black) — 暗色主题
  - 彩色 (Colorful) — 亮色主题，带蓝色强调色
  - 跟随系统 (FollowSystem) — 自动跟随操作系统的亮/暗模式

- **语言设置** (Language)
  - 中文 (Chinese)
  - 英文 (English)

- **其他首选项**
  - 保持日志面板显示 (Keep Logs Visible)
  - 记忆编辑器 EXE 路径
  - 设置自动保存和恢复

### 故障排除

| 问题 | 解决方案 |
|------|---------|
| 无法加载存档文件 | 确保文件路径正确；检查文件是否损坏 |
| UI 显示不清晰 | 尝试更改主题设置；检查系统 DPI 设置 |
| 编辑后存档无效 | 确保在保存前未关闭游戏；尝试从备份恢复 |
| 中文显示乱码 | 检查系统字体设置；更新应用到最新版本 |

### 开发贡献

欢迎提交 Issue 和 Pull Request！

### 许可证

本项目基于 MIT License 开源。详见 [LICENSE](LICENSE) 文件。

### 作者

**ABaLaQiYaShanMaiI**

### 更新日志

- **v0.1.0** - 初始版本发布

---

## English

### Project Description

**BadNorthSaveModifier** is a powerful GUI application designed for the *Bad North* game to quickly and conveniently modify game save data. The tool provides an intuitive user interface that allows players to easily manage heroes, upgrades, inventory items, and other game data. Save conversion is built-in, no external tools required.

### Key Features

- **Hero Management**
  - View all recruited hero information
  - Edit hero attributes (level, experience, etc.)
  - Manage hero status
  - Modify hero Class, Item, and Trait

- **Upgrade System**
  - Grail Upgrade
  - Bomb Upgrade
  - Mine Upgrade
  - Philosopher's Stone Upgrade
  - Size Upgrade
  - Warhammer Upgrade
  - Cornucopia Upgrade
  - War Horn Upgrade

- **Inventory Management**
  - View and edit inventory item quantities
  - Automatic inventory capacity checks (max 20 items)
  - Quick add/remove items
  - Custom item addition

- **Mod Support**
  - Mod version exclusive equipment and traits
  - Fusion version exclusive equipment and traits
  - Rebirth Flag exclusive traits

- **UI Features**
  - Chinese and English bilingual support
  - Three color modes: Black, Colorful, Follow System
  - Smooth theme transition animations
  - Settings auto-save and restore
  - User-friendly error messages and operation logs
  - Export save data as JSON

### Tech Stack

- **Language**: Rust (edition 2021)
- **UI Framework**: egui 0.24 / eframe 0.24
- **Serialization**: serde, serde_json
- **Other**:
  - walkdir (directory traversal)
  - rfd (file picker dialogs)
  - anyhow, thiserror (error handling)
  - log, env_logger (logging)
  - paste (macro helpers)

### Project Structure

```
BadNorthSaveModifier/
├── src/
│   ├── main.rs                 # Application entry and main logic (~1887 lines)
│   ├── lib.rs                  # Library entry point
│   ├── models.rs               # Data models
│   ├── save_manager.rs         # Save file read/write management
│   ├── settings.rs             # Application settings
│   ├── class_dictionary.rs     # Class type dictionary
│   ├── upgrade_dictionary.rs   # Upgrade dictionary
│   └── ui/
│       ├── mod.rs              # UI module entry
│       ├── styles.rs           # UI styles
│       └── components/
│           └── mod.rs          # UI components
├── Cargo.toml                  # Project configuration and dependencies
├── Cargo.lock                  # Dependency lock file
├── LICENSE                     # License
├── .gitignore                  # Git ignore rules
└── README.md                   # Project documentation
```

### Installation & Compilation

#### Prerequisites
- Rust 1.56+ (latest stable recommended)
- Cargo

#### Build Steps

1. **Clone or download the project**
   ```bash
   git clone https://github.com/ABaLaQiYaShanMaiI/BadNorthSaveModifier.git
   cd BadNorthSaveModifier
   ```

2. **Build the project**
   ```bash
   cargo build --release
   ```

3. **Run the application**
   ```bash
   cargo run --release
   ```

The compiled executable will be at `target/release/BadNorthSaveModifier.exe`

### Usage Guide

1. **Launch the application**
   - Run the compiled `BadNorthSaveModifier.exe`

2. **Select a save file**
   - After launch, browse and select the Bad North save file to edit
   - Save conversion is handled automatically by the built-in converter

3. **Edit the save**
   - Select the function to modify from the left menu (Settings, Commanders, Currency & Items)
   - Modify heroes, upgrades, items, and other data in the right panel
   - Changes are previewed in real-time with instant log feedback

4. **Save changes**
   - Click the "Save & Backup" button to write modified data back to the save file
   - The app automatically creates a backup of the original save

5. **Export JSON**
   - Export the current save data as a JSON file for viewing or backup

### Core Modules Overview

#### `main.rs` (~1887 lines)
The application's main entry point and core UI logic, including:
- Application state management (Select Save / Load Save / Edit Save)
- Color theme system (with smooth transition animations)
- Chinese/English i18n translation functions
- System dark mode detection for Windows/macOS/Linux

#### `save_manager.rs`
Handles save file read/write operations with the following modules:
- File I/O and serialization/deserialization
- Hero data queries and modifications
- Currency (coinBank) queries and modifications
- Grail queries and modifications
- Inventory item CRUD operations
- Shortcut helper methods (generated via macros)

#### `ui/` Directory
Contains all user interface code using the egui framework for cross-platform support.

### Configuration & Settings

The application supports the following settings (managed via `settings.rs`, auto-saved as `settings.json`):

- **Color Mode** (ColorMode)
  - Black — Dark theme
  - Colorful — Light theme with blue accent colors
  - Follow System — Automatically follows the OS light/dark mode

- **Language** (Language)
  - Chinese
  - English

- **Other Preferences**
  - Keep Logs Visible
  - Remember editor EXE path
  - Settings auto-save and restore

### Troubleshooting

| Issue | Solution |
|-------|----------|
| Cannot load save file | Verify file path is correct; check if file is corrupted |
| UI display not clear | Try changing theme settings; check system DPI settings |
| Save file invalid after editing | Ensure game is closed during saving; try restoring from backup |
| Chinese characters display incorrectly | Check system font settings; update app to latest version |

### Contributing

Issues and Pull Requests are welcome!

### License

This project is open-sourced under the MIT License. See the [LICENSE](LICENSE) file for details.

### Author

**ABaLaQiYaShanMaiI**

### Changelog

- **v0.1.0** - Initial release