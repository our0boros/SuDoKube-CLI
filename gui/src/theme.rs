use bevy::prelude::*;
use sudokube_core::theme::Theme;

/// 用于标记动态主题色的 UI 组件角色。
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ThemeColorRole {
    Panel,
    PanelBorder,
    Button,
    ButtonText,
    TextGiven,
}

/// 标记需要随主题更新的背景色。
#[derive(Component, Clone, Copy)]
pub struct ThemeBackground(pub ThemeColorRole);

/// 标记需要随主题更新的边框色。
#[derive(Component, Clone, Copy)]
pub struct ThemeBorder(pub ThemeColorRole);

/// 标记需要随主题更新的文字色。
#[derive(Component, Clone, Copy)]
pub struct ThemeText(pub ThemeColorRole);

#[derive(Debug, Clone, Resource)]
pub struct ThemeColors {
    pub background: Color,
    pub face_background: Color,
    pub cell_default: Color,
    pub cell_alt: Color,
    pub cell_highlight: Color,
    pub cell_related: Color,
    pub cell_selected: Color,
    pub cell_error: Color,
    pub text_given: Color,
    pub text_user: Color,
    pub text_note: Color,
    pub panel: Color,
    pub panel_border: Color,
    pub button: Color,
    pub button_hover: Color,
    pub button_text: Color,
}

impl Default for ThemeColors {
    fn default() -> Self {
        Self::for_theme(Theme::default())
    }
}

impl ThemeColors {
    pub fn for_theme(theme: Theme) -> Self {
        match theme {
            Theme::Dark => Self {
                background: Color::srgb(0.08, 0.08, 0.10),
                face_background: Color::srgb(0.12, 0.12, 0.15),
                cell_default: Color::srgb(0.14, 0.14, 0.18),
                cell_alt: Color::srgb(0.16, 0.16, 0.20),
                cell_highlight: Color::srgb(0.25, 0.35, 0.55),
                cell_related: Color::srgb(0.20, 0.20, 0.28),
                cell_selected: Color::srgb(0.35, 0.45, 0.65),
                cell_error: Color::srgb(0.75, 0.25, 0.25),
                text_given: Color::srgb(0.95, 0.95, 0.95),
                text_user: Color::srgb(0.45, 0.75, 0.95),
                text_note: Color::srgb(0.55, 0.55, 0.60),
                panel: Color::srgb(0.10, 0.10, 0.13),
                panel_border: Color::srgb(0.30, 0.30, 0.35),
                button: Color::srgb(0.22, 0.22, 0.28),
                button_hover: Color::srgb(0.30, 0.30, 0.38),
                button_text: Color::srgb(0.90, 0.90, 0.92),
            },
            Theme::Light => Self {
                background: Color::srgb(0.92, 0.92, 0.94),
                face_background: Color::srgb(0.88, 0.88, 0.90),
                cell_default: Color::srgb(0.98, 0.98, 0.99),
                cell_alt: Color::srgb(0.94, 0.94, 0.96),
                cell_highlight: Color::srgb(0.60, 0.75, 0.95),
                cell_related: Color::srgb(0.82, 0.85, 0.92),
                cell_selected: Color::srgb(0.55, 0.70, 0.90),
                cell_error: Color::srgb(0.90, 0.35, 0.35),
                text_given: Color::srgb(0.10, 0.10, 0.12),
                text_user: Color::srgb(0.15, 0.45, 0.75),
                text_note: Color::srgb(0.45, 0.45, 0.50),
                panel: Color::srgb(0.95, 0.95, 0.97),
                panel_border: Color::srgb(0.70, 0.70, 0.75),
                button: Color::srgb(0.82, 0.82, 0.86),
                button_hover: Color::srgb(0.72, 0.72, 0.78),
                button_text: Color::srgb(0.15, 0.15, 0.18),
            },
        }
    }

    pub fn color(&self, role: ThemeColorRole) -> Color {
        match role {
            ThemeColorRole::Panel => self.panel,
            ThemeColorRole::PanelBorder => self.panel_border,
            ThemeColorRole::Button => self.button,
            ThemeColorRole::ButtonText => self.button_text,
            ThemeColorRole::TextGiven => self.text_given,
        }
    }
}
