//! 公共类型定义：屏幕枚举、位置类型、生成状态、设置辅助类型

use std::sync::{Arc, Mutex};
use std::time::Instant;

use sudokube_core::cube::Difficulty;

use crate::config;

// ── 单元位置类型 ──

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

// ── 屏幕枚举 ──

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
    KeymapConfig,
}

// ── 异步生成状态 ──

/// 异步生成状态
pub struct GeneratingState {
    pub difficulty: Difficulty,
    pub result: Arc<Mutex<Option<sudokube_core::game_state::GameState>>>,
    pub spinner: u8,
    pub started: Instant,
}

// ── 设置辅助类型 ──

/// 键位映射编辑状态
#[derive(Debug, Clone)]
pub struct KeymapEditState {
    /// 所有可绑定的 Action 列表
    pub actions: Vec<config::Action>,
    /// 当前选中的 Action 索引
    pub selected: usize,
    /// 滚动偏移
    pub scroll: u16,
    /// 是否正在等待用户按键(重新绑定模式)
    pub awaiting_key: bool,
    /// 正在重新绑定的 Action 索引
    pub rebinding_index: Option<usize>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsArrow {
    Left,
    Right,
}
