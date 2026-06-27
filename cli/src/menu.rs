//! 菜单相关类型：MenuItem, MenuState

use sudokube_core::cube::Difficulty;

use crate::save::{GameRecord, self};

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
