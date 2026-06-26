# SuDoKube

> **3D Sudoku on a Cube — Play Sudoku across six faces of a cube**

[![Rust](https://img.shields.io/badge/Rust-2024%20edition-orange.svg)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

SuDoKube is a terminal-based 3D Sudoku game where players solve puzzles across the six faces of a cube. Each face is a standard 9×9 Sudoku grid, and adjacent faces share edges and corner cells, creating a unique spatial puzzle experience.

---

## Features

- **3D Cube Puzzle**: 386 cells across 6 interconnected faces
- **Cross-face Constraints**: Adjacent faces share edges and corners, adding a new dimension to traditional Sudoku
- **Multi-language Support**: English, 中文, 日本語 (auto-detected)
- **Lively Naming**: Game IDs can use playful names like "Excited Panda #1" or simple numeric IDs
- **Auto-save**: Progress is automatically saved to SQLite
- **Import/Export**: Share puzzles via clipboard with XOR+Base64 encryption
- **Difficulty Levels**: Easy, Medium, Hard with varying initial clues
- **3D Preview**: Real-time cube preview showing current progress
- **Theme Customization**: Adjust colors, sizes, and visual preferences

---

## Quick Start

### Prerequisites

- Rust 1.75+ (2024 edition support)
- Cargo

### Build & Run

```bash
# Clone the repository
git clone https://github.com/yourusername/SuDoKube.git
cd SuDoKube

# Build release version
cargo build --release

# Run the game
cargo run -p sudokube-cli
```

---

## Controls

### Main Menu

| Key | Action |
|-----|--------|
| ↑/↓ | Navigate menu |
| Enter | Confirm selection |
| D | Delete selected save |
| E | Export selected game |
| I | Import game |
| Q | Quit |

### In-Game

| Key | Action |
|-----|--------|
| 1-9 | Input number |
| Backspace/Delete | Erase number |
| W/A/S/D | Move cursor |
| ↑/↓/←/→ | Switch faces |
| M | Toggle render mode (grid/compact) |
| G | Toggle guide mode |
| H | Show hint for current cell |
| Z | Undo |
| N | New game |
| Q | Return to menu |
| Alt+H | Debug: fill current face (requires Debug Mode) |

---

## Project Structure

```
SuDoKube/
├── core/               # Core game logic library
│   └── src/
│       ├── cube.rs       # Cube coordinates, grid, face definitions
│       ├── game_state.rs # Game state management
│       ├── puzzle.rs     # Puzzle generation & difficulty
│       ├── wfc.rs        # Wave Function Collapse algorithm
│       └── theme.rs      # Theme configuration
├── cli/                # Terminal UI client
│   └── src/
│       ├── main.rs       # Application entry point
│       ├── render.rs     # UI rendering (ratatui)
│       ├── input.rs      # Event handling
│       ├── i18n.rs       # Internationalization
│       ├── widgets.rs    # Custom TUI widgets
│       └── save.rs       # Persistence (SQLite)
├── assets/             # Fonts and icons
└── doc/                # Documentation (Chinese)
```

---

## Technical Stack

- **Rust** 2024 edition
- **ratatui** — Terminal UI framework
- **crossterm** — Terminal input/output
- **rusqlite** — SQLite database for saves
- **chrono** — Date/time handling

---

## Difficulty Levels

| Level | Description |
|-------|-------------|
| Easy | More initial clues given |
| Medium | Standard difficulty |
| Hard | Fewer initial clues |

---

## Documentation

For detailed documentation in Chinese, please refer to [doc/README_zh.md](doc/README_zh.md).

---

## License

This project is licensed under the MIT License. See [LICENSE](LICENSE) for details.
