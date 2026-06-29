//! 应用核心状态：App struct 及其实现

use std::collections::VecDeque;
use std::time::{Duration, Instant};

use sudokube_core::cube::{CubeCoord, Face};
use sudokube_core::game_state::GameState;

use crate::config;
use crate::game_utils::{flush_elapsed, new_game, now_secs, total_elapsed};
use crate::i18n;
use crate::menu::MenuState;
use crate::render::{ButtonId, RenderMode};
use crate::save;
use crate::settings::{AppSettings, SettingsState};
use crate::shop;
use crate::types::{AppScreen, GeneratingState, PositionKind, position_kind};

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
    pub cube_angle_y: f64,                     // 3D cube Y-axis rotation angle
    pub cube_angle_x: f64,                     // 3D cube X-axis rotation angle
    pub victory_countdown: Option<Instant>,    // Victory screen countdown
    pub import_buffer: String,                 // Import input buffer
    pub import_paste_started: Option<Instant>, // 一键粘贴开始时间
    pub import_last_input: Option<Instant>,    // 上次输入时间(用于检测连续粘贴)
    pub export_select: usize,                  // 0=encrypted, 1=plaintext
    pub action_log: VecDeque<String>,          // Recent action messages (newest at back)
    pub overflow_notice_elapsed: Option<Instant>, // Until when to suppress overflow mode-switch notice
    /// 待删除的存档 ID(确认弹窗打开时设置)
    pub confirm_delete_id: Option<i64>,
    /// 每个数字的剩余数量(默认 54),用于防漏题
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
    /// 全局容错上限(开局默认 3，购买后增加)
    pub global_errors_max: u32,
    /// 冻结状态下购买选项光标: 0=当局容错, 1=全局容错
    pub frozen_select: u8,
    /// 游戏数值配置(集中化 magic number)
    pub config: config::GameConfig,
    /// 键位映射(可自定义)
    pub keymap: config::Keymap,
}

impl App {
    pub fn new_menu() -> Self {
        let mut app = Self {
            screen: AppScreen::Menu,
            menu: MenuState::new(),
            game: GameState::new(
                sudokube_core::cube::CubeGrid {
                    cells: Default::default(),
                },
                Default::default(),
                sudokube_core::cube::Difficulty::Medium,
            ),
            current_face: Face::Front,
            cursor: (4, 4),
            render_mode: RenderMode::Standard,
            // guidance 在下方 settings 加载后赋值(避免重复 load_from_db)
            guidance: false,
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
            digit_remaining: [config::GameConfig::default().digit_total; 9],
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
            global_errors_max: save::load_setting("global_errors_max")
                .ok()
                .flatten()
                .and_then(|v| v.parse().ok())
                .unwrap_or(3),
            frozen_select: 0,
            config: config::GameConfig::default(),
            keymap: config::Keymap::load_from_db(),
        };
        // 从 settings 派生 guidance 状态
        app.guidance = app.settings.guide_active();
        app
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

    /// 检查指定坐标放置的值是否为错误答案。
    /// 如果错误，消耗容错次数(优先当局，再全局)；
    /// 若容错耗尽则冻结对局。返回 true 表示输入错误。
    pub fn check_and_consume_error(&mut self, coord: CubeCoord, value: u8) -> bool {
        let is_wrong = self
            .game
            .solution
            .get(&coord)
            .map_or(true, |&sol| sol != value);
        if !is_wrong {
            return false;
        }
        // 计算总可用容错 = 当局剩余 + 全局剩余
        let local_remaining = self.game.errors_max.saturating_sub(self.game.errors);
        let global_remaining = self.global_errors_max;
        if local_remaining > 0 {
            // 优先消耗当局容错
            self.game.errors += 1;
        } else if global_remaining > 0 {
            // 消耗全局容错: global_errors_max 代表全局剩余量，消耗后减1
            self.global_errors_max = self.global_errors_max.saturating_sub(1);
            let _ = save::save_setting(
                "global_errors_max",
                &self.global_errors_max.to_string(),
            );
        }
        // 检查是否耗尽
        let local_rem = self.game.errors_max.saturating_sub(self.game.errors);
        let global_rem = self.global_errors_max;
        if local_rem == 0 && global_rem == 0 {
            self.freeze_game();
        }
        true
    }

    /// 冻结当前对局：暂停计时，标记 frozen，存档状态改为"失败"
    pub fn freeze_game(&mut self) {
        flush_elapsed(self);
        self.game.frozen = true;
        self.game.started_at = 0.0; // 停止计时
        let _ = save::save_game(&self.game);
    }

    /// 解冻对局：购买容错后恢复（刷新 errors_max 并解冻）
    pub fn unfreeze_game(&mut self, extra_local: u32, extra_global: u32) {
        self.game.errors_max += extra_local;
        self.global_errors_max += extra_global;
        let _ = save::save_setting(
            "global_errors_max",
            &self.global_errors_max.to_string(),
        );
        self.game.frozen = false;
        self.game.started_at = now_secs();
        let _ = save::save_game(&self.game);
    }

    /// 购买一个商店商品,扣减金币。
    /// 容错道具 / Guide 解锁直接生效(不入背包)，其余道具加到背包。
    /// 成功返回 true;金币不足返回 false。
    pub fn buy_item(&mut self, item: shop::ItemType) -> bool {
        let price = item.price();
        if self.gold < price {
            return false;
        }
        self.gold -= price;
        let _ = save::save_setting("player_gold", &self.gold.to_string());
        if item.is_revive() || item.is_guide_unlock() {
            // 容错道具 / Guide 购买后直接生效;Guide 仅可买一次(目录已过滤)
            shop::apply_tool(self, item);
        } else {
            *self.inventory.entry(item).or_insert(0) += 1;
        }
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
                Duration::from_secs(2),
            );
            return false;
        }
        let _ = self.consume_item(item);
        let msg = shop::apply_tool(self, item);
        if let Some(m) = msg {
            self.set_message(m, Duration::from_secs(2));
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

    /// 重算每个数字的剩余数量
    pub fn recompute_digit_remaining(&mut self) {
        let mut counts = [self.config.digit_total; 9];
        for (coord, cell) in self.game.grid.cells.iter() {
            if let Some(value) = cell.user_value {
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
        if let Some(v) = old_value {
            if (1..=9).contains(&v) {
                let idx = (v - 1) as usize;
                self.digit_remaining[idx] += decrement;
            }
        }
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

    pub fn start_generating(&mut self, difficulty: sudokube_core::cube::Difficulty) {
        let result = std::sync::Arc::new(std::sync::Mutex::new(None));
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
        let reward = shop::calculate_gold_reward(self.game.difficulty, elapsed, &self.config);
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
