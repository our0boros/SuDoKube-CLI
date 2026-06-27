//! 游戏配置参数集中管理
//!
//! 将散落在 main.rs / render.rs / input.rs / shop.rs 中的
//! magic number 统一收敛到此模块，方便调整和维护。

pub mod keymap;
pub use keymap::{Action, Keymap};

/// 游戏核心数值配置
#[derive(Debug, Clone)]
pub struct GameConfig {
    // ── 数字剩余 ──
    /// 每个数字(1-9)的理论总出现次数
    pub digit_total: i32,

    // ── 渲染布局 ──
    /// 标准模式默认格子宽度(奇数)
    pub default_cell_width: usize,
    /// 方向指示条宽度
    pub dir_border: u16,
    /// 按钮栏高度
    pub btn_bar_height: u16,
    /// 按钮栏与数独面板之间的间距
    pub btn_gap: u16,
    /// 左侧面板最小宽度
    pub left_panel_min_w: u16,
    /// 左侧面板最大宽度
    pub left_panel_max_w: u16,
    /// 翻页控件预留宽度: ◁(1) + 空格(1) + 页码(3) + 空格(1) + ▷(1)
    pub pager_reserve_w: u16,
    /// 设置弹窗期望宽度
    pub settings_popup_w: u16,
    /// 设置弹窗最小可见行数
    pub settings_min_content_h: u16,

    // ── 导入/粘贴 ──
    /// 粘贴最大持续时间(秒)
    pub paste_max_duration_secs: u64,
    /// 粘贴突发间隔(毫秒)
    pub paste_burst_gap_ms: u64,

    // ── 金币/商店 ──
    /// 简单难度目标完成时间(秒)
    pub gold_easy_target_secs: u64,
    /// 中等难度目标完成时间(秒)
    pub gold_medium_target_secs: u64,
    /// 困难难度目标完成时间(秒)
    pub gold_hard_target_secs: u64,
    /// 金币奖励最大倍率
    pub gold_reward_max_multiplier: f64,
    /// 金币奖励最低倍率
    pub gold_reward_min_multiplier: f64,

    // ── 贪吃蛇 ──
    /// 蛇移步间隔(毫秒)
    pub snake_step_interval_ms: u64,
    /// 蛇超时(秒)
    pub snake_timeout_secs: u64,
    /// 蛇初始身体长度
    pub snake_initial_body_len: usize,
    /// 商店每页显示项数
    pub shop_items_per_page: usize,

    // ── 生成 ──
    /// WFC 生成最大尝试次数
    pub wfc_max_attempts: u32,
    /// 生成界面进度条宽度
    pub generating_bar_w: u16,

    // ── 胜利 ──
    /// 胜利画面自动返回倒计时(秒)
    pub victory_countdown_secs: u64,
    /// 胜利弹窗宽度
    pub victory_box_w: u16,
    /// 胜利弹窗高度
    pub victory_box_h: u16,

    // ── 菜单 ──
    /// 菜单侧边栏宽度
    pub menu_sidebar_w: u16,
    /// 菜单 LOGO 行数
    pub menu_logo_lines: u16,

    // ── 状态面板 ──
    /// 状态面板高度
    pub status_panel_h: u16,
    /// 导航面板高度
    pub navigator_panel_h: u16,

    // ── 3D 立方体 ──
    /// 立方体内框左偏移
    pub cube_inner_left_offset: u16,
    /// 立方体内框上偏移
    pub cube_inner_top_offset: u16,

    // ── 消息 ──
    /// 消息行高度
    pub msg_line_h: u16,
    /// 消息与网格间距
    pub msg_gap: u16,

    // ── 日志 ──
    /// 操作日志最大条目数
    pub action_log_max: usize,
}

impl Default for GameConfig {
    fn default() -> Self {
        Self {
            digit_total: 54,
            default_cell_width: 7,
            dir_border: 1,
            btn_bar_height: 3,
            btn_gap: 1,
            left_panel_min_w: 24,
            left_panel_max_w: 30,
            pager_reserve_w: 7,
            settings_popup_w: 46,
            settings_min_content_h: 6,
            paste_max_duration_secs: 5,
            paste_burst_gap_ms: 100,
            gold_easy_target_secs: 180,
            gold_medium_target_secs: 480,
            gold_hard_target_secs: 1500,
            gold_reward_max_multiplier: 1.5,
            gold_reward_min_multiplier: 0.1,
            snake_step_interval_ms: 200,
            snake_timeout_secs: 30,
            snake_initial_body_len: 6,
            shop_items_per_page: 4,
            wfc_max_attempts: 2000,
            generating_bar_w: 40,
            victory_countdown_secs: 3,
            victory_box_w: 36,
            victory_box_h: 7,
            menu_sidebar_w: 28,
            menu_logo_lines: 10,
            status_panel_h: 16,
            navigator_panel_h: 9,
            cube_inner_left_offset: 4,
            cube_inner_top_offset: 2,
            msg_line_h: 1,
            msg_gap: 1,
            action_log_max: 50,
        }
    }
}

impl GameConfig {
    /// 根据 difficulty 返回目标完成时间
    pub fn gold_target_secs(&self, difficulty: sudokube_core::cube::Difficulty) -> u64 {
        match difficulty {
            sudokube_core::cube::Difficulty::Easy => self.gold_easy_target_secs,
            sudokube_core::cube::Difficulty::Medium => self.gold_medium_target_secs,
            sudokube_core::cube::Difficulty::Hard => self.gold_hard_target_secs,
        }
    }
}
