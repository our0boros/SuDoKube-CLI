use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    text::Line,
    widgets::Widget,
};

/// 按钮状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonState {
    Normal,
    Selected,
    Hovered,
    Active,
}

/// 按钮主题
#[derive(Debug, Clone, Copy)]
pub struct ButtonTheme {
    pub text: Color,
    pub background: Color,
    pub highlight: Color,
    pub shadow: Color,
}

pub const THEME_PRIMARY: ButtonTheme = ButtonTheme {
    text: Color::Rgb(16, 24, 48),
    background: Color::Rgb(48, 72, 144),
    highlight: Color::Rgb(64, 96, 192),
    shadow: Color::Rgb(32, 48, 96),
};

pub const THEME_SUCCESS: ButtonTheme = ButtonTheme {
    text: Color::Rgb(16, 48, 16),
    background: Color::Rgb(48, 144, 48),
    highlight: Color::Rgb(64, 192, 64),
    shadow: Color::Rgb(32, 96, 32),
};

pub const THEME_DANGER: ButtonTheme = ButtonTheme {
    text: Color::Rgb(48, 16, 16),
    background: Color::Rgb(144, 48, 48),
    highlight: Color::Rgb(192, 64, 64),
    shadow: Color::Rgb(96, 32, 32),
};

pub const THEME_NEUTRAL: ButtonTheme = ButtonTheme {
    text: Color::White,
    background: Color::DarkGray,
    highlight: Color::Gray,
    shadow: Color::Black,
};

/// 自定义按钮 Widget
#[derive(Debug, Clone)]
pub struct Button<'a> {
    pub label: Line<'a>,
    pub theme: ButtonTheme,
    pub state: ButtonState,
    pub show_border: bool,
}

impl<'a> Button<'a> {
    pub fn new<T: Into<Line<'a>>>(label: T) -> Self {
        Self {
            label: label.into(),
            theme: THEME_NEUTRAL,
            state: ButtonState::Normal,
            show_border: true,
        }
    }

    pub const fn theme(mut self, theme: ButtonTheme) -> Self {
        self.theme = theme;
        self
    }

    pub const fn state(mut self, state: ButtonState) -> Self {
        self.state = state;
        self
    }

    pub const fn border(mut self, show: bool) -> Self {
        self.show_border = show;
        self
    }

    fn colors(&self) -> (Color, Color, Color, Color) {
        let theme = self.theme;
        match self.state {
            ButtonState::Normal => (theme.background, theme.text, theme.shadow, theme.highlight),
            ButtonState::Selected => (theme.highlight, theme.text, theme.shadow, theme.highlight),
            ButtonState::Hovered => (theme.highlight, theme.text, theme.shadow, theme.highlight),
            ButtonState::Active => (theme.background, theme.text, theme.highlight, theme.shadow),
        }
    }
}

impl Widget for Button<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.width == 0 || area.height == 0 {
            return;
        }

        let (background, text, shadow, highlight) = self.colors();
        let bg_style = Style::new().bg(background).fg(text);

        // 填充背景
        for y in area.top()..area.bottom() {
            for x in area.left()..area.right() {
                buf[(x, y)].set_style(bg_style);
            }
        }

        // 渲染上边框高光
        if self.show_border && area.height > 2 {
            for x in area.left()..area.right() {
                buf[(x, area.top())]
                    .set_symbol("▔")
                    .set_fg(highlight)
                    .set_bg(background);
            }
        }

        // 渲染下边框阴影
        if self.show_border && area.height > 1 {
            for x in area.left()..area.right() {
                buf[(x, area.bottom() - 1)]
                    .set_symbol("▁")
                    .set_fg(shadow)
                    .set_bg(background);
            }
        }

        // 渲染标签居中
        let label_width = self.label.width() as u16;
        let label_x = area.x + (area.width.saturating_sub(label_width)) / 2;
        let label_y = area.y + (area.height.saturating_sub(1)) / 2;

        buf.set_line(label_x, label_y, &self.label, area.width);
    }
}
