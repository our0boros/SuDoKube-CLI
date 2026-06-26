# SuDoKube

> **3D 数独立方体 — 在立方体表面玩数独**

[![Rust](https://img.shields.io/badge/Rust-2024%20edition-orange.svg)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/our0b/SuDoKube/blob/main/LICENSE)

SuDoKube 是一款终端 3D 数独游戏。玩家在由 6 个 9×9 面组成的立方体表面解题，每个面是标准数独，相邻面共享边和角点，共 386 个格子。

---

## 功能特性

- **3D 立方体谜题**: 6 个相互关联的面，共 386 个格子
- **跨面约束**: 相邻面共享边和角点，为传统数独增添空间维度
- **多语言支持**: 中文、English、日本語（自动检测系统语言）
- **生动命名**: 游戏 ID 可使用趣味命名，如"兴奋的熊猫#1"或简单数字编号
- **自动存档**: 进度自动保存至 SQLite 数据库
- **导入/导出**: 支持通过剪贴板分享谜题，采用 XOR+Base64 加密
- **难度等级**: 简单、中等、困难，含不同初始提示数量
- **3D 预览**: 实时显示当前进度的立方体预览
- **主题自定义**: 可调整颜色、大小和视觉偏好

---

## 快速开始

### 环境要求

- Rust 1.75+ (支持 2024 edition)
- Cargo

### 构建与运行

```bash
# 克隆仓库
git clone https://github.com/yourusername/SuDoKube.git
cd SuDoKube

# 构建发布版本
cargo build --release

# 运行游戏
cargo run -p sudokube-cli
```

---

## 操作说明

### 主菜单

| 按键 | 功能 |
|------|------|
| ↑/↓ | 导航菜单 |
| Enter | 确认选择 |
| D | 删除选中的存档 |
| E | 导出选中的对局 |
| I | 导入对局 |
| Q | 退出游戏 |

### 游戏中

| 按键 | 功能 |
|------|------|
| 1-9 | 填入数字 |
| Backspace/Delete | 擦除数字 |
| W/A/S/D | 移动光标 |
| ↑/↓/←/→ | 切换面 |
| M | 切换渲染模式（网格/紧凑） |
| G | 切换辅助模式 |
| H | 显示当前格提示 |
| Z | 撤销 |
| N | 新游戏 |
| Q | 返回菜单 |
| Alt+H | 调试：填充当前面（需开启调试模式） |

---

## 项目结构

```
SuDoKube/
├── core/               # 核心游戏逻辑库
│   └── src/
│       ├── cube.rs       # 立方体坐标、网格、面定义
│       ├── game_state.rs # 游戏状态管理
│       ├── puzzle.rs     # 谜题生成与难度控制
│       ├── wfc.rs        # 波函数坍缩算法
│       └── theme.rs      # 主题配置
├── cli/                # 终端 UI 客户端
│   └── src/
│       ├── main.rs       # 应用入口、状态管理
│       ├── render.rs     # UI 渲染（ratatui）
│       ├── input.rs      # 事件处理
│       ├── i18n.rs       # 多语言翻译
│       ├── widgets.rs    # 自定义 TUI 组件
│       └── save.rs       # 存档持久化（SQLite）
├── assets/             # 字体和图标
└── doc/                # 文档（中文）
```

---

## 技术栈

- **Rust** 2024 edition
- **ratatui** — 终端 UI 框架
- **crossterm** — 终端输入输出
- **rusqlite** — SQLite 数据库存档
- **chrono** — 日期时间处理

---

## 难度等级

| 等级 | 说明 |
|------|------|
| 简单 (Easy) | 更多初始提示 |
| 中等 (Medium) | 标准难度 |
| 困难 (Hard) | 较少初始提示 |

---

## 文档

更多详细信息请参阅 [英文版 README](../README.md)。

---

## 许可证

本项目基于 MIT 许可证开源。详见 [LICENSE](../LICENSE)。
