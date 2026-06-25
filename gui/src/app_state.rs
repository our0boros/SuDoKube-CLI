use bevy::prelude::*;

/// 全局应用状态机。
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum AppState {
    #[default]
    Loading,
    InGame,
    History,
}
