mod i18n;
mod input;
mod render;
mod save;
mod shop;
mod widgets;

pub use widgets::{
    Button, ButtonState, ButtonTheme, EdgeDecor, THEME_PRIMARY, THEME_SUCCESS, THEME_DANGER,
    THEME_NEUTRAL,
};

use std::io;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use crossterm::event;
use rand::SeedableRng;
use rand::rngs::StdRng;
use ratatui::Terminal;
use sudokube_core::cube::{CubeCoord, Difficulty, Face};
use sudokube_core::game_state::GameState;
use sudokube_core::puzzle::generate_puzzle;
use sudokube_core::wfc::WfcGenerator;

use input::{EventResult, handle_event};

/// 单元位置类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PositionKind {
    /// 角点(顶点): 3 个轴坐标都处于边界 (0 或 8)
    Corner,
    /// 边: 恰好 2 个轴坐标处于边界
    Edge,
    /// 面中心: 恰好 1 个轴坐标处于边界
    Center,
}

/// 判定 9x9x9 立方体中单元 (x, y, z) 的位置类型
pub fn position_kind(x: u8, y: u8, z: u8) -> PositionKind {
    let on_boundary = |v: u8| v == 0 || v == 8;
    let count = [x, y, z].iter().filter(|&&v| on_boundary(v)).count();
    match count {
        3 => PositionKind::Corner,
        2 => PositionKind::Edge,
        1 => PositionKind::Center,
        _ => PositionKind::Center, // 内部: 退化为面中心
    }
}
use render::{ButtonId, RenderMode};
use save::{GameRecord, save_game};
use std::collections::VecDeque;

/// 当前画面
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppScreen {
    Menu,
    Game,
    Settings,
    Generating,
    Victory,
    ExportSelect,
    ImportInput,
}

/// 可配置项
#[derive(Debug, Clone)]
pub struct AppSettings {
    pub standard_cell_width: usize, // 奇数
    pub bg_color: String,           // "black", "darkgray"
    pub border_color: String,       // "cyan", "white", "green"
    pub guide_group_color: String,  // "green", "blue", "magenta"
    pub guide_same_color: String,   // "blue", "magenta", "red"
    pub cube_scale: String,         // "0.3", "0.35", "0.4", "0.45", "0.5"
    pub show_cube: String,          // "yes", "no"
    pub cube_width: String,         // "16", "18", "20", "22", "24"
    pub cube_height: String,        // "14", "16", "18", "20", "22"
    pub cube_aspect: String,        // "0.8", "1.0", "1.2", "1.5" — width/height of cube content
    pub debug_mode: String,         // "off", "on"
    pub language: String,           // "zh", "en", "ja"
    pub naming_mode: String,        // "vivid", "numeric"
    pub blink_highlight: String,    // "off", "on" — 选中数字时是否闪烁
}

impl Default for AppSettings {
    fn default() -> Self {
        let lang = detect_language();
        Self {
            standard_cell_width: 7,
            bg_color: "black".into(),
            border_color: "cyan".into(),
            guide_group_color: "green".into(),
            guide_same_color: "blue".into(),
            cube_scale: "0.38".into(),
            show_cube: "yes".into(),
            cube_width: "20".into(),
            cube_height: "18".into(),
            cube_aspect: "1.0".into(),
            debug_mode: "off".into(),
            language: lang,
            naming_mode: "vivid".into(),
            blink_highlight: "off".into(),
        }
    }
}

impl AppSettings {
    pub fn load_from_db() -> Self {
        let def = Self::default();
        Self {
            standard_cell_width: save::load_setting("standard_cell_width")
                .ok()
                .flatten()
                .and_then(|v| v.parse().ok())
                .unwrap_or(def.standard_cell_width),
            bg_color: save::load_setting("bg_color")
                .ok()
                .flatten()
                .unwrap_or(def.bg_color),
            border_color: save::load_setting("border_color")
                .ok()
                .flatten()
                .unwrap_or(def.border_color),
            guide_group_color: save::load_setting("guide_group_color")
                .ok()
                .flatten()
                .unwrap_or(def.guide_group_color),
            guide_same_color: save::load_setting("guide_same_color")
                .ok()
                .flatten()
                .unwrap_or(def.guide_same_color),
            cube_scale: save::load_setting("cube_scale")
                .ok()
                .flatten()
                .unwrap_or(def.cube_scale),
            show_cube: save::load_setting("show_cube")
                .ok()
                .flatten()
                .unwrap_or(def.show_cube),
            cube_width: save::load_setting("cube_width")
                .ok()
                .flatten()
                .unwrap_or(def.cube_width),
            cube_height: save::load_setting("cube_height")
                .ok()
                .flatten()
                .unwrap_or(def.cube_height),
            cube_aspect: save::load_setting("cube_aspect")
                .ok()
                .flatten()
                .unwrap_or(def.cube_aspect),
            debug_mode: save::load_setting("debug_mode")
                .ok()
                .flatten()
                .unwrap_or(def.debug_mode),
            language: save::load_setting("language")
                .ok()
                .flatten()
                .unwrap_or(def.language),
            naming_mode: save::load_setting("naming_mode")
                .ok()
                .flatten()
                .unwrap_or(def.naming_mode),
            blink_highlight: save::load_setting("blink_highlight")
                .ok()
                .flatten()
                .unwrap_or(def.blink_highlight),
        }
    }

    pub fn save_to_db(&self) {
        let _ = save::save_setting("standard_cell_width", &self.standard_cell_width.to_string());
        let _ = save::save_setting("bg_color", &self.bg_color);
        let _ = save::save_setting("border_color", &self.border_color);
        let _ = save::save_setting("guide_group_color", &self.guide_group_color);
        let _ = save::save_setting("guide_same_color", &self.guide_same_color);
        let _ = save::save_setting("cube_scale", &self.cube_scale);
        let _ = save::save_setting("show_cube", &self.show_cube);
        let _ = save::save_setting("cube_width", &self.cube_width);
        let _ = save::save_setting("cube_height", &self.cube_height);
        let _ = save::save_setting("cube_aspect", &self.cube_aspect);
        let _ = save::save_setting("debug_mode", &self.debug_mode);
        let _ = save::save_setting("language", &self.language);
        let _ = save::save_setting("naming_mode", &self.naming_mode);
        let _ = save::save_setting("blink_highlight", &self.blink_highlight);
    }
}

fn detect_language() -> String {
    // 根据系统时区检测语言
    let tz = std::env::var("TZ").unwrap_or_default();
    let lang = std::env::var("LANG").unwrap_or_default();
    let locale =
        std::env::var("LC_ALL").unwrap_or_else(|_| std::env::var("LC_CTYPE").unwrap_or_default());

    let check = format!("{}{}{}", tz, lang, locale).to_lowercase();
    if check.contains("cn") || check.contains("zh") {
        "zh".into()
    } else if check.contains("jp") || check.contains("ja") {
        "ja".into()
    } else {
        "en".into()
    }
}

pub struct SettingsState {
    pub fields: Vec<SettingsField>,
    pub selected: usize,
    /// 弹窗中可见区域的滚动偏移（用于内容过多时滚动）
    pub scroll: u16,
    /// 鼠标悬停的字段下标
    pub hover_field: Option<usize>,
    /// 鼠标悬停的方向（None / Left / Right）
    pub hover_arrow: Option<SettingsArrow>,
    /// 弹窗是否可见（默认隐藏,需用户主动打开）
    pub visible: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsArrow {
    Left,
    Right,
}

#[derive(Debug, Clone)]
pub struct SettingsField {
    pub label: String,
    pub value: String,
    pub options: Vec<String>,
    pub option_index: usize,
}

/// 启动菜单选项。
#[derive(Debug, Clone)]
pub enum MenuItem {
    NewGame(Difficulty),
    Continue(GameRecord),
    Settings,
    Export,
    Import,
}

pub struct MenuState {
    pub items: Vec<MenuItem>,
    pub selected: usize,
    pub victories: Vec<GameRecord>, // completed games for sidebar
}

impl MenuState {
    pub fn new() -> Self {
        let mut items = vec![
            MenuItem::NewGame(Difficulty::Easy),
            MenuItem::NewGame(Difficulty::Medium),
            MenuItem::NewGame(Difficulty::Hard),
            MenuItem::Settings,
            MenuItem::Export,
            MenuItem::Import,
        ];
        let unfinished = save::load_unfinished(20).unwrap_or_default();
        for record in unfinished {
            if !record.completed {
                items.push(MenuItem::Continue(record));
            }
        }
        let victories = save::load_completed(20).unwrap_or_default();
        Self {
            items,
            selected: 0,
            victories,
        }
    }
}

/// 异步生成状态
pub struct GeneratingState {
    pub difficulty: Difficulty,
    pub result: Arc<Mutex<Option<GameState>>>,
    pub spinner: u8,
    pub started: Instant,
}

/// 应用状态
pub struct App {
    pub screen: AppScreen,
    pub menu: MenuState,
    pub game: GameState,
    pub current_face: Face,
    pub cursor: (u8, u8),
    pub render_mode: RenderMode,
    pub guidance: bool,
    pub blink_on: bool,
    pub message: String,
    pub message_until: Option<Instant>,
    pub hover_button: Option<ButtonId>,
    pub btn_page: u16, // 当前按钮栏页码 (0-based)
    pub settings: AppSettings,
    pub settings_ui: SettingsState,
    pub generating: Option<GeneratingState>,
    pub cube_angle_y: f64,                  // 3D cube Y-axis rotation angle
    pub cube_angle_x: f64,                  // 3D cube X-axis rotation angle
    pub victory_countdown: Option<Instant>, // Victory screen countdown
    pub import_buffer: String,              // Import input buffer
    pub import_paste_started: Option<Instant>, // 一键粘贴开始时间
    pub import_last_input: Option<Instant>,    // 上次输入时间(用于检测连续粘贴)
    pub export_select: usize,               // 0=encrypted, 1=plaintext
    pub action_log: VecDeque<String>,       // Recent action messages (newest at back)
    pub overflow_notice_elapsed: Option<Instant>, // Until when to suppress overflow mode-switch notice
    /// 待删除的存档 ID(确认弹窗打开时设置)
    pub confirm_delete_id: Option<i64>,
    /// 每个数字的剩余数量(默认 54),用于防漏题
    /// - 角点(顶点)位置: 放置时 -3
    /// - 边位置: 放置时 -2
    /// - 面中心位置: 放置时 -1
    /// 擦除时反向加回
    pub digit_remaining: [i32; 9],
    /// 局外金币
    pub gold: i32,
    /// 道具背包(每种道具持有数量,购买后增加,使用时减少)
    pub inventory: std::collections::HashMap<shop::ItemType, u32>,
    /// 商店当前选中项
    pub shop_selected: usize,
    /// 商店当前页(每页 4 项)
    pub shop_page: u16,
    /// 商店是否获得焦点(激活时方向键/Enter 用于浏览与购买)
    pub shop_focused: bool,
    /// 最近一次结算获得的金币(胜利画面展示用)
    pub last_reward: i32,
    /// 贪吃蛇小游戏状态(运行中时为 Some)
    pub snake: Option<shop::SnakeState>,
}

impl App {
    pub fn new_menu() -> Self {
        Self {
            screen: AppScreen::Menu,
            menu: MenuState::new(),
            game: GameState::new(
                sudokube_core::cube::CubeGrid {
                    cells: Default::default(),
                },
                Default::default(),
                Difficulty::Medium,
            ),
            current_face: Face::Front,
            cursor: (4, 4),
            render_mode: RenderMode::Standard,
            guidance: true,
            blink_on: true,
            message: String::new(),
            message_until: None,
            hover_button: None,
            btn_page: 0,
            settings: AppSettings::load_from_db(),
            settings_ui: SettingsState::from_settings(&AppSettings::load_from_db()),
            generating: None,
            cube_angle_y: 0.0,
            cube_angle_x: 0.0,
            victory_countdown: None,
            import_buffer: String::new(),
            import_paste_started: None,
            import_last_input: None,
            export_select: 0,
            action_log: VecDeque::new(),
            overflow_notice_elapsed: None,
            confirm_delete_id: None,
            digit_remaining: [54; 9],
            gold: save::load_setting("player_gold")
                .ok()
                .flatten()
                .and_then(|v| v.parse().ok())
                .unwrap_or(0),
            inventory: std::collections::HashMap::new(),
            shop_selected: 0,
            shop_page: 0,
            shop_focused: false,
            last_reward: 0,
            snake: None,
        }
    }

    pub fn start_game(game: GameState) -> Self {
        let mut app = Self {
            screen: AppScreen::Game,
            game,
            victory_countdown: None,
            shop_focused: false,
            ..Self::new_menu()
        };
        app.btn_page = 0;
        app.recompute_digit_remaining();
        app
    }

    /// 购买一个商店商品,扣减金币并加到背包。
    /// 成功返回 true;金币不足返回 false。
    pub fn buy_item(&mut self, item: shop::ItemType) -> bool {
        let price = item.price();
        if self.gold < price {
            return false;
        }
        self.gold -= price;
        let _ = save::save_setting("player_gold", &self.gold.to_string());
        *self.inventory.entry(item).or_insert(0) += 1;
        true
    }

    /// 使用一个道具(扣减背包数量),debug 模式下不消耗。
    /// 数量不足返回 false。
    pub fn consume_item(&mut self, item: shop::ItemType) -> bool {
        if self.settings.debug_mode == "on" {
            return true;
        }
        let count = self.inventory.get(&item).copied().unwrap_or(0);
        if count == 0 {
            return false;
        }
        if count == 1 {
            self.inventory.remove(&item);
        } else {
            *self.inventory.entry(item).or_insert(0) -= 1;
        }
        true
    }

    /// 切换商店焦点状态
    pub fn toggle_shop_focus(&mut self) {
        self.shop_focused = !self.shop_focused;
    }

    /// 使用一个道具:扣减背包(debug 模式不消耗)并执行对应效果。
    /// 数量不足(非 debug 模式)时设置警告消息并返回 false。
    pub fn use_tool(&mut self, item: shop::ItemType) -> bool {
        let lang = i18n::Lang::from_code(&self.settings.language);
        let count = self.inventory.get(&item).copied().unwrap_or(0);
        if count == 0 && self.settings.debug_mode != "on" {
            self.set_message(
                i18n::t("tool.no_count", lang).to_string(),
                std::time::Duration::from_secs(2),
            );
            return false;
        }
        // 扣减
        let _ = self.consume_item(item);
        // 实际效果
        let msg = shop::apply_tool(self, item);
        if let Some(m) = msg {
            self.set_message(m, std::time::Duration::from_secs(2));
        }
        true
    }

    pub fn set_message(&mut self, text: impl Into<String>, duration: Duration) {
        self.message = text.into();
        self.message_until = Some(Instant::now() + duration);
    }

    /// Push a line into the action log (keeps at most `max` entries).
    pub fn push_log(&mut self, line: impl Into<String>, max: usize) {
        self.action_log.push_back(line.into());
        while self.action_log.len() > max {
            self.action_log.pop_front();
        }
    }

    /// 重算每个数字的剩余数量(默认 54):
    /// - 角点(顶点)位置: 放置时 -3
    /// - 边位置: 放置时 -2
    /// - 面中心位置: 放置时 -1
    /// 擦除时反向加回
    pub fn recompute_digit_remaining(&mut self) {
        // 默认 54
        let mut counts = [54i32; 9];
        // 遍历 game.grid 每个 cell,如果是用户填入则扣减
        for (coord, cell) in self.game.grid.cells.iter() {
            if let Some(value) = cell.user_value {
                // 计算 cell 的位置类型
                let (x, y, z) = (coord.x, coord.y, coord.z);
                let kind = position_kind(x, y, z);
                let decrement = match kind {
                    PositionKind::Corner => 3,
                    PositionKind::Edge => 2,
                    PositionKind::Center => 1,
                };
                if (1..=9).contains(&value) {
                    let idx = (value - 1) as usize;
                    counts[idx] -= decrement;
                }
            }
        }
        self.digit_remaining = counts;
    }

    /// 当玩家在 (x,y,z) 位置放置/擦除数字时,调整对应数字的剩余数量
    /// `old_value` 为 None 表示擦除,`new_value` 为 Some 表示放置
    pub fn adjust_digit_remaining(
        &mut self,
        x: u8,
        y: u8,
        z: u8,
        old_value: Option<u8>,
        new_value: Option<u8>,
    ) {
        let kind = position_kind(x, y, z);
        let decrement = match kind {
            PositionKind::Corner => 3,
            PositionKind::Edge => 2,
            PositionKind::Center => 1,
        };
        // 擦除:加回
        if let Some(v) = old_value {
            if (1..=9).contains(&v) {
                let idx = (v - 1) as usize;
                self.digit_remaining[idx] += decrement;
            }
        }
        // 放置:扣减
        if let Some(v) = new_value {
            if (1..=9).contains(&v) {
                let idx = (v - 1) as usize;
                self.digit_remaining[idx] -= decrement;
            }
        }
    }

    pub fn clear_message_if_expired(&mut self) {
        if let Some(until) = self.message_until {
            if Instant::now() >= until {
                self.message.clear();
                self.message_until = None;
            }
        }
        if let Some(until) = self.overflow_notice_elapsed {
            if Instant::now() >= until {
                self.overflow_notice_elapsed = None;
            }
        }
    }

    pub fn start_generating(&mut self, difficulty: Difficulty) {
        let result = Arc::new(Mutex::new(None));
        let result_clone = result.clone();
        std::thread::spawn(move || {
            let game = new_game(difficulty);
            *result_clone.lock().unwrap() = Some(game);
        });
        self.generating = Some(GeneratingState {
            difficulty,
            result,
            spinner: 0,
            started: Instant::now(),
        });
        self.screen = AppScreen::Generating;
    }

    /// 触发胜利结算: 计算金币奖励,加入 gold 并持久化,跳转到 Victory 画面
    pub fn trigger_victory(&mut self) {
        let elapsed = total_elapsed(self);
        let reward = shop::calculate_gold_reward(self.game.difficulty, elapsed);
        self.last_reward = reward;
        self.gold += reward;
        let _ = save::save_setting("player_gold", &self.gold.to_string());
        self.game.completed = true;
        self.screen = AppScreen::Victory;
        self.victory_countdown = Some(Instant::now() + Duration::from_secs(3));
        let _ = save::save_game(&self.game);
    }

    pub fn check_generating(&mut self) -> Option<GameState> {
        if let Some(ref gen_state) = self.generating {
            if let Ok(mut guard) = gen_state.result.lock() {
                if guard.is_some() {
                    return guard.take();
                }
            }
        }
        None
    }
}

impl SettingsState {
    pub fn from_settings(s: &AppSettings) -> Self {
        let widths: Vec<String> = (3..=15).step_by(2).map(|n| n.to_string()).collect();
        let colors = vec!["black".into(), "darkgray".into()];
        let border_colors = vec![
            "cyan".into(),
            "white".into(),
            "green".into(),
            "yellow".into(),
        ];
        let guide_colors = vec![
            "green".into(),
            "blue".into(),
            "magenta".into(),
            "red".into(),
        ];
        let cube_scales = vec![
            "0.3".into(),
            "0.35".into(),
            "0.38".into(),
            "0.4".into(),
            "0.45".into(),
            "0.5".into(),
        ];
        let yes_no = vec!["yes".into(), "no".into()];
        let cube_widths = vec![
            "16".into(),
            "18".into(),
            "20".into(),
            "22".into(),
            "24".into(),
            "26".into(),
            "28".into(),
            "30".into(),
        ];
        let cube_heights = vec![
            "14".into(),
            "16".into(),
            "18".into(),
            "20".into(),
            "22".into(),
            "24".into(),
            "26".into(),
            "28".into(),
            "30".into(),
        ];
        let cube_aspects = vec![
            "0.7".into(),
            "0.85".into(),
            "1.0".into(),
            "1.2".into(),
            "1.4".into(),
            "1.6".into(),
        ];
        let debug_modes = vec!["off".into(), "on".into()];
        let languages = vec!["zh".into(), "en".into(), "ja".into()];
        let naming_modes = vec!["vivid".into(), "numeric".into()];
        let blink_modes = vec!["off".into(), "on".into()];

        let fields = vec![
            SettingsField::new("Cell Width", &s.standard_cell_width.to_string(), widths),
            SettingsField::new("BG Color", &s.bg_color, colors),
            SettingsField::new("Border Color", &s.border_color, border_colors.clone()),
            SettingsField::new("Guide-Group", &s.guide_group_color, guide_colors.clone()),
            SettingsField::new("Guide-Same", &s.guide_same_color, guide_colors),
            SettingsField::new("Cube Scale", &s.cube_scale, cube_scales),
            SettingsField::new("Show Cube", &s.show_cube, yes_no),
            SettingsField::new("Cube Width", &s.cube_width, cube_widths),
            SettingsField::new("Cube Height", &s.cube_height, cube_heights),
            SettingsField::new("Cube Aspect", &s.cube_aspect, cube_aspects),
            SettingsField::new("Debug Mode", &s.debug_mode, debug_modes),
            SettingsField::new("Language", &s.language, languages),
            SettingsField::new("Naming Mode", &s.naming_mode, naming_modes),
            SettingsField::new("Blink Highlight", &s.blink_highlight, blink_modes),
        ];
        Self {
            fields,
            selected: 0,
            scroll: 0,
            hover_field: None,
            hover_arrow: None,
            visible: false,
        }
    }

    pub fn apply_to(&self, s: &mut AppSettings) {
        s.standard_cell_width = self.fields[0].value.parse().unwrap_or(7);
        s.bg_color = self.fields[1].value.clone();
        s.border_color = self.fields[2].value.clone();
        s.guide_group_color = self.fields[3].value.clone();
        s.guide_same_color = self.fields[4].value.clone();
        s.cube_scale = self.fields[5].value.clone();
        s.show_cube = self.fields[6].value.clone();
        s.cube_width = self.fields[7].value.clone();
        s.cube_height = self.fields[8].value.clone();
        s.cube_aspect = self.fields[9].value.clone();
        s.debug_mode = self.fields[10].value.clone();
        s.language = self.fields[11].value.clone();
        s.naming_mode = self.fields[12].value.clone();
        s.blink_highlight = self.fields[13].value.clone();
    }
}

impl SettingsField {
    fn new(label: &str, current: &str, options: Vec<String>) -> Self {
        let idx = options.iter().position(|o| o == current).unwrap_or(0);
        Self {
            label: label.into(),
            value: current.into(),
            options,
            option_index: idx,
        }
    }

    pub fn cycle_next(&mut self) {
        self.option_index = (self.option_index + 1) % self.options.len();
        self.value = self.options[self.option_index].clone();
    }

    pub fn cycle_prev(&mut self) {
        if self.option_index == 0 {
            self.option_index = self.options.len() - 1;
        } else {
            self.option_index -= 1;
        }
        self.value = self.options[self.option_index].clone();
    }
}

fn main() -> io::Result<()> {
    let terminal = ratatui::init();
    crossterm::execute!(io::stdout(), crossterm::event::EnableMouseCapture)?;
    let result = run_app(terminal);
    crossterm::execute!(io::stdout(), crossterm::event::DisableMouseCapture)?;
    ratatui::restore();
    result
}

fn run_app(
    mut terminal: Terminal<ratatui::backend::CrosstermBackend<io::Stdout>>,
) -> io::Result<()> {
    let mut app = App::new_menu();
    let mut last_blink = Instant::now();

    loop {
        // 闪烁定时器(仅在设置开启时切换)
        let now = Instant::now();
        if app.settings.blink_highlight == "on"
            && now.duration_since(last_blink) >= Duration::from_millis(500)
        {
            app.blink_on = !app.blink_on;
            last_blink = now;
        }
        app.clear_message_if_expired();

        // 3D立方体自动旋转（两轴不同周期）
        app.cube_angle_y += 0.03;
        app.cube_angle_x += 0.02;

        // 胜利倒计时
        if app.screen == AppScreen::Victory {
            if let Some(until) = app.victory_countdown {
                if Instant::now() >= until {
                    app = App::new_menu();
                }
            }
        }

        // 检查生成进度
        if app.screen == AppScreen::Generating {
            if let Some(game) = app.check_generating() {
                app.generating = None;
                app = App::start_game(game);
            } else if let Some(ref mut gen_state) = app.generating {
                gen_state.spinner = (gen_state.spinner + 1) % 4;
            }
        }

        // 贪吃蛇: 每帧推进 + 胜负结算时退出
        if app.snake.is_some() {
            shop::snake_step(&mut app);
            if let Some(snake) = app.snake.as_ref() {
                if snake.outcome != shop::SnakeOutcome::Running {
                    if let Some(msg) = shop::end_snake_game(&mut app) {
                        app.set_message(msg, Duration::from_secs(2));
                    }
                }
            }
        }

        // 渲染
        // 自动溢出检测：当内部元素（数独网格、按钮栏）超出可用区域时，
        // 自动切换到 Scrollbar 模式以适应。
        if app.screen == AppScreen::Game {
            let area: ratatui::layout::Rect = terminal.size()?.into();
            let needs_overflow_switch = render::needs_scrollbar_mode(area, &app);
            if needs_overflow_switch && app.render_mode != render::RenderMode::Scrollbar {
                let prev = app.render_mode;
                app.render_mode = render::RenderMode::Scrollbar;
                if prev != render::RenderMode::Scrollbar && app.overflow_notice_elapsed.is_none()
                {
                    let lang = crate::i18n::Lang::from_code(&app.settings.language);
                    let label = render::mode_label(app.render_mode, lang);
                    app.set_message(
                        format!("⚠ {} ({})", i18n::t("msg.overflow_auto", lang), label),
                        Duration::from_secs(3),
                    );
                    app.overflow_notice_elapsed = Some(Instant::now() + Duration::from_secs(3));
                }
            }
        }

        terminal.draw(|f| render::draw(f, &mut app))?;

        // 事件处理
        if event::poll(Duration::from_millis(50))? {
            let ev = event::read()?;
            let area: ratatui::layout::Rect = terminal.size()?.into();
            match handle_event(&mut app, ev, area) {
                EventResult::Continue => {}
                EventResult::StartGame(game) => {
                    app = App::start_game(game);
                }
                EventResult::StartGenerating(difficulty) => {
                    app.start_generating(difficulty);
                }
                EventResult::BackToMenu => {
                    if app.screen == AppScreen::Game {
                        flush_elapsed(&mut app);
                        let _ = save_game(&app.game);
                    }
                    app = App::new_menu();
                }
                EventResult::Quit => {
                    if app.screen == AppScreen::Game {
                        flush_elapsed(&mut app);
                        let _ = save_game(&app.game);
                    }
                    break;
                }
            }
        }
    }

    Ok(())
}

pub fn new_game(difficulty: Difficulty) -> GameState {
    let mut rng = StdRng::from_entropy();
    let mut generator = WfcGenerator::new();
    let solution = generator.generate(&mut rng).expect("生成题解失败");
    let grid = generate_puzzle(&solution, difficulty, &mut rng);
    let mut game = GameState::new(grid, solution, difficulty);
    game.started_at = now_secs();
    game.selected = Some(Face::Front.to_cube(4, 4));
    game
}

pub fn continue_game(record: &GameRecord) -> GameState {
    let difficulty = match record.difficulty.as_str() {
        "简单" => Difficulty::Easy,
        "困难" => Difficulty::Hard,
        _ => Difficulty::Medium,
    };
    let given: std::collections::HashSet<CubeCoord> = record.given.keys().copied().collect();
    let mut game = GameState::new(
        sudokube_core::cube::CubeGrid::from_solution(record.answer.clone(), &given),
        record.answer.clone(),
        difficulty,
    );
    for (coord, value) in &record.puzzle {
        if !given.contains(coord) {
            if let Some(cell) = game.grid.get_mut(coord) {
                cell.user_value = Some(*value);
            }
        }
    }
    game.id = Some(record.id);
    game.elapsed_seconds = record.elapsed_seconds as u64;
    game.started_at = now_secs();
    game.selected = Some(Face::Front.to_cube(4, 4));
    game
}

pub fn now_secs() -> f64 {
    use std::time::SystemTime;
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.as_secs_f64())
        .unwrap_or(0.0)
}

pub fn total_elapsed(app: &App) -> u64 {
    let session = if app.game.started_at > 0.0 {
        (now_secs() - app.game.started_at) as u64
    } else {
        0
    };
    app.game.elapsed_seconds + session
}

pub fn flush_elapsed(app: &mut App) {
    let session = if app.game.started_at > 0.0 {
        (now_secs() - app.game.started_at) as u64
    } else {
        0
    };
    app.game.elapsed_seconds += session;
    app.game.started_at = now_secs();
}

pub fn current_coord(app: &App) -> CubeCoord {
    app.current_face.to_cube(app.cursor.0, app.cursor.1)
}
