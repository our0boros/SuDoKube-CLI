//! 键位映射系统
//!
//! 提供统一的 Action 枚举和 KeyBinding 映射，
//! 替代 input.rs 中分散的硬编码键位。
//!
//! 支持：
//! - 按界面/模式分组的有效键位
//! - 从 DB 加载/保存自定义键位
//! - 在设置中配置映射界面

use crossterm::event::{KeyCode, KeyModifiers};
use serde::{Deserialize, Serialize};

use crate::AppScreen;

/// 可绑定的操作
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Action {
    // ── 通用 ──
    Quit,
    Confirm,
    Cancel,

    // ── 菜单 ──
    MenuUp,
    MenuDown,
    MenuSelect,
    MenuDelete,
    MenuForceDelete,
    MenuExport,
    MenuImport,

    // ── 游戏-光标移动 ──
    CursorUp,
    CursorDown,
    CursorLeft,
    CursorRight,

    // ── 游戏-面切换 ──
    FaceUp,
    FaceDown,
    FaceLeft,
    FaceRight,

    // ── 游戏-面直跳 ──
    FaceFront,
    FaceBack,
    FaceLeftJump,
    FaceRightJump,
    FaceTop,
    FaceBottom,

    // ── 游戏-操作 ──
    Number(u8),
    Erase,
    Hint,
    Undo,
    ToggleGuidance,
    ToggleMode,
    NewGame,

    // ── 按钮 ──
    BtnPagePrev,
    BtnPageNext,

    // ── 商店 ──
    ShopFocus,
    ShopUp,
    ShopDown,
    ShopBuy,

    // ── 贪吃蛇 ──
    SnakeUp,
    SnakeDown,
    SnakeLeft,
    SnakeRight,
    SnakeQuit,

    // ── 设置 ──
    SettingsUp,
    SettingsDown,
    SettingsLeft,
    SettingsRight,

    // ── 工具快捷键 ──
    ToolCube,
    ToolSnake3,
    ToolFace,
    ToolSnake5,
    ToolTarget,

    // ── 调试 ──
    DebugHintFace,
    DebugWin,
    DebugGoldUp,
    DebugGoldDown,

    // ── 导入/导出 ──
    ExportUp,
    ExportDown,
    ImportChar,
    ImportBackspace,
}

impl Action {
    /// 操作的显示名称（用于设置界面）
    pub fn display_name(&self) -> &'static str {
        match self {
            Action::Quit => "Quit",
            Action::Confirm => "Confirm",
            Action::Cancel => "Cancel",
            Action::MenuUp => "Menu Up",
            Action::MenuDown => "Menu Down",
            Action::MenuSelect => "Menu Select",
            Action::MenuDelete => "Delete Game",
            Action::MenuForceDelete => "Force Delete",
            Action::MenuExport => "Export",
            Action::MenuImport => "Import",
            Action::CursorUp => "Cursor Up",
            Action::CursorDown => "Cursor Down",
            Action::CursorLeft => "Cursor Left",
            Action::CursorRight => "Cursor Right",
            Action::FaceUp => "Face Up",
            Action::FaceDown => "Face Down",
            Action::FaceLeft => "Face Left",
            Action::FaceRight => "Face Right",
            Action::FaceFront => "Jump Front",
            Action::FaceBack => "Jump Back",
            Action::FaceLeftJump => "Jump Left",
            Action::FaceRightJump => "Jump Right",
            Action::FaceTop => "Jump Top",
            Action::FaceBottom => "Jump Bottom",
            Action::Number(n) => match n {
                1 => "Number 1",
                2 => "Number 2",
                3 => "Number 3",
                4 => "Number 4",
                5 => "Number 5",
                6 => "Number 6",
                7 => "Number 7",
                8 => "Number 8",
                9 => "Number 9",
                _ => "Number ?",
            },
            Action::Erase => "Erase",
            Action::Hint => "Hint",
            Action::Undo => "Undo",
            Action::ToggleGuidance => "Toggle Guide",
            Action::ToggleMode => "Toggle Mode",
            Action::NewGame => "New Game",
            Action::BtnPagePrev => "Btn Page ←",
            Action::BtnPageNext => "Btn Page →",
            Action::ShopFocus => "Shop Focus",
            Action::ShopUp => "Shop Up",
            Action::ShopDown => "Shop Down",
            Action::ShopBuy => "Shop Buy",
            Action::SnakeUp => "Snake Up",
            Action::SnakeDown => "Snake Down",
            Action::SnakeLeft => "Snake Left",
            Action::SnakeRight => "Snake Right",
            Action::SnakeQuit => "Snake Quit",
            Action::SettingsUp => "Settings Up",
            Action::SettingsDown => "Settings Down",
            Action::SettingsLeft => "Settings Left",
            Action::SettingsRight => "Settings Right",
            Action::ToolCube => "Tool: Cube",
            Action::ToolSnake3 => "Tool: Snake×3",
            Action::ToolFace => "Tool: Face",
            Action::ToolSnake5 => "Tool: Snake×5",
            Action::ToolTarget => "Tool: Target",
            Action::DebugHintFace => "Debug: Hint Face",
            Action::DebugWin => "Debug: Win",
            Action::DebugGoldUp => "Debug: Gold +100",
            Action::DebugGoldDown => "Debug: Gold -50",
            Action::ExportUp => "Export Up",
            Action::ExportDown => "Export Down",
            Action::ImportChar => "Import: Type",
            Action::ImportBackspace => "Import: Backspace",
        }
    }

    /// 操作所属的分组（用于设置界面分类显示）
    pub fn group(&self) -> ActionGroup {
        match self {
            Action::Quit | Action::Confirm | Action::Cancel => ActionGroup::General,
            Action::MenuUp
            | Action::MenuDown
            | Action::MenuSelect
            | Action::MenuDelete
            | Action::MenuForceDelete
            | Action::MenuExport
            | Action::MenuImport => ActionGroup::Menu,
            Action::CursorUp
            | Action::CursorDown
            | Action::CursorLeft
            | Action::CursorRight
            | Action::FaceUp
            | Action::FaceDown
            | Action::FaceLeft
            | Action::FaceRight => ActionGroup::Navigation,
            Action::FaceFront
            | Action::FaceBack
            | Action::FaceLeftJump
            | Action::FaceRightJump
            | Action::FaceTop
            | Action::FaceBottom => ActionGroup::FaceJump,
            Action::Number(_)
            | Action::Erase
            | Action::Hint
            | Action::Undo
            | Action::ToggleGuidance
            | Action::ToggleMode
            | Action::NewGame => ActionGroup::Gameplay,
            Action::BtnPagePrev | Action::BtnPageNext => ActionGroup::ButtonBar,
            Action::ShopFocus | Action::ShopUp | Action::ShopDown | Action::ShopBuy => {
                ActionGroup::Shop
            }
            Action::SnakeUp
            | Action::SnakeDown
            | Action::SnakeLeft
            | Action::SnakeRight
            | Action::SnakeQuit => ActionGroup::Snake,
            Action::SettingsUp
            | Action::SettingsDown
            | Action::SettingsLeft
            | Action::SettingsRight => ActionGroup::Settings,
            Action::ToolCube
            | Action::ToolSnake3
            | Action::ToolFace
            | Action::ToolSnake5
            | Action::ToolTarget => ActionGroup::Tools,
            Action::DebugHintFace
            | Action::DebugWin
            | Action::DebugGoldUp
            | Action::DebugGoldDown => ActionGroup::Debug,
            Action::ExportUp | Action::ExportDown => ActionGroup::Export,
            Action::ImportChar | Action::ImportBackspace => ActionGroup::Import,
        }
    }
}

/// 操作分组
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ActionGroup {
    General,
    Menu,
    Navigation,
    FaceJump,
    Gameplay,
    ButtonBar,
    Shop,
    Snake,
    Settings,
    Tools,
    Debug,
    Export,
    Import,
}

/// 修饰键组合（可序列化的版本）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ModKey {
    pub shift: bool,
    pub alt: bool,
    pub ctrl: bool,
}

impl From<KeyModifiers> for ModKey {
    fn from(m: KeyModifiers) -> Self {
        Self {
            shift: m.contains(KeyModifiers::SHIFT),
            alt: m.contains(KeyModifiers::ALT),
            ctrl: m.contains(KeyModifiers::CONTROL),
        }
    }
}

impl From<ModKey> for KeyModifiers {
    fn from(m: ModKey) -> Self {
        let mut mods = KeyModifiers::NONE;
        if m.shift {
            mods.insert(KeyModifiers::SHIFT);
        }
        if m.alt {
            mods.insert(KeyModifiers::ALT);
        }
        if m.ctrl {
            mods.insert(KeyModifiers::CONTROL);
        }
        mods
    }
}

/// 可序列化的 KeyCode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Key {
    Char(char),
    F(u8),
    Enter,
    Esc,
    Backspace,
    Delete,
    Up,
    Down,
    Left,
    Right,
    Home,
    End,
    PageUp,
    PageDown,
    Tab,
    Space,
}

impl From<KeyCode> for Key {
    fn from(kc: KeyCode) -> Self {
        match kc {
            KeyCode::Char(c) => Key::Char(c),
            KeyCode::F(n) => Key::F(n),
            KeyCode::Enter => Key::Enter,
            KeyCode::Esc => Key::Esc,
            KeyCode::Backspace => Key::Backspace,
            KeyCode::Delete => Key::Delete,
            KeyCode::Up => Key::Up,
            KeyCode::Down => Key::Down,
            KeyCode::Left => Key::Left,
            KeyCode::Right => Key::Right,
            KeyCode::Home => Key::Home,
            KeyCode::End => Key::End,
            KeyCode::PageUp => Key::PageUp,
            KeyCode::PageDown => Key::PageDown,
            KeyCode::Tab => Key::Tab,
            _ => Key::Space,
        }
    }
}

/// 一条键位绑定
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct KeyBinding {
    pub action: Action,
    pub key: Key,
    pub mods: ModKey,
}

impl KeyBinding {
    pub fn new(action: Action, key: Key) -> Self {
        Self {
            action,
            key,
            mods: ModKey {
                shift: false,
                alt: false,
                ctrl: false,
            },
        }
    }

    pub fn with_mods(action: Action, key: Key, mods: ModKey) -> Self {
        Self { action, key, mods }
    }

    /// 检查是否匹配一个 crossterm KeyEvent
    pub fn matches(&self, key_code: KeyCode, key_mods: KeyModifiers) -> bool {
        let key_matches = match (self.key, key_code) {
            (Key::Char(a), KeyCode::Char(b)) => a.eq_ignore_ascii_case(&b),
            (Key::F(a), KeyCode::F(b)) => a == b,
            (Key::Enter, KeyCode::Enter) => true,
            (Key::Esc, KeyCode::Esc) => true,
            (Key::Backspace, KeyCode::Backspace) => true,
            (Key::Delete, KeyCode::Delete) => true,
            (Key::Up, KeyCode::Up) => true,
            (Key::Down, KeyCode::Down) => true,
            (Key::Left, KeyCode::Left) => true,
            (Key::Right, KeyCode::Right) => true,
            (Key::Home, KeyCode::Home) => true,
            (Key::End, KeyCode::End) => true,
            (Key::PageUp, KeyCode::PageUp) => true,
            (Key::PageDown, KeyCode::PageDown) => true,
            (Key::Tab, KeyCode::Tab) => true,
            (Key::Space, KeyCode::Char(' ')) => true,
            _ => false,
        };
        if !key_matches {
            return false;
        }

        // 修饰键匹配: 我们忽略 SHIFT 对字母键的影响(大小写不敏感)
        let binding_mods: KeyModifiers = self.mods.into();
        let relevant_mods = KeyModifiers::ALT | KeyModifiers::CONTROL;
        binding_mods.intersects(relevant_mods) == key_mods.intersects(relevant_mods)
    }

    /// 显示名称（用于设置界面）
    pub fn display_label(&self) -> String {
        let mut parts = Vec::new();
        if self.mods.ctrl {
            parts.push("Ctrl");
        }
        if self.mods.alt {
            parts.push("Alt");
        }
        if self.mods.shift {
            parts.push("Shift");
        }
        let key_name = match self.key {
            Key::Char(c) => c.to_ascii_uppercase().to_string(),
            Key::F(n) => format!("F{}", n),
            Key::Enter => "Enter".into(),
            Key::Esc => "Esc".into(),
            Key::Backspace => "Bksp".into(),
            Key::Delete => "Del".into(),
            Key::Up => "↑".into(),
            Key::Down => "↓".into(),
            Key::Left => "←".into(),
            Key::Right => "→".into(),
            Key::Home => "Home".into(),
            Key::End => "End".into(),
            Key::PageUp => "PgUp".into(),
            Key::PageDown => "PgDn".into(),
            Key::Tab => "Tab".into(),
            Key::Space => "Space".into(),
        };
        parts.push(&key_name);
        parts.join("+")
    }
}

/// 完整的键位映射表
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Keymap {
    /// 全局绑定
    pub global: Vec<KeyBinding>,
    /// 菜单界面绑定
    pub menu: Vec<KeyBinding>,
    /// 游戏界面绑定
    pub game: Vec<KeyBinding>,
    /// 设置弹窗绑定
    pub settings: Vec<KeyBinding>,
    /// 贪吃蛇模式绑定
    pub snake: Vec<KeyBinding>,
    /// 导入弹窗绑定
    pub import: Vec<KeyBinding>,
    /// 导出弹窗绑定
    pub export: Vec<KeyBinding>,
    /// 键位配置界面绑定
    pub keymap_config: Vec<KeyBinding>,
}

impl Default for Keymap {
    fn default() -> Self {
        Self {
            global: vec![KeyBinding::new(Action::Cancel, Key::Esc)],
            menu: vec![
                KeyBinding::new(Action::MenuUp, Key::Up),
                KeyBinding::new(Action::MenuUp, Key::Char('w')),
                KeyBinding::new(Action::MenuDown, Key::Down),
                KeyBinding::new(Action::MenuDown, Key::Char('s')),
                KeyBinding::new(Action::MenuSelect, Key::Enter),
                KeyBinding::new(Action::MenuDelete, Key::Char('d')),
                KeyBinding::new(Action::MenuForceDelete, Key::Char('d')),
                KeyBinding::new(Action::MenuExport, Key::Char('e')),
                KeyBinding::new(Action::MenuImport, Key::Char('i')),
                KeyBinding::new(Action::Quit, Key::Char('q')),
            ],
            game: vec![
                // 光标移动
                KeyBinding::new(Action::CursorUp, Key::Char('w')),
                KeyBinding::new(Action::CursorDown, Key::Char('s')),
                KeyBinding::new(Action::CursorLeft, Key::Char('a')),
                KeyBinding::new(Action::CursorRight, Key::Char('d')),
                // 面切换
                KeyBinding::new(Action::FaceUp, Key::Up),
                KeyBinding::new(Action::FaceDown, Key::Down),
                KeyBinding::new(Action::FaceLeft, Key::Left),
                KeyBinding::new(Action::FaceRight, Key::Right),
                // 面直跳
                KeyBinding::new(Action::FaceFront, Key::Char('f')),
                KeyBinding::new(Action::FaceBack, Key::Char('b')),
                KeyBinding::new(Action::FaceLeftJump, Key::Char('l')),
                KeyBinding::new(Action::FaceRightJump, Key::Char('r')),
                KeyBinding::new(Action::FaceTop, Key::Char('t')),
                KeyBinding::new(Action::FaceBottom, Key::Char('u')),
                // 操作
                KeyBinding::new(Action::Erase, Key::Char('x')),
                KeyBinding::new(Action::Erase, Key::Char('e')),
                KeyBinding::new(Action::Hint, Key::Char('h')),
                KeyBinding::new(Action::Undo, Key::Char('z')),
                KeyBinding::new(Action::ToggleGuidance, Key::Char('g')),
                KeyBinding::new(Action::ToggleMode, Key::Char('m')),
                KeyBinding::new(Action::NewGame, Key::Char('n')),
                KeyBinding::new(Action::Quit, Key::Char('q')),
                KeyBinding::new(Action::Erase, Key::Backspace),
                KeyBinding::new(Action::Erase, Key::Delete),
                // 数字 1-9
                KeyBinding::new(Action::Number(1), Key::Char('1')),
                KeyBinding::new(Action::Number(2), Key::Char('2')),
                KeyBinding::new(Action::Number(3), Key::Char('3')),
                KeyBinding::new(Action::Number(4), Key::Char('4')),
                KeyBinding::new(Action::Number(5), Key::Char('5')),
                KeyBinding::new(Action::Number(6), Key::Char('6')),
                KeyBinding::new(Action::Number(7), Key::Char('7')),
                KeyBinding::new(Action::Number(8), Key::Char('8')),
                KeyBinding::new(Action::Number(9), Key::Char('9')),
                // 按钮
                KeyBinding::new(Action::BtnPagePrev, Key::Char('[')),
                KeyBinding::new(Action::BtnPageNext, Key::Char(']')),
                // 商店
                KeyBinding::new(Action::ShopFocus, Key::Tab),
                KeyBinding::new(Action::ShopUp, Key::PageUp),
                KeyBinding::new(Action::ShopDown, Key::PageDown),
                KeyBinding::new(Action::ShopBuy, Key::Enter),
                KeyBinding::new(Action::ShopBuy, Key::Char('b')),
                // 调试
                KeyBinding::with_mods(
                    Action::DebugHintFace,
                    Key::Char('h'),
                    ModKey {
                        shift: false,
                        alt: true,
                        ctrl: false,
                    },
                ),
                KeyBinding::with_mods(
                    Action::DebugWin,
                    Key::Char('w'),
                    ModKey {
                        shift: false,
                        alt: true,
                        ctrl: false,
                    },
                ),
                KeyBinding::with_mods(
                    Action::DebugGoldUp,
                    Key::Char('='),
                    ModKey {
                        shift: false,
                        alt: true,
                        ctrl: false,
                    },
                ),
                KeyBinding::with_mods(
                    Action::DebugGoldUp,
                    Key::Char('+'),
                    ModKey {
                        shift: false,
                        alt: true,
                        ctrl: false,
                    },
                ),
                KeyBinding::with_mods(
                    Action::DebugGoldDown,
                    Key::Char('-'),
                    ModKey {
                        shift: false,
                        alt: true,
                        ctrl: false,
                    },
                ),
                KeyBinding::with_mods(
                    Action::DebugGoldDown,
                    Key::Char('_'),
                    ModKey {
                        shift: false,
                        alt: true,
                        ctrl: false,
                    },
                ),
            ],
            settings: vec![
                KeyBinding::new(Action::SettingsUp, Key::Up),
                KeyBinding::new(Action::SettingsDown, Key::Down),
                KeyBinding::new(Action::SettingsLeft, Key::Left),
                KeyBinding::new(Action::SettingsRight, Key::Right),
                KeyBinding::new(Action::Confirm, Key::Enter),
            ],
            snake: vec![
                KeyBinding::new(Action::SnakeUp, Key::Up),
                KeyBinding::new(Action::SnakeUp, Key::Char('w')),
                KeyBinding::new(Action::SnakeDown, Key::Down),
                KeyBinding::new(Action::SnakeDown, Key::Char('s')),
                KeyBinding::new(Action::SnakeLeft, Key::Left),
                KeyBinding::new(Action::SnakeLeft, Key::Char('a')),
                KeyBinding::new(Action::SnakeRight, Key::Right),
                KeyBinding::new(Action::SnakeRight, Key::Char('d')),
                KeyBinding::new(Action::SnakeQuit, Key::Esc),
                KeyBinding::new(Action::SnakeQuit, Key::Char('q')),
            ],
            import: vec![
                KeyBinding::new(Action::Confirm, Key::Enter),
                KeyBinding::new(Action::Cancel, Key::Esc),
                KeyBinding::new(Action::ImportBackspace, Key::Backspace),
            ],
            export: vec![
                KeyBinding::new(Action::ExportUp, Key::Up),
                KeyBinding::new(Action::ExportDown, Key::Down),
                KeyBinding::new(Action::Confirm, Key::Enter),
            ],
            keymap_config: vec![
                KeyBinding::new(Action::SettingsUp, Key::Up),
                KeyBinding::new(Action::SettingsDown, Key::Down),
                KeyBinding::new(Action::SettingsLeft, Key::Left),
                KeyBinding::new(Action::SettingsRight, Key::Right),
                KeyBinding::new(Action::Confirm, Key::Enter),
                KeyBinding::new(Action::Cancel, Key::Esc),
            ],
        }
    }
}

impl Keymap {
    /// 根据当前界面和按键查找对应 Action
    pub fn resolve(
        &self,
        screen: AppScreen,
        snake_active: bool,
        key_code: KeyCode,
        key_mods: KeyModifiers,
    ) -> Option<Action> {
        // 贪吃蛇模式：跳过全局绑定（让蛇操作优先），只检查蛇和当前界面
        if snake_active {
            // 先检查蛇操作
            if let Some(binding) = self.snake.iter().find(|b| b.matches(key_code, key_mods)) {
                return Some(binding.action);
            }
            // 再检查当前界面绑定（game 界面）
            let bindings: &[KeyBinding] = match screen {
                AppScreen::Game => &self.game,
                _ => &[],
            };
            if let Some(binding) = bindings.iter().find(|b| b.matches(key_code, key_mods)) {
                return Some(binding.action);
            }
            return None;
        }

        // 正常模式：全局绑定
        if let Some(binding) = self.global.iter().find(|b| b.matches(key_code, key_mods)) {
            return Some(binding.action);
        }

        // 当前界面绑定
        let bindings: &[KeyBinding] = match screen {
            AppScreen::Menu => &self.menu,
            AppScreen::Game => &self.game,
            AppScreen::Settings => &self.settings,
            AppScreen::Generating => &[],
            AppScreen::Victory => &[],
            AppScreen::ExportSelect => &self.export,
            AppScreen::ImportInput => &self.import,
            AppScreen::KeymapConfig => &self.keymap_config,
        };

        bindings
            .iter()
            .find(|b| b.matches(key_code, key_mods))
            .map(|b| b.action)
    }

    /// 获取某个 Action 在指定分组中的当前绑定
    pub fn find_binding(&self, action: Action) -> Vec<&KeyBinding> {
        let mut results = Vec::new();
        for b in self
            .global
            .iter()
            .chain(self.menu.iter())
            .chain(self.game.iter())
            .chain(self.settings.iter())
            .chain(self.snake.iter())
            .chain(self.import.iter())
            .chain(self.export.iter())
            .chain(self.keymap_config.iter())
        {
            if b.action == action {
                results.push(b);
            }
        }
        results
    }

    /// 重新绑定一个 Action（替换第一条匹配的绑定）
    /// 返回 Ok(()) 表示成功，Err(msg) 表示失败（保留键或冲突）
    pub fn rebind(
        &mut self,
        action: Action,
        new_key: Key,
        new_mods: ModKey,
    ) -> Result<(), &'static str> {
        // 检查是否是保留键
        if matches!(new_key, Key::Esc) || matches!(new_key, Key::Enter) {
            return Err("reserved");
        }

        // 获取该 Action 所属的场景分组
        let scene = self.action_scene(&action);

        // 检查是否与同场景的其他 Action 冲突（跳过同 Action 的现有绑定）
        let new_binding = KeyBinding::with_mods(action, new_key, new_mods);
        let scene_bindings = self.get_scene_bindings(scene);
        for b in scene_bindings {
            if b.action != action && b.key == new_key && b.mods == new_mods {
                return Err("conflict");
            }
        }

        // 执行绑定
        let groups_mut: Vec<&mut Vec<KeyBinding>> = vec![
            &mut self.global,
            &mut self.menu,
            &mut self.game,
            &mut self.settings,
            &mut self.snake,
            &mut self.import,
            &mut self.export,
            &mut self.keymap_config,
        ];
        for group in groups_mut {
            if let Some(pos) = group.iter().position(|b| b.action == action) {
                group[pos] = new_binding;
                return Ok(());
            }
        }
        // 如果没找到已有绑定，添加到 game 分组
        self.game.push(new_binding);
        Ok(())
    }

    /// 获取 Action 所属的场景
    fn action_scene(&self, action: &Action) -> &'static str {
        // 根据 Action 所在的分组确定场景
        if self.global.iter().any(|b| &b.action == action) {
            return "global";
        }
        if self.menu.iter().any(|b| &b.action == action) {
            return "menu";
        }
        if self.game.iter().any(|b| &b.action == action) {
            return "game";
        }
        if self.settings.iter().any(|b| &b.action == action) {
            return "settings";
        }
        if self.snake.iter().any(|b| &b.action == action) {
            return "snake";
        }
        if self.import.iter().any(|b| &b.action == action) {
            return "import";
        }
        if self.export.iter().any(|b| &b.action == action) {
            return "export";
        }
        if self.keymap_config.iter().any(|b| &b.action == action) {
            return "keymap_config";
        }
        "game"
    }

    /// 获取指定场景的所有绑定
    fn get_scene_bindings(&self, scene: &str) -> Vec<&KeyBinding> {
        let mut bindings = Vec::new();
        match scene {
            "global" => bindings.extend(self.global.iter()),
            "menu" => bindings.extend(self.menu.iter()),
            "game" => bindings.extend(self.game.iter()),
            "settings" => bindings.extend(self.settings.iter()),
            "snake" => bindings.extend(self.snake.iter()),
            "import" => bindings.extend(self.import.iter()),
            "export" => bindings.extend(self.export.iter()),
            "keymap_config" => bindings.extend(self.keymap_config.iter()),
            _ => {}
        }
        bindings
    }

    /// 重置为默认键位
    pub fn reset_to_default(&mut self) {
        *self = Keymap::default();
    }

    /// 序列化为 JSON 字符串
    pub fn serialize_json(&self) -> Option<String> {
        serde_json::to_string(self).ok()
    }

    /// 从 JSON 字符串反序列化
    pub fn deserialize_json(json: &str) -> Option<Self> {
        serde_json::from_str(json).ok()
    }

    /// 从 DB 加载自定义键位
    pub fn load_from_db() -> Self {
        if let Some(json) = crate::save::load_setting("keymap").ok().flatten() {
            Self::deserialize_json(&json).unwrap_or_default()
        } else {
            Self::default()
        }
    }

    /// 保存自定义键位到 DB
    pub fn save_to_db(&self) {
        if let Some(json) = self.serialize_json() {
            let _ = crate::save::save_setting("keymap", &json);
        }
    }

    /// 收集所有已定义的 Action（去重）
    pub fn all_actions(&self) -> Vec<Action> {
        use std::collections::HashSet;
        let mut seen = HashSet::new();
        let mut result = Vec::new();
        for b in self
            .global
            .iter()
            .chain(self.menu.iter())
            .chain(self.game.iter())
            .chain(self.settings.iter())
            .chain(self.snake.iter())
            .chain(self.import.iter())
            .chain(self.export.iter())
            .chain(self.keymap_config.iter())
        {
            if seen.insert(b.action) {
                result.push(b.action);
            }
        }
        result
    }
}
