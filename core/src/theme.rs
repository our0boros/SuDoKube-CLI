#[cfg(feature = "bevy")]
use bevy::prelude::Resource;

/// 主题类型，CLI/GUI 各自解释具体颜色。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "bevy", derive(Resource))]
pub enum Theme {
    #[default]
    Dark,
    Light,
}

impl Theme {
    pub fn toggle(&self) -> Self {
        match self {
            Theme::Dark => Theme::Light,
            Theme::Light => Theme::Dark,
        }
    }
}
