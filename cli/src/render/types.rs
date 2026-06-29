//! 渲染相关类型定义

use ratatui::layout::Rect;

use crate::i18n::{self, Lang};
use crate::settings::AppSettings;

// ── RenderMode ──

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderMode {
    Scrollbar, // 使用滚动条显示完整数独
    Compact,   // 精简模式，最小空间占用
    Standard,  // 标准模式
}

impl RenderMode {
    pub fn toggle(self) -> Self {
        match self {
            RenderMode::Scrollbar => RenderMode::Compact,
            RenderMode::Compact => RenderMode::Standard,
            RenderMode::Standard => RenderMode::Scrollbar,
        }
    }

    pub fn cell_width(self, settings: &AppSettings) -> usize {
        match self {
            RenderMode::Scrollbar => 3,
            RenderMode::Compact => 1,
            RenderMode::Standard => settings.standard_cell_width,
        }
    }

    pub fn cell_height(self) -> usize {
        match self {
            RenderMode::Scrollbar => 1,
            RenderMode::Compact => 1,
            RenderMode::Standard => 3,
        }
    }
}

pub fn mode_label(mode: RenderMode, lang: Lang) -> &'static str {
    match mode {
        RenderMode::Scrollbar => i18n::t("game.mode_scrollbar", lang),
        RenderMode::Compact => i18n::t("game.mode_compact", lang),
        RenderMode::Standard => i18n::t("game.mode_standard", lang),
    }
}

// ── ButtonId ──

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ButtonId {
    Number(u8),
    Erase,
    Hint,
    Undo,
    ToggleGuidance,
    ToggleMode,
    ToggleDraft,
    Quit,
    // 道具工具
    ToolCube,
    ToolSnake3,
    ToolSnake5,
    ToolFace,
    ToolTarget,
}

// ── ButtonLayout ──

/// 按钮布局信息（用于鼠标点击检测）。
pub struct ButtonLayout {
    pub id: ButtonId,
    pub label: String,
    pub col: u16,
    pub row: u16,
    pub width: u16,
    pub height: u16,
    /// 按钮主题（用于自定义 Widget 渲染）。
    pub theme: crate::ButtonTheme,
}

// ── ButtonPagerLayout ──

/// 翻页控件位置（用于鼠标交互）
pub struct ButtonPagerLayout {
    /// 左翻页按钮 (◁) 的矩形
    pub prev_rect: Rect,
    /// 右翻页按钮 (▷) 的矩形
    pub next_rect: Rect,
    /// 当前页码标签矩形
    pub page_label_rect: Rect,
    /// 总页数
    pub total_pages: u16,
}

// ── GameLayout ──

/// 游戏画面布局（三列式：左 = 控制面板，中间 = 数独网格，右 = 3D 立方体 / 预留）。
#[allow(dead_code)]
pub struct GameLayout {
    /// 整个游戏区域
    pub game_area: Rect,
    /// 左侧列外框
    pub left_column: Rect,
    /// 左侧第一个面板 (Status)
    pub status_panel: Rect,
    /// 左侧第二个面板 (Navigator)
    pub navigator_panel: Rect,
    /// 左侧第三个面板 (Logs)
    pub logs_panel: Rect,
    /// 中间列外框（整个区域，含 sudoku_panel + 按钮栏）
    pub center_column: Rect,
    /// 数独面板外框区域（第一层边框）
    pub sudoku_outer_frame: Rect,
    /// 中间面板的顶方向指示条（实心方块）
    pub sudoku_dir_top: Rect,
    /// 中间面板的底方向指示条（实心方块）
    pub sudoku_dir_bot: Rect,
    /// 中间面板的左方向指示条（实心方块）
    pub sudoku_dir_left: Rect,
    /// 中间面板的右方向指示条（实心方块）
    pub sudoku_dir_right: Rect,
    /// 数独面板内框区域（第二层边框，尺寸比外框上下各-2）
    pub sudoku_inner_frame: Rect,
    /// 中间数独网格内容（去掉内框，去掉方向边）
    pub grid_area: Rect,
    /// 数独外框（╔═╗ 双线）区域
    pub grid_frame: Rect,
    /// 消息行（位于网格下方）
    pub msg_area: Rect,
    /// 按钮栏（独立于 sudoku_panel，位于 center_column 底部）
    pub btn_area: Rect,
    /// 按钮实际起始列
    pub btn_content_x: u16,
    /// 右侧列外框
    pub right_column: Rect,
    /// 3D 立方体面板外框区域
    pub cube_outer_frame: Rect,
    /// 立方体面板的顶方向指示条
    pub cube_dir_top: Rect,
    /// 立方体面板的底方向指示条
    pub cube_dir_bot: Rect,
    /// 立方体面板的左方向指示条
    pub cube_dir_left: Rect,
    /// 立方体面板的右方向指示条
    pub cube_dir_right: Rect,
    /// 立方体内框区域（第二层边框）
    pub cube_inner_frame: Rect,
    /// 3D 立方体实际渲染区域（已按 aspect ratio 居中）
    pub cube_area: Rect,
    /// 商店预留区域
    pub shop_area: Rect,
    /// 当前页可见的按钮列表
    pub buttons: Vec<ButtonLayout>,
    /// 翻页控件位置（None 表示无翻页）
    pub pager: Option<ButtonPagerLayout>,
}

// ── PagerAction ──

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PagerAction {
    Prev,
    Next,
}

// ── SettingsPopupLayout ──

pub struct SettingsPopupLayout {
    pub popup_area: Rect,
    #[allow(dead_code)]
    pub content_area: Rect,
    pub fields: Vec<SettingsFieldLayout>,
}

pub struct SettingsFieldLayout {
    #[allow(dead_code)]
    pub row_y: u16,
    pub label_rect: Rect,
    pub left_arrow_rect: Rect,
    pub value_rect: Rect,
    pub right_arrow_rect: Rect,
}
